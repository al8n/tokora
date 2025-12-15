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
// :
//   BatchEmitter<'inp, L, UnexpectedLeadingOf<'inp, Sep, L, Lang>, Lang>
//   + BatchEmitter<'inp, L, UnexpectedTrailingOf<'inp, Sep, L, Lang>, Lang>
//   + BatchEmitter<'inp, L, UnexpectedRepeatedOf<'inp, Sep, L, Lang>, Lang>
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

  /// Emits an error or warning for a repeated separators found during parsing.
  ///
  /// The `span` covers all the repeated separators.
  fn emit_unexpected_repeated_separator(
    &mut self,
    err: UnexpectedRepeatedOf<'inp, Sep, L, Lang>,
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
  fn emit_unexpected_repeated_separator(
    &mut self,
    err: UnexpectedRepeatedOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_unexpected_repeated_separator(err)
  }
}

/// A trait bound for converting separated-by emitter errors into emitter errors.
pub trait FromSeparatedError<'inp, O, Sep, L, Lang: ?Sized = ()>:
  FromEmitterError<'inp, L, Lang>
// + From<UnexpectedLeadingOf<'inp, Sep, L, Lang>>
// + From<UnexpectedTrailingOf<'inp, Sep, L, Lang>>
// + From<UnexpectedRepeatedOf<'inp, Sep, L, Lang>>
where
  L: Lexer<'inp>,
{
  /// Creates an emitter error from a missing separator error.
  fn from_missing_separator(err: MissingSeparatorOf<'inp, Sep, L, Lang>) -> Self
  where
    L: Lexer<'inp>;

  /// Creates an emitter error from an unexpected repeated separator error.
  fn from_unexpected_repeated_separator(err: UnexpectedRepeatedOf<'inp, Sep, L, Lang>) -> Self
  where
    L: Lexer<'inp>;
}

impl<'inp, T, O, Sep, L, Lang: ?Sized> FromSeparatedError<'inp, O, Sep, L, Lang> for T
where
  L: Lexer<'inp>,
  T: From<MissingSeparatorOf<'inp, Sep, L, Lang>>
    // + From<MissingSyntaxOf<'inp, O, L, Lang>>
    // + From<MissingLeadingOf<'inp, Sep, L, Lang>>
    // + From<MissingTrailingOf<'inp, Sep, L, Lang>>
    + From<UnexpectedRepeatedOf<'inp, Sep, L, Lang>>
    // + From<UnexpectedLeadingOf<'inp, Sep, L, Lang>>
    // + From<UnexpectedTrailingOf<'inp, Sep, L, Lang>>
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
  fn from_unexpected_repeated_separator(err: UnexpectedRepeatedOf<'inp, Sep, L, Lang>) -> Self
  where
    L: Lexer<'inp>,
  {
    err.into()
  }
}
