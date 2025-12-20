use core::{convert::identity, mem};

use mayber::Maybe::{Owned, Ref};

use crate::{
  container::{DelimiterContainer, SeparatorsContainer},
  emitter::{DelimitedEmitter, FullContainerEmitter, SeparatedEmitter},
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
    SeparatedBy<&'c mut P, &'c mut Sep, &'c mut Condition, O, W, L, Ctx, Lang>,
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
    Ctx::Emitter: DelimitedEmitter<'inp, Delim, L, Lang>
      + SeparatedEmitter<'inp, O, Sep, L, Lang>
      + FullContainerEmitter<'inp, O, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
    Container: DelimiterContainer<Spanned<L::Token, L::Span>, Spanned<L::Token, L::Span>, O>
      + SeparatorsContainer<Spanned<L::Token, L::Span>, O>,
    EH: EndStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang>,
    CH: ContinueStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang>,
    SP: SeparatorStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang>,
  {
    // Sync the input to the next token boundary, any lexer errors will be emitted during this process.
    let ckp = inp.save();
    let first = inp.sync_until_token()?;

    let state = match first {
      // End of input reached
      None => return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into()),
      Some(maybe_tok) => {
        let ct = maybe_tok.as_maybe_ref().map(identity, |t| t.as_ref());

        let tok = ct.token().copied().into_data();
        match self.left_classifier.check(tok) {
          Err(knd) => {
            let (span, tok) = maybe_tok
              .map(|t| t.into_token().cloned(), |t| t.into_token())
              .into_inner()
              .into_components();

            Err(
              UnexpectedToken::<_, _, _, Lang>::with_expected_of(span, Expected::one(knd))
                .with_found(tok),
            )
          }
          Ok(_) => {
            // consume the opening delimiter token
            let tok = match maybe_tok {
              Ref(_) => inp
                .next()
                .expect("peeked guarantee there is a next token")
                .map_data(|t| t.unwrap_token()),
              Owned(ct) => ct.into_token(),
            };
            Ok(tok)
          }
        }
      }
    };

    // we already handled the first token above
    let mut left_span = None;
    let has_open = match state {
      Ok(left) => {
        left_span = Some(left.span_ref().clone());
        container.push_open(left);
        true
      }
      Err(err) => {
        inp.emitter().emit_unexpected_token(err)?;
        false
      }
    };

    let mut state: State<L::Token, L::Span> = State::Start;
    let parser = &mut self.parser;
    let mut num_elems = 0;

    let elems_start = inp.cursor().clone();
    let (elems_span, right) = loop {
      let (peeked, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

      let peek_span = match peeked.front() {
        None => {
          drop(peeked);
          break (
            parser.handle_end(state, inp, &ckp, num_elems, end_state_handler)?,
            None,
          );
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
            t if parser.sep.check(t) => {
              drop(peeked);
              state = parser.handle_separator(state, inp, container, separator_state_handler)?;

              continue;
            }
            t => match self.right_classifier.check(t) {
              Ok(_) => {
                drop(peeked);

                let Ok(Some(tok)) = inp.next_token() else {
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
            &ckp,
            &peek_span,
            &mut num_elems,
            container,
            continue_state_handler,
          )?;
        }
      }
    };

    match right.or_else(|| match inp.peek_one() {
      None => None,
      Some(tok) => {
        let t = tok
          .as_maybe_ref()
          .map(
            |t| t.token().map(|t| *t, |t| t.unwrap_token_ref()),
            |t| t.token().map_data(|t| t.unwrap_token_ref()),
          )
          .into_inner();

        if self.right_classifier.check(t.data()).is_ok() {
          inp.next().map(|t| t.map_data(|t| t.unwrap_token()))
        } else {
          None
        }
      }
    }) {
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
        let _ = container.push_close(right);
      }
    }

    Ok(elems_span)
  }
}
