use crate::{
  Lexer,
  span::{SimpleSpan, Span, Spanned},
};

use super::super::{
  separated::{
    MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  *,
};

use std::{collections::BTreeMap, vec::Vec};

use core::marker::PhantomData;

mod full_container;
mod missing_leading_separator;
mod missing_trailing_separator;
mod pratt;
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
/// - [`SeparatedEmitter`](super::super::SeparatedEmitter) - Separator errors
/// - And other atomic traits for specific parsing scenarios
///
/// The errors are stored in a `BTreeMap` indexed by span, ensuring they are ordered by their
/// position in the source code. Multiple errors can share a single span (for example a
/// zero-width missing-element and missing-separator reported at the same offset), so each span
/// maps to a `Vec` of errors that accumulate in emission order rather than overwriting one
/// another. You can retrieve all collected errors via the [`errors()`](Self::errors) method.
///
/// # Examples
///
/// ```ignore
/// use tokit::emitter::Verbose;
///
/// // Create a verbose emitter
/// let emitter = Verbose::<MyError>::new();
///
/// // After parsing, retrieve all errors (each span may carry several).
/// for (span, errors) in emitter.errors() {
///     for error in errors {
///         println!("Error at {:?}: {}", span, error);
///     }
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
  errs: BTreeMap<S, Vec<Error>>,
  /// The span of every emission, in emission order. An entry's index in this log
  /// is its monotonic sequence number; [`checkpoint`](Emitter::checkpoint) is the
  /// log length and [`rewind`](Emitter::rewind) unwinds the tail back to a mark,
  /// popping the matching error off each span's `Vec`. This is what lets rewind
  /// drop a speculative zero-width diagnostic while keeping an earlier one at the
  /// same span — a distinction span-ordered storage alone cannot make.
  log: Vec<S>,
  _lang: PhantomData<Lang>,
}

impl<Error, Span, Lang: ?Sized> Default for Verbose<Error, Span, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self {
      errs: BTreeMap::new(),
      log: Vec::new(),
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
      log: self.log.clone(),
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
      log: Vec::new(),
      _lang: PhantomData,
    }
  }

  /// Records `err` at `span`, appending it to the span's group and logging the
  /// emission order so a later [`rewind`](Emitter::rewind) can undo it precisely.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn record(&mut self, span: S, err: Error)
  where
    S: Ord + Clone,
  {
    self.log.push(span.clone());
    self.errs.entry(span).or_default().push(err);
  }

  /// Returns a reference to all collected errors.
  ///
  /// The errors are stored in a `BTreeMap` indexed by their span, which means they are
  /// automatically sorted by their position in the source code. Each span maps to a `Vec`
  /// of every error reported at that span, in emission order, so same-span errors are all
  /// retained rather than overwritten.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// use tokit::emitter::Verbose;
  ///
  /// let mut emitter = Verbose::<MyError>::new();
  /// // ... perform parsing ...
  ///
  /// // Iterate through all errors in source order (flattening the per-span groups).
  /// for (span, error) in emitter.errors().iter().flat_map(|(s, es)| es.iter().map(move |e| (s, e))) {
  ///     println!("Error at position {}: {}", span.start(), error);
  /// }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn errors(&self) -> &BTreeMap<S, Vec<Error>> {
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
    let err = Error::from_lexer_error(Spanned::new(span.clone(), err));
    self.record(span, err);
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, err: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error> {
    let (span, err) = err.into_components();
    self.record(span, err);
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
    let span = err.span_ref().clone();
    self.record(span, Error::from_unexpected_token(err));
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn checkpoint(&self) -> u64 {
    self.log.len() as u64
  }

  /// Rewind the error state to a checkpoint, emission-aware.
  ///
  /// `checkpoint` is the [`checkpoint`](Emitter::checkpoint) mark captured at the
  /// save point: the emission-log length at that instant. Every diagnostic
  /// recorded *after* it — exactly the emissions of the abandoned branch — is
  /// dropped, newest first, by popping the matching entry off its span's group;
  /// everything recorded before survives. The decision is purely by emission
  /// order, so a zero-width error emitted during a speculative branch is removed
  /// while an earlier zero-width error at the *same* offset is kept — a
  /// distinction the former span-end offset heuristic could not make. `cursor`
  /// is unused.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, cursor: &Cursor<'inp, '_, L>, checkpoint: u64)
  where
    L: Lexer<'inp>,
  {
    let _ = cursor;
    let mark = (checkpoint as usize).min(self.log.len());
    while self.log.len() > mark {
      // Unwind newest-first: each span's `Vec` grows in emission order, so the
      // matching entry to drop is always its last one.
      let span = self.log.pop().expect("log length exceeds the mark");
      if let Some(group) = self.errs.get_mut(&span) {
        group.pop();
        if group.is_empty() {
          self.errs.remove(&span);
        }
      }
    }
  }
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests;
