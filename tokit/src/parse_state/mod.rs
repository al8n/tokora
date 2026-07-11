use super::{input::Cursor, *};

#[cfg(any(feature = "std", feature = "alloc"))]
use super::input::Checkpoint;

/// A parsing state passed to parser functions.
///
/// Alongside the span/slice/state/emitter accessors it lends a combinator callback, a
/// `ParseState` in an allocator build owns an internal stack of **session points** — the
/// non-lexical form of speculation.
///
/// # Session points: owned, externally-driven speculation
///
/// The transaction guards ([`Transaction`](crate::Transaction),
/// [`StackedTransaction`](crate::StackedTransaction)) borrow the input for a lexical scope, so a
/// guard cannot be stored beside the input it borrows. That rules out one shape: a driver object
/// that owns its input and is stepped across separate method calls — a REPL or an IDE that parses
/// a fragment, speculates, then commits or rolls back on a *later* call — cannot hold a borrowing
/// guard next to the input it owns (the value would be self-referential).
///
/// Session points close that shape by moving the speculation *inside* the state.
/// [`begin_point`](Self::begin_point) saves a checkpoint onto an internal stack and pins it
/// exactly as a guard pins its begin point; [`commit_point`](Self::commit_point) keeps the
/// progress and [`rollback_point`](Self::rollback_point) returns to it, each settling the newest
/// point. The checkpoints are plain values on the state, not a borrow, so the state — and the
/// whole open session with it — can be owned by a driver and moved between calls.
///
/// The internal stack *is* the last-in, first-out order: points settle newest-first, so nesting
/// needs no id and no validation beyond what [`restore`](crate::InputRef::restore) already
/// enforces. Like the guards' savepoint stack it exists only where an allocator does; a
/// no-allocator `ParseState` simply lacks the session API.
///
/// # No implicit rollback on drop
///
/// Unlike a guard — whose drop rolls back (or, under the [`Commit`](crate::Commit) policy, keeps)
/// its undecided scope — dropping a `ParseState` with live session points does **nothing** for
/// them: the checkpoints are discarded with the state and the input dies alongside it. A session
/// ends *explicitly*, through [`commit_point`](Self::commit_point) or
/// [`rollback_point`](Self::rollback_point); the lineage pin a live point holds is likewise
/// released only by settling it. Rolling a whole owned session back implicitly on drop would
/// silently paper over a driver that lost track of its own points — the deliberate opposite of a
/// guard's drop policy — so the end is left explicit to surface that bug instead.
pub struct ParseState<'a, 'inp, 'closure, L, Ctx, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  inp: &'a mut InputRef<'inp, 'closure, L, Ctx, Lang>,
  start: Cursor<'inp, 'closure, L>,
  /// The live session points, oldest first: the checkpoints
  /// [`begin_point`](Self::begin_point) has saved and neither committed nor rolled back. The
  /// vector *is* the last-in, first-out stack — [`commit_point`](Self::commit_point) and
  /// [`rollback_point`](Self::rollback_point) pop its back — so nesting is structural and needs no
  /// id validation. It never allocates until the first [`begin_point`](Self::begin_point). Gated
  /// with the session API to the allocator builds, exactly like the guards' savepoint stack.
  #[cfg(any(feature = "std", feature = "alloc"))]
  points: std::vec::Vec<Checkpoint<'inp, 'closure, L>>,
}

impl<'a, 'inp, 'closure, L, Ctx, Lang: ?Sized> ParseState<'a, 'inp, 'closure, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Create a new `ParseState`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(
    inp: &'a mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    start: Cursor<'inp, 'closure, L>,
  ) -> Self {
    Self {
      inp,
      start,
      #[cfg(any(feature = "std", feature = "alloc"))]
      points: std::vec::Vec::new(),
    }
  }

  /// Returns the span covering the output being parsed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span(&self) -> L::Span {
    self.inp.span_since(&self.start)
  }

  /// Returns a mutable reference to an emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn emitter(&mut self) -> &mut Ctx::Emitter {
    self.inp.emitter()
  }

  /// Returns the state of the lexer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn state(&self) -> &L::State {
    self.inp.state()
  }

  /// Returns a mutable reference to the state of the lexer.
  ///
  /// # State replacement re-keys the input's offset-dependent facts
  ///
  /// Delegates to [`InputRef::state_mut`](crate::InputRef::state_mut): taking the state
  /// mutably eagerly re-keys every offset-dependent fact the input tracks — the token cache
  /// is cleared, the poison boundary is dropped, and the lexer-error dedup watermark is
  /// reset to the current committed cursor. The re-key is itself transactional, not
  /// invalidating: checkpoints and savepoints saved before the state mutation remain
  /// valid, and restoring one afterwards simply undoes the surgery — the prior regime,
  /// boundary, watermark, and position all return.
  ///
  /// State surgery with outstanding speculative diagnostics may re-report the re-lexed
  /// region under the new regime, so callers should complete or roll back speculation
  /// before replacing state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn state_mut(&mut self) -> &mut L::State {
    self.inp.state_mut()
  }

  /// Returns the source slice covering the output being parsed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice(&self) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    self.inp.slice_since(&self.start)
  }
}

