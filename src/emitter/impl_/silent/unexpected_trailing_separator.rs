use super::*;

impl<'a, O: ?Sized, Sep: ?Sized, L, E, Lang: ?Sized>
  UnexpectedTrailingSeparatorEmitter<'a, O, Sep, L, Lang> for Silent<E, Lang>
where
  L: Lexer<'a>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: UnexpectedTrailingOf<'a, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }
}
