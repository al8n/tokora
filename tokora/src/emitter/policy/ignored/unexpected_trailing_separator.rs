use crate::utils::CowStr;

use super::*;

impl<'inp, L, Lang: ?Sized> UnexpectedTrailingSeparatorEmitter<'inp, L, Lang> for Ignored {
  #[inline(always)]
  fn emit_unexpected_trailing_separator(
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
