use crate::{
  Check,
  container::SeparatorsContainer,
  emitter::{FullContainerEmitter, SeparatedEmitter},
  error::{
    syntax::{FullContainer, MissingSyntaxOf},
    token::MissingSeparatorOf,
  },
  lexer::{Checkpoint, Span},
};

use super::*;

use core::mem;

mod allow_leading;
mod allow_surrounded;
mod allow_trailing;
mod at_least;
mod at_most;
mod bounded;
mod require_leading;
mod require_surrounded;
mod require_trailing;
mod unbounded;

trait EndStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized> {
  fn handle_start_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  fn handle_element_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  fn handle_leading_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    leading_sep: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  fn handle_separator_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    sep: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

trait ContinueStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized> {
  fn handle_start_state(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    off: L::Offset,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

trait SeparatorStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized> {
  fn handle_start_state(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    sep_tok: &Spanned<L::Token, L::Span>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;
}

impl<'c, 'inp, F, SepClassifier, Condition, O, W, L, Ctx, Lang: ?Sized>
  SeparatedBy<&'c mut F, &'c mut SepClassifier, &'c mut Condition, O, W, L, Ctx, Lang>
{
  fn parse<'closure, Container, CH, SP, EH>(
    &mut self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    container: &mut Container,
    continue_state_handler: &CH,
    separator_state_handler: &SP,
    end_state_handler: &EH,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    F: ParseInput<'inp, L, O, Ctx, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    SepClassifier: Check<L::Token>,
    Ctx::Emitter:
      SeparatedEmitter<'inp, O, SepClassifier, L, Lang> + FullContainerEmitter<'inp, O, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    Container: SeparatorsContainer<Spanned<L::Token, L::Span>, O>,
    W: Window,
    EH: EndStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
    CH: ContinueStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
    SP: SeparatorStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
  {
    let mut state = State::Start;
    let ckp = inp.save();
    let mut num_elems = 0;

    loop {
      let (peeked, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

      let peek_span = match peeked.front() {
        None => {
          drop(peeked);
          return self.handle_end(state, inp, &ckp, num_elems, end_state_handler);
        }
        Some(tok) => {
          // the sync_until_token_then_peek_with_emitter guarantees the first token is not a `Lexed::Error`
          let tok = tok
            .as_maybe_ref()
            .map(
              |t| t.token().map(|t| *t, |t| t.unwrap_token_ref()),
              |t| t.token().map_data(|t| t.unwrap_token_ref()),
            )
            .into_inner();
          let peek_span = tok.span();
          match tok.data() {
            tok if self.sep.check(tok) => {
              drop(peeked);
              state = self.handle_separator(state, inp, container, separator_state_handler)?;

              continue;
            }
            _ => peek_span.clone(),
          }
        }
      };

      match self.condition.decide(peeked, emitter)? {
        Action::Stop => {
          return self.handle_end(state, inp, &ckp, num_elems, end_state_handler);
        }
        Action::Continue => {
          // if the peeked token belongs to an element, check the current state
          state = self.handle_continue(
            state,
            inp,
            &ckp,
            &peek_span,
            &mut num_elems,
            container,
            continue_state_handler,
          )?;
        }
      }
    }
  }

  pub(in crate::parser) fn handle_separator<'closure, Handler, Container>(
    &mut self,
    mut state: State<L::Token, L::Span>,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    container: &mut Container,
    handler: &Handler,
  ) -> Result<State<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>,
    Handler: SeparatorStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
    Container: SeparatorsContainer<Spanned<L::Token, L::Span>, O>,
  {
    let sep_tok = inp
      .next()
      .expect("peeked token already confirmed there must be a token");
    match state {
      // happy path, we found a separator after an element
      State::Element => {
        // Change the current state to Separator.
        state = State::Separator(sep_tok.map_data(|t| t.unwrap_token()));
      }
      // First token is a separator, we found another leading separator
      State::Leading(_) => {
        // whatever the leading spec is, multiple leading separators are not allowed
        // so we treat the old one as a unexpected token, emit it via the emitter,
        // and let the emitter decide whether to return early
        inp
          .emitter()
          .emit_missing_element(MissingSyntaxOf::<'_, O, L, Lang>::of(
            sep_tok.span_ref().start(),
          ))?;

        // As we have emitted the missing element error, so the behavior of the state machine
        // should be as if we have successfully parsed an element here.
        // So we push the new separator token into the container,
        // and change the state to Separator.
        // TODO(al8n): return error when separator container is full?
        let sep = sep_tok.map_data(|t| t.unwrap_token());
        container.push_separator(sep.clone());
        state = State::Separator(sep);
      }
      // first token is a separator
      State::Start => {
        // we do not need to check leading spec here, as we cached the leading separator token,
        // the check will be done when we find the first element or reach the end of input
        let st = sep_tok.map_data(|t| t.unwrap_token());
        handler.handle_start_state(inp, &st)?;
        // TODO(al8n): return error when separator container is full?
        container.push_separator(st.clone());
        state = State::Leading(st);
      }
      // we are in separator state, so the next token should be an element,
      State::Separator(_) => {
        // We found consecutive separators, emit missing element error via the emitter
        inp
          .emitter()
          .emit_missing_element(MissingSyntaxOf::<'_, O, L, Lang>::of(
            sep_tok.span_ref().start(),
          ))?;

        // TODO(al8n): return error when separator container is full?
        let sep_tok = sep_tok.map_data(|t| t.unwrap_token());
        container.push_separator(sep_tok.clone());
        state = State::Separator(sep_tok);
      }
    }
    Ok(state)
  }

  #[allow(clippy::too_many_arguments)]
  pub(in crate::parser) fn handle_continue<'closure, Container, Handler>(
    &mut self,
    mut state: State<L::Token, L::Span>,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    peek_span: &L::Span,
    num_elems: &mut usize,
    container: &mut Container,
    handler: &Handler,
  ) -> Result<State<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    F: ParseInput<'inp, L, O, Ctx, Lang>,
    W: Window,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    SepClassifier: Check<L::Token>,
    Ctx::Emitter:
      SeparatedEmitter<'inp, O, SepClassifier, L, Lang> + FullContainerEmitter<'inp, O, L, Lang>,
    Container: SeparatorsContainer<Spanned<L::Token, L::Span>, O>,
    Handler: ContinueStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
  {
    match state {
      // happy path, we found a separator before an element
      State::Separator(_) => {
        // parse the next element
        let element = self.f.parse_input(inp)?;
        if push(num_elems, container, element).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            container.len(),
            Container::capacity(),
          ))?;
        }
        state = State::Element;
      }
      // we are in leading state,
      State::Leading(_) => {
        // match leading_spec {
        //   // no leading separators allowed
        //   SepFixSpec::Deny(_) => {
        //     let (sep_span, sep_token) = leading_tok.into_components();
        //     inp
        //       .emitter()
        //       .emit_unexpected_leading_separator(
        //         UnexpectedLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(sep_span, sep_token),
        //       )?;
        //   }
        //   SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {}
        // }

        // let the passing handler deal with the leading separator

        // handler.handle_leading_state(
        //   *num_elems,
        //   inp,
        //   ckp,
        //   leading_tok,
        // )?;

        // parse the first element
        let element = self.f.parse_input(inp)?;
        if push(num_elems, container, element).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            *num_elems,
            Container::capacity(),
          ))?;
        }
        state = State::Element;
      }
      // nothing before element, parse the first element
      State::Start => {
        // match leading_spec {
        //   SepFixSpec::Require(_) => {
        //     let off = peek_span.start();
        //     // unhappy, missing the required leading separator
        //     inp
        //       .emitter()
        //       .emit_missing_leading_separator(
        //         MissingLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(off),
        //       )?;
        //   }
        //   SepFixSpec::Deny(_) | SepFixSpec::Allow(_) => {
        //     // so happyyyyy, no leading separators, just parse the first element
        //   }
        // }

        // let the passing handler deal with the start state
        handler.handle_start_state(inp, peek_span.start())?;

        // parse the first element
        let element = self.f.parse_input(inp)?;
        if push(num_elems, container, element).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            *num_elems,
            Container::capacity(),
          ))?;
        }

        state = State::Element;
      }
      // we are in element state, so the next token should be a separator,
      // so missing separator case, let's construct a missing separator error,
      // and emit it via the emitter, and let the emitter decide whether to return early
      State::Element => {
        let off = peek_span.start();
        inp
          .emitter()
          .emit_missing_separator(MissingSeparatorOf::<'_, SepClassifier, L, Lang>::of(off))?;

        // parse the next element
        let element = self.f.parse_input(inp)?;
        if push(num_elems, container, element).is_some() {
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_full_container(FullContainer::of(
            span,
            *num_elems,
            Container::capacity(),
          ))?;
        }
        state = State::Element;
      }
    }

    Ok(state)
  }

  pub(in crate::parser) fn handle_end<'closure, Handler>(
    &mut self,
    state: State<L::Token, L::Span>,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    num_elems: usize,
    handler: &Handler,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    F: ParseInput<'inp, L, O, Ctx, Lang>,
    W: Window,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    SepClassifier: Check<L::Token>,
    Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>,
    Handler: EndStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
  {
    // let minimum = self.config.minimum();
    // let maximum = self.config.maximum();
    // let leading_spec = self.config.leading();
    // let trailing_spec = self.config.trailing();

    Ok(match state {
      // we are in the start state, so no elements were found
      State::Start => {
        // let span = inp.span_since(ckp.cursor());
        // if minimum > 0 {
        //   inp
        //     .emitter()
        //     .emit_too_few(TooFew::of(span.clone(), num_elems, minimum))?;
        // }
        // span
        handler.handle_start_state(num_elems, inp, ckp)?
      }
      // we are in element state, so all good, check for trailing separator, and the minimum, maximum constraints
      State::Element => {
        // let full_span = inp.span_since(ckp.cursor());
        // if num_elems < minimum {
        //   inp
        //     .emitter()
        //     .emit_too_few(TooFew::of(full_span.clone(), num_elems, minimum))?;
        // }

        // if num_elems > maximum {
        //   inp
        //     .emitter()
        //     .emit_too_many(TooMany::of(full_span.clone(), num_elems, maximum))?;
        // }

        // if trailing_spec.is_require() {
        //   let off = inp.span().end();
        //   inp
        //     .emitter()
        //     .emit_missing_trailing_separator(
        //       MissingTrailingOf::<'_, SepClassifier, L, Lang>::trailing_of(off),
        //     )?;
        // }
        // full_span
        handler.handle_element_state(num_elems, inp, ckp)?
      }
      State::Leading(spanned) => {
        // // only find leading separators, no element
        // let (sep_span, sep_token) = spanned.into_components();
        // match leading_spec {
        //   SepFixSpec::Deny(_) => {
        //     // we are not allowed to have leading separators
        //     inp
        //       .emitter()
        //       .emit_unexpected_leading_separator(
        //         UnexpectedLeadingOf::<'_, SepClassifier, L, Lang>::leading_of(sep_span, sep_token),
        //       )?;
        //   }
        //   SepFixSpec::Allow(_) | SepFixSpec::Require(_) => {
        //     // we should emit an error as we are missing the element followed the leading separator
        //     inp
        //       .emitter()
        //       .emit_missing_element(MissingSyntaxOf::<'_, O, L, Lang>::of(sep_span.end()))?;
        //   }
        // }
        // inp.span_since(ckp.cursor())
        handler.handle_leading_state(num_elems, inp, ckp, spanned)?
      }
      // we have a trailing separator
      State::Separator(spanned) => {
        // let (sep_span, sep_token) = spanned.into_components();

        // // we have a trailing separator, but the spec says no trailing separators allowed
        // if trailing_spec.is_deny() {
        //   inp
        //     .emitter()
        //     .emit_unexpected_trailing_separator(
        //       UnexpectedTrailingOf::<'_, SepClassifier, L, Lang>::trailing_of(sep_span, sep_token),
        //     )?;
        // }

        // let full_span = inp.span_since(ckp.cursor());
        // let nums = container.len();
        // if nums < minimum {
        //   inp
        //     .emitter()
        //     .emit_too_few(TooFew::of(full_span.clone(), nums, minimum))?;
        // }

        // if nums > maximum {
        //   inp
        //     .emitter()
        //     .emit_too_many(TooMany::of(full_span.clone(), nums, maximum))?;
        // }

        // full_span
        handler.handle_separator_state(num_elems, inp, ckp, spanned)?
      }
    })
  }
}

#[cfg_attr(not(tarpaulin), inline(always))]
fn push<C, T>(nums: &mut usize, container: &mut C, item: T) -> Option<T>
where
  C: crate::container::Container<T>,
{
  match container.push(item) {
    None => {
      *nums += 1;
      None
    }
    Some(item) => Some(item),
  }
}
