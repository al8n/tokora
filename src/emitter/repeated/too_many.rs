use super::*;

/// A trait bound for creating emitter errors from too many elements errors.
pub trait FromTooManyError<'a, O: ?Sized, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from a too many elements error.
  fn from_too_many(err: TooMany<O, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, O: ?Sized, L, Lang: ?Sized> FromTooManyError<'a, O, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<TooMany<O, L::Span, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_too_many(err: TooMany<O, <L>::Span, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}

/// An emitter that handles too many elements error for repeated parsers.
pub trait TooManyEmitter<'a, O: ?Sized, L, Lang: ?Sized = ()>: Emitter<'a, L, Lang> {
  /// Emits an error indicating that too many elements were found.
  fn emit_too_many(&mut self, err: TooMany<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;
}

impl<'a, O, L, Lang: ?Sized, U> TooManyEmitter<'a, O, L, Lang> for &mut U
where
  U: TooManyEmitter<'a, O, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_many(&mut self, err: TooMany<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_too_many(err)
  }
}
