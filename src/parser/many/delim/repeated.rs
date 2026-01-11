use core::mem;

use crate::{
  TryParseInput,
  container::Container as ContainerT,
  delimiter::DelimiterSelector,
  emitter::DelimitedEmitter,
  error::{Unclosed, Undelimited},
  try_parse_input::{Accept, Decline},
};

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

impl<'inp, L, P, O, Ctx, Delim, Lang: ?Sized>
  DelimitedBy<&mut Repeated<P, O, L, Ctx, Lang>, Delim>
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
    Delim: DelimiterSelector<'inp, L, Lang>,
    L: Lexer<'inp>,
    P: TryParseInput<'inp, L, O, Ctx, Lang>,
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
        Ok(Accept(nxt)) => {
          container.push(nxt);
          nums += 1;
        }
        // no more elemnts.
        Ok(Decline) => {
          let mut close_kind = None;
          match inp.try_expect(|t| match self.right_classifier.check(t.data()) {
            Ok(_) => true,
            Err(knd) => {
              close_kind = Some(knd);
              false
            }
          })? {
            None => on_missing_close(inp, inp.span_since(ckp.cursor()))?,
            Some(close) => {
              container.on_close_delimiter(close);
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
