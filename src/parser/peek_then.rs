use super::*;

/// a
pub struct PeekThen<P, D, T, Window> {
  parser: P,
  handler: D,
  _token: PhantomData<T>,
  _capacity: PhantomData<Window>,
}

impl<P, D, T, Window> Apply<OrNot<Self>> for PeekThen<P, D, T, Window> {
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, _: Self::Options) -> OrNot<Self> {
    OrNot::new(self)
  }
}

impl<P, D, T, W: Window> PeekThen<P, D, T, W> {
  /// Creates a new `PeekThen` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new<'inp, L, O, Ctx>(parser: P, condition: D) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseInput<'inp, L, O, Ctx, ()>,
    D: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self::of(parser, condition)
  }

  /// Creates a new `PeekThen` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of<'inp, L, O, Ctx, Lang>(parser: P, condition: D) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    D: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Lang: ?Sized,
  {
    Self {
      parser,
      handler: condition,
      _token: PhantomData,
      _capacity: PhantomData,
    }
  }

  /// Creates a new `OrNot<PeekThen>` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn or_not<'inp, L, O, Ctx>(parser: P, condition: D) -> OrNot<Self>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseInput<'inp, L, O, Ctx, ()>,
    D: Decision<'inp, L, Ctx::Emitter, W, ()>,
  {
    Self::or_not_of(parser, condition)
  }

  /// Creates a new `OrNot<PeekThen>` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn or_not_of<'inp, L, O, Ctx, Lang>(parser: P, condition: D) -> OrNot<Self>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    D: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    Lang: ?Sized,
  {
    OrNot::new(Self {
      parser,
      handler: condition,
      _token: PhantomData,
      _capacity: PhantomData,
    })
  }
}

impl<'inp, P, D, L, O, Ctx, Lang, W> ParseInput<'inp, L, O, Ctx, Lang>
  for PeekThen<P, D, L::Token, W>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  D: FnMut(
    Peeked<'_, 'inp, L, W>,
    &mut Ctx::Emitter,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  W: Window,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let (output, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;
    (self.handler)(output, emitter).and_then(|_| self.parser.parse_input(inp))
  }
}

impl<'inp, P, D, L, O, Ctx, Lang, W> ParseInput<'inp, L, Option<O>, Ctx, Lang>
  for OrNot<PeekThen<P, D, L::Token, W>>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  D: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  W: Window,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<Option<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let (output, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

    if output.is_empty() {
      return Ok(None);
    }

    self
      .0
      .handler
      .decide(output, emitter)
      .and_then(|val| match val {
        Action::Continue => self.0.parser.parse_input(inp).map(Some),
        Action::Stop => Ok(None),
      })
  }
}

#[cfg(test)]
mod tests {
  use generic_arraydeque::typenum::U2;

  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_peek_then_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Option<DummyToken>, ()> {
    use crate::emitter::Fatal;
    Parser::new().apply(Any::new().peek_then_or_not::<_, U2>(
      |_toks: Peeked<'_, '_, DummyLexer, U2>, _: &mut Fatal<()>| Ok(Action::Continue),
    ))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_peek_then_parse_impl();
  }
}
