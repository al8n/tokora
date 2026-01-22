use crate::utils::CowStr;

use super::*;

impl<'inp, L, E, Lang: ?Sized> UnexpectedTrailingSeparatorEmitter<'inp, L, Lang> for Fatal<E, Lang>
where
  E: FromUnexpectedTrailingSeparatorError<'inp, L, Lang>,
  Fatal<E, Lang>: SeparatedEmitter<'inp, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    name: CowStr,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unexpected_trailing_separator(name, err))
  }
}
