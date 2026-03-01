use crate::error::{UnexpectedEoLhs, UnexpectedEoRhs};

use super::*;

impl<'a, L, E, Lang: ?Sized> PrattEmitter<'a, L, Lang> for Silent<E, Lang>
where
  L: Lexer<'a>,
  E: FromEmitterError<'a, L, Lang>
    + From<UnexpectedEoLhs<L::Offset, Lang>>
    + From<UnexpectedEoRhs<L::Offset, Lang>>,
  Silent<E, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_end_of_lhs(
    &mut self,
    _: UnexpectedEoLhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_end_of_rhs(
    &mut self,
    _: UnexpectedEoRhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }
}
