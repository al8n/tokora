use crate::{emitter::TooFewEmitter, error::syntax::TooFew};

use super::*;

impl<'inp, L, F, O, Container, Ctx, Lang: ?Sized> ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<AtLeast<Repeated<F, O, L, Ctx, Lang>>, Container, Ctx, Lang>
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Ctx::Emitter: TooFewEmitter<'inp, O, L, Lang>,
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

impl<'inp, L, F, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for Collect<AtLeast<Repeated<F, O, L, Ctx, Lang>>, Container, Ctx, Lang>
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Ctx::Emitter: TooFewEmitter<'inp, O, L, Lang>,
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

impl<'inp, 'c, L, F, O, Container, Ctx, Lang: ?Sized> ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<&'c mut AtLeast<Repeated<F, O, L, Ctx, Lang>>, &'c mut Container, Ctx, Lang>
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Ctx::Emitter: TooFewEmitter<'inp, O, L, Lang>,
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
    let min = self.parser.minimum().get();

    self
      .parser
      .parser
      .parse(inp, &mut self.container, |nums, inp, span| {
        if min > nums {
          inp
            .emitter()
            .emit_too_few(TooFew::of(span.clone(), nums, min))?;
        }
        Ok(())
      })
  }
}
