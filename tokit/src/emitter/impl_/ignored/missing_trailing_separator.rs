use crate::{error::token::MissingTokenOf, utils::CowStr};

use super::*;

impl<'inp, L, Lang: ?Sized> MissingTrailingSeparatorEmitter<'inp, L, Lang> for Ignored
where
  L: Lexer<'inp>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_trailing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}
