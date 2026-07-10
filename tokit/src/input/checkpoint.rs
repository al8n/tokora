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
  /// The emitter's emission mark at save time (see
  /// [`Emitter::checkpoint`](crate::emitter::Emitter::checkpoint)). Restoring
  /// replays it into [`Emitter::rewind`](crate::emitter::Emitter::rewind) so an
  /// emission-aware emitter drops exactly the diagnostics of the abandoned branch.
  pub(crate) emitter_checkpoint: u64,
  /// The lexer-error dedup high-water mark at save time.
  ///
  /// A speculative branch may seal (emit) a lexer error whose span end sits
  /// *above* the checkpoint cursor — e.g. a `peek` that scans past the cursor.
  /// [`Emitter::rewind`](crate::emitter::Emitter::rewind) keeps that error (it
  /// predates the emission checkpoint), so restoring the watermark to the cursor
  /// would drop it below the retained error and let a re-lex emit it a second
  /// time. Restoring *this* saved mark instead keeps the watermark above the
  /// retained error, preserving exactly-once emission; errors sealed *after* the
  /// checkpoint were unwound from the emitter, and this mark (predating them)
  /// correctly permits their re-emission if the committed path re-lexes them.
  pub(crate) emitted_error_end: L::Offset,
  /// The input-level sticky limit-error boundary at save time.
  ///
  /// `None` is unpoisoned; `Some(off)` is the durable frontier a trip latched (see
  /// [`Input::poison_boundary`](crate::input::Input)). It is checkpointed alongside
  /// the emitter mark and the dedup watermark because the three move together: a
  /// speculative peek that trips the limit latches the frontier *and* emits the
  /// limit diagnostic *and* lifts the watermark. A
  /// [`restore`](crate::InputRef::restore) that rewinds the diagnostic must also
  /// relax the frontier, or it would outlive its diagnostic and a post-restore
  /// drain would stop on a diagnostic-less poison — truncation masquerading as
  /// clean EOF. Restore only ever moves the frontier toward the *less-poisoned* of
  /// this saved value and the current one (a max under the ordering where a smaller
  /// offset is more poisoned and `None` is +infinity), never more poisoned.
  pub(crate) poison_boundary: Option<L::Offset>,
  _m: PhantomData<fn(&'closure ()) -> &'closure ()>,
}

impl<'a, 'closure, L: Lexer<'a>> Checkpoint<'a, 'closure, L> {
  /// Creates a new checkpoint.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(
    cursor: Cursor<'a, 'closure, L>,
    span: L::Span,
    state: L::State,
    emitter_checkpoint: u64,
    emitted_error_end: L::Offset,
    poison_boundary: Option<L::Offset>,
  ) -> Self {
    Self {
      cursor,
      span,
      state,
      emitter_checkpoint,
      emitted_error_end,
      poison_boundary,
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
