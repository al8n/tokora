use super::*;

/// A trait bound for creating emitter errors from too few elements errors.
pub trait FromTooFewError<'a, O: ?Sized, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from a too few elements error.
  fn from_too_few(err: TooFew<O, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, O: ?Sized, L, Lang: ?Sized> FromTooFewError<'a, O, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<TooFew<O, L::Span, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_too_few(err: TooFew<O, <L>::Span, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}

/// An emitter that handles too few elements error for repeated parsers.
pub trait TooFewEmitter<'a, O: ?Sized, L, Lang: ?Sized = ()>: Emitter<'a, L, Lang> {
  /// Emits an error indicating that too few elements were found.
  fn emit_too_few(&mut self, err: TooFew<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;
}

impl<'a, O, L, Lang: ?Sized, U> TooFewEmitter<'a, O, L, Lang> for &mut U
where
  U: TooFewEmitter<'a, O, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, err: TooFew<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_too_few(err)
  }
}
