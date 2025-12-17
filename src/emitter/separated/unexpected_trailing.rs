use super::*;

/// An emitter that handles unexpected trailing separator.
pub trait UnexpectedTrailingSeparatorEmitter<'inp, O, Sep, L, Lang: ?Sized = ()>:
  SeparatedEmitter<'inp, O, Sep, L, Lang>
where
  L: Lexer<'inp>,
{
  /// Emits an error or warning for an unexpected trailing separator found during parsing.
  fn emit_unexpected_trailing_separator(
    &mut self,
    err: UnexpectedTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, O, Sep, L, Lang, U> UnexpectedTrailingSeparatorEmitter<'inp, O, Sep, L, Lang> for &mut U
where
  U: UnexpectedTrailingSeparatorEmitter<'inp, O, Sep, L, Lang>,
  L: Lexer<'inp>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    err: UnexpectedTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_unexpected_trailing_separator(err)
  }
}

/// A trait bound for creating emitter errors from unexpected trailing separator errors.
pub trait FromUnexpectedTrailingSeparatorError<'a, O: ?Sized, Sep, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from an unexpected trailing separator error.
  fn from_unexpected_trailing_separator(err: UnexpectedTrailingOf<'a, Sep, L, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, O: ?Sized, Sep, L, Lang: ?Sized>
  FromUnexpectedTrailingSeparatorError<'a, O, Sep, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<UnexpectedTrailingOf<'a, Sep, L, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_unexpected_trailing_separator(err: UnexpectedTrailingOf<'a, Sep, L, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}
