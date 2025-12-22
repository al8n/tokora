use super::*;

impl<'inp, L, O, Sep, E, Lang: ?Sized> SeparatedEmitter<'inp, O, Sep, L, Lang> for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: FromSeparatedError<'inp, O, Sep, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    err: MissingSeparatorOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_missing_separator(err))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, O, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_missing_element(err))
  }
}
