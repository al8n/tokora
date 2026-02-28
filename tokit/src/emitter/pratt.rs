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

impl<'inp, L, U, Lang> PrattEmitter<'inp, L, Lang> for &mut U
where
  U: PrattEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_end_of_lhs(
    &mut self,
    err: UnexpectedEoLhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_unexpected_end_of_lhs(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_end_of_rhs(
    &mut self,
    err: UnexpectedEoRhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_unexpected_end_of_rhs(err)
  }
}

/// A trait bound for converting pratt emitter errors into emitter errors.
pub trait FromPrattError<'inp, L, Lang: ?Sized = ()>: FromEmitterError<'inp, L, Lang> {
  /// Creates an emitter error from an unexpected end of left hand side error.
  fn from_unexpected_end_of_lhs(err: UnexpectedEoLhs<L::Offset, Lang>) -> Self
  where
    L: Lexer<'inp>;

  /// Creates an emitter error from an unexpected end of right hand side error.
  fn from_unexpected_end_of_rhs(err: UnexpectedEoRhs<L::Offset, Lang>) -> Self
  where
    L: Lexer<'inp>;
}

impl<'inp, T, L, Lang: ?Sized> FromPrattError<'inp, L, Lang> for T
where
  L: Lexer<'inp>,
  T: FromEmitterError<'inp, L, Lang>
    + From<UnexpectedEoLhs<L::Offset, Lang>>
    + From<UnexpectedEoRhs<L::Offset, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_unexpected_end_of_lhs(err: UnexpectedEoLhs<L::Offset, Lang>) -> Self
  where
    L: Lexer<'inp>,
  {
    err.into()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_unexpected_end_of_rhs(err: UnexpectedEoRhs<L::Offset, Lang>) -> Self
  where
    L: Lexer<'inp>,
  {
    err.into()
  }
}
