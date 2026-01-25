use crate::utils::CowStr;

use super::*;

impl<'inp, L, S, E, Lang: ?Sized> UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>
  for Verbose<E, S, Lang>
where
  L: Lexer<'inp, Span = S, Offset = S::Offset>,
  E: FromUnexpectedTrailingSeparatorError<'inp, L, Lang>,
  Verbose<E, S, Lang>: SeparatedEmitter<'inp, L, Lang, Error = E>,
  S: Span + Ord + Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    name: CowStr,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    let span = err.span_ref().clone();
    self
      .errs
      .insert(span, E::from_unexpected_trailing_separator(name, err));
    Ok(())
  }
}
