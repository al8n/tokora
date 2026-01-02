use super::*;

impl<'inp, L, F, SepClassifier, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    AllowLeading<AllowTrailing<Separated<F, SepClassifier, O, L, Ctx, Lang>>>,
    Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>,
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
        .map_parser(|p| p.map_parser_mut(|p| p.map_parser_mut(|p| p.as_mut()))),
    )
    .parse_input(inp)
    .map(|_| mem::take(&mut self.container))
  }
}

impl<'inp, L, F, SepClassifier, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<
    Collect<
      AllowLeading<AllowTrailing<Separated<F, SepClassifier, O, L, Ctx, Lang>>>,
      Container,
      Ctx,
      Lang,
    >,
    PhantomSpan,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>,
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
        .map_parser(|p| p.map_parser_mut(|p| p.map_parser_mut(|p| p.as_mut()))),
    )
    .parse_input(inp)
    .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
  }
}

impl<'inp, 'c, L, F, SepClassifier, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut AllowLeading<AllowTrailing<Separated<F, SepClassifier, O, L, Ctx, Lang>>>,
    &'c mut Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L>,
{
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let Self {
      parser:
        AllowLeading {
          parser: AllowTrailing {
            parser: Separated { f, sep, .. },
          },
        },
      container,
      ..
    } = self;

    let parser = AllowLeading::new(AllowTrailing::new(Separated {
      f: &mut *f,
      sep: &mut *sep,
      _m: PhantomData,
      _ctx: PhantomData,
      _l: PhantomData,
      _lang: PhantomData,
    }));

    Wrapper(Collect::new(parser, container)).parse_input(input)
  }
}

struct Wrapper<T>(T);

impl<'inp, 'c, L, F, SepClassifier, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Wrapper<
    Collect<
      AllowLeading<AllowTrailing<Separated<&'c mut F, &'c mut SepClassifier, O, L, Ctx, Lang>>>,
      &'c mut Container,
      Ctx,
      Lang,
    >,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L>,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    const HANDLER: &AllowLeading<AllowTrailing<Unbounded>> =
      &AllowLeading::new(AllowTrailing::new(Unbounded));
    let Collect {
      parser, container, ..
    } = &mut self.0;

    parser
      .parser_mut()
      .parser_mut()
      .parse(inp, container, HANDLER, HANDLER, HANDLER)
  }
}
