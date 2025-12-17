use super::*;

/// An emitter that handles unexpected leading separator.
pub trait UnexpectedLeadingSeparatorEmitter<'inp, O, Sep, L, Lang: ?Sized = ()>:
  SeparatedEmitter<'inp, O, Sep, L, Lang>
where
  L: Lexer<'inp>,
{
  /// Emits an error or warning for an unexpected leading separator found during parsing.
  fn emit_unexpected_leading_separator(
    &mut self,
    err: UnexpectedLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, O, Sep, L, Lang, U> UnexpectedLeadingSeparatorEmitter<'inp, O, Sep, L, Lang> for &mut U
where
  U: UnexpectedLeadingSeparatorEmitter<'inp, O, Sep, L, Lang>,
  L: Lexer<'inp>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    err: UnexpectedLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_unexpected_leading_separator(err)
  }
}

/// A trait bound for creating emitter errors from unexpected leading separator errors.
pub trait FromUnexpectedLeadingSeparatorError<'a, O: ?Sized, Sep, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from an unexpected leading separator error.
  fn from_unexpected_leading_separator(err: UnexpectedLeadingOf<'a, Sep, L, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, O: ?Sized, Sep, L, Lang: ?Sized>
  FromUnexpectedLeadingSeparatorError<'a, O, Sep, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<UnexpectedLeadingOf<'a, Sep, L, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_unexpected_leading_separator(err: UnexpectedLeadingOf<'a, Sep, L, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}
