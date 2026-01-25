use core::mem;

use crate::{
  TryParseInput,
  container::Container as ContainerT,
  delimiter::Delimiter,
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
    Delim: Delimiter<'inp, L, Lang>,
    L: Lexer<'inp>,
    P: TryParseInput<'inp, L, O, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
    Container: Default + ContainerT<O> + DelimiterHandler<'inp, L>,
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

    match left_delimiter {
      None if inp.is_eoi() => {
        return Err(UnexpectedEot::eot_of(inp.cursor().as_inner().clone()).into());
      }
      None => {
        // safe unwrap as we know when left_delimiter is None, first_kind is Some
        inp.emitter().emit_unexpected_token(first_kind.unwrap())?;
      }
      Some(open) => {
        container.on_open_delimiter(open);
      }
    };

    let mut nums = 0;
    let mut elem_cur = inp.cursor().clone();

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
          let mut err = None;
          match inp.try_expect(|t| match Delim::is_close(&t.data.kind()) {
            true => true,
            false => {
              err = Some(Delim::unexpected_close_token(t.cloned()));
              false
            }
          })? {
            None => {
              inp.emitter().emit_unexpected_token(err.unwrap())?;
            }
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
