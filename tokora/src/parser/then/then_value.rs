use super::*;

/// A parser that sequences a parser with a fixed value.
///
/// This combinator runs the first parser, then returns a fixed value
/// regardless of the first parser's output. Useful for cases where
/// you want to parse some input but always yield the same result.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ThenValue<F, T, O, U, L, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  parser: F,
  value: T,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _cmpl: PhantomData<Cmpl>,
}

impl<F, O, T, U, L, Ctx, Lang: ?Sized, Cmpl> ThenValue<F, T, O, U, L, Ctx, Lang, Cmpl> {
  /// Creates a new `ThenValue` combinator.
  #[inline(always)]
  pub(crate) const fn new(parser: F, value: T) -> Self {
    Self {
      parser,
      value,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _o: PhantomData,
      _u: PhantomData,
      _cmpl: PhantomData,
    }
  }
}

impl<'inp, F, T, L, O, U, Ctx, Lang, Cmpl> ParseInput<'inp, L, U, Ctx, Lang, Cmpl>
  for ThenValue<F, T, O, U, L, Ctx, Lang, Cmpl>
where
  F: ParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
  T: FnMut() -> U,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  #[inline(always)]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self.parser.parse_input(input).map(|_| (self.value)())
  }
}

impl<'inp, F, T, L, O, U, Ctx, Lang, Cmpl> TryParseInput<'inp, L, U, Ctx, Lang, Cmpl>
  for ThenValue<F, T, O, U, L, Ctx, Lang, Cmpl>
where
  F: TryParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
  T: FnMut() -> U,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  #[inline(always)]
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  ) -> Result<ParseAttempt<U>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self
      .parser
      .try_parse_input(input)
      .map(|val| val.map(|_| (self.value)()))
  }
}
