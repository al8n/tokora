use core::{
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicU64, Ordering},
};

use std::vec::Vec;

use super::{Checkpoint, InputRef, Lexer, ParseContext};

/// Process-wide source of stacked-transaction nonces.
///
/// Every [`begin_stacked`](InputRef::begin_stacked) in the process takes the next value, so
/// each transaction — across every [`Input`](crate::input::Input), not just successive
/// transactions on one input — carries a distinct nonce. A [`SavepointId`] is therefore
/// foreign to every transaction but the one that issued it, and the transaction-nonce
/// check rejects a stale or cross-input id instead of it matching another transaction's
/// first savepoint (whose per-input nonce and `seq` once coincided). Relaxed ordering
/// suffices: only uniqueness matters, not inter-thread ordering.
static TXN_NONCE: AtomicU64 = AtomicU64::new(0);

/// Returns a fresh, process-unique stacked-transaction nonce.
#[cfg_attr(not(tarpaulin), inline)]
pub(super) fn next_txn_nonce() -> u64 {
  TXN_NONCE.fetch_add(1, Ordering::Relaxed)
}

/// An opaque handle to one savepoint inside a [`StackedTransaction`].
///
/// Returned by [`savepoint`](StackedTransaction::savepoint) and consumed by
/// [`rollback_to`](StackedTransaction::rollback_to) and
/// [`release`](StackedTransaction::release). It is a small `Copy` token that borrows
/// nothing, so it can be stashed in a list of candidates or returned up the call stack
/// while the transaction stays open.
///
/// An id is valid only for the transaction that issued it, and only while its savepoint
/// is live: rolling back to an older savepoint destroys the younger ones (SQL
/// `ROLLBACK TO`), and releasing forgets a savepoint and everything above it. Using a
/// destroyed or foreign id panics — see [`rollback_to`](StackedTransaction::rollback_to).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SavepointId {
  /// Identifies the issuing transaction. Stamped once per
  /// [`begin_stacked`](InputRef::begin_stacked) from a process-wide counter, so an id is
  /// unique to its transaction across every input and outlives it as a detectable foreign
  /// token rather than silently matching another transaction's savepoint.
  txn_nonce: u64,
  /// The savepoint's position in its transaction's issue order.
  seq: u64,
}

