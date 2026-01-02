use core::{convert::identity, mem};

use mayber::Maybe::{Owned, Ref};

use crate::{
  TryParseInput,
  container::Container as ContainerT,
  emitter::DelimitedEmitter,
  error::{Unclosed, Undelimited},
};

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

impl<'inp, L, P, Open, Close, O, Ctx, Delim, Lang: ?Sized>
  DelimitedBy<&mut Repeated<P, O, L, Ctx, Lang>, &Open, &Close, &Delim>
{
  fn parse_repeated<Container>(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    container: &mut Container,
    on_stop: impl FnOnce(
      usize,
      &mut InputRef<'inp, '_, L, Ctx, Lang>,
      &L::Span,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
    Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
    Delim: Clone,
    L: Lexer<'inp>,
    P: TryParseInput<'inp, L, O, Ctx, Lang>,
    Ctx::Emitter: DelimitedEmitter<'inp, Delim, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
    Container: Default + ContainerT<O> + DelimiterHandler<'inp, L>,
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
    let has_open = match state {
      Ok(left) => {
        container.on_open_delimiter(left);
        true
      }
      Err(err) => {
        inp.emitter().emit_unexpected_token(err)?;
        false
      }
    };

    let mut nums = 0;

    let mut elem_cur = inp.cursor().clone();

    let on_missing_close = |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
                            span: L::Span|
     -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
      if has_open {
        inp
          .emitter()
          .emit_unclosed(Unclosed::of(span, self.delimiter.clone()))?;
      } else {
        inp
          .emitter()
          .emit_undelimited(Undelimited::of(span, self.delimiter.clone()))?;
      }
      Ok(())
    };

    loop {
      match self.parser.f.try_parse_input(inp) {
        Err(err) => {
          let span = inp.span_since(&elem_cur);
          inp.emitter().emit_error(Spanned::new(span, err))?;
        }
        Ok(Some(nxt)) => {
          container.push(nxt);
          nums += 1;
        }
        // no more elemnts.
        Ok(None) => {
          let nxt = inp.sync_until_token()?;
          match nxt {
            None => on_missing_close(inp, inp.span_since(ckp.cursor()))?,
            Some(nxt) => {
              let tok = nxt
                .map(|t| t.into_token().cloned(), |t| t.into_token())
                .into_inner();

              if self.right_classifier.check(tok.data()).is_ok() {
                container.on_close_delimiter(tok);
                inp.skip_one();
              } else {
                on_missing_close(inp, inp.span_since(ckp.cursor()))?;
              }
            }
          }

          let span = inp.span_since(ckp.cursor());
          return on_stop(nums, inp, &span).map(|_| mem::take(container));
        }
      }

      elem_cur = inp.cursor().clone();
    }
  }
}
