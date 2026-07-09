use core::marker::PhantomData;

use super::{Cursor, Lexer};

/// A checkpoint that captures the lexer's state for backtracking.
///
/// A `Checkpoint` stores a snapshot of the lexer's position and state
/// at a specific point in time. This allows you to save the current state using
/// [`InputRef::save`](crate::InputRef::save) and later restore it using [`InputRef::restore`](crate::InputRef::restore), enabling
/// efficient backtracking in parsers.
///
/// Checkpoints include:
/// - The cursor position (byte offset in the input)
/// - The input span at save time (`InputRef::span` / last-consumed-token span)
/// - The lexer's extras state (for stateful lexers)
/// - Cache state (implicitly through the cursor)
///
/// # Example
///
/// ```ignore
/// let checkpoint = tokenizer.save();
/// // Try parsing something that might fail...
/// if should_backtrack {
///     tokenizer.restore(checkpoint); // Restore to saved state
/// }
/// ```
pub struct Checkpoint<'a, 'closure, L: Lexer<'a>> {
  cursor: Cursor<'a, 'closure, L>,
  /// The actual `InputRef::span` at save time.
  ///
  /// This is the span of the last consumed token, which may differ from the
  /// cursor when the cache is non-empty.  Restoring with `self.span` (rather
  /// than the cursor's offset) ensures that the lexer position is placed *before*
  /// any cached tokens, so they can be re-lexed after a restore.
  pub(crate) span: L::Span,
  pub(crate) state: L::State,
  _m: PhantomData<fn(&'closure ()) -> &'closure ()>,
}

impl<'a, 'closure, L: Lexer<'a>> Checkpoint<'a, 'closure, L> {
  /// Creates a new checkpoint.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(cursor: Cursor<'a, 'closure, L>, span: L::Span, state: L::State) -> Self {
    Self {
      cursor,
      span,
      state,
      _m: PhantomData,
    }
  }

  /// Returns the cursor of the checkpoint.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn cursor(&self) -> &Cursor<'a, 'closure, L> {
    &self.cursor
  }

  /// Returns the state of the checkpoint.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn state(&self) -> &L::State {
    &self.state
  }
}
