use super::*;

/// A parser that sequences two parsers, keeping only the second result.
///
/// This combinator runs the first parser, discards its result, then runs
/// the second parser and returns its result. Useful for skipping over
/// expected tokens or syntax.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IgnoreThen<F, G, O, U, L, Ctx, Lang: ?Sized> {
  first: F,
  second: G,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, G, O, U, L, Ctx, Lang: ?Sized> IgnoreThen<F, G, O, U, L, Ctx, Lang> {
  /// Creates a new `IgnoreThen` combinator.
  #[inline(always)]
  pub(crate) const fn new(first: F, second: G) -> Self {
    Self {
      first,
      second,
      _o: PhantomData,
      _u: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<'inp, F, G, L, O, U, Ctx, Lang: ?Sized> ParseInput<'inp, L, U, Ctx, Lang>
  for IgnoreThen<F, G, O, U, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  G: ParseInput<'inp, L, U, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[inline(always)]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let _ = self.first.parse_input(input)?;
    self.second.parse_input(input)
  }
}
