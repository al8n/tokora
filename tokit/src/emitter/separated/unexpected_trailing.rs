use crate::{
  error::token::{SeparatedError, SeparatedErrorOf, UnexpectedTokenOf},
  utils::CowStr,
};

use super::*;

/// An emitter that handles unexpected trailing separator.
pub trait UnexpectedTrailingSeparatorEmitter<'inp, L, Lang: ?Sized = ()>:
  SeparatedEmitter<'inp, L, Lang>
{
  /// Emits an error or warning for an unexpected trailing separator found during parsing.
  fn emit_unexpected_trailing_separator(
    &mut self,
    name: CowStr,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, L, Lang, U> UnexpectedTrailingSeparatorEmitter<'inp, L, Lang> for &mut U
where
  U: UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>,
  L: Lexer<'inp>,
  Lang: ?Sized,
{
  #[inline(always)]
  fn emit_unexpected_trailing_separator(
    &mut self,
    name: CowStr,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_unexpected_trailing_separator(name, err)
  }
}

/// A trait bound for creating emitter errors from unexpected trailing separator errors.
pub trait FromUnexpectedTrailingSeparatorError<'a, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from an unexpected trailing separator error.
  fn from_unexpected_trailing_separator(name: CowStr, err: UnexpectedTokenOf<'a, L, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, L, Lang: ?Sized> FromUnexpectedTrailingSeparatorError<'a, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<SeparatedErrorOf<'a, L, Lang>>,
{
  #[inline(always)]
  fn from_unexpected_trailing_separator(_name: CowStr, err: UnexpectedTokenOf<'a, L, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    SeparatedError::trailing(err).into()
  }
}
