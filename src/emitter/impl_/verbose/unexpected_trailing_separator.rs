use super::*;

impl<'inp, L, O, S, Sep, E, Lang: ?Sized> UnexpectedTrailingSeparatorEmitter<'inp, O, Sep, L, Lang>
  for Verbose<E, S, Lang>
where
  L: Lexer<'inp, Span = S, Offset = S::Offset>,
  E: FromUnexpectedTrailingSeparatorError<'inp, O, Sep, L, Lang>,
  Verbose<E, S, Lang>: SeparatedEmitter<'inp, O, Sep, L, Lang, Error = E>,
  S: Span + Ord + Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    err: UnexpectedTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    let span = err.span_ref().clone();
    self
      .errs
      .insert(span, E::from_unexpected_trailing_separator(err));
    Ok(())
  }
}
