use crate::{error::token::MissingTokenOf, utils::CowStr};

use super::*;

impl<'inp, L, Lang: ?Sized> SeparatedEmitter<'inp, L, Lang> for Ignored {
  #[inline(always)]
  fn emit_missing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[inline(always)]
  fn emit_missing_element(&mut self, _: MissingSyntaxOf<'inp, L, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}
