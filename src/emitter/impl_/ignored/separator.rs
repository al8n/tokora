use super::*;

impl<'inp, L, Sep, Lang: ?Sized> SeparatedEmitter<'inp, Sep, L, Lang> for Ignored
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
  fn emit_missing_element(&mut self, _: MissingSyntaxOf<'inp, L, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}
