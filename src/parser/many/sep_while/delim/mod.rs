use core::mem;

use crate::{
  container::Container as ContainerT,
  emitter::{DelimitedEmitter, SeparatedEmitter},
  error::{Unclosed, Undelimited},
};

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

mod allow_leading;
mod allow_leading_require_trailing;
mod allow_surrounded;
mod allow_trailing;

mod require_leading;
mod require_leading_allow_trailing;
mod require_surrounded;
mod require_trailing;

impl<'c, 'inp, L, P, Sep, O, Condition, Ctx, Delim, W, Lang: ?Sized>
  DelimitedBy<SeparatedWhile<&'c mut P, &'c mut Sep, &'c mut Condition, O, W, L, Ctx, Lang>, Delim>
{
  fn parse_separated<'closure, Container, CH, SP, EH>(
    &mut self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    container: &mut Container,
    continue_state_handler: &CH,
    separator_state_handler: &SP,
    end_state_handler: &EH,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Delim: DelimiterSelector<'inp, L, Lang>,
    Sep: Check<L::Token>,
    L: Lexer<'inp>,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    W: Window,
    Ctx::Emitter: DelimitedEmitter<'inp, Delim, L, Lang> + SeparatedEmitter<'inp, Sep, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
    Container: DelimiterHandler<'inp, L> + SeparatorHandler<'inp, L> + ContainerT<O>,
    EH: EndStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang>,
    CH: ContinueStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang>,
    SP: SeparatorStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang>,
  {
    // Sync the input to the next token boundary, any lexer errors will be emitted during this process.
    let ckp = inp.save();
    let mut first_kind = None;
    let left_delimiter = inp.try_expect(|tok| {
      let (span, tok) = tok.into_components();
      match Delim::is_open(&tok.kind()) {
        false => {
          first_kind = Some(Delim::unexpected_open_token(Spanned::new(
            span.clone(),
            tok.clone(),
          )));
          false
        }
        true => true,
      }
    })?;

    let mut left_span = None;
    let _has_open = match left_delimiter {
      None if inp.is_eoi() => {
        return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into());
      }
      None => {
        // safe unwrap as we know when left_delimiter is None, first_kind is Some
        // inp.emitter().emit_unexpected_token(first_kind.unwrap())?;
        false
      }
      Some(open) => {
        left_span = Some(open.span_ref().clone());
        container.on_open_delimiter(open);
        true
      }
    };

    let mut state: State<L::Token, L::Span> = State::Start;
    let parser = &mut self.parser;
    let mut num_elems = 0;

    let elems_start = inp.cursor().clone();
    loop {
      let mut is_sep = false;
      match inp.try_expect(|tok| {
        if parser.sep.check(tok.data) {
          is_sep = true;
          true
        } else {
          Delim::is_close(&tok.kind())
        }
      })? {
        Some(tok) => {
          if is_sep {
            state = parser.handle_separator(state, inp, tok, container, separator_state_handler)?;
            continue;
          }

          parser.handle_end(state, inp, &ckp, num_elems, end_state_handler)?;
          container.on_close_delimiter(tok);
          return Ok(inp.span_since(ckp.cursor()));
        }
        None => {
          let (peeked, emitter) = inp.peek_with_emitter::<W>()?;

          if peeked.is_empty() {
            drop(peeked);
            parser.handle_end(state, inp, &ckp, num_elems, end_state_handler)?;

            let span = inp.span_since(ckp.cursor());
            inp
              .emitter()
              .emit_unclosed(Unclosed::of(span, Delim::name()))?;

            return Ok(inp.span_since(&elems_start));
          }

          let first_span = peeked
            .front()
            .unwrap()
            .as_maybe_ref()
            .map(|t| t.token().copied(), |t| t.token())
            .into_inner()
            .span()
            .clone();

          match parser.condition.decide(peeked, emitter)? {
            Action::Stop => {
              parser.handle_end(state, inp, &ckp, num_elems, end_state_handler)?;

              return match inp.try_expect(|tok| Delim::is_close(&tok.kind()))? {
                Some(closed) => {
                  container.on_close_delimiter(closed);
                  Ok(inp.span_since(&elems_start))
                }
                None => {
                  let span = inp.span_since(ckp.cursor());
                  inp
                    .emitter()
                    .emit_undelimited(Undelimited::of(span, Delim::name()))?;

                  Ok(inp.span_since(&elems_start))
                }
              };
            }
            Action::Continue => {
              // if the peeked token belongs to an element, check the current state
              state = parser.handle_continue(
                state,
                inp,
                &first_span,
                &mut num_elems,
                container,
                continue_state_handler,
              )?;
            }
          }
        }
      }
    }
  }
}
