use crate::{emitter::FullContainerEmitter, error::syntax::FullContainer};

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

impl<'inp, 'c, L, F, Condition, O, Ctx, Lang: ?Sized, W>
  Repeated<F, Condition, O, W, L, Ctx, Lang>
{
  fn parse<Container>(
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
    F: ParseInput<'inp, L, O, Ctx, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    W: Window,
    Ctx::Emitter: FullContainerEmitter<'inp, O, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    Container: crate::container::Container<O>,
  {
    let ckp = inp.save();
    let mut nums = 0;

    loop {
      let (peeked, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

      match self.condition.decide(peeked, emitter) {
        Err(err) => return Err(err),
        Ok(action) => match action {
          Action::Stop => {
            let span = inp.span_since(ckp.cursor());
            return on_stop(nums, inp, &span).map(|_| span);
          }
          Action::Continue => {
            if container.push(self.f.parse_input(inp)?).is_some() {
              let span = inp.span_since(ckp.cursor());
              inp.emitter().emit_full_container(FullContainer::of(
                span,
                nums,
                Container::capacity(),
              ))?;
            }
            nums += 1;
          }
        },
      }
    }
  }
}
