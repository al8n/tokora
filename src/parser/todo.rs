use super::*;

/// A parser that is not yet implemented.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Todo<O: ?Sized>(PhantomData<O>);

impl<O: ?Sized> Todo<O> {
  /// Creates a parser that is not yet implemented.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self(PhantomData)
  }
}

impl<'inp, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for Todo<O>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    _inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    todo!()
  }
}
