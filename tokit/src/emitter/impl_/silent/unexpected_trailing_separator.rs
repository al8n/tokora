use crate::utils::CowStr;

use super::*;

impl<'a, L, E, Lang: ?Sized> UnexpectedTrailingSeparatorEmitter<'a, L, Lang> for Silent<E, Lang> {
  #[inline(always)]
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'a, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }
}
