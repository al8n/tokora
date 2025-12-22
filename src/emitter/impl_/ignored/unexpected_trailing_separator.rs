use super::*;

impl<'inp, L, O, Sep, Lang: ?Sized> UnexpectedTrailingSeparatorEmitter<'inp, O, Sep, L, Lang>
  for Ignored
where
  L: Lexer<'inp>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: UnexpectedTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}
