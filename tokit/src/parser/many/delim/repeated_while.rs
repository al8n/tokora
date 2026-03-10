use core::mem;

use crate::{
  container::Container as ContainerT, delimiter::Delimiter, emitter::FullContainerEmitter,
  error::syntax::FullContainer,
};

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

impl<'inp, L, P, O, Condition, Ctx, Delim, W, Lang: ?Sized>
  DelimitedBy<&mut RepeatedWhile<P, Condition, O, W, L, Ctx, Lang>, Delim>
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
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    W: Window,
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
    }

    let mut nums = 0;

    loop {
      let mut err = None;
      match inp.try_expect(|tok| match Delim::is_close(&tok.kind()) {
        true => true,
        false => {
          err = Some(Delim::unexpected_close_token(tok.cloned()));
          false
        }
      })? {
        Some(closed) => {
          container.on_close_delimiter(closed);
          let span = inp.span_since(ckp.cursor());
          return on_stop(nums, inp, &span).map(|_| mem::take(container));
        }
        None => {
          let (peeked, emitter) = inp.peek_with_emitter::<W>()?;
          match self.parser.condition.decide(peeked, emitter) {
            Err(err) => return Err(err),
            Ok(action) => match action {
              // missing ending delimiter
              Action::Stop => {
                if let Some(err) = err {
                  inp.emitter().emit_unexpected_token(err)?;
                }
                return Ok(mem::take(container));
              }
              Action::Continue => {
                // TODO(al8n): tracing dropped element
                if let Err(_e) = container.push(self.parser.f.parse_input(inp)?) {
                  let span = inp.span_since(ckp.cursor());
                  inp.emitter().emit_full_container(FullContainer::of(
                    span,
                    nums,
                    container.max_capacity(),
                  ))?;
                }
                nums += 1;
              }
            },
          }
        }
      }
    }
  }
}
