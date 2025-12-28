use super::*;

impl<'inp, L, O: ?Sized, Sep: ?Sized, E, Lang: ?Sized> SeparatedEmitter<'inp, O, Sep, L, Lang>
  for Silent<E, Lang>
where
  L: Lexer<'inp>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    _: MissingSeparatorOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    _: MissingSyntaxOf<'inp, O, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}
