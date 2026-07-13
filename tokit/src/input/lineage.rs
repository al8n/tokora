//! The lineage memos an [`Input`](super::Input) carries so backtracking can rewind an
//! abandoned continuation exactly, gathered behind one guardian.
//!
//! # Single-writer taxonomy
//!
//! Every cell an [`Input`](super::Input) owns belongs to exactly one of three classes, and
//! each class has a single writer:
//!
//! - **Ground truth** — the live scanning position and regime the next token is lexed from:
//!   the source, the lexer state, the last-consumed span, the token cache, the poison
//!   boundary, and the lexer-error dedup watermark. These live directly on
//!   [`Input`](super::Input); the scan/consume paths are their sole writer. They are *not*
//!   part of this module, and the layout keeps them packed ahead of it.
//! - **Lineage memos** — the bookkeeping gathered here: the ids [`save`](super::InputRef::save)
//!   pushes and [`restore`](super::InputRef::restore) pops so an abandoned continuation
//!   rewinds exactly, the pin set that protects a live guard's begin point, and the monotone
//!   counters that keep checkpoint ids, cache-push counts, and savepoint sequences unique.
//!   Backtracking (save/restore/commit and the guards) is their sole writer.
//! - **Witness** — the debug-only, process-unique cross-input identity a checkpoint is stamped
//!   with, so a restore can reject a checkpoint from a foreign input. It is a class of its own,
//!   kept separately cfg'd on [`Input`](super::Input) (see [`Witness`](super::Witness)); its
//!   atomic id source keeps it behind the debug + `target_has_atomic = "ptr"` gate.
//!
//! This module is the guardian of the lineage memos: the cells are private to it and reachable
//! only through the operations below, each of which carries the invariant it maintains. Every
//! memo but the cache-push counter is a live-checkpoint concept that only exists where an
//! allocator does, so all of them except [`cache_pushes`](Lineage::cache_pushes) sit behind the
//! allocator gate; an allocator-less build keeps only the counter and every stack operation
//! compiles out.

#[cfg(any(feature = "std", feature = "alloc"))]
use super::LineageStack;

/// The lineage memos of one [`Input`](super::Input) — the class of cell backtracking owns (see
/// the [module docs](self) for the full taxonomy).
///
/// The fields are private: callers reach them only through the operations, so every invariant
/// that governs a cell lives on the one method that maintains it. In an allocator-less build
/// only [`cache_pushes`](Self::cache_pushes) exists; the live-checkpoint stack, the pin set, and
/// their counters all compile out with the backtracking machinery that needs them.
pub(crate) struct Lineage {
  /// Monotone count of tokens the cache has accepted over the input's life, bumped by every
  /// successful cache push (see [`record_cache_push`](Self::record_cache_push)). A
  /// [`Checkpoint`](super::Checkpoint) captures it at save time and
  /// [`restore`](super::InputRef::restore) uses the difference to drop the entries pushed on
  /// the abandoned continuation. It is correctness state in **every** build, so it is the one
  /// memo present without an allocator.
  cache_pushes: u64,
  /// Input-global savepoint sequence counter for [`StackedTransaction`](super::StackedTransaction),
  /// handed out monotonically by [`next_savepoint_seq`](Self::next_savepoint_seq) and never reset.
  /// It is monotone across every stacked transaction of this input, so a
  /// [`SavepointId`](super::SavepointId)'s `seq` is unique for the whole life of the input: an id
  /// that crosses transactions (nested or sequential) can never collide with a live savepoint's
  /// `seq` in another transaction's stack. There is no atomic and no process-wide state — the
  /// counter is per-input.
  #[cfg(any(feature = "std", feature = "alloc"))]
  savepoint_seq: u64,
  /// The live-checkpoint lineage stack: the ids of the checkpoints that have been saved and
  /// neither restored nor invalidated by restoring an older one, youngest last.
  /// [`open`](Self::open) pushes a fresh id, [`pop_through`](Self::pop_through) pops down through
  /// a restored id (invalidating it and every younger one), and a committed checkpoint is
  /// dropped by [`forget`](Self::forget). State surgery leaves it untouched — checkpoints survive
  /// state replacement, which is transactional.
  ///
  /// It is the single source of truth for lineage validity in **every** allocator build — no
  /// atomics, no interior mutability, just a stack — so [`StackedTransaction`](super::StackedTransaction)
  /// can reject a savepoint whose checkpoint a raw restore below it invalidated, on release and
  /// no-`target_has_atomic`-ptr targets alike. In debug + ptr builds the same stack also backs
  /// [`restore`](super::InputRef::restore)'s non-LIFO panic.
  #[cfg(any(feature = "std", feature = "alloc"))]
  live_ckpts: LineageStack,
  /// Monotone id source for [`live_ckpts`](Self::live_ckpts): each [`open`](Self::open) takes the
  /// current value and bumps it, so an id is never reused for the life of the input and a popped
  /// id can never be mistaken for a live one.
  #[cfg(any(feature = "std", feature = "alloc"))]
  next_ckp_id: u64,
  /// The pinned checkpoint ids: the begin-point checkpoint of every currently-live transaction
  /// guard, [`attempt`](super::InputRef::attempt)/[`try_attempt`](super::InputRef::try_attempt), and
  /// [session point](super::InputRef::begin_point).
  /// A guard/attempt logically borrows the timeline from its begin point forward, so a raw
  /// [`restore`](super::InputRef::restore) that would pop a pinned id off
  /// [`live_ckpts`](Self::live_ckpts) — tearing that begin point out from under a live guard —
  /// **panics at the restore** rather than silently invalidating it. Every guard/attempt
  /// constructor pins its held id on entry and every settle path unpins, so this holds exactly
  /// the live begin points and stays bounded across commit-heavy loops. Allocator-less builds
  /// maintain no pin set and fall back on the detect-at-use backstops.
  ///
  /// "Every settle path" includes one that is **not** a verb: an [`InputRef`](super::InputRef)
  /// dropped with session points still open releases their pins in its `Drop`. A guard cannot leak
  /// a pin (it borrows the handle, so it must settle before the handle can die), but a session point
  /// is a *value on* the handle while its pin lives out here on the longer-lived
  /// [`Input`](super::Input) — so the handle's death is a settle path, and unpinning there is what
  /// keeps "exactly the live begin points" true.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pinned: LineageStack,
}

