use crate::utils::CowStr;

use super::*;

impl<'inp, L, E, Lang: ?Sized> UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>
  for Silent<E, Lang>
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}
