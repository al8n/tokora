use crate::{
  emitter::{MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter, TooManyEmitter},
  error::token::{MissingLeadingOf, MissingTrailingOf},
};

use super::*;

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  EndStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for RequireLeading<RequireTrailing<Maximum>>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, Sep, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, Sep, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, Sep, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.parser.parser.check(inp, ckp, num_elems)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_element_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let span = inp.span_since(ckp.cursor());
    inp
      .emitter()
      .emit_missing_trailing_separator(MissingTrailingOf::<'_, Sep, L, Lang>::of(span.end()))
      .and_then(|_| self.parser.parser.check(inp, ckp, num_elems))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_leading_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    spanned: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    inp
      .emitter()
      .emit_missing_element(MissingSyntaxOf::<'_, O, L, Lang>::of(
        spanned.span_ref().end(),
      ))
      .and_then(|_| self.parser.parser.check(inp, ckp, num_elems))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_separator_state(
    &self,
    num_elems: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    _: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.parser.parser.check(inp, ckp, num_elems)
  }
}

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  ContinueStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang>
  for RequireLeading<RequireTrailing<Maximum>>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter:
    SeparatedEmitter<'inp, O, Sep, L, Lang> + MissingLeadingSeparatorEmitter<'inp, O, Sep, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    off: L::Offset,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    inp
      .emitter()
      .emit_missing_leading_separator(MissingLeadingOf::<'_, Sep, L, Lang>::of(off))
  }
}

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  SeparatorStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang>
  for RequireLeading<RequireTrailing<Maximum>>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, Sep, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, Sep, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, Sep, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    _: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    _: &Spanned<L::Token, L::Span>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(())
  }
}

impl<'inp, L, F, SepClassifier, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    RequireLeading<RequireTrailing<AtMost<Separated<F, SepClassifier, O, L, Ctx, Lang>>>>,
    Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
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

impl<'inp, L, F, SepClassifier, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<
    Collect<
      RequireLeading<RequireTrailing<AtMost<Separated<F, SepClassifier, O, L, Ctx, Lang>>>>,
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
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: Default + ContainerT<O> + SeparatorHandler<'inp, L>,
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

impl<'inp, 'c, L, F, SepClassifier, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut RequireLeading<
      RequireTrailing<AtMost<Separated<&'c mut F, &'c mut SepClassifier, O, L, Ctx, Lang>>>,
    >,
    &'c mut Container,
    Ctx,
    Lang,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>,
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
      parser:
        RequireLeading {
          parser:
            RequireTrailing {
              parser:
                AtMost {
                  parser: Separated { f, sep, .. },
                  maximum,
                },
            },
        },
      container,
      ..
    } = self;
    let parser = RequireLeading::new(RequireTrailing::new(AtMost::new(
      Separated {
        f: &mut **f,
        sep: &mut **sep,
        _m: PhantomData,
        _ctx: PhantomData,
        _l: PhantomData,
        _lang: PhantomData,
      },
      maximum.get(),
    )));

    Wrapper(Collect::new(parser, &mut *container)).parse_input(input)
  }
}

struct Wrapper<T>(T);

impl<'inp, 'c, L, F, SepClassifier, O, Container, Ctx, Lang: ?Sized>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Wrapper<
    Collect<
      RequireLeading<
        RequireTrailing<AtMost<Separated<&'c mut F, &'c mut SepClassifier, O, L, Ctx, Lang>>>,
      >,
      &'c mut Container,
      Ctx,
      Lang,
    >,
  >
where
  L: Lexer<'inp>,
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  SepClassifier: Check<L::Token>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingTrailingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + MissingLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>,
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

    let limitation = RequireLeading::new(RequireTrailing::new(parser.parser.parser.maximum()));

    parser.parser_mut().parser_mut().parser_mut().parse(
      inp,
      container,
      &limitation,
      &limitation,
      &limitation,
    )
  }
}
