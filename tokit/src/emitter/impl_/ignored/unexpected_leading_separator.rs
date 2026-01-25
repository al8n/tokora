use crate::utils::CowStr;

use super::*;

impl<'inp, L, Lang: ?Sized> UnexpectedLeadingSeparatorEmitter<'inp, L, Lang> for Ignored {
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
