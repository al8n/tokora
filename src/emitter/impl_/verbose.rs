use crate::{
  Lexer,
  span::{SimpleSpan, Span, Spanned},
};

use super::super::{
  separated::{UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter},
  *,
};

use std::collections::BTreeMap;

use core::marker::PhantomData;

mod full_container;
mod separator;
mod too_few;
mod too_many;
mod unexpected_leading_separator;
mod unexpected_trailing_separator;

/// A verbose emitter that collects all errors during parsing.
///
/// Unlike [`Fatal`](super::fatal::Fatal) which stops at the first error, or [`Silent`](super::silent::Silent)
/// which ignores errors silently, `Verbose` collects all errors encountered during parsing and
/// continues parsing where possible. This makes it ideal for compiler diagnostics, IDE integration,
/// and development scenarios where you need comprehensive error reporting.
///
/// `Verbose` is a **complete implementation** of all atomic emitter traits, providing a pre-built bundle
/// for comprehensive error collection. It implements:
/// - [`Emitter`](super::super::Emitter) - Core error handling
/// - [`TooFewEmitter`](super::super::TooFewEmitter) - "Too few elements" errors
/// - [`TooManyEmitter`](super::super::TooManyEmitter) - "Too many elements" errors
/// - [`DelimitedEmitter`](super::super::DelimitedEmitter) - Delimiter errors
/// - [`SeparatedEmitter`](super::super::SeparatedEmitter) - Separator errors
/// - And other atomic traits for specific parsing scenarios
///
/// The errors are stored in a `BTreeMap` indexed by span, ensuring they are ordered by their
/// position in the source code. You can retrieve all collected errors via the [`errors()`](Self::errors) method.
///
/// # Examples
///
/// ```ignore
/// use tokit::emitter::Verbose;
///
/// // Create a verbose emitter
/// let emitter = Verbose::<MyError>::new();
///
/// // After parsing, retrieve all errors
/// for (span, error) in emitter.errors() {
///     println!("Error at {:?}: {}", span, error);
/// }
/// ```
///
/// # Use Cases
///
/// - **Compiler Diagnostics**: Collect all errors in a single pass to show users all issues at once
/// - **IDE Integration**: Provide comprehensive error highlighting and diagnostics
/// - **Development & Debugging**: Understand all parsing issues without having to fix them one at a time
/// - **Error Recovery**: Continue parsing after errors to provide better context and suggestions
///
/// # Comparison with Other Emitters
///
/// | Emitter | Behavior | Atomic Traits | Use Case |
/// |---------|----------|---------------|----------|
/// | [`Fatal`](super::fatal::Fatal) | Stop on first error | Implements all | Runtime, REPL, fail-fast scenarios |
/// | [`Silent`](super::silent::Silent) | Ignore all errors | Implements all | Error recovery, best-effort parsing |
/// | `Verbose` | Collect all errors | Implements all | Compilers, IDEs, comprehensive diagnostics |
/// | Custom | User-defined | Implement only what you need | Specialized use cases |
///
/// Thanks to Tokit's **atomically composable trait design**, you can implement only the emitter traits
/// your parser needs. `Verbose`, `Fatal`, and `Silent` are pre-built bundles that implement all atomic
/// traits with consistent behavior, but you're encouraged to create custom emitters by implementing just
/// the specific traits relevant to your parser.
#[derive(Debug)]
pub struct Verbose<Error, S = SimpleSpan, Lang: ?Sized = ()> {
  errs: BTreeMap<S, Error>,
  _lang: PhantomData<Lang>,
}

impl<Error, Span, Lang: ?Sized> Default for Verbose<Error, Span, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self {
      errs: BTreeMap::new(),
      _lang: PhantomData,
    }
  }
}

impl<Error, Span, Lang: ?Sized> Clone for Verbose<Error, Span, Lang>
where
  Error: Clone,
  Span: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    Self {
      errs: self.errs.clone(),
      _lang: PhantomData,
    }
  }
}

impl<Error, S, Lang: ?Sized> Verbose<Error, S, Lang> {
  /// Creates a new `Verbose` emitter with an empty error collection.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// use tokit::emitter::Verbose;
  ///
  /// let emitter = Verbose::<MyError>::new();
  /// assert_eq!(emitter.errors().len(), 0);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      errs: BTreeMap::new(),
      _lang: PhantomData,
    }
  }

  /// Returns a reference to all collected errors.
  ///
  /// The errors are stored in a `BTreeMap` indexed by their span, which means they are
  /// automatically sorted by their position in the source code.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// use tokit::emitter::Verbose;
  ///
  /// let mut emitter = Verbose::<MyError>::new();
  /// // ... perform parsing ...
  ///
  /// // Iterate through all errors in source order
  /// for (span, error) in emitter.errors() {
  ///     println!("Error at position {}: {}", span.start(), error);
  /// }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn errors(&self) -> &BTreeMap<S, Error> {
    &self.errs
  }
}

impl<'inp, L, S, Error, Lang: ?Sized> Emitter<'inp, L, Lang> for Verbose<Error, S, Lang>
where
  L: Lexer<'inp, Span = S, Offset = S::Offset>,
  Error: FromEmitterError<'inp, L, Lang>,
  S: Span + Ord + Clone,
{
  type Error = Error;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    err: Spanned<<L::Token as Token<'inp>>::Error, L::Span>,
  ) -> Result<(), Self::Error> {
    let (span, err) = err.into_components();
    self.errs.insert(
      span.clone(),
      Error::from_lexer_error(Spanned::new(span, err)),
    );
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, err: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error> {
    let (span, err) = err.into_components();
    self.errs.insert(span, err);
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_token(
    &mut self,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self
      .errs
      .insert(err.span_ref().clone(), Error::from_unexpected_token(err));
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, cursor: &Cursor<'inp, '_, L>)
  where
    L: Lexer<'inp>,
  {
    let offset = cursor.as_inner();
    self.errs.retain(|k, _| k.end_ref().lt(offset));
  }
}
