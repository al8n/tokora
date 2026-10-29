use crate::error::{UnexpectedEoLhs, UnexpectedEoRhs};

use super::*;

impl<'a, L, Lang: ?Sized> PrattEmitter<'a, L, Lang> for Ignored
where
  L: Lexer<'a>,
{
  #[inline(always)]
  fn emit_unexpected_end_of_lhs(
    &mut self,
    _: UnexpectedEoLhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[inline(always)]
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
