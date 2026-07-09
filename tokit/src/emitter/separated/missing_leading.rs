use super::*;

/// An emitter that handles missing leading separator.
pub trait MissingLeadingSeparatorEmitter<'inp, L, Lang: ?Sized = ()>:
  SeparatedEmitter<'inp, L, Lang>
where
  L: Lexer<'inp>,
{
  /// Emits an error or warning for a missing a leading separator found during parsing.
  fn emit_missing_leading_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, L, Lang, U> MissingLeadingSeparatorEmitter<'inp, L, Lang> for &mut U
where
  U: MissingLeadingSeparatorEmitter<'inp, L, Lang>,
  L: Lexer<'inp>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_leading_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_leading_separator(name, err)
  }
}

/// A trait bound for creating emitter errors from missing leading separator errors.
pub trait FromMissingLeadingSeparatorError<'a, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from a missing leading separator error.
  fn from_missing_leading_separator(name: CowStr, err: MissingTokenOf<'a, L, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, L, Lang: ?Sized> FromMissingLeadingSeparatorError<'a, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<MissingTokenOf<'a, L, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_missing_leading_separator(_name: CowStr, err: MissingTokenOf<'a, L, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}
