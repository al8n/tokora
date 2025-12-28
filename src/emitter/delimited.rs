use super::*;

/// An emitter that emits delimiter errors
pub trait DelimitedEmitter<'a, Delim, L, Lang: ?Sized = ()>: Emitter<'a, L, Lang> {
  /// Emits an error indicating that there are unclosed.
  fn emit_unclosed(&mut self, err: Unclosed<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;

  /// Emits an error indicating that there are unopened.
  fn emit_unopened(&mut self, err: Unopened<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;

  /// Emits an error indicating that undelimited content was found.
  fn emit_undelimited(&mut self, err: Undelimited<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;
}

impl<'a, Delim, L, U, Lang: ?Sized> DelimitedEmitter<'a, Delim, L, Lang> for &mut U
where
  U: DelimitedEmitter<'a, Delim, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unclosed(&mut self, err: Unclosed<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_unclosed(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unopened(&mut self, err: Unopened<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_unopened(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_undelimited(&mut self, err: Undelimited<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_undelimited(err)
  }
}

/// A trait bound for converting delimiter errors into emitter errors.
pub trait FromDelimitedError<'a, Delim, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from an unclosed delimiter error.
  fn from_unclosed(err: Unclosed<Delim, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>;

  /// Creates an emitter error from an unopened delimiter error.
  fn from_unopened(err: Unopened<Delim, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>;

  /// Creates an emitter error from an undelimited content error.
  fn from_undelimited(err: Undelimited<Delim, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, Delim, L, Lang: ?Sized> FromDelimitedError<'a, Delim, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<Unclosed<Delim, L::Span, Lang>>
    + From<Unopened<Delim, L::Span, Lang>>
    + From<Undelimited<Delim, L::Span, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_unclosed(err: Unclosed<Delim, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_unopened(err: Unopened<Delim, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_undelimited(err: Undelimited<Delim, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}
