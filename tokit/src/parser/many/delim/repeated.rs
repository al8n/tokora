use core::mem;

use crate::{
  TryParseInput,
  container::Container as ContainerT,
  delimiter::Delimiter,
  emitter::FullContainerEmitter,
  error::syntax::FullContainer,
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
    Ctx::Emitter: FullContainerEmitter<'inp, L, Lang>,
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
          // TODO(al8n): tracing dropped element
          if let Err(_e) = container.push(nxt) {
            let span = inp.span_since(ckp.cursor());
            inp.emitter().emit_full_container(FullContainer::of(
              span,
              nums,
              container.max_capacity(),
            ))?;
          }
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
            None if err.is_some() => {
              inp.emitter().emit_unexpected_token(err.unwrap())?;
            }
            None => {
              // EOI — no close delimiter found
            }
            Some(close) => {
              container.on_close_delimiter(close);
            }
          }

          let span = inp.span_since(ckp.cursor());
          return on_stop(nums, inp, &span).map(|_| mem::take(container));
        }
      }

      let new_cursor = inp.cursor().clone();
      if new_cursor.as_inner() == elem_cur.as_inner() {
        break;
      }
      elem_cur = new_cursor;
    }

    // No progress was made — treat as end of elements
    let mut close_err = None;
    match inp.try_expect(|t| match Delim::is_close(&t.data.kind()) {
      true => true,
      false => {
        close_err = Some(Delim::unexpected_close_token(t.cloned()));
        false
      }
    })? {
      None if close_err.is_some() => {
        inp.emitter().emit_unexpected_token(close_err.unwrap())?;
      }
      None => {
        // EOI — no tokens left, no close delimiter
      }
      Some(close) => {
        container.on_close_delimiter(close);
      }
    }

    let span = inp.span_since(ckp.cursor());
    on_stop(nums, inp, &span).map(|_| mem::take(container))
  }
}
