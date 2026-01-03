use crate::{
  Check, TryParseInput,
  container::Container as ContainerT,
  emitter::SeparatedEmitter,
  error::{syntax::MissingSyntaxOf, token::MissingSeparatorOf},
  input::Checkpoint,
  span::Span,
  try_parse_input::{Accept, Decline},
};

use super::*;

use core::mem;

mod allow_leading;
mod allow_leading_require_trailing;
mod allow_surrounded;
mod allow_trailing;
mod at_least;
mod at_most;
mod bounded;
mod require_leading;
mod require_leading_allow_trailing;
mod require_surrounded;
mod require_trailing;
mod unbounded;

impl<'c, 'inp, F, SepClassifier, O, L, Ctx, Lang: ?Sized>
  Separated<&'c mut F, &'c mut SepClassifier, O, L, Ctx, Lang>
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
    F: TryParseInput<'inp, L, O, Ctx, Lang>,
    SepClassifier: Check<L::Token>,
    Ctx::Emitter: SeparatedEmitter<'inp, SepClassifier, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    Container: ContainerT<O> + SeparatorHandler<'inp, L>,
    EH: EndStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
    CH: ContinueStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
    SP: SeparatorStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
  {
    let mut state = State::Start;
    let ckp = inp.save();
    let mut cursor = ckp.cursor().clone();
    let mut num_elems = 0;

    loop {
      let tok = inp.sync_until_token()?;

      let peek_span = match tok {
        None => {
          return self.handle_end(state, inp, &ckp, num_elems, end_state_handler);
        }
        Some(tok) => {
          // the sync_until_token guarantees the first token is not a `Lexed::Error`
          let tok = tok
            .as_maybe_ref()
            .map(|t| t.token().copied(), |t| t.token())
            .into_inner();
          let peek_span = tok.span();
          match tok.data() {
            tok if self.sep.check(tok) => {
              state = self.handle_separator(state, inp, container, separator_state_handler)?;
              cursor = inp.cursor().clone();
              continue;
            }
            _ => peek_span.clone(),
          }
        }
      };

      match self.f.try_parse_input(inp) {
        Err(e) => {
          let span = inp.span_since(&cursor);
          inp.emitter().emit_error(Spanned::new(span, e))?;
        }
        Ok(Decline) => return self.handle_end(state, inp, &ckp, num_elems, end_state_handler),
        Ok(Accept(elem)) => {
          // if the peeked token belongs to an element, check the current state
          state = self.handle_continue(
            state,
            inp,
            peek_span,
            elem,
            &mut num_elems,
            container,
            continue_state_handler,
          )?;
        }
      }

      cursor = inp.cursor().clone();
    }
  }

  pub(super) fn handle_separator<'closure, Handler, Container>(
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
    Ctx::Emitter: SeparatedEmitter<'inp, SepClassifier, L, Lang>,
    Handler: SeparatorStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
    Container: ContainerT<O> + SeparatorHandler<'inp, L>,
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
          .emit_missing_element(MissingSyntaxOf::<'_, L, Lang>::of(
            sep_tok.span_ref().start(),
          ))?;

        // As we have emitted the missing element error, so the behavior of the state machine
        // should be as if we have successfully parsed an element here.
        // So we push the new separator token into the container,
        // and change the state to Separator.
        // TODO(al8n): return error when separator container is full?
        let sep = sep_tok.map_data(|t| t.unwrap_token());
        container.on_separator(sep.clone());
        state = State::Separator(sep);
      }
      // first token is a separator
      State::Start => {
        // we do not need to check leading spec here, as we cached the leading separator token,
        // the check will be done when we find the first element or reach the end of input
        let st = sep_tok.map_data(|t| t.unwrap_token());
        handler.handle_start_state(inp, &st)?;
        // TODO(al8n): return error when separator container is full?
        container.on_separator(st.clone());
        state = State::Leading(st);
      }
      // we are in separator state, so the next token should be an element,
      State::Separator(_) => {
        // We found consecutive separators, emit missing element error via the emitter
        inp
          .emitter()
          .emit_missing_element(MissingSyntaxOf::<'_, L, Lang>::of(
            sep_tok.span_ref().start(),
          ))?;

        // TODO(al8n): return error when separator container is full?
        let sep_tok = sep_tok.map_data(|t| t.unwrap_token());
        container.on_separator(sep_tok.clone());
        state = State::Separator(sep_tok);
      }
    }
    Ok(state)
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn handle_continue<'closure, Container, Handler>(
    &mut self,
    mut state: State<L::Token, L::Span>,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    peek_span: L::Span,
    element: O,
    num_elems: &mut usize,
    container: &mut Container,
    handler: &Handler,
  ) -> Result<State<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    'inp: 'closure,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    F: TryParseInput<'inp, L, O, Ctx, Lang>,
    SepClassifier: Check<L::Token>,
    Ctx::Emitter: SeparatedEmitter<'inp, SepClassifier, L, Lang>,
    Container: ContainerT<O> + SeparatorHandler<'inp, L>,
    Handler: ContinueStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
  {
    match state {
      // happy path, we found a separator before an element
      State::Separator(_) => {
        push(num_elems, container, element);
        state = State::Element;
      }
      // we are in leading state,
      State::Leading(_) => {
        push(num_elems, container, element);
        state = State::Element;
      }
      // nothing before element, parse the first element
      State::Start => {
        // let the passing handler deal with the start state
        handler.handle_start_state(inp, peek_span.start())?;

        push(num_elems, container, element);

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
        push(num_elems, container, element);
        state = State::Element;
      }
    }

    Ok(state)
  }

  pub(super) fn handle_end<'closure, Handler>(
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
    F: TryParseInput<'inp, L, O, Ctx, Lang>,
    SepClassifier: Check<L::Token>,
    Ctx::Emitter: SeparatedEmitter<'inp, SepClassifier, L, Lang>,
    Handler: EndStateHandler<'inp, 'closure, SepClassifier, O, L, Ctx, Lang>,
  {
    Ok(match state {
      // we are in the start state, so no elements were found
      State::Start => handler.handle_start_state(num_elems, inp, ckp)?,
      // we are in element state, so all good, check for trailing separator, and the minimum, maximum constraints
      State::Element => handler.handle_element_state(num_elems, inp, ckp)?,
      State::Leading(spanned) => handler.handle_leading_state(num_elems, inp, ckp, spanned)?,
      // we have a trailing separator
      State::Separator(spanned) => handler.handle_separator_state(num_elems, inp, ckp, spanned)?,
    })
  }
}

#[cfg_attr(not(tarpaulin), inline(always))]
fn push<C, T>(nums: &mut usize, container: &mut C, item: T)
where
  C: crate::container::Container<T>,
{
  container.push(item);
  *nums += 1;
}
