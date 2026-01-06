use core::{convert::identity, mem};

use mayber::Maybe::{Owned, Ref};

use crate::{
  TryParseInput,
  container::Container as ContainerT,
  emitter::{DelimitedEmitter, SeparatedEmitter},
  error::{Unclosed, Undelimited},
  try_parse_input::{Accept, Decline},
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

impl<'c, 'inp, L, P, Open, Close, Sep, O, Ctx, Delim, Lang: ?Sized>
  DelimitedBy<Separated<&'c mut P, &'c mut Sep, O, L, Ctx, Lang>, &Open, &Close, &Delim>
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
    P: TryParseInput<'inp, L, O, Ctx, Lang>,
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
    let first = inp.sync_errors()?;

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
        container.on_open_delimiter(left);
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
    let mut cursor = elems_start.clone();
    let (elems_span, right) = loop {
      let peeked = inp.sync_errors()?;

      let peek_span = match peeked {
        None => {
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
              state = parser.handle_separator(state, inp, container, separator_state_handler)?;
              cursor = inp.cursor().clone();
              continue;
            }
            t => match self.right_classifier.check(t) {
              Ok(_) => {
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

      match parser.f.try_parse_input(inp) {
        Err(e) => {
          let span = inp.span_since(&cursor);
          inp.emitter().emit_error(Spanned::new(span, e))?;
        }
        Ok(Decline) => {
          break (
            parser.handle_end(state, inp, &ckp, num_elems, end_state_handler)?,
            None,
          );
        }
        Ok(Accept(elem)) => {
          // if the peeked token belongs to an element, check the current state
          state = parser.handle_continue(
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
        container.on_close_delimiter(right);
      }
    }

    Ok(elems_span)
  }
}
