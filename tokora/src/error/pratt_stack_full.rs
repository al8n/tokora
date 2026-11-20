use core::marker::PhantomData;

/// The explicit stack of a flat expression driver could not hold another pending operator.
///
/// A flat driver keeps the operators an expression has pending on a stack of its own instead of
/// recursing once for each, and this is its refusal when that stack is full: at a cap the caller
/// set, or because a storage that does not grow holds all it can.
///
/// # Always terminal
///
/// [`is_terminal`](crate::error::MaybeTerminal::is_terminal) is `true` for **every** value of this
/// type: no amount of further input empties a full stack, so a recoverer handed this value must
/// re-raise it untouched rather than synthesize a node from input it was forbidden to read. It is
/// the law [`RecursionLimitReached`](crate::error::RecursionLimitReached) carries for the recursion
/// budget, read through the same [`MaybeTerminal`](crate::error::MaybeTerminal) gate.
///
/// The stop does not rest on the grammar's error type either. The driver that returns this counts
/// the trip on the input session before it builds the value, as
/// [`InputRef::descend`](crate::InputRef::descend) does for the recursion budget, and the three
/// recovery combinators read that count beside the trait. So a grammar whose error discards this
/// value, `()` included, loses the offset and the bound and still re-raises.
///
/// # Its two numbers
///
/// - [`offset`](Self::offset) is committed consumption,
///   [`InputRef::span().end()`](crate::InputRef::span), after the operator that could not be
///   stacked: the end of that operator.
/// - [`capacity`](Self::capacity) is the bound that refused, in pending operators: the cap, where
///   the count reached it, and the storage's own capacity, where the storage refused below the
///   cap.
///
/// # Example
///
/// ```rust
/// use tokora::error::{MaybeTerminal, PrattStackFull};
///
/// let error: PrattStackFull = PrattStackFull::of(12, 32);
/// assert_eq!(error.offset(), 12);
/// assert_eq!(error.capacity(), 32);
/// assert!(error.is_terminal());
/// assert_eq!(error.to_string(), "expression operator stack is full at 12: capacity 32");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("expression operator stack is full at {at}: capacity {capacity}")]
pub struct PrattStackFull<O = usize, Lang: ?Sized = ()> {
  /// Committed consumption after the operator that could not be stacked.
  at: O,
  /// The bound that refused, in pending operators.
  capacity: usize,
  _lang: PhantomData<Lang>,
}

impl<O, Lang: ?Sized> PrattStackFull<O, Lang> {
  /// Creates the error for an operator ending at `at` that a bound of `capacity` pending operators
  /// refused.
  #[inline]
  pub const fn of(at: O, capacity: usize) -> Self {
    Self {
      at,
      capacity,
      _lang: PhantomData,
    }
  }

  /// Returns the offset: committed consumption after the operator that could not be stacked.
  #[inline]
  pub const fn offset(&self) -> O
  where
    O: Copy,
  {
    self.at
  }

  /// Returns a reference to the offset.
  #[inline]
  pub const fn offset_ref(&self) -> &O {
    &self.at
  }

  /// Returns the bound that refused, in pending operators.
  #[inline]
  pub const fn capacity(&self) -> usize {
    self.capacity
  }

  /// Rewrites the offset — the shape a nested parse uses to lift an inner offset into its
  /// enclosing document's coordinates.
  #[inline]
  pub fn map_offset<U, F>(self, f: F) -> PrattStackFull<U, Lang>
  where
    F: FnOnce(O) -> U,
  {
    PrattStackFull {
      at: f(self.at),
      capacity: self.capacity,
      _lang: PhantomData,
    }
  }
}

/// **Always** terminal: no amount of input empties a full stack, so recovery must re-raise rather
/// than spend it.
impl<O, Lang: ?Sized> crate::error::MaybeTerminal for PrattStackFull<O, Lang> {
  #[inline]
  fn is_terminal(&self) -> bool {
    true
  }
}

/// The unit error sink absorbs it like every other error. The payload, the offset and the bound,
/// is lost with the value, and `()` is never terminal, so the
/// [`MaybeTerminal`](crate::error::MaybeTerminal) gate reads `false` on the converted value.
impl<O, Lang: ?Sized> From<PrattStackFull<O, Lang>> for () {
  #[inline]
  fn from(_: PrattStackFull<O, Lang>) -> Self {}
}
