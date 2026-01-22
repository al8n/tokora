use crate::utils::CowStr;

use super::*;

impl<'inp, L, E, Lang: ?Sized> UnexpectedLeadingSeparatorEmitter<'inp, L, Lang> for Fatal<E, Lang>
where
  E: FromUnexpectedLeadingSeparatorError<'inp, L, Lang>,
  Fatal<E, Lang>: SeparatedEmitter<'inp, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    name: CowStr,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unexpected_leading_separator(name, err))
  }
}
