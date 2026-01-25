use crate::{error::token::UnexpectedTokenOf, utils::CowStr};

use super::*;

/// An emitter that handles unexpected leading separator.
pub trait UnexpectedLeadingSeparatorEmitter<'inp, L, Lang: ?Sized = ()>:
  SeparatedEmitter<'inp, L, Lang>
{
  /// Emits an error or warning for an unexpected leading separator found during parsing.
  fn emit_unexpected_leading_separator(
    &mut self,
    name: CowStr,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, L, Lang, U> UnexpectedLeadingSeparatorEmitter<'inp, L, Lang> for &mut U
where
  U: UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>,
  L: Lexer<'inp>,
  Lang: ?Sized,
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
    (**self).emit_unexpected_leading_separator(name, err)
  }
}

/// A trait bound for creating emitter errors from unexpected leading separator errors.
pub trait FromUnexpectedLeadingSeparatorError<'a, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from an unexpected leading separator error.
  fn from_unexpected_leading_separator(name: CowStr, err: UnexpectedTokenOf<'a, L, Lang>) -> Self
  where
    L: Lexer<'a>;
}

// impl<'a, T, L, Lang: ?Sized> FromUnexpectedLeadingSeparatorError<'a, L, Lang> for T
// where
//   L: Lexer<'a>,
//   T: From<UnexpectedTokenOf<'a, L, Lang>>,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn from_unexpected_leading_separator(err: UnexpectedTokenOf<'a, L, Lang>) -> Self
//   where
//     L: Lexer<'a>,
//   {
//     err.into()
//   }
// }
