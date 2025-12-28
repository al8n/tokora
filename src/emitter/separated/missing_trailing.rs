use super::*;

/// An emitter that handles missing trailing separator.
pub trait MissingTrailingSeparatorEmitter<'inp, O, Sep, L, Lang: ?Sized = ()>:
  SeparatedEmitter<'inp, O, Sep, L, Lang>
where
  L: Lexer<'inp>,
{
  /// Emits an error or warning for a missing a trailing separator found during parsing.
  fn emit_missing_trailing_separator(
    &mut self,
    err: MissingTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, O, Sep, L, Lang, U> MissingTrailingSeparatorEmitter<'inp, O, Sep, L, Lang> for &mut U
where
  U: MissingTrailingSeparatorEmitter<'inp, O, Sep, L, Lang>,
  L: Lexer<'inp>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_trailing_separator(
    &mut self,
    err: MissingTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_trailing_separator(err)
  }
}

/// A trait bound for creating emitter errors from missing trailing separator errors.
pub trait FromMissingTrailingSeparatorError<'a, O: ?Sized, Sep, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from a missing trailing separator error.
  fn from_missing_trailing_separator(err: MissingTrailingOf<'a, Sep, L, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, O: ?Sized, Sep, L, Lang: ?Sized> FromMissingTrailingSeparatorError<'a, O, Sep, L, Lang>
  for T
where
  L: Lexer<'a>,
  T: From<MissingTrailingOf<'a, Sep, L, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_missing_trailing_separator(err: MissingTrailingOf<'a, Sep, L, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}