/// Session points — the owned, non-lexical speculation surface (see the [type
/// docs](ParseState#session-points-owned-externally-driven-speculation)).
///
/// Gated with the internal point stack to the allocator builds and bounded by
/// `L::State: Clone` (the [`save`](crate::InputRef::save) / [`restore`](crate::InputRef::restore)
/// requirement the guards share): a no-allocator `ParseState` has neither the stack nor these
/// methods.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
impl<'inp, L, Ctx, Lang: ?Sized> ParseState<'_, 'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Opens a session point: saves a checkpoint of the current position onto the internal stack
  /// and **pins** its lineage id, exactly as a transaction guard pins its begin point.
  ///
  /// A session point is the base of a non-lexical speculative scope, so the same hazard a guard
  /// base carries applies until the point is settled: a raw [`restore`](crate::InputRef::restore)
  /// that would rewind the lineage *below* this point would tear its foundation out, and the pin
  /// makes such a restore **panic where it is requested** rather than corrupt the timeline
  /// silently. That raw restore is reachable only with the `unstable-raw` feature, so without it
  /// the hazard is unrepresentable downstream and session points are the whole story. Settle the
  /// point with [`commit_point`](Self::commit_point) (keep the progress) or
  /// [`rollback_point`](Self::rollback_point) (return to it); points settle newest-first.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn begin_point(&mut self) {
    let ckp = self.inp.save();
    // Pin the base exactly like a guard: a raw restore reaching below this point now panics at
    // that restore instead of silently invalidating the session's foundation.
    self.inp.pin_checkpoint(ckp.ckp_id);
    self.points.push(ckp);
  }

  /// Settles the newest session point by **committing** it: pops it off the internal stack,
  /// releases its pin, and keeps every bit of progress made since it opened — the consuming
  /// [`commit`](crate::InputRef::commit) that releases the checkpoint's lineage entry.
  ///
  /// # Panics
  ///
  /// Panics with a message prefixed `no live session point` when there is no open point to
  /// commit.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn commit_point(&mut self) {
    let ckp = self.points.pop().expect("no live session point to commit");
    // Kept, not restored: unpin the base, then the raw consuming commit keeps the progress and
    // releases the lineage entry.
    self.inp.unpin_checkpoint(ckp.ckp_id);
    self.inp.commit(ckp);
  }

  /// Settles the newest session point by **rolling back** to it: pops it off the internal stack,
  /// releases its pin **first** — so restoring to the point does not trip its own pin, mirroring
  /// the guards' settle ordering — then performs the checked
  /// [`restore`](crate::InputRef::restore). Position, span, lexer state, emission log, dedup
  /// watermark, and poison boundary all return to where the point opened.
  ///
  /// # Panics
  ///
  /// Panics with a message prefixed `no live session point` when there is no open point to roll
  /// back.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn rollback_point(&mut self) {
    let ckp = self
      .points
      .pop()
      .expect("no live session point to roll back");
    // Unpin the base FIRST so the checked restore below does not see the point's own begin point
    // as pinned — rolling back to it is legal. A raw restore *below* it would already have
    // panicked at that restore (the pin's detect-at-cause check).
    self.inp.unpin_checkpoint(ckp.ckp_id);
    self.inp.restore(ckp);
  }

  /// The number of live session points — the depth of the internal speculation stack, for a
  /// driver tracking where it sits in a nested speculation.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn points(&self) -> usize {
    self.points.len()
  }
}

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests;
