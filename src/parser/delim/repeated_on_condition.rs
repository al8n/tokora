use core::{convert::identity, mem};

use mayber::Maybe::{Owned, Ref};

use crate::{
  container::Container as ContainerT,
  emitter::DelimitedEmitter,
  error::{Unclosed, Undelimited},
};

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

impl<'inp, L, P, Open, Close, O, Condition, Ctx, Delim, W, Lang: ?Sized>
  DelimitedBy<&mut RepeatedOnCondition<P, Condition, O, W, L, Ctx, Lang>, &Open, &Close, &Delim>
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
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    W: Window,
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

    loop {
      let (mut peeked, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

      if let Some(front) = peeked.front() {
        let tok = front
          .as_maybe_ref()
          .map(
            |t| t.token().data().unwrap_token_ref(),
            |t| t.as_ref().token().data().unwrap_token_ref(),
          )
          .into_inner();

        // find the ending delimiter
        if self.right_classifier.check(tok).is_ok() {
          let front = peeked.pop_front().expect("just checked there is a front");
          drop(peeked);
          let close = match front {
            Ref(_) => inp
              .next()
              .expect("peeked guarantee there is a next token")
              .map_data(|t| t.unwrap_token()),
            Owned(ct) => ct.into_token().map_data(|t| t.unwrap_token()),
          };
          container.on_close_delimiter(close);

          let span = inp.span_since(ckp.cursor());
          return on_stop(nums, inp, &span).map(|_| mem::take(container));
        }
      }

      match self.parser.condition.decide(peeked, emitter) {
        Err(err) => return Err(err),
        Ok(action) => match action {
          // missing ending delimiter
          Action::Stop => {
            if has_open {
              let span = inp.span_since(ckp.cursor());
              inp
                .emitter()
                .emit_unclosed(Unclosed::of(span, self.delimiter.clone()))?;
            } else {
              let span = inp.span_since(ckp.cursor());
              inp
                .emitter()
                .emit_undelimited(Undelimited::of(span, self.delimiter.clone()))?;
            }

            return Ok(mem::take(container));
          }
          Action::Continue => {
            container.push(self.parser.f.parse_input(inp)?);
            nums += 1;
          }
        },
      }
    }
  }
}
