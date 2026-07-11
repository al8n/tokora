use crate::{
  Lexer,
  error::{
    syntax::{FullContainer, TooFew, TooMany},
    token::UnexpectedTokenOf,
  },
  input::Cursor,
  span::Spanned,
};

use super::Token;

pub use impl_::*;
pub use pratt::*;
pub use repeated::*;
pub use separated::*;
pub use severity::*;

mod impl_;
mod pratt;
mod repeated;
mod separated;
mod severity;

/// A trait for handling and emitting errors during tokenization and parsing.
///
/// `Emitter` provides a unified interface for error handling in the tokenization pipeline.
/// Implementations can decide whether errors are fatal (stop processing) or non-fatal
/// (logged and processing continues). This is particularly useful when you want to collect
/// multiple errors before stopping, or when implementing error recovery.
///
/// # Atomically Composable Trait Design
///
/// Tokit's emitter system uses an **atomically composable trait design**. Instead of one monolithic
/// emitter interface, error handling is broken down into small, focused traits, each responsible for
/// a specific parsing scenario:
///
/// - **Core**: [`Emitter`] - Base error handling (lexer errors, unexpected tokens)
/// - **Repetition**: [`TooFewEmitter`], [`TooManyEmitter`], [`FullContainerEmitter`]
/// - **Separation**: [`SeparatedEmitter`], [`UnexpectedLeadingSeparatorEmitter`], [`UnexpectedTrailingSeparatorEmitter`]
///
/// This atomic design provides:
/// - ✅ **Fine-grained control**: Implement only the traits you need for your use case
/// - ✅ **Composability**: Mix and match traits to build custom error handling strategies
/// - ✅ **Pre-built bundles**: [`Fatal`], [`Verbose`], and [`Silent`] implement all traits with consistent behavior
/// - ✅ **Extensibility**: Create specialized emitters by implementing a subset of traits
///
/// Tokit provides several complete implementations: [`Fatal`], [`Verbose`],
/// [`Silent`], and [`Ignored`](crate::utils::marker::Ignored). However, the atomic trait system
/// encourages you to create custom emitters tailored to your specific needs by implementing only the
/// traits relevant to your parser.
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
/// - **Custom Strategies**: Implement domain-specific error handling (e.g., max error limits, severity filtering, telemetry)
///
/// # Example: Custom Emitter with Error Limit
///
/// ```ignore
/// use tokit::emitter::{Emitter, TooFewEmitter, TooManyEmitter};
///
/// struct MaxErrorsEmitter {
///     errors: Vec<String>,
///     max_errors: usize,
/// }
///
/// // Implement the core Emitter trait
/// impl<'a, L> Emitter<'a, L> for MaxErrorsEmitter {
///     type Error = String;
///
///     fn emit_lexer_error(&mut self, err: Spanned<...>) -> Result<(), Self::Error> {
///         self.errors.push(format!("Lexer error: {:?}", err));
///         if self.errors.len() >= self.max_errors {
///             Err("Too many errors".to_string())
///         } else {
///             Ok(())
///         }
///     }
///     // ... other Emitter methods
/// }
///
/// // Optionally implement atomic traits for specific error scenarios
/// impl<'a, O, L> TooFewEmitter<'a, O, L> for MaxErrorsEmitter {
///     fn emit_too_few(&mut self, err: TooFew<...>) -> Result<(), Self::Error> {
///         self.errors.push(format!("Too few elements: {:?}", err));
///         if self.errors.len() >= self.max_errors {
///             Err("Too many errors".to_string())
///         } else {
///             Ok(())
///         }
///     }
/// }
/// // Implement other atomic traits as needed: TooManyEmitter, SeparatedEmitter, etc.
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
    err: UnexpectedTokenOf<'a, L, Lang>,
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

  /// Emits a warning — a diagnostic that, by contract, does not stop parsing.
  ///
  /// A warning carries the *same* payload as [`emit_error`](Self::emit_error) (a
  /// [`Spanned<Self::Error, _>`](Spanned)): a warning **is** a diagnostic, just one classified
  /// at the [`Severity::Warning`] tier rather than [`Severity::Error`]. This is an additive,
  /// second channel for future callers (e.g. a lossless collecting parse) — nothing in the
  /// existing emit paths reclassifies through it.
  ///
  /// Like the diagnostic-label capabilities, this is a method with a **blanket no-op default**:
  /// stateless emitters ([`Fatal`], [`Silent`], [`Ignored`](crate::utils::marker::Ignored))
  /// inherit the empty body — a fail-fast parse has no warning sink, so the warning is dropped
  /// and parsing continues (`Ok(())`). A collecting emitter like [`Verbose`] overrides this to
  /// record the warning into a channel parallel to its errors. The `Result` return is what lets
  /// a bespoke emitter escalate a warning to fatal if it wishes; the built-in emitters never do.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_warning(&mut self, warning: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    let _ = warning;
    Ok(())
  }

  /// Captures the emitter's current emission checkpoint for a later [`rewind`](Self::rewind).
  ///
  /// A checkpoint is a monotonically increasing emission mark: emitters that
  /// retain per-emission state (e.g. [`Verbose`]) return a value that grows with
  /// every recorded error, so a subsequent `rewind` can drop *exactly* the
  /// emissions made after this point. Stateless emitters ([`Fatal`], [`Silent`],
  /// [`Ignored`](crate::utils::marker::Ignored)) keep nothing to rewind and use
  /// the default `0`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn checkpoint(&self) -> u64 {
    0
  }

  /// Rewinds the emitter state to a previously captured [`checkpoint`](Self::checkpoint).
  ///
  /// `checkpoint` is the mark returned by [`checkpoint`](Self::checkpoint) at the
  /// save point; `cursor` is the restore offset. Emission-aware emitters
  /// ([`Verbose`]) drop every diagnostic recorded after `checkpoint` — precisely
  /// the emissions of the abandoned branch, regardless of their span — and ignore
  /// `cursor`. `cursor` is retained for emitters that key their own rollback on
  /// the source offset. Stateless emitters ignore both.
  fn rewind(&mut self, cursor: &Cursor<'a, '_, L>, checkpoint: u64)
  where
    L: Lexer<'a>;

  /// Pushes a diagnostic label onto the emitter's open-label stack, opening a
  /// *"while parsing X"* context for the duration of a [`labelled`](crate::labelled)
  /// sub-parse.
  ///
  /// This is an additive capability with a **blanket no-op default**: stateless
  /// emitters ([`Fatal`], [`Silent`], [`Ignored`](crate::utils::marker::Ignored))
  /// inherit the empty body, so a label pair around them costs nothing — the two
  /// calls inline away. A collecting emitter like [`Verbose`] overrides this to
  /// maintain the stack and snapshot it into every diagnostic it records.
  ///
  /// Labels are `&'static str` (parser names are static), so a push never allocates.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn enter_label(&mut self, label: &'static str) {
    let _ = label;
  }

  /// Pops the most recently [`enter_label`](Self::enter_label)ed label as its
  /// [`labelled`](crate::labelled) scope closes.
  ///
  /// No-op by default (see [`enter_label`](Self::enter_label)); [`Verbose`] overrides
  /// it to pop its open-label stack. The stack therefore follows the call structure of
  /// the `labelled` wrappers exactly, so a checkpoint restore needs no label handling —
  /// no label state lives outside the wrapper scopes and the recorded log entries.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn exit_label(&mut self) {}
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
    err: UnexpectedTokenOf<'a, L, Lang>,
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
  fn emit_warning(&mut self, warning: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_warning(warning)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn checkpoint(&self) -> u64 {
    (**self).checkpoint()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, cursor: &Cursor<'a, '_, L>, checkpoint: u64)
  where
    L: Lexer<'a>,
  {
    (**self).rewind(cursor, checkpoint)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn enter_label(&mut self, label: &'static str) {
    (**self).enter_label(label)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn exit_label(&mut self) {
    (**self).exit_label()
  }
}

/// A trait bound for generic emitter error conversion.
pub trait FromEmitterError<'a, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from a lexer error.
  fn from_lexer_error(err: Spanned<<L::Token as Token<'a>>::Error, L::Span>) -> Self
  where
    L: Lexer<'a>;

  /// Creates an emitter error from an unexpected token error.
  fn from_unexpected_token(err: UnexpectedTokenOf<'a, L, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, L, Lang: ?Sized> FromEmitterError<'a, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<<L::Token as Token<'a>>::Error> + From<UnexpectedTokenOf<'a, L, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_lexer_error(err: Spanned<<L::Token as Token<'a>>::Error, L::Span>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into_data().into()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_unexpected_token(err: UnexpectedTokenOf<'a, L, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}