impl Lineage {
  /// A fresh set of memos for a new input: an unadvanced cache-push counter and — in allocator
  /// builds — an empty live-checkpoint stack, an empty pin set, and zeroed counters.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn new() -> Self {
    Self {
      cache_pushes: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      savepoint_seq: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      live_ckpts: LineageStack::new(),
      #[cfg(any(feature = "std", feature = "alloc"))]
      next_ckp_id: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      pinned: LineageStack::new(),
    }
  }

  /// The memos a **clone** of the input starts with. A clone is a *new* input that happens to
  /// share the original's cache contents:
  ///
  /// - the **cache-push counter** carries forward, so the clone's own future saves and restores
  ///   stay consistent with the shared cache contents;
  /// - the **savepoint sequence** carries forward so the clone's savepoint seqs stay monotone;
  ///   the clone is a distinct struct with a distinct nonce anyway, so its ids never cross the
  ///   original's regardless of the starting value;
  /// - the **live-checkpoint stack** and its **id counter** reset — a clone starts with an empty
  ///   lineage and a fresh id source, so a checkpoint from the original is never mistaken for one
  ///   of the clone's (restoring it is caught as a foreign input in debug + ptr builds);
  /// - the **pin set** resets — a clone has no live guards.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn forked(&self) -> Self {
    Self {
      cache_pushes: self.cache_pushes,
      #[cfg(any(feature = "std", feature = "alloc"))]
      savepoint_seq: self.savepoint_seq,
      #[cfg(any(feature = "std", feature = "alloc"))]
      live_ckpts: LineageStack::new(),
      #[cfg(any(feature = "std", feature = "alloc"))]
      next_ckp_id: 0,
      #[cfg(any(feature = "std", feature = "alloc"))]
      pinned: LineageStack::new(),
    }
  }

  /// The current cache-push count, snapshotted into a [`Checkpoint`](super::Checkpoint) at save
  /// time so a later restore can drop exactly the entries pushed since.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn cache_pushes(&self) -> u64 {
    self.cache_pushes
  }

  /// Records one accepted cache push. Every cache push flows through this on success — the peek
  /// fill and the `try_expect` put-backs — so the count tracks exactly the tokens the cache
  /// accepted: a full cache that hands the token back leaves the count unchanged, and a blackhole
  /// cache — which accepts no push — keeps it at 0.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn record_cache_push(&mut self) {
    self.cache_pushes += 1;
  }

  /// Rewinds the cache-push counter to a checkpoint's saved value on restore.
  ///
  /// The count is per-lineage state, exactly like the dedup watermark and the poison boundary:
  /// under the last-in, first-out contract a restore returns to the saved lineage exactly, so the
  /// counter copies back verbatim. It is re-anchored to the push history of the lineage now live
  /// (the restore's tail-drop has already consumed the pre-rewind value), so future
  /// `cache_pushes − saved` deltas stay exact. State surgery deliberately leaves the counter
  /// untouched — a re-key clears the cache but not the count, so a checkpoint saved before the
  /// surgery still restores an exact delta.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn restore_cache_pushes(&mut self, saved: u64) {
    self.cache_pushes = saved;
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Lineage {
  /// Opens a checkpoint entry: takes a fresh, never-reused id, records it on the live-checkpoint
  /// lineage stack (youngest last), and returns it to be stamped into the
  /// [`Checkpoint`](super::Checkpoint). [`restore`](super::InputRef::restore) later pops the stack
  /// down through this id, and a [`StackedTransaction`](super::StackedTransaction) checks the id
  /// is still present before honoring a savepoint — the check that makes stale savepoints panic on
  /// release and no-ptr targets. Opening never invalidates another checkpoint; only restoring does.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn open(&mut self) -> u64 {
    let id = self.next_ckp_id;
    self.next_ckp_id += 1;
    self.live_ckpts.push(id);
    id
  }

  /// Returns whether `id` is still live on the lineage stack. Backs both the
  /// [`StackedTransaction`](super::StackedTransaction) savepoint-staleness check (every allocator
  /// build) and, in debug + ptr builds, [`restore`](super::InputRef::restore)'s non-LIFO panic.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn contains(&self, id: u64) -> bool {
    self.live_ckpts.contains(&id)
  }

  /// Pops the lineage stack down through `id` inclusive, invalidating it and every checkpoint
  /// saved after it. A no-op if `id` is already gone — a raw restore to a checkpoint an earlier
  /// restore already invalidated (release's unspecified-but-bounded posture; debug + ptr asserts
  /// presence in [`restore`](super::InputRef::restore) first).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn pop_through(&mut self, id: u64) {
    if let Some(pos) = self.live_ckpts.iter().position(|&x| x == id) {
      self.live_ckpts.truncate(pos);
    }
  }

  /// Drops `id` from the live-checkpoint lineage stack because its checkpoint was **kept**
  /// (committed) rather than restored.
  ///
  /// A restored checkpoint is popped off the stack by [`pop_through`](Self::pop_through); a
  /// *committed* one never reaches a restore, so without this its id would linger and grow the
  /// stack across commit-heavy loops. Removing it keeps the stack exact and bounded. `O(1)` when
  /// `id` is the stack top (the common case for a committed checkpoint); a linear removal
  /// otherwise (e.g. a raw checkpoint saved above it was dropped without restoring). Removing a
  /// non-top id keeps the rest of the stack in order, so an older restore still pops cleanly
  /// through it. Committing an already-invalidated id is a harmless no-op: it is simply absent.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn forget(&mut self, id: u64) {
    if self.live_ckpts.last() == Some(&id) {
      self.live_ckpts.pop();
    } else if let Some(pos) = self.live_ckpts.iter().position(|&x| x == id) {
      self.live_ckpts.remove(pos);
    }
  }

  /// Pins `id` — the begin-point checkpoint of a transaction guard or an
  /// [`attempt`](super::InputRef::attempt) — so a raw [`restore`](super::InputRef::restore) that
  /// would pop it off the lineage (a restore reaching *below* the guard's begin point) panics at
  /// the restore instead of silently tearing out the guard's foundation. Every guard constructor
  /// and attempt pins on entry; the matching [`unpin`](Self::unpin) runs on every settle path.
  ///
  /// Nested guards are borrowck-serialized: an inner guard mutably borrows its parent for its
  /// whole life, so the inner settles (and unpins) before the outer is usable again. An outer
  /// rollback therefore never finds a live inner pin sitting above its base — only its own
  /// (just-unpinned) begin point and any LIFO-clean raw checkpoints, none pinned.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn pin(&mut self, id: u64) {
    self.pinned.push(id);
  }

  /// Removes `id` from the pin set when its guard, attempt, or session point settles. Mirrors
  /// [`forget`](Self::forget): `O(1)` when `id` is the top (the LIFO common case — guards and
  /// attempts are borrowck-serialized, so the settling one is innermost), a linear removal
  /// otherwise. Called on **every** settle path (commit, explicit rollback, `Drop`, both closure
  /// arms of the attempts, both session-point verbs, and the [`InputRef`](super::InputRef) `Drop`
  /// that releases session points abandoned with the handle), so the pin set stays bounded and holds
  /// exactly the begin points of the currently-live guards, attempts, and session points.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn unpin(&mut self, id: u64) {
    if self.pinned.last() == Some(&id) {
      self.pinned.pop();
    } else if let Some(pos) = self.pinned.iter().position(|&x| x == id) {
      self.pinned.remove(pos);
    }
  }

  /// Panics if restoring to `target_id` would pop a **pinned** checkpoint off the live lineage —
  /// i.e. if it would tear the begin point out from under a still-live transaction guard or
  /// attempt. This is the detect-at-cause check: a raw restore below a live guard/attempt begin
  /// point is refused right where it is requested, in every allocator build.
  ///
  /// A [`restore`](super::InputRef::restore) pops the target and every younger checkpoint. A
  /// guard's own settle unpins its held id **before** routing through the restore, so a guard
  /// rolling back to its own base never trips its own pin; only a restore reaching *below* a live
  /// begin point finds that begin point still pinned above the target. A stacked-transaction
  /// savepoint `rollback_to` restores a checkpoint *above* the base, so it can never reach the
  /// pinned base. A target that is not live pops nothing, so it cannot invalidate anything pinned.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn assert_restore_preserves_pins(&self, target_id: u64) {
    let Some(pos) = self.live_ckpts.iter().position(|&x| x == target_id) else {
      // The target is already gone: the restore will pop nothing, so nothing pinned can be
      // invalidated (release's unspecified-but-bounded posture for an already-dead target).
      return;
    };
    // The restore truncates `live_ckpts` at `pos`, popping the target and every younger
    // checkpoint. If any of those is pinned, the restore would invalidate a live guard/attempt.
    if self.live_ckpts[pos..]
      .iter()
      .any(|id| self.pinned.contains(id))
    {
      panic!(
        "restore would invalidate a live transaction guard or attempt (the target predates its begin point)"
      );
    }
  }

  /// Hands out the next input-global savepoint sequence number, bumping the counter. Because the
  /// counter lives on the input (not on any one transaction) and is never reset, a
  /// [`SavepointId`](super::SavepointId)'s `seq` is unique for the whole life of the input: an id
  /// that crosses transactions can never collide with a live savepoint's `seq` in another
  /// transaction's stack, so the membership scan in `rollback_to`/`release` panics deterministically
  /// wherever the lifetime brand does not already reject the id.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn next_savepoint_seq(&mut self) -> u64 {
    let seq = self.savepoint_seq;
    self.savepoint_seq += 1;
    seq
  }

  /// The number of live checkpoints — test-only observability for the no-growth guarantee that
  /// committing (and a success-path recover) gives the lineage stack.
  #[cfg(all(test, feature = "logos", feature = "std"))]
  pub(crate) fn live_len(&self) -> usize {
    self.live_ckpts.len()
  }

  /// The number of pinned checkpoints — observability for the law this set states about itself:
  /// it holds **exactly** the begin points of the currently-live guards, attempts, and session
  /// points, and is therefore empty whenever none is live. It backs the drop-path release the
  /// [`InputRef`](super::InputRef)'s `Drop` performs for abandoned session points (a pin whose
  /// point nobody can settle would otherwise sit here for the life of the input).
  ///
  /// Reachable from the owning [`Input`](super::Input), not just from a handle, because that is
  /// exactly where the question is asked: *after* the handle that opened the points is gone.
  /// Gated to its callers — the `logos` + `std` session tests — so it is never dead code under
  /// `cargo hack --each-feature --tests`.
  #[cfg(all(test, feature = "logos", feature = "std"))]
  pub(crate) fn pinned_len(&self) -> usize {
    self.pinned.len()
  }
}
