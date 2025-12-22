use super::*;

impl<'inp, L, O: ?Sized, E, Sep: ?Sized, Lang: ?Sized> UnexpectedLeadingSeparatorEmitter<'inp, O, Sep, L, Lang>
  for Silent<E, Lang>
where
  L: Lexer<'inp>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    _: UnexpectedLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}
