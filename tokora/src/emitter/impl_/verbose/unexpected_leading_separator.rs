use crate::utils::CowStr;

use super::*;

impl<'inp, L, S, E, Lang: ?Sized> UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
  for Verbose<E, S, Lang>
where
  L: Lexer<'inp, Span = S, Offset = S::Offset>,
  E: FromUnexpectedLeadingSeparatorError<'inp, L, Lang>,
  S: Span + Ord + Clone,
  Verbose<E, S, Lang>: SeparatedEmitter<'inp, L, Lang, Error = E>,
{
  #[inline(always)]
  fn emit_unexpected_leading_separator(
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
      .entry(span)
      .or_default()
      .push(E::from_unexpected_leading_separator(name, err));
    Ok(())
  }
}
