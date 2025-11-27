use crate::{
  Lexer,
  error::{
    UnclosedParen,
    syntax::{MissingSyntaxOf, TooFew, TooMany},
    token::{
      MissingLeadingOf, MissingTokenOf, MissingTrailingOf, UnexpectedLeadingOf,
      UnexpectedRepeatedOf, UnexpectedTrailingOf,
    },
  },
  utils::{Message, Spanned},
};

use super::Token;

mod blackhole;
mod noop;

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
pub trait Emitter<'a, L> {
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
  ) -> Result<(), Spanned<Self::Error, L::Span>>
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
  fn emit_error(
    &mut self,
    err: Spanned<Self::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>;
}

impl<'a, L, U> Emitter<'a, L> for &mut U
where
  U: Emitter<'a, L>,
{
  type Error = U::Error;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    err: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    (**self).emit_lexer_error(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(
    &mut self,
    err: Spanned<Self::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    (**self).emit_error(err)
  }
}

/// An emitter that supports batching of errors for more efficient reporting.
pub trait BatchEmitter<'a, L, Error>: Emitter<'a, L> {
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
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>;

  /// Emits an single error into the specified batch.
  ///
  /// If this error can trigger a fatal condition, the emitter can return an `Err`.
  fn emit_to_batch(
    &mut self,
    id: &L::Span,
    err: Spanned<Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>;

  /// Emits all errors collected in the specified batch.
  ///
  /// If the batch does not exist or is empty, this method does nothing.
  ///
  /// If emitting the batch triggers a fatal condition, the emitter can return an `Err`.
  fn emit_batch(&mut self, id: &L::Span) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>;

  /// Drops the specified batch without emitting its errors.
  ///
  /// This can be used to discard non-fatal errors that are replaced by other errors.
  fn drop_batch(&mut self, id: &L::Span)
  where
    L: Lexer<'a>;
}

impl<'a, L, Error, U> BatchEmitter<'a, L, Error> for &mut U
where
  U: BatchEmitter<'a, L, Error>,
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
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    (**self).create_batch_with_error(description, err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_to_batch(
    &mut self,
    id: &L::Span,
    err: Spanned<Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    (**self).emit_to_batch(id, err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_batch(&mut self, id: &L::Span) -> Result<(), Spanned<Self::Error, L::Span>>
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

/// An emitter that emits unclosed parenthesis errors.
pub trait UnclosedEmitter<'a, L>: Emitter<'a, L> {
  /// Emits an error indicating that there are unclosed parentheses.
  fn emit_unclosed(&mut self, err: UnclosedParen) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>;
}

/// An emitter that handles errors related to repeated elements during parsing.
pub trait RepeatedEmitter<'a, O, L>: Emitter<'a, L> {
  /// Emits an error indicating that too few elements were found.
  fn emit_too_few(&mut self, err: TooFew<O, L::Span>) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>;

  /// Emits an error indicating that too many elements were found.
  fn emit_too_many(
    &mut self,
    err: TooMany<O, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>;
}

impl<'a, O, L, U> RepeatedEmitter<'a, O, L> for &mut U
where
  U: RepeatedEmitter<'a, O, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, err: TooFew<O, L::Span>) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    (**self).emit_too_few(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_many(&mut self, err: TooMany<O, L::Span>) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    (**self).emit_too_many(err)
  }
}

/// An emitter that handles missing separator or repeated separators found during parsing.
pub trait SeparatedByEmitter<'inp, O, Sep, L>:
  RepeatedEmitter<'inp, O, L>
  + BatchEmitter<'inp, L, UnexpectedLeadingOf<'inp, Sep, L>>
  + BatchEmitter<'inp, L, UnexpectedTrailingOf<'inp, Sep, L>>
  + BatchEmitter<'inp, L, UnexpectedRepeatedOf<'inp, Sep, L>>
  + BatchEmitter<'inp, L, <L::Token as Token<'inp>>::Error>
where
  L: Lexer<'inp>,
{
  /// Emits an error or warning for a missing separator found during parsing.
  fn emit_missing_separator(
    &mut self,
    err: MissingTokenOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>;

  /// Emits an error or warning for a missing an element after a leading separator.
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, O, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>;

  /// Emits an error or warning for a missing a leading separator found during parsing.
  fn emit_missing_leading_separator(
    &mut self,
    err: MissingLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>;

  /// Emits an error or warning for a missing a trailing separator found during parsing.
  fn emit_missing_trailing_separator(
    &mut self,
    err: MissingTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>;

  /// Emits an error or warning for a repeated separators found during parsing.
  ///
  /// The `span` covers all the repeated separators.
  fn emit_unexpected_repeated_separator(
    &mut self,
    err: UnexpectedRepeatedOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>;

  /// Emits an error or warning for a leading separator(s) found during parsing.
  ///
  /// The `leadings` covers the leading separator(s).
  fn emit_unexpected_leading_separator(
    &mut self,
    err: UnexpectedLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>;

  /// Emits an error or warning for a trailing separator(s) found during parsing.
  ///
  /// The `trailings` covers the trailing separator(s).
  fn emit_unexpected_trailing_separator(
    &mut self,
    err: UnexpectedTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>;
}

impl<'inp, O, L, Sep, U> SeparatedByEmitter<'inp, O, Sep, L> for &mut U
where
  L: Lexer<'inp>,
  U: SeparatedByEmitter<'inp, O, Sep, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    err: MissingTokenOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_separator(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, O, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_element(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_leading_separator(
    &mut self,
    err: MissingLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_leading_separator(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_trailing_separator(
    &mut self,
    err: MissingTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_missing_trailing_separator(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_repeated_separator(
    &mut self,
    err: UnexpectedRepeatedOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_unexpected_repeated_separator(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    err: UnexpectedLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_unexpected_leading_separator(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    err: UnexpectedTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    (**self).emit_unexpected_trailing_separator(err)
  }
}