/// A scoped backtracking transaction that holds several live savepoints at once,
/// mirroring SQL savepoint semantics.
///
/// The lean [`Transaction`](super::Transaction) captures a single begin point;
/// `StackedTransaction` adds an internal last-in, first-out stack of savepoints so a
/// parser can keep **several fallback positions live simultaneously** and return to any
/// of them. Reach for it when a single alternative is not enough — best- or
/// longest-match selection (mark a savepoint after each parsed stage, score them, then
/// [`rollback_to`](Self::rollback_to) the winner), multi-segment speculation with
/// fallback to any earlier boundary, or recovery scans juggling several anchor
/// candidates. For a single speculative alternative, prefer
/// [`Transaction`](super::Transaction); for closure-shaped speculation,
/// [`attempt`](InputRef::attempt) / [`try_attempt`](InputRef::try_attempt).
///
/// # SQL savepoint semantics
///
/// The four operations map onto SQL exactly:
///
/// | this type | SQL | effect |
/// |---|---|---|
/// | [`savepoint`](Self::savepoint) | `SAVEPOINT` | mark the current position, return an id |
/// | [`rollback_to`](Self::rollback_to) | `ROLLBACK TO` | return to a mark, destroy the younger savepoints, keep the mark |
/// | [`release`](Self::release) | `RELEASE SAVEPOINT` | forget a mark and the younger ones, keep the parsed progress |
/// | [`commit`](Self::commit) | `COMMIT` | keep everything, forget all savepoints |
/// | [`rollback`](Self::rollback) | `ROLLBACK` | return to the begin point, discard everything |
///
/// Rolling back to an older savepoint always destroys every newer one, so out-of-order
/// revival is impossible by construction — the [`restore`](InputRef::restore) discipline
/// holds because the internal stack only ever shrinks from the top. A destroyed or
/// foreign [`SavepointId`] panics in every build (a foreign or stale id is a logic bug,
/// not a backtracking choice); see [`rollback_to`](Self::rollback_to).
///
/// [`commit`](Self::commit) and [`rollback`](Self::rollback) consume the transaction; an
/// undecided transaction rolls back to the begin point on drop, discarding all
/// savepoints — the database default. Cost when unused is zero: the empty savepoint
/// `Vec` never allocates, and a begin costs one relaxed atomic increment for the nonce.
///
/// ```ignore
/// // Best-match selection across three stages: keep a fallback after each, then return
/// // to the highest-scoring one and resume from exactly there.
/// let mut txn = input.begin_stacked();
///
/// let mut best = None;
/// let mut best_score = i32::MIN;
/// for _ in 0..3 {
///   let score = parse_one_stage(&mut txn);         // parse through the guard (DerefMut)
///   let sp = txn.savepoint();                       // fallback point after this stage
///   if score > best_score {
///     best_score = score;
///     best = Some(sp);
///   }
/// }
///
/// if let Some(sp) = best {
///   txn.rollback_to(sp);   // resume right after the best stage; younger savepoints die
/// }
/// txn.commit();            // keep the winning prefix
/// ```
pub struct StackedTransaction<'txn, 'inp, 'closure, L, Ctx, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  pub(super) input: &'txn mut InputRef<'inp, 'closure, L, Ctx, Lang>,
  /// The begin point. `Some` while the transaction is undecided; `None` once
  /// [`commit`](Self::commit) / [`rollback`](Self::rollback) (or a rolling-back drop)
  /// has consumed it. Routing the whole-transaction decision through this one
  /// `Option::take` is what keeps `commit`, `rollback`, and `Drop` from restoring the
  /// base twice — the same funnel the plain [`Transaction`](super::Transaction) uses.
  pub(super) base: Option<Checkpoint<'inp, 'closure, L>>,
  /// The live savepoints, youngest last. Each entry pairs a savepoint's `seq` with the
  /// checkpoint saved at that mark. `rollback_to` / `release` truncate this vector from
  /// the top, which is what makes destroy-younger structural rather than a runtime check.
  pub(super) saves: Vec<(u64, Checkpoint<'inp, 'closure, L>)>,
  /// This transaction's nonce, stamped into every [`SavepointId`] it issues.
  pub(super) txn_nonce: u64,
  /// The next savepoint `seq` to hand out.
  pub(super) next_seq: u64,
}

