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

impl<'c, 'inp, L, P, Open, Close, Sep, O, Condition, Ctx, Delim, W, Lang: ?Sized>
  DelimitedBy<
    SeparatedWhile<&'c mut P, &'c mut Sep, &'c mut Condition, O, W, L, Ctx, Lang>,
    &Open,
    &Close,
    &Delim,
  >
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
    Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
    Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
    Delim: Clone,
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
      match self.left_classifier.check(tok) {
        Err(knd) => {
          first_kind =
            Some(UnexpectedToken::expected_one(span.clone(), knd).with_found(tok.clone()));
          false
        }
        Ok(_) => true,
      }
    })?;

    let mut left_span = None;
    let has_open = match left_delimiter {
      None if inp.is_eoi() => {
        return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into());
      }
      None => {
        // safe unwrap as we know when left_delimiter is None, first_kind is Some
        inp.emitter().emit_unexpected_token(first_kind.unwrap())?;
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
    let (elems_span, right) = loop {
      let (peeked, emitter) = inp.peek_with_emitter::<W>()?;

      let peek_span = match peeked.front() {
        None => {
          drop(peeked);
          break (
            parser.handle_end(state, inp, &ckp, num_elems, end_state_handler)?,
            None,
          );
        }
        Some(tok) => {
          let tok = tok
            .as_maybe_ref()
            .map(|t| t.token().copied(), |t| t.token())
            .into_inner();

          let peek_span = tok.span();
          match tok.data() {
            t if parser.sep.check(t) => {
              drop(peeked);
              state = parser.handle_separator(state, inp, container, separator_state_handler)?;

              continue;
            }
            t => match self.right_classifier.check(t) {
              Ok(_) => {
                drop(peeked);

                let Ok(Some(tok)) = inp.next() else {
                  unreachable!("peeked guarantee there is a next token")
                };

                break (inp.span_since(&elems_start), Some(tok));
              }
              Err(_) => peek_span.clone(),
            },
          }
        }
      };

      match parser.condition.decide(peeked, emitter)? {
        Action::Stop => {
          break (
            parser.handle_end(state, inp, &ckp, num_elems, end_state_handler)?,
            None,
          );
        }
        Action::Continue => {
          // if the peeked token belongs to an element, check the current state
          state = parser.handle_continue(
            state,
            inp,
            &peek_span,
            &mut num_elems,
            container,
            continue_state_handler,
          )?;
        }
      }
    };

    let right = match right {
      Some(tok) => Some(tok),
      None => inp.try_expect(|t| self.right_classifier.check(t.data()).is_ok())?,
    };

    match right {
      // missing closing delimiter
      None if has_open => {
        let span = inp.span_since(ckp.cursor());
        inp
          .emitter()
          .emit_unclosed(Unclosed::of(span, self.delimiter.clone()))?;
      }
      // no open and close delimiters
      None => {
        let span = inp.span_since(ckp.cursor());
        inp
          .emitter()
          .emit_undelimited(Undelimited::of(span, self.delimiter.clone()))?;
      }
      Some(right) => {
        container.on_close_delimiter(right);
      }
    }

    Ok(elems_span)
  }
}
