use core::mem::MaybeUninit;

use super::*;

/// a
pub struct PeekThen<P, H, T, const N: usize> {
  parser: P,
  handler: H,
  _token: PhantomData<T>,
}

impl<P, H, T, const N: usize> Apply<OrNot<Self>> for PeekThen<P, H, T, N> {
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, _: Self::Options) -> OrNot<Self> {
    OrNot::new(self)
  }
}

impl<P, H, T, const N: usize> PeekThen<P, H, T, N> {
  /// Creates a new `PeekThen` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new<'inp, L, O, Ctx>(parser: P, condition: H) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseInput<'inp, L, O, Ctx, ()>,
    H: FnMut(
      &PeekBuf<'inp, '_, L>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self::of(parser, condition)
  }

  /// Creates a new `PeekThen` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of<'inp, L, O, Ctx, Lang>(parser: P, condition: H) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    H: FnMut(
      &PeekBuf<'inp, '_, L>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Lang: ?Sized,
  {
    Self {
      parser,
      handler: condition,
      _token: PhantomData,
    }
  }

  /// Creates a new `OrNot<PeekThen>` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn or_not<'inp, L, O, Ctx>(parser: P, condition: H) -> OrNot<Self>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseInput<'inp, L, O, Ctx, ()>,
    H: FnMut(
      &PeekBuf<'inp, '_, L>,
      &mut Ctx::Emitter,
    ) -> Result<bool, <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self::or_not_of(parser, condition)
  }

  /// Creates a new `OrNot<PeekThen>` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn or_not_of<'inp, L, O, Ctx, Lang>(parser: P, condition: H) -> OrNot<Self>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    H: FnMut(
      &PeekBuf<'inp, '_, L>,
      &mut Ctx::Emitter,
    ) -> Result<bool, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Lang: ?Sized,
  {
    OrNot::new(Self {
      parser,
      handler: condition,
      _token: PhantomData,
    })
  }
}

impl<'inp, P, H, L, O, Ctx, Lang, const N: usize> ParseInput<'inp, L, O, Ctx, Lang>
  for PeekThen<P, H, L::Token, N>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  H: FnMut(
    &PeekBuf<'inp, '_, L>,
    &mut Ctx::Emitter,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let mut buf = [const { MaybeUninit::uninit() }; N];
    let (output, emitter) = inp.peek_with_emitter(&mut buf);
    (self.handler)(output, emitter).and_then(|_| self.parser.parse_input(inp))
  }
}

impl<'inp, P, H, L, O, Ctx, Lang, const N: usize> ParseInput<'inp, L, Option<O>, Ctx, Lang>
  for or_not::OrNot<PeekThen<P, H, L::Token, N>>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  H: FnMut(
    &PeekBuf<'inp, '_, L>,
    &mut Ctx::Emitter,
  ) -> Result<bool, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, <Ctx>::Emitter, <Ctx>::Cache, Lang>,
  ) -> Result<Option<O>, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let mut buf = [const { MaybeUninit::uninit() }; N];
    let (output, emitter) = inp.peek_with_emitter(&mut buf);

    if output.is_empty() {
      return Ok(None);
    }

    (self.0.handler)(output, emitter).and_then(|val| {
      if !val {
        Ok(None)
      } else {
        self.0.parser.parse_input(inp).map(Some)
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::{DummyLexer, DummyToken};

  use super::*;

  fn assert_peek_then_parse_impl<'inp>()
  -> impl Parse<'inp, DummyLexer, Option<Spanned<DummyToken>>, ()> {
    Parser::new().apply(
      Any::new().peek_then_or_not::<_, 2>(|_toks: &PeekBuf<'inp, '_, DummyLexer>, _| Ok(true)),
    )
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_peek_then_parse_impl();
  }
}
