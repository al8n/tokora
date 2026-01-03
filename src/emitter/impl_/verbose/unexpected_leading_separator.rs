use super::*;

impl<'inp, L, Sep, S, E, Lang: ?Sized> UnexpectedLeadingSeparatorEmitter<'inp, Sep, L, Lang>
  for Verbose<E, S, Lang>
where
  L: Lexer<'inp, Span = S, Offset = S::Offset>,
  E: FromUnexpectedLeadingSeparatorError<'inp, Sep, L, Lang>,
  S: Span + Ord + Clone,
  Verbose<E, S, Lang>: SeparatedEmitter<'inp, Sep, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    err: UnexpectedLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    let span = err.span_ref().clone();
    self
      .errs
      .insert(span, E::from_unexpected_leading_separator(err));
    Ok(())
  }
}
