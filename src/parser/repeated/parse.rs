use crate::TryParseInput;

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

impl<'inp, 'c, L, F, O, Ctx, Lang: ?Sized> Repeated<F, O, L, Ctx, Lang> {
  pub(super) fn parse<Container>(
    &mut self,
    inp: &mut InputRef<'inp, 'c, L, Ctx, Lang>,
    container: &mut Container,
    on_stop: impl FnOnce(
      usize,
      &mut InputRef<'inp, 'c, L, Ctx, Lang>,
      &L::Span,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    F: TryParseInput<'inp, L, O, Ctx, Lang>,
    Ctx::Emitter: Emitter<'inp, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    Container: crate::container::Container<O>,
  {
    let mut num = 0;
    let ckp = inp.save();
    let mut cursor = ckp.cursor().clone();

    loop {
      match self.f.try_parse_input(inp) {
        Ok(Some(item)) => {
          container.push(item);
          num += 1;
        }
        Ok(None) => break,
        Err(err) => {
          let span = inp.span_since(&cursor);
          inp.emitter().emit_error(Spanned::new(span, err))?;
        }
      }
      cursor = inp.cursor().clone();
    }

    let span = inp.span_since(ckp.cursor());
    on_stop(num, inp, &span).map(|_| span)
  }
}
