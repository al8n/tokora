use crate::error::syntax::MissingSyntaxOf;

use super::*;

pub use missing_leading::*;
pub use missing_trailing::*;
pub use unexpected_leading::*;
pub use unexpected_trailing::*;

mod missing_leading;
mod missing_trailing;
mod unexpected_leading;
mod unexpected_trailing;

/// An emitter that handles missing separator or repeated separators found during parsing.
pub trait SeparatedEmitter<'inp, O, Sep, L, Lang: ?Sized = ()>: Emitter<'inp, L, Lang>
where
  L: Lexer<'inp>,
{
  /// Emits an error or warning for a missing separator found during parsing.
  fn emit_missing_separator(
    &mut self,
    err: MissingSeparatorOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;

  /// Emits an error or warning for a missing separator found during parsing.
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, O, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, O, L, Sep, U, Lang: ?Sized> SeparatedEmitter<'inp, O, Sep, L, Lang> for &mut U
where
  L: Lexer<'inp>,
  U: SeparatedEmitter<'inp, O, Sep, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    err: MissingSeparatorOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_separator(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, O, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_element(err)
  }
}

/// A trait bound for converting separated-by emitter errors into emitter errors.
pub trait FromSeparatedError<'inp, O, Sep, L, Lang: ?Sized = ()>:
  FromEmitterError<'inp, L, Lang>
where
  L: Lexer<'inp>,
{
  /// Creates an emitter error from a missing separator error.
  fn from_missing_separator(err: MissingSeparatorOf<'inp, Sep, L, Lang>) -> Self
  where
    L: Lexer<'inp>;

  /// Creates an emitter error from a missing element error.
  fn from_missing_element(err: MissingSyntaxOf<'inp, O, L, Lang>) -> Self
  where
    L: Lexer<'inp>;
}

impl<'inp, T, O, Sep, L, Lang: ?Sized> FromSeparatedError<'inp, O, Sep, L, Lang> for T
where
  L: Lexer<'inp>,
  T: From<MissingSeparatorOf<'inp, Sep, L, Lang>>
    + From<MissingSyntaxOf<'inp, O, L, Lang>>
    + FromEmitterError<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_missing_separator(err: MissingSeparatorOf<'inp, Sep, L, Lang>) -> Self
  where
    L: Lexer<'inp>,
  {
    err.into()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_missing_element(err: MissingSyntaxOf<'inp, O, L, Lang>) -> Self
  where
    L: Lexer<'inp>,
  {
    err.into()
  }
}
