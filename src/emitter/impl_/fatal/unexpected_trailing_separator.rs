use super::*;

impl<'inp, L, Sep, E, Lang: ?Sized> UnexpectedTrailingSeparatorEmitter<'inp, Sep, L, Lang>
  for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: FromUnexpectedTrailingSeparatorError<'inp, Sep, L, Lang>,
  Fatal<E, Lang>: SeparatedEmitter<'inp, Sep, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    err: UnexpectedTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unexpected_trailing_separator(err))
  }
}
