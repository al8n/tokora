use super::*;

/// A parser that sequences two parsers, keeping only the first result.
///
/// This combinator runs the first parser, then runs the second parser,
/// but only returns the first parser's result. Useful for parsing required
/// trailing tokens or syntax that you want to validate but don't need.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ThenIgnore<F, G, O, U, L, Ctx, Lang: ?Sized> {
  first: F,
  second: G,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, G, O, U, L, Ctx, Lang: ?Sized> ThenIgnore<F, G, O, U, L, Ctx, Lang> {
  /// Creates a new `ThenIgnore` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
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

impl<'inp, F, G, L, O, U, Ctx, Lang: ?Sized> ParseInput<'inp, L, O, Ctx, Lang>
  for ThenIgnore<F, G, O, U, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  G: ParseInput<'inp, L, U, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let first_result = self.first.parse_input(input)?;
    self.second.parse_input(input).map(|_| first_result)
  }
}
