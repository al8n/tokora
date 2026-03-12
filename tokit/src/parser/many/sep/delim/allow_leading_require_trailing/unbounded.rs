use crate::emitter::MissingTrailingSeparatorEmitter;

use super::*;

impl<'inp, L, F, Sep, O, Delim, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    DelimitedBy<AllowLeading<RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang>>>, Delim>,
    Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
    + FullContainerEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  Delim: Delimiter<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Wrapper(
      self.as_mut().map_parser(|p| {
        p.map_parser_mut(|p| p.map_parser_mut(|p| p.map_parser_mut(|p| p.as_mut())))
      }),
    )
    .parse_input(inp)
    .map(|_| mem::take(&mut self.container))
  }
}

impl<'inp, L, F, Sep, O, Delim, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<
    Collect<
      DelimitedBy<AllowLeading<RequireTrailing<Separated<F, Sep, O, L, Ctx, Lang>>>, Delim>,
      Container,
      Ctx,
      Lang,
    >,
    PhantomSpan,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
    + FullContainerEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  Delim: Delimiter<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Wrapper(
      self.primary_mut().as_mut().map_parser(|p| {
        p.map_parser_mut(|p| p.map_parser_mut(|p| p.map_parser_mut(|p| p.as_mut())))
      }),
    )
    .parse_input(inp)
    .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
  }
}

impl<'inp, 'c, L, F, Sep, O, Delim, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut DelimitedBy<
      AllowLeading<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>,
      Delim,
    >,
    &'c mut Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
    + FullContainerEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  Delim: Delimiter<'inp, L, Lang>,
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
    let (delim, container) = self.parts_mut();
    let f = delim.parser.parser_mut().parser_mut().fn_mut();
    let parser = DelimitedBy::<_, Delim>::new(AllowLeading::new(RequireTrailing::new(
      Separated::new::<Sep>(&mut **f),
    )));

    Wrapper(Collect::new(parser, &mut **container)).parse_input(input)
  }
}

struct Wrapper<T>(T);

impl<'inp, 'c, L, F, Sep, O, Delim, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Wrapper<
    Collect<
      DelimitedBy<AllowLeading<RequireTrailing<Separated<&'c mut F, Sep, O, L, Ctx, Lang>>>, Delim>,
      &'c mut Container,
      Ctx,
      Lang,
    >,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  Sep: Punctuator<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
    + FullContainerEmitter<'inp, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  Delim: Delimiter<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    const UNBOUNDED: &AllowLeading<RequireTrailing<Unbounded>> =
      &AllowLeading::new(RequireTrailing::new(Unbounded));

    let (parser, container) = self.0.parts_mut();

    let f = parser.parser.parser_mut().parser_mut().fn_mut();

    DelimitedBy::<_, Delim>::new(Separated::new::<Sep>(&mut **f))
      .parse_separated(inp, container, UNBOUNDED, UNBOUNDED, UNBOUNDED)
  }
}
