use core::mem;

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
  DelimitedBy<&mut RepeatedWhile<P, Condition, O, W, L, Ctx, Lang>, &Open, &Close, &Delim>
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
        container.on_open_delimiter(open);
        true
      }
    };

    let mut nums = 0;

    loop {
      let (mut peeked, emitter) = inp.sync_errors_then_peek_with_emitter::<W>()?;

      if let Some(front) = peeked.front() {
        let tok = front
          .as_maybe_ref()
          .map(|t| t.token().copied(), |t| t.token())
          .into_inner();

        // find the ending delimiter
        if self.right_classifier.check(tok.data()).is_ok() {
          let front = peeked.pop_front().expect("just checked there is a front");
          drop(peeked);
          let close = match front {
            Ref(_) => inp.next()?.expect("peeked guarantee there is a next token"),
            Owned(ct) => ct.into_token(),
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
