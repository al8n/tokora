use super::*;

/// a
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeekThenChoice<P, H, L, Ctx, W, Lang: ?Sized = ()> {
  parser: P,
  handler: H,
  _capacity: PhantomData<W>,
  _ctx: PhantomData<Ctx>,
  _l: PhantomData<L>,
  _lang: PhantomData<Lang>,
}

impl<P, H, L, Ctx, W: Window, Lang: ?Sized> PeekThenChoice<P, H, L, Ctx, W, Lang> {
  /// Creates a new `PeekThenChoice` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn of<'inp, O>(parser: P, condition: H) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseChoice<'inp, L, O, Ctx, Lang>,
    H: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<P::Id, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    Self {
      parser,
      handler: condition,
      _capacity: PhantomData,
      _ctx: PhantomData,
      _l: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Creates a new `PeekThenChoice` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn or_not_of<'inp, O>(parser: P, condition: H) -> OrNot<Self>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseChoice<'inp, L, O, Ctx, Lang>,
    H: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<Option<P::Id>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    OrNot::new(Self {
      parser,
      handler: condition,
      _capacity: PhantomData,
      _ctx: PhantomData,
      _l: PhantomData,
      _lang: PhantomData,
    })
  }
}

impl<'inp, P, H, L, O, Ctx, Lang, W: Window> ParseInput<'inp, L, O, Ctx, Lang>
  for PeekThenChoice<P, H, L, Ctx, W, Lang>
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
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let id = {
      let (output, emitter) = inp.sync_until_token_then_peek_with_emitter::<W>()?;
      (self.handler)(output, emitter)?
    };
    self.parser.parse_choice(inp, &id)
  }
}

impl<'inp, P, H, L, O, Ctx, Lang, W: Window> ParseInput<'inp, L, Option<O>, Ctx, Lang>
  for OrNot<PeekThenChoice<P, H, L, Ctx, W>>
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
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
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
