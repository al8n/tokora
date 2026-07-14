use super::*;

/// A trait bound for creating emitter errors from too many elements errors.
pub trait FromTooManyError<'a, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from a too many elements error.
  fn from_too_many(err: TooMany<L::Span, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, L, Lang: ?Sized> FromTooManyError<'a, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<TooMany<L::Span, Lang>>,
{
  #[inline(always)]
  fn from_too_many(err: TooMany<L::Span, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}

/// An emitter that handles too many elements error for repeated parsers.
pub trait TooManyEmitter<'a, L, Lang: ?Sized = ()>: Emitter<'a, L, Lang> {
  /// Emits an error indicating that too many elements were found.
  fn emit_too_many(&mut self, err: TooMany<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;
}

impl<'a, L, Lang: ?Sized, U> TooManyEmitter<'a, L, Lang> for &mut U
where
  U: TooManyEmitter<'a, L, Lang>,
{
  #[inline(always)]
  fn emit_too_many(&mut self, err: TooMany<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_too_many(err)
  }
}
