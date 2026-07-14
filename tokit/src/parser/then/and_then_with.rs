use super::*;

/// A parser that sequentially composes two parsers.
///
/// This combinator runs the first parser, then uses its output to determine
/// the second parser to run. This enables context-dependent parsing where
/// the result of one parser influences what comes next.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AndThenWith<F, T, O, U, L, Ctx, Lang: ?Sized = ()> {
  parser: F,
  then: T,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, O, T, U, L, Ctx, Lang: ?Sized> AndThenWith<F, T, O, U, L, Ctx, Lang> {
  /// Creates a new `AndThen` combinator.
  #[inline(always)]
  pub(crate) const fn new(parser: F, then: T) -> Self {
    Self {
      parser,
      then,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _o: PhantomData,
      _u: PhantomData,
    }
  }
}

impl<'inp, F, T, L, O, U, Ctx, Lang> ParseInput<'inp, L, U, Ctx, Lang>
  for AndThenWith<F, T, O, U, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  T: FnMut(
    O,
    ParseState<'_, 'inp, '_, L, Ctx, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[inline(always)]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let start = input.cursor().clone();
    self
      .parser
      .parse_input(input)
      .and_then(|output| (self.then)(output, ParseState::new(input, start)))
  }
}

impl<'inp, F, T, L, O, U, Ctx, Lang> TryParseInput<'inp, L, U, Ctx, Lang>
  for AndThenWith<F, T, O, U, L, Ctx, Lang>
where
  F: TryParseInput<'inp, L, O, Ctx, Lang>,
  T: FnMut(
    O,
    ParseState<'_, 'inp, '_, L, Ctx, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[inline(always)]
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<ParseAttempt<U>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let start = input.cursor().clone();
    self
      .parser
      .try_parse_input(input)
      .and_then(|val| val.and_then(|output| (self.then)(output, ParseState::new(input, start))))
  }
}
