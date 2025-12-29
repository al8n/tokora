use core::marker::PhantomData;

use super::{Cursor, Lexer};

/// A checkpoint that captures the lexer's state for backtracking.
///
/// A `Checkpoint` stores a snapshot of the lexer's position and state
/// at a specific point in time. This allows you to save the current state using
/// [`InputRef::save`](crate::lexer::InputRef::save) and later restore it using [`InputRef::restore`](crate::lexer::InputRef::restore), enabling
/// efficient backtracking in parsers.
///
/// Checkpoints include:
/// - The cursor position (byte offset in the input)
/// - The lexer's extras state (for stateful lexers)
/// - Cache state (implicitly through the cursor)
///
/// # Example
///
/// ```ignore
/// let checkpoint = tokenizer.save();
/// // Try parsing something that might fail...
/// if should_backtrack {
///     tokenizer.go(checkpoint); // Restore to saved state
/// }
/// ```
pub struct Checkpoint<'a, 'closure, L: Lexer<'a>> {
  cursor: Cursor<'a, 'closure, L>,
  pub(crate) state: L::State,
  _m: PhantomData<fn(&'closure ()) -> &'closure ()>,
}

impl<'a, 'closure, L: Lexer<'a>> Checkpoint<'a, 'closure, L> {
  /// Creates a new checkpoint.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(cursor: Cursor<'a, 'closure, L>, state: L::State) -> Self {
    Self {
      cursor,
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
