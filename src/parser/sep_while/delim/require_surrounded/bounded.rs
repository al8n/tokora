use crate::emitter::{
  MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter, TooFewEmitter, TooManyEmitter,
};

use super::*;

impl<'inp, L, F, SepClassifier, Condition, O, Open, Close, Delim, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    DelimitedBy<
      RequireLeading<
        RequireTrailing<Bounded<SeparatedWhile<F, SepClassifier, Condition, O, W, L, Ctx, Lang>>>,
      >,
      Open,
      Close,
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
  SepClassifier: Check<L::Token>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + DelimitedEmitter<'inp, Delim, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>
    + TooFewEmitter<'inp, O, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  W: Window,
  Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Delim: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Wrapper(self.as_mut().map_parser(|p| {
      p.map_parser_mut(|p| {
        p.map_parser_mut(|p| p.map_parser_mut(|p| p.map_parser_mut(|p| p.as_mut())))
      })
    }))
    .parse_input(inp)
    .map(|_| mem::take(&mut self.container))
  }
}

impl<'inp, L, F, SepClassifier, Condition, O, Open, Close, Delim, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<
    Collect<
      DelimitedBy<
        RequireLeading<
          RequireTrailing<Bounded<SeparatedWhile<F, SepClassifier, Condition, O, W, L, Ctx, Lang>>>,
        >,
        Open,
        Close,
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
  SepClassifier: Check<L::Token>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + DelimitedEmitter<'inp, Delim, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>
    + TooFewEmitter<'inp, O, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  W: Window,
  Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Delim: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<Container, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Wrapper(self.primary_mut().as_mut().map_parser(|p| {
      p.map_parser_mut(|p| {
        p.map_parser_mut(|p| p.map_parser_mut(|p| p.map_parser_mut(|p| p.as_mut())))
      })
    }))
    .parse_input(inp)
    .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
  }
}

impl<
  'inp,
  'c,
  L,
  F,
  SepClassifier,
  Condition,
  O,
  Open,
  Close,
  Delim,
  Container,
  Ctx,
  Lang: ?Sized,
  W,
> ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut DelimitedBy<
      RequireLeading<
        RequireTrailing<
          Bounded<SeparatedWhile<&'c mut F, &'c mut SepClassifier, Condition, O, W, L, Ctx, Lang>>,
        >,
      >,
      Open,
      Close,
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
  SepClassifier: Check<L::Token>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + DelimitedEmitter<'inp, Delim, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>
    + TooFewEmitter<'inp, O, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  W: Window,
  Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Delim: Clone,
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
            RequireLeading {
              parser:
                RequireTrailing {
                  parser:
                    Bounded {
                      parser:
                        SeparatedWhile {
                          f, sep, condition, ..
                        },
                      maximum,
                      minimum,
                    },
                },
            },
          left_classifier,
          right_classifier,
          delimiter,
        },
      container,
      ..
    } = self;
    let parser = DelimitedBy::new_in(
      RequireLeading::new(RequireTrailing::new(Bounded::new(
        SeparatedWhile::new(&mut **f, &mut **sep, &mut *condition),
        maximum.get(),
        minimum.get(),
      ))),
      &*left_classifier,
      &*right_classifier,
      &*delimiter,
    );

    Wrapper(Collect::new(parser, &mut *container)).parse_input(input)
  }
}

struct Wrapper<T>(T);

impl<
  'inp,
  'c,
  L,
  F,
  SepClassifier,
  Condition,
  O,
  Open,
  Close,
  Delim,
  Container,
  Ctx,
  Lang: ?Sized,
  W,
> ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Wrapper<
    Collect<
      DelimitedBy<
        RequireLeading<
          RequireTrailing<
            Bounded<
              SeparatedWhile<
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
        >,
        &'c Open,
        &'c Close,
        &'c Delim,
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
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + DelimitedEmitter<'inp, Delim, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>
    + TooFewEmitter<'inp, O, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L> + DelimiterHandler<'inp, L>,
  W: Window,
  Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Delim: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let Collect {
      parser, container, ..
    } = &mut self.0;

    let limitation =
      RequireLeading::new(RequireTrailing::new(parser.parser.parser.parser.to_with()));

    let DelimitedBy {
      parser:
        RequireTrailing {
          parser:
            Bounded {
              parser: SeparatedWhile {
                f, sep, condition, ..
              },
              ..
            },
        },
      left_classifier,
      right_classifier,
      delimiter,
    } = parser.map_parser_mut(|p| p.parser_mut());

    DelimitedBy::new_in(
      SeparatedWhile::new(&mut **f, &mut **sep, &mut **condition),
      *left_classifier,
      *right_classifier,
      *delimiter,
    )
    .parse_separated(inp, container, &limitation, &limitation, &limitation)
  }
}
