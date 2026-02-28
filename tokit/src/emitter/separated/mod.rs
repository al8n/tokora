use crate::{
  error::{syntax::MissingSyntaxOf, token::MissingTokenOf},
  utils::CowStr,
};

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
pub trait SeparatedEmitter<'inp, L, Lang: ?Sized = ()>: Emitter<'inp, L, Lang> {
  /// Emits an error or warning for a missing separator found during parsing.
  fn emit_missing_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;

  /// Emits an error or warning for a missing separator found during parsing.
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, L, U, Lang> SeparatedEmitter<'inp, L, Lang> for &mut U
where
  U: SeparatedEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_separator(name, err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(&mut self, err: MissingSyntaxOf<'inp, L, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_element(err)
  }
}

/// A trait bound for converting separated-by emitter errors into emitter errors.
pub trait FromSeparatedError<'inp, L, Lang: ?Sized = ()>: FromEmitterError<'inp, L, Lang> {
  /// Creates an emitter error from a missing separator error.
  fn from_missing_separator(name: CowStr, err: MissingTokenOf<'inp, L, Lang>) -> Self
  where
    L: Lexer<'inp>;

  /// Creates an emitter error from a missing element error.
  fn from_missing_element(err: MissingSyntaxOf<'inp, L, Lang>) -> Self
  where
    L: Lexer<'inp>;
}
