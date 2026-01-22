use crate::emitter::{
  MissingTrailingSeparatorEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
};

use super::*;

impl<'inp, L, F, SepClassifier, Condition, O, Delim, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    DelimitedBy<
      RequireTrailing<AtMost<SeparatedWhile<F, SepClassifier, Condition, O, W, L, Ctx, Lang>>>,
      Delim,
    >,
    Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Punctuator<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, SepClassifier, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  W: Window,
  Delim: DelimiterSelector<'inp, L, Lang>,
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

impl<'inp, L, F, SepClassifier, Condition, O, Delim, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<
    Collect<
      DelimitedBy<
        RequireTrailing<AtMost<SeparatedWhile<F, SepClassifier, Condition, O, W, L, Ctx, Lang>>>,
        Delim,
      >,
      Container,
      Ctx,
      Lang,
    >,
    PhantomSpan,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Punctuator<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, SepClassifier, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  W: Window,
  Delim: DelimiterSelector<'inp, L, Lang>,
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

impl<'inp, 'c, L, F, SepClassifier, Condition, O, Delim, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut DelimitedBy<
      RequireTrailing<
        AtMost<SeparatedWhile<&'c mut F, SepClassifier, Condition, O, W, L, Ctx, Lang>>,
      >,
      Delim,
    >,
    &'c mut Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Punctuator<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, SepClassifier, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  W: Window,
  Delim: DelimiterSelector<'inp, L, Lang>,
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
      parser:
        DelimitedBy {
          parser:
            RequireTrailing {
              parser:
                AtMost {
                  parser: SeparatedWhile { f, condition, .. },
                  maximum,
                },
            },
          ..
        },
      container,
      ..
    } = self;
    let parser = DelimitedBy::<_, Delim>::new_in(RequireTrailing::new(AtMost::new(
      SeparatedWhile::new(&mut **f, &mut *condition),
      maximum.get(),
    )));

    Wrapper(Collect::new(parser, &mut *container)).parse_input(input)
  }
}

struct Wrapper<T>(T);

impl<'inp, 'c, L, F, SepClassifier, Condition, O, Delim, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Wrapper<
    Collect<
      DelimitedBy<
        RequireTrailing<
          AtMost<SeparatedWhile<&'c mut F, SepClassifier, &'c mut Condition, O, W, L, Ctx, Lang>>,
        >,
        Delim,
      >,
      &'c mut Container,
      Ctx,
      Lang,
    >,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Punctuator<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, SepClassifier, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  W: Window,
  Delim: DelimiterSelector<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let Collect {
      parser, container, ..
    } = &mut self.0;

    let maximum = RequireTrailing::new(parser.parser.parser.maximum());

    let DelimitedBy {
      parser: AtMost {
        parser: SeparatedWhile { f, condition, .. },
        ..
      },
      ..
    } = parser.map_parser_mut(|p| p.parser_mut());

    DelimitedBy::<_, Delim>::new_in(SeparatedWhile::new(&mut **f, &mut **condition))
      .parse_separated(inp, container, &maximum, &maximum, &maximum)
  }
}
