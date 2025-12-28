use super::*;

impl<'inp, L, O, Sep, E, Lang: ?Sized> UnexpectedLeadingSeparatorEmitter<'inp, O, Sep, L, Lang>
  for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: FromUnexpectedLeadingSeparatorError<'inp, O, Sep, L, Lang>,
  Fatal<E, Lang>: SeparatedEmitter<'inp, O, Sep, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    err: UnexpectedLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unexpected_leading_separator(err))
  }
}
