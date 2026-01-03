use super::*;

/// An emitter that handles missing leading separator.
pub trait MissingLeadingSeparatorEmitter<'inp, Sep, L, Lang: ?Sized = ()>:
  SeparatedEmitter<'inp, Sep, L, Lang>
where
  L: Lexer<'inp>,
{
  /// Emits an error or warning for a missing a leading separator found during parsing.
  fn emit_missing_leading_separator(
    &mut self,
    err: MissingLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, Sep, L, Lang, U> MissingLeadingSeparatorEmitter<'inp, Sep, L, Lang> for &mut U
where
  U: MissingLeadingSeparatorEmitter<'inp, Sep, L, Lang>,
  L: Lexer<'inp>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_leading_separator(
    &mut self,
    err: MissingLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_leading_separator(err)
  }
}

/// A trait bound for creating emitter errors from missing leading separator errors.
pub trait FromMissingLeadingSeparatorError<'a, Sep, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from a missing leading separator error.
  fn from_missing_leading_separator(err: MissingLeadingOf<'a, Sep, L, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, Sep, L, Lang: ?Sized> FromMissingLeadingSeparatorError<'a, Sep, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<MissingLeadingOf<'a, Sep, L, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_missing_leading_separator(err: MissingLeadingOf<'a, Sep, L, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}
