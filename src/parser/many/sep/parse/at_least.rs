use crate::emitter::{
  TooFewEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};

use super::*;

impl<'inp, L, F, Sep, O, Container, Ctx, Lang: ?Sized> ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<AtLeast<Separated<F, Sep, O, L, Ctx, Lang>>, Container, Ctx, Lang>
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Wrapper(
      self
        .as_mut()
        .map_parser(|p| p.map_parser_mut(|p| p.as_mut())),
    )
    .parse_input(inp)
    .map(|_| mem::take(&mut self.container))
  }
}

impl<'inp, L, F, Sep, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<Collect<AtLeast<Separated<F, Sep, O, L, Ctx, Lang>>, Container, Ctx, Lang>, PhantomSpan>
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Wrapper(
      self
        .primary_mut()
        .as_mut()
        .map_parser(|p| p.map_parser_mut(|p| p.as_mut())),
    )
    .parse_input(inp)
    .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
  }
}

impl<'inp, 'c, L, F, Sep, O, Container, Ctx, Lang: ?Sized> ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>,
    &'c mut Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let Self {
      parser: AtLeast {
        parser: Separated { f, .. },
        minimum,
      },
      container,
      ..
    } = self;
    let parser = AtLeast::new(Separated::new::<Sep>(&mut **f), minimum.get());

    Wrapper(Collect::new(parser, &mut *container)).parse_input(input)
  }
}

struct Wrapper<T>(T);

impl<'inp, 'c, L, F, Sep, O, Container, Ctx, Lang: ?Sized> ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Wrapper<
    Collect<AtLeast<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>, &'c mut Container, Ctx, Lang>,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
    + UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let Collect {
      parser, container, ..
    } = &mut self.0;

    let minimum = parser.minimum();

    parser
      .parser_mut()
      .parse(inp, container, &minimum, &minimum, &minimum)
  }
}
