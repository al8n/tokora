use crate::emitter::FullContainerEmitter;

use super::*;

impl<'inp, L, F, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<RepeatedOnCondition<F, Condition, O, W, L, Ctx, Lang>, Container, Ctx, Lang>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  W: Window,
  Ctx::Emitter: FullContainerEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + crate::container::Container<O>,
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

impl<'inp, L, F, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for Collect<RepeatedOnCondition<F, Condition, O, W, L, Ctx, Lang>, Container, Ctx, Lang>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  W: Window,
  Ctx::Emitter: FullContainerEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + crate::container::Container<O>,
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

impl<'inp, 'c, L, F, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut RepeatedOnCondition<F, Condition, O, W, L, Ctx, Lang>,
    &'c mut Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  W: Window,
  Ctx::Emitter: FullContainerEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: crate::container::Container<O>,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    // let ckp = inp.save();
    // let mut nums = 0;

    // loop {
    //   let (peeked, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

    //   match self.parser.condition.decide(peeked, emitter) {
    //     Err(err) => return Err(err),
    //     Ok(action) => match action {
    //       Action::Stop => return Ok(inp.span_since(ckp.cursor())),
    //       Action::Continue => {
    //         if self
    //           .container
    //           .push(self.parser.f.parse_input(inp)?)
    //           .is_some()
    //         {
    //           let span = inp.span_since(ckp.cursor());
    //           inp.emitter().emit_full_container(FullContainer::of(
    //             span,
    //             nums,
    //             Container::capacity(),
    //           ))?;
    //         }
    //         nums += 1;
    //       }
    //     },
    //   }
    // }
    self
      .parser
      .parse(inp, &mut self.container, |_, _, _| Ok(()))
  }
}
