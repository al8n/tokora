use crate::{
  emitter::RepeatedEmitter,
  error::syntax::{FullContainer, TooFew, TooMany},
};

use super::*;

impl<'inp, L, F, Condition, O, Container, Ctx, Max, Min, Lang: ?Sized, Window>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<Repeated<F, Condition, O, Window, RepeatedOptions<Max, Min>>, Container>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, Window::CAPACITY, Lang>,
  Window: Capacity,
  Ctx::Emitter: RepeatedEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + crate::container::Container<O>,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let ckp = inp.save();
    let mut nums = 0;
    let max = self.parser.maximum();
    let min = self.parser.minimum();

    loop {
      let (peeked, emitter) = inp.peek_with_emitter::<Window::CAPACITY>();

      match self.parser.condition.decide(peeked, emitter) {
        Err(err) => return Err(err),
        Ok(action) => match action {
          Action::End => {
            if min > nums {
              let span = inp.span_since(ckp.cursor());
              inp.emitter().emit_too_few(TooFew::of(span, nums, min))?;
            }

            if nums > max {
              let span = inp.span_since(ckp.cursor());
              inp.emitter().emit_too_many(TooMany::of(span, nums, max))?;
            }

            return Ok(core::mem::take(&mut self.container));
          }
          Action::Continue => {
            if self
              .container
              .push(self.parser.f.parse_input(inp)?)
              .is_some()
            {
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
