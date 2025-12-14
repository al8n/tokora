use crate::{
  emitter::RepeatedEmitter,
  error::syntax::{FullContainer, TooFew, TooMany},
};

use super::*;

impl<'inp, L, F, Condition, O, Container, Ctx, Max, Min, Lang: ?Sized, W>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    Repeated<F, Condition, O, W, L, Ctx, RepeatedOptions<Max, Min>, Lang>,
    Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  W: Window,
  Ctx::Emitter: RepeatedEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + crate::container::Container<O>,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self
      .as_mut()
      .parse_input(inp)
      .map(|_| core::mem::take(&mut self.container))
  }
}

impl<'inp, L, F, Condition, O, Container, Ctx, Max, Min, Lang: ?Sized, W>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for Collect<
    Repeated<F, Condition, O, W, L, Ctx, RepeatedOptions<Max, Min>, Lang>,
    Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  W: Window,
  Ctx::Emitter: RepeatedEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + crate::container::Container<O>,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self
      .as_mut()
      .parse_input(inp)
      .map(|span| Spanned::new(span, core::mem::take(&mut self.container)))
  }
}

impl<'inp, 'c, L, F, Condition, O, Container, Ctx, Max, Min, Lang: ?Sized, W>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut Repeated<F, Condition, O, W, L, Ctx, RepeatedOptions<Max, Min>, Lang>,
    &'c mut Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  W: Window,
  Ctx::Emitter: RepeatedEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: crate::container::Container<O>,
  Max: super::MaxSpec,
  Min: super::MinSpec,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let ckp = inp.save();
    let mut nums = 0;
    let max = self.parser.maximum();
    let min = self.parser.minimum();

    loop {
      let (peeked, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

      match self.parser.condition.decide(peeked, emitter) {
        Err(err) => return Err(err),
        Ok(action) => match action {
          Action::Stop => {
            if min > nums {
              let span = inp.span_since(ckp.cursor());
              inp.emitter().emit_too_few(TooFew::of(span, nums, min))?;
            }

            if nums > max {
              let span = inp.span_since(ckp.cursor());
              inp.emitter().emit_too_many(TooMany::of(span, nums, max))?;
            }

            return Ok(inp.span_since(ckp.cursor()));
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
