use crate::{emitter::UnexpectedLeadingSeparatorEmitter, error::token::UnexpectedLeadingOf};

use super::*;

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  EndStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for AllowTrailing<Unbounded>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, Sep, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, O, Sep, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    _: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(inp.span_since(ckp.cursor()))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_element_state(
    &self,
    _: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(inp.span_since(ckp.cursor()))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_leading_state(
    &self,
    _: usize,
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
        spanned.span_ref().start(),
      ))
      .map(|_| inp.span_since(ckp.cursor()))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_separator_state(
    &self,
    _: usize,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    _: Spanned<L::Token, L::Span>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(inp.span_since(ckp.cursor()))
  }
}

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  ContinueStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for AllowTrailing<Unbounded>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, Sep, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    _: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    _: L::Offset,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(())
  }
}

impl<'inp, 'closure, Sep, O, L, Ctx, Lang: ?Sized>
  SeparatorStateHandler<'inp, 'closure, Sep, O, L, Ctx, Lang> for AllowTrailing<Unbounded>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: SeparatedEmitter<'inp, O, Sep, L, Lang>
    + UnexpectedLeadingSeparatorEmitter<'inp, O, Sep, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn handle_start_state(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    sep_tok: &Spanned<L::Token, L::Span>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let (span, tok) = sep_tok.clone().into_components();
    inp.emitter().emit_unexpected_leading_separator(
      UnexpectedLeadingOf::<'_, Sep, L, Lang>::of(span).with_found(tok),
    )
  }
}

impl<'inp, L, F, SepClassifier, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    AllowTrailing<SeparatedOnCondition<F, SepClassifier, Condition, O, W, L, Ctx, Lang>>,
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
    + UnexpectedLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>,
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
        .map_parser(|p| p.map_parser_mut(|p| p.as_mut())),
    )
    .parse_input(inp)
    .map(|_| mem::take(&mut self.container))
  }
}

impl<'inp, L, F, SepClassifier, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, Spanned<Container, L::Span>, Ctx, Lang>
  for With<
    Collect<
      AllowTrailing<SeparatedOnCondition<F, SepClassifier, Condition, O, W, L, Ctx, Lang>>,
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
    + UnexpectedLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>,
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
        .map_parser(|p| p.map_parser_mut(|p| p.as_mut())),
    )
    .parse_input(inp)
    .map(|span| Spanned::new(span, mem::take(&mut self.primary.container)))
  }
}

impl<'inp, 'c, L, F, SepClassifier, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Collect<
    &'c mut AllowTrailing<SeparatedOnCondition<F, SepClassifier, Condition, O, W, L, Ctx, Lang>>,
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
    + UnexpectedLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L>,
  W: Window,
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
        AllowTrailing {
          parser: SeparatedOnCondition {
            f, sep, condition, ..
          },
        },
      container,
      ..
    } = self;

    let parser = AllowTrailing::new(SeparatedOnCondition {
      f: &mut *f,
      sep: &mut *sep,
      condition: &mut *condition,
      _m: PhantomData,
      _decision_window: PhantomData,
      _ctx: PhantomData,
      _l: PhantomData,
      _lang: PhantomData,
    });

    Wrapper(Collect::new(parser, container)).parse_input(input)
  }
}

struct Wrapper<T>(T);

impl<'inp, 'c, L, F, SepClassifier, Condition, O, Container, Ctx, Lang: ?Sized, W>
  ParseInput<'inp, L, L::Span, Ctx, Lang>
  for Wrapper<
    Collect<
      AllowTrailing<
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
    + UnexpectedLeadingSeparatorEmitter<'inp, O, SepClassifier, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Container: ContainerT<O> + SeparatorHandler<'inp, L>,
  W: Window,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    const HANDLER: &AllowTrailing<Unbounded> = &AllowTrailing::new(Unbounded);
    let Collect {
      parser, container, ..
    } = &mut self.0;

    parser
      .parser_mut()
      .parse(inp, container, HANDLER, HANDLER, HANDLER)
  }
}
