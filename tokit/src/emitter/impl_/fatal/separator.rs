use crate::{error::token::MissingTokenOf, utils::CowStr};

use super::*;

impl<'inp, L, E, Lang: ?Sized> SeparatedEmitter<'inp, L, Lang> for Fatal<E, Lang>
where
  E: FromSeparatedError<'inp, L, Lang>,
{
  #[inline(always)]
  fn emit_missing_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_missing_separator(name, err))
  }

  #[inline(always)]
  fn emit_missing_element(&mut self, err: MissingSyntaxOf<'inp, L, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_missing_element(err))
  }
}
