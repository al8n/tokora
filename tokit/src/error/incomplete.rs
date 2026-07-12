//! The partial-input sentinel: the [`Incomplete`] marker error and the [`MaybeIncomplete`]
//! discrimination trait that enforces its never-recoverable law.

/// The partial-input sentinel — signals that the input ended *mid-construct*.
///
/// `Incomplete` is the marker a partial/streaming parse raises when it runs out of input while a
/// construct is still open (a half-read token, an unterminated list): "there may be more input;
/// come back with it". It is defined now so the error family is coherent before the partial-input
/// wave wires it — the type is always compiled but inert until then.
///
/// # The never-recoverable law
///
/// An `Incomplete` is **never recoverable**, and this is a hard contract, not a heuristic:
///
/// - **Recovery combinators must re-raise it untouched.** A recoverer exists to synthesize a value
///   from a *malformed* construct; an incomplete one is not malformed, only unfinished, so
///   recovering it would fabricate a value from input that has not arrived.
///   [`Recover`](crate::parser::Recover) (and the skip-retry driver when it lands) therefore
///   re-raises an `Incomplete` on the `Err` channel instead of invoking the recoverer — see
///   [`MaybeIncomplete`].
/// - **It must never be emitted as a diagnostic.** `Incomplete` rides the `Err` channel *only*; it
///   is a control signal to the caller ("feed me more"), not a user-facing error, so it never
///   flows through an [`Emitter`](crate::Emitter). Reporting it would misclassify a
///   resume-and-retry condition as a failure.
///
/// The payload is the [`offset`](Self::offset) at which the input ran out.
///
/// ```
/// use tokit::error::{Incomplete, MaybeIncomplete};
///
/// let inc = Incomplete::new(42usize);
/// assert_eq!(inc.offset(), 42);
/// // The marker reports itself as incomplete; that is what recovery keys off of.
/// assert!(inc.is_incomplete());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("incomplete input: construct unfinished at offset {offset}")]
pub struct Incomplete<O = usize> {
  /// The offset at which the input ran out mid-construct — the frontier the caller should
  /// resume from once more input arrives.
  offset: O,
}

impl<O> Incomplete<O> {
  /// Creates an `Incomplete` sentinel for input that ran out at `offset`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(offset: O) -> Self {
    Self { offset }
  }

  /// Returns the offset at which the input ended mid-construct.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset(&self) -> O
  where
    O: Copy,
  {
    self.offset
  }

  /// Returns a reference to the offset at which the input ended mid-construct.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset_ref(&self) -> &O {
    &self.offset
  }

  /// Returns a mutable reference to the offset at which the input ended mid-construct.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset_mut(&mut self) -> &mut O {
    &mut self.offset
  }

  /// Consumes the sentinel and returns its offset.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_offset(self) -> O {
    self.offset
  }
}

/// Discriminates whether an error value *is* an [`Incomplete`] partial-input sentinel.
///
/// Recovery machinery is generic over the emitter's error type, so it cannot name [`Incomplete`]
/// directly to honor its [never-recoverable law](Incomplete#the-never-recoverable-law). This
/// trait is the minimal hook that makes the law testable on any error type: the single method
/// [`is_incomplete`](Self::is_incomplete) has a **blanket `false` default**, so an error type
/// opts in with an empty `impl MaybeIncomplete for MyError {}` and only overrides the method if it
/// can actually carry an incomplete signal. [`Incomplete`] itself overrides it to `true`.
///
/// [`Recover`](crate::parser::Recover) requires this bound and re-raises rather than recovers when
/// `is_incomplete()` holds, so an unfinished construct is never fabricated into a value.
pub trait MaybeIncomplete {
  /// Returns `true` iff this error value is (or currently represents) an [`Incomplete`]
  /// partial-input sentinel. Defaults to `false`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_incomplete(&self) -> bool {
    false
  }
}

impl<O> MaybeIncomplete for Incomplete<O> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_incomplete(&self) -> bool {
    true
  }
}

/// The unit error sink is never an incomplete signal.
impl MaybeIncomplete for () {}
