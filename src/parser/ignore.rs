use super::*;

/// A parser that ignores any output.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ignore<P, O> {
  parser: P,
  _output: PhantomData<O>,
}

impl<P, O> Ignore<P, O> {
  /// Creates a parser that ignores any output.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(parser: P) -> Self {
    Self {
      parser,
      _output: PhantomData,
    }
  }
}

impl<'inp, P, L, O, Ctx, Lang> ParseInput<'inp, L, (), Ctx, Lang> for Ignore<P, O>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.parser.parse_input(inp).map(|_| ())
  }
}
