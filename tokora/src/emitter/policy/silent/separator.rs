use crate::{error::token::MissingTokenOf, utils::CowStr};

use super::*;

impl<'inp, L, E, Lang: ?Sized> SeparatedEmitter<'inp, L, Lang> for Silent<E, Lang> {
  #[inline(always)]
  fn emit_missing_separator(
    &mut self,
    _name: CowStr,
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