impl<'inp, L, Ctx, Lang: ?Sized> StackedTransaction<'_, 'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Marks the current position as a savepoint and returns its id (SQL `SAVEPOINT`).
  ///
  /// The returned [`SavepointId`] stays usable for [`rollback_to`](Self::rollback_to)
  /// and [`release`](Self::release) until an older savepoint destroys it or it is
  /// released.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn savepoint(&mut self) -> SavepointId {
    let seq = self.next_seq;
    self.next_seq += 1;
    let ckp = self.input.save();
    self.saves.push((seq, ckp));
    SavepointId {
      txn_nonce: self.txn_nonce,
      seq,
    }
  }

  /// Rolls back to `sp` (SQL `ROLLBACK TO`): returns the input to `sp`'s position —
  /// cursor, span, lexer state, emission log, dedup watermark, and poison boundary all
  /// restored — and destroys every savepoint created after it, while keeping `sp` itself
  /// valid for a later rollback.
  ///
  /// `Checkpoint` is single-use, so keeping `sp` reusable is done by restoring the stored
  /// checkpoint and immediately re-saving at the now-current position, swapping the fresh
  /// checkpoint into `sp`'s slot. This preserves the classic SQL loop of rolling back to
  /// the same savepoint any number of times; it costs one extra `O(1)` save per call on
  /// this cold path.
  ///
  /// # Panics
  ///
  /// Panics if `sp` was issued by a different transaction
  /// (`stacked transaction: savepoint belongs to a different transaction`) or was
  /// destroyed by an earlier `rollback_to` / [`release`](Self::release)
  /// (`stacked transaction: savepoint is stale (destroyed by an earlier rollback or
  /// release)`). Both checks — a nonce compare and a short stack scan — run in every
  /// build.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn rollback_to(&mut self, sp: SavepointId) {
    let idx = self.slot(sp);
    // Drop the younger savepoints' checkpoints. Their live-checkpoint ids are still on
    // the input stack at this point; the restore below pops the stack down through the
    // target, which sweeps every one of them off in the same step.
    self.saves.truncate(idx + 1);
    let (seq, ckp) = self
      .saves
      .pop()
      .expect("slot() returned a valid index into `saves`");
    // Restore consumes the stored checkpoint; re-save at the restored position and
    // reinstall it under the same `seq` so `sp` survives for repeated rollbacks.
    self.input.restore(ckp);
    let fresh = self.input.save();
    self.saves.push((seq, fresh));
  }

  /// Releases `sp` (SQL `RELEASE SAVEPOINT`): forgets `sp` and every savepoint created
  /// after it, **keeping the parsed progress**. The input position does not move.
  ///
  /// # Panics
  ///
  /// Same as [`rollback_to`](Self::rollback_to): a foreign or already-destroyed id panics.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn release(&mut self, sp: SavepointId) {
    let idx = self.slot(sp);
    // Forget from the youngest down to `sp` inclusive, so each removed id is the
    // live-stack top when it is forgotten (the `O(1)` fast path). Progress is kept: no
    // checkpoint is restored.
    while self.saves.len() > idx {
      let (_, ckp) = self.saves.pop().expect("len > idx implies a value to pop");
      #[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
      self.input.forget_checkpoint(ckp.ckp_id);
      #[cfg(not(all(debug_assertions, any(feature = "std", feature = "alloc"))))]
      let _ = ckp;
    }
  }

  /// Validates `sp` and returns its index in `saves`, panicking on a foreign or a
  /// destroyed id. Two integer compares plus a short scan, on a cold path.
  #[cfg_attr(not(tarpaulin), inline)]
  fn slot(&self, sp: SavepointId) -> usize {
    assert!(
      sp.txn_nonce == self.txn_nonce,
      "stacked transaction: savepoint belongs to a different transaction"
    );
    match self.saves.iter().position(|(seq, _)| *seq == sp.seq) {
      Some(idx) => idx,
      None => panic!(
        "stacked transaction: savepoint is stale (destroyed by an earlier rollback or release)"
      ),
    }
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> StackedTransaction<'_, 'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Commits the whole transaction: keeps every parsed byte and forgets all savepoints
  /// and the begin point without restoring.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn commit(mut self) {
    // Forget youngest-first (each is the live-stack top when popped), then the base last
    // (it is the deepest, so it is the top once the savepoints are gone). Taking `base`
    // leaves the `Drop` guard nothing to restore.
    while let Some((_, ckp)) = self.saves.pop() {
      #[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
      self.input.forget_checkpoint(ckp.ckp_id);
      #[cfg(not(all(debug_assertions, any(feature = "std", feature = "alloc"))))]
      let _ = ckp;
    }
    if let Some(base) = self.base.take() {
      #[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
      self.input.forget_checkpoint(base.ckp_id);
      #[cfg(not(all(debug_assertions, any(feature = "std", feature = "alloc"))))]
      let _ = base;
    }
  }

  /// Rolls the whole transaction back to the begin point, discarding every savepoint and
  /// all parsed progress.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn rollback(mut self) {
    // Restoring the base pops the live stack down through it, carrying off every
    // savepoint id in one step; the savepoint checkpoints then just drop with `self`.
    if let Some(base) = self.base.take() {
      self.input.restore(base);
    }
  }
}

impl<'inp, 'closure, L, Ctx, Lang: ?Sized> Deref
  for StackedTransaction<'_, 'inp, 'closure, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  type Target = InputRef<'inp, 'closure, L, Ctx, Lang>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.input
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> DerefMut for StackedTransaction<'_, 'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.input
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> Drop for StackedTransaction<'_, 'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Rolls an undecided transaction back to the begin point (database default:
  /// uncommitted work is discarded). After [`commit`](Self::commit) /
  /// [`rollback`](Self::rollback) the base is already taken, so this is a no-op. Silent
  /// rather than a panicking drop-bomb: tokit is `no_std`, where `thread::panicking()`
  /// is unavailable, so a panic here could double-panic during unwinding.
  #[cfg_attr(not(tarpaulin), inline)]
  fn drop(&mut self) {
    if let Some(base) = self.base.take() {
      self.input.restore(base);
    }
  }
}

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests;
