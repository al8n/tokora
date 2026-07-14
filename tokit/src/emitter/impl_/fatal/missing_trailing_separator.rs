use crate::{error::token::MissingTokenOf, utils::CowStr};

use super::*;

impl<'inp, L, E, Lang: ?Sized> MissingTrailingSeparatorEmitter<'inp, L, Lang> for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: FromMissingTrailingSeparatorError<'inp, L, Lang>,
  Fatal<E, Lang>: SeparatedEmitter<'inp, L, Lang, Error = E>,
{
  #[inline(always)]
  fn emit_missing_trailing_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_missing_trailing_separator(name, err))
  }
}
