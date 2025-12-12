use super::*;

/// a
pub struct PeekThenChoice<P, H, T, W> {
  parser: P,
  handler: H,
  _token: PhantomData<T>,
  _capacity: PhantomData<W>,
}

impl<P, H, T, W: Window> PeekThenChoice<P, H, T, W> {
  /// Creates a new `PeekThenChoice` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new<'inp, L, O, Ctx>(parser: P, condition: H) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseChoice<'inp, L, O, Ctx, ()>,
    H: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<P::Id, <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self::of(parser, condition)
  }

  /// Creates a new `PeekThenChoice` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of<'inp, L, O, Ctx, Lang>(parser: P, condition: H) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseChoice<'inp, L, O, Ctx, Lang>,
    H: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<P::Id, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Lang: ?Sized,
  {
    Self {
      parser,
      handler: condition,
      _token: PhantomData,
      _capacity: PhantomData,
    }
  }

  /// Creates a new `PeekThenChoice` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn or_not<'inp, L, O, Ctx>(parser: P, condition: H) -> OrNot<Self>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseChoice<'inp, L, O, Ctx, ()>,
    H: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<Option<P::Id>, <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self::or_not_of(parser, condition)
  }

  /// Creates a new `PeekThenChoice` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn or_not_of<'inp, L, O, Ctx, Lang>(parser: P, condition: H) -> OrNot<Self>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseChoice<'inp, L, O, Ctx, Lang>,
    H: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<Option<P::Id>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
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

impl<'inp, P, H, L, O, Ctx, Lang, W: Window> ParseInput<'inp, L, O, Ctx, Lang>
  for PeekThenChoice<P, H, L::Token, W>
where
  P: ParseChoice<'inp, L, O, Ctx, Lang>,
  H: FnMut(
    Peeked<'_, 'inp, L, W>,
    &mut Ctx::Emitter,
  ) -> Result<P::Id, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let id = {
      let (output, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;
      (self.handler)(output, emitter)?
    };
    self.parser.parse_choice(inp, &id)
  }
}

impl<'inp, P, H, L, O, Ctx, Lang, W: Window> ParseInput<'inp, L, Option<O>, Ctx, Lang>
  for or_not::OrNot<PeekThenChoice<P, H, L::Token, W>>
where
  P: ParseChoice<'inp, L, O, Ctx, Lang>,
  H: FnMut(
    Peeked<'_, 'inp, L, W>,
    &mut Ctx::Emitter,
  ) -> Result<Option<P::Id>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<Option<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let id = {
      let (output, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;

      if output.is_empty() {
        return Ok(None);
      }

      (self.0.handler)(output, emitter)?
    };
    match id {
      Some(id) => self.0.parser.parse_choice(inp, &id).map(Some),
      None => Ok(None),
    }
  }
}

#[cfg(test)]
mod tests {
  use generic_arraydeque::typenum::U2;

  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_peek_then_choice_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(
      (Any::new(), Any::new())
        .peek_then_choice::<_, U2>(|_toks, _| Ok(deranged::RangedU8::<0, 1>::new(0).unwrap())),
    )
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_peek_then_choice_parse_impl();
  }
}
