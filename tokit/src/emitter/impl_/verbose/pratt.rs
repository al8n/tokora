use crate::error::{UnexpectedEoLhs, UnexpectedEoRhs};

use super::*;

impl<'a, L, S, E, Lang: ?Sized> PrattEmitter<'a, L, Lang> for Verbose<E, S, Lang>
where
  L: Lexer<'a, Span = S, Offset = S::Offset>,
  E: FromPrattError<'a, L, Lang>,
  S: Span + Ord + Clone,
  Verbose<E, S, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_end_of_lhs(
    &mut self,
    err: UnexpectedEoLhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(E::from_unexpected_end_of_lhs(err))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_end_of_rhs(
    &mut self,
    err: UnexpectedEoRhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(E::from_unexpected_end_of_rhs(err))
  }
}
