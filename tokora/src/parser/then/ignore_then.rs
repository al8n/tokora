use super::*;

/// A parser that sequences two parsers, keeping only the second result.
///
/// This combinator runs the first parser, discards its result, then runs
/// the second parser and returns its result. Useful for skipping over
/// expected tokens or syntax.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IgnoreThen<F, G, O, U, L, Ctx, Lang: ?Sized, Cmpl = Complete> {
  first: F,
  second: G,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _cmpl: PhantomData<Cmpl>,
}

impl<F, G, O, U, L, Ctx, Lang: ?Sized, Cmpl> IgnoreThen<F, G, O, U, L, Ctx, Lang, Cmpl> {
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
      _cmpl: PhantomData,
    }
  }
}

impl<'inp, F, G, L, O, U, Ctx, Lang: ?Sized, Cmpl> ParseInput<'inp, L, U, Ctx, Lang, Cmpl>
  for IgnoreThen<F, G, O, U, L, Ctx, Lang, Cmpl>
where
  F: ParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
  G: ParseInput<'inp, L, U, Ctx, Lang, Cmpl>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  #[inline(always)]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let _ = self.first.parse_input(input)?;
    self.second.parse_input(input)
  }
}
