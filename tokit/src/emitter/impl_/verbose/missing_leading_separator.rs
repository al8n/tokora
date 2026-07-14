use crate::{error::token::MissingTokenOf, utils::CowStr};

use super::*;

impl<'inp, L, S, E, Lang: ?Sized> MissingLeadingSeparatorEmitter<'inp, L, Lang>
  for Verbose<E, S, Lang>
where
  L: Lexer<'inp, Span = S, Offset = S::Offset>,
  E: FromMissingLeadingSeparatorError<'inp, L, Lang>,
  Verbose<E, S, Lang>: SeparatedEmitter<'inp, L, Lang, Error = E>,
  S: Span + Ord + Clone,
{
  #[inline(always)]
  fn emit_missing_leading_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    let off = err.offset_ref().clone();
    self
      .errs
      .entry(S::new(off.clone(), off))
      .or_default()
      .push(E::from_missing_leading_separator(name, err));
    Ok(())
  }
}
