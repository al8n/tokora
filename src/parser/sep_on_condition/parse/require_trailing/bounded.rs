use crate::emitter::{
  MissingTrailingSeparatorEmitter, TooFewEmitter, TooManyEmitter,
  UnexpectedLeadingSeparatorEmitter,
};

use super::*;

impl<'inp, L, F, SepClassifier, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    RequireTrailing<Bounded<SeparatedOnCondition<F, SepClassifier, Condition, O, W, L, Ctx, Lang>>>,
    Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooFewEmitter<'inp, O, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
  W: Window,
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

impl<'inp, L, F, SepClassifier, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<
    Collect<
      RequireTrailing<
        Bounded<SeparatedOnCondition<F, SepClassifier, Condition, O, W, L, Ctx, Lang>>,
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
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooFewEmitter<'inp, O, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
  W: Window,
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

impl<'inp, 'c, L, F, SepClassifier, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut RequireTrailing<
      Bounded<
        SeparatedOnCondition<&'c mut F, &'c mut SepClassifier, Condition, O, W, L, Ctx, Lang>,
      >,
    >,
    &'c mut Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooFewEmitter<'inp, O, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L>,
  W: Window,
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
        RequireTrailing {
          parser:
            Bounded {
              parser: SeparatedOnCondition {
                f, sep, condition, ..
              },
              maximum,
              minimum,
            },
        },
      container,
      ..
    } = self;
    let parser = RequireTrailing::new(Bounded::new(
      SeparatedOnCondition {
        f: &mut **f,
        sep: &mut **sep,
        condition: &mut *condition,
        _m: PhantomData,
        _decision_window: PhantomData,
        _ctx: PhantomData,
        _l: PhantomData,
        _lang: PhantomData,
      },
      maximum.get(),
      minimum.get(),
    ));

    Wrapper(Collect::new(parser, &mut *container)).parse_input(input)
  }
}

struct Wrapper<T>(T);

impl<'inp, 'c, L, F, SepClassifier, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Wrapper<
    Collect<
      RequireTrailing<
        Bounded<
          SeparatedOnCondition<
            &'c mut F,
            &'c mut SepClassifier,
            &'c mut Condition,
            O,
            W,
            L,
            Ctx,
            Lang,
          >,
        >,
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
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooFewEmitter<'inp, O, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L>,
  W: Window,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let Collect {
      parser, container, ..
    } = &mut self.0;

    let limitation = RequireTrailing::new(parser.parser.to_with());

    parser
      .parser_mut()
      .parser_mut()
      .parse(inp, container, &limitation, &limitation, &limitation)
  }
}
