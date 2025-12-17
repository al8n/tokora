use crate::{
  Lexer,
  error::{
    Unclosed, Undelimited, Unopened,
    syntax::{FullContainer, TooFew, TooMany},
    token::{
      MissingLeadingOf, MissingSeparatorOf, MissingTrailingOf, UnexpectedLeadingOf,
      UnexpectedRepeatedOf, UnexpectedToken, UnexpectedTrailingOf,
    },
  },
  lexer::Cursor,
  utils::{Message, Spanned},
};

use super::Token;

pub use delimited::*;
pub use impl_::*;
pub use repeated::*;
pub use separated::*;

mod delimited;
mod impl_;
mod repeated;
mod separated;

/// A trait for handling and emitting errors during tokenization and parsing.
///
/// `Emitter` provides a unified interface for error handling in the tokenization pipeline.
/// Implementations can decide whether errors are fatal (stop processing) or non-fatal
/// (logged and processing continues). This is particularly useful when you want to collect
/// multiple errors before stopping, or when implementing error recovery.
///
/// # Error Handling Strategy
///
/// The emitter uses a `Result`-based approach where:
/// - `Ok(())` means the error was handled as non-fatal and processing should continue
/// - `Err(error)` means the error is fatal and processing should stop immediately
///
/// # Use Cases
///
/// - **Error Collection**: Accumulate multiple errors before reporting them all at once
/// - **Error Recovery**: Log errors but continue parsing to find more issues
/// - **Fail-Fast**: Stop on the first error by always returning `Err`
/// - **Filtering**: Only treat certain error types as fatal
///
/// # Example
///
/// ```ignore
/// struct MyEmitter {
///     errors: Vec<String>,
///     max_errors: usize,
/// }
///
/// impl<'a, T: Token<'a>> Emitter<'a, T> for MyEmitter {
///     type Error = String;
///
///     fn emit_token_error(&mut self, err: Spanned<...>) -> Result<(), Self::Error> {
///         self.errors.push(format!("Lexer error at {:?}", err.span));
///         if self.errors.len() >= self.max_errors {
///             Err("Too many errors".to_string())
///         } else {
///             Ok(())
///         }
///     }
///
///     fn emit_error(&mut self, err: Spanned<Self::Error>) -> Result<(), Self::Error> {
///         self.errors.push(err.data);
///         if self.errors.len() >= self.max_errors {
///             Err("Too many errors".to_string())
///         } else {
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait Emitter<'a, L, Lang: ?Sized = ()> {
  /// The error type that this emitter produces.
  ///
  /// This is the type returned when a fatal error occurs (via `Err(Self::Error)`).
  /// It can be any type that represents your application's error model.
  type Error;

  /// Emits a lexer error from the underlying Logos tokenizer.
  ///
  /// This method is called when Logos encounters an error during lexing (e.g.,
  /// invalid input that doesn't match any token pattern). The implementation
  /// decides whether to treat it as fatal or non-fatal.
  ///
  /// # Parameters
  ///
  /// - `err`: The lexer error wrapped with its source span
  ///
  /// # Returns
  ///
  /// - `Ok(())` if the error should be treated as non-fatal (processing continues)
  /// - `Err(Self::Error)` if the error is fatal (processing stops immediately)
  fn emit_lexer_error(
    &mut self,
    err: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;

  /// Emits an unexpected token error encountered during parsing.
  fn emit_unexpected_token(
    &mut self,
    err: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;

  /// Emits a custom error from the application or parser.
  ///
  /// This method is called for application-level errors (not lexer errors).
  /// Like `emit_token_error`, the implementation decides whether the error
  /// is fatal or should be logged and processing continued.
  ///
  /// # Parameters
  ///
  /// - `err`: The application error wrapped with its source span
  ///
  /// # Returns
  ///
  /// - `Ok(())` if the error should be treated as non-fatal (processing continues)
  /// - `Err(Self::Error)` if the error is fatal (processing stops immediately)
  fn emit_error(&mut self, err: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;

  /// Rewinds the emitter state to the specified cursor.
  fn rewind(&mut self, cursor: &Cursor<'a, '_, L>)
  where
    L: Lexer<'a>;
}

impl<'a, L, U, Lang: ?Sized> Emitter<'a, L, Lang> for &mut U
where
  U: Emitter<'a, L, Lang>,
{
  type Error = U::Error;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    err: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_lexer_error(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_token(
    &mut self,
    err: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_unexpected_token(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, err: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_error(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, cursor: &Cursor<'a, '_, L>)
  where
    L: Lexer<'a>,
  {
    (**self).rewind(cursor)
  }
}

/// A trait bound for generic emitter error conversion.
pub trait FromEmitterError<'a, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from a lexer error.
  fn from_lexer_error(err: Spanned<<L::Token as Token<'a>>::Error, L::Span>) -> Self
  where
    L: Lexer<'a>;

  /// Creates an emitter error from an unexpected token error.
  fn from_unexpected_token(
    err: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>,
  ) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, L, Lang: ?Sized> FromEmitterError<'a, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<<L::Token as Token<'a>>::Error>
    + From<UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_lexer_error(err: Spanned<<L::Token as Token<'a>>::Error, L::Span>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into_data().into()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_unexpected_token(
    err: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>,
  ) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}

/// An emitter that supports batching of errors for more efficient reporting.
pub trait BatchEmitter<'a, L, Error, Lang: ?Sized = ()>: Emitter<'a, L, Lang> {
  /// Creates a new empty batch for collecting errors, returning its ID.
  ///
  /// The given `span` represents the starting span of the batch, and `description`
  /// provides a message describing the batch.
  fn create_batch(&mut self, span: L::Span, description: Message)
  where
    L: Lexer<'a>;

  /// Creates a new batch for collecting errors with an initial error.
  ///
  /// If the initial error is kind of fatal error, it returns an `Err`.
  fn create_batch_with_error(
    &mut self,
    description: Message,
    err: Spanned<Error, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;

  /// Emits an single error into the specified batch.
  ///
  /// If this error can trigger a fatal condition, the emitter can return an `Err`.
  fn emit_to_batch(
    &mut self,
    id: &L::Span,
    err: Spanned<Error, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;

  /// Emits all errors collected in the specified batch.
  ///
  /// If the batch does not exist or is empty, this method does nothing.
  ///
  /// If emitting the batch triggers a fatal condition, the emitter can return an `Err`.
  fn emit_batch(&mut self, id: &L::Span) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;

  /// Drops the specified batch without emitting its errors.
  ///
  /// This can be used to discard non-fatal errors that are replaced by other errors.
  fn drop_batch(&mut self, id: &L::Span)
  where
    L: Lexer<'a>;
}

impl<'a, L, Error, U, Lang: ?Sized> BatchEmitter<'a, L, Error, Lang> for &mut U
where
  U: BatchEmitter<'a, L, Error, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch(&mut self, span: L::Span, description: Message)
  where
    L: Lexer<'a>,
  {
    (**self).create_batch(span, description)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch_with_error(
    &mut self,
    description: Message,
    err: Spanned<Error, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).create_batch_with_error(description, err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_to_batch(&mut self, id: &L::Span, err: Spanned<Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_to_batch(id, err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_batch(&mut self, id: &L::Span) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_batch(id)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn drop_batch(&mut self, id: &L::Span)
  where
    L: Lexer<'a>,
  {
    (**self).drop_batch(id)
  }
}
