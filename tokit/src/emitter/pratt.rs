use crate::error::{UnexpectedEoLhs, UnexpectedEoRhs};

use super::*;

/// An emitter that handles pratt related errors.
pub trait PrattEmitter<'inp, L, Lang: ?Sized = ()>: Emitter<'inp, L, Lang> {
  /// Emits an error or warning for an unexpected end of left hand side error while parsing pratt expression.
  fn emit_unexpected_end_of_lhs(
    &mut self,
    err: UnexpectedEoLhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;

  /// Emits an error or warning for an unexpected end of right hand side error while parsing pratt expression.
  fn emit_unexpected_end_of_rhs(
    &mut self,
    err: UnexpectedEoRhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}
