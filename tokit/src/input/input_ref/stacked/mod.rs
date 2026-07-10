use core::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

use super::super::SavepointStack;
use super::{
  Checkpoint, InputRef, Lexer, ParseContext,
  drop_policy::{DropPolicy, Rollback},
};

/// An opaque handle to one savepoint inside a [`StackedTransaction`].
///
/// Returned by [`savepoint`](StackedTransaction::savepoint) and consumed by
/// [`rollback_to`](StackedTransaction::rollback_to) and
/// [`release`](StackedTransaction::release). It is a small `Copy` token that holds no
/// runtime borrow, so it can be stashed in a list of candidates or returned up the call
/// stack while the transaction stays open.
///
/// # Not a durable position token
///
/// A `SavepointId` is *branded* with the lifetime of the transaction that issued it, so
/// it cannot outlive that transaction or the input loan behind it — keeping one past the
/// parse is a compile error, not a dangling handle. For a position that must survive
/// beyond the transaction, capture a [`Cursor`](super::Cursor) or a span instead.
///
/// # How a misused id is caught
///
/// Identity is layered, from compile time down to a runtime check, with no global state
/// — no counter, no atomic — behind any of it:
///
/// - **Temporal misuse** — using an id after its transaction ended, or holding one across
///   the next [`begin_stacked`](InputRef::begin_stacked) on the same input — is a
///   **compile error**: the id's invariant lifetime brand keeps the input loan open while
///   the id is live, so the borrow checker rejects reopening it.
/// - **Cross-parser misuse** — an id from another, simultaneously-live transaction over a
///   *different* input — **panics in every build**: each id carries the address of an
///   Input-owned field, and two live inputs occupy distinct addresses.
/// - **Intra-transaction staleness** — an id destroyed by an earlier `rollback_to` /
///   `release` on the same transaction, or one whose checkpoint a raw restore below it
///   invalidated (see the mixing rules on [`StackedTransaction`]) — **panics in every
///   build** via a membership scan of the live savepoints and their lineage (see
///   [`rollback_to`](StackedTransaction::rollback_to)). State surgery is transactional and
///   does *not* invalidate a savepoint.
///
/// # Compile-time rejections
///
/// The temporal and nesting misuses never reach a runtime check — the borrow checker
/// rejects them. Each snippet below fails to compile.
///
/// Reusing an id after its transaction ended (here, across the next `begin_stacked` on the
/// same input) — the id keeps the first transaction's loan on `input` open, so the second
/// `begin_stacked` cannot re-borrow it:
///
/// ```compile_fail
/// use tokit::{InputRef, Lexer, ParseContext};
///
/// fn temporal_misuse<'inp, L, Ctx>(input: &mut InputRef<'inp, '_, L, Ctx>)
/// where
///   L: Lexer<'inp>,
///   L::State: Clone,
///   Ctx: ParseContext<'inp, L>,
/// {
///   let sp = {
///     let mut txn = input.begin_stacked();
///     let sp = txn.savepoint();
///     txn.commit();
///     sp
///   };
///   let mut txn2 = input.begin_stacked(); // error[E0499]: `input` is still borrowed by `sp`
///   txn2.rollback_to(sp);
/// }
/// ```
///
/// Storing an id past the parse (returning it out of the transaction) — the brand cannot
/// outlive the input loan:
///
/// ```compile_fail
/// use tokit::{InputRef, Lexer, ParseContext, SavepointId};
///
/// fn durable_misuse<'inp, 'closure, L, Ctx>(
///   input: &mut InputRef<'inp, 'closure, L, Ctx>,
/// ) -> SavepointId<'closure>
/// where
///   L: Lexer<'inp>,
///   L::State: Clone,
///   Ctx: ParseContext<'inp, L>,
/// {
///   let mut txn = input.begin_stacked();
///   let sp = txn.savepoint();
///   txn.commit();
///   sp // error: the id's brand does not outlive the transaction's loan
/// }
/// ```
///
/// Passing a parent savepoint into a nested child transaction — the parent's brand region
/// strictly contains the child's (the parent is used before the child exists), so the
/// invariant brands cannot unify:
///
/// ```compile_fail
/// use tokit::{InputRef, Lexer, ParseContext};
///
/// fn parent_id_in_child<'inp, L, Ctx>(input: &mut InputRef<'inp, '_, L, Ctx>)
/// where
///   L: Lexer<'inp>,
///   L::State: Clone,
///   Ctx: ParseContext<'inp, L>,
/// {
///   let mut parent = input.begin_stacked();
///   let sp_parent = parent.savepoint();
///   let mut child = parent.begin_stacked();
///   child.rollback_to(sp_parent); // error[E0597]: brands cannot unify parent with child
/// }
/// ```
///
/// The mirror — keeping a child savepoint to use in the parent after the child ends — also
/// fails: the child id keeps the child's borrow of the parent alive, so the parent cannot
/// be used:
///
/// ```compile_fail
/// use tokit::{InputRef, Lexer, ParseContext};
///
/// fn child_id_in_parent<'inp, L, Ctx>(input: &mut InputRef<'inp, '_, L, Ctx>)
/// where
///   L: Lexer<'inp>,
///   L::State: Clone,
///   Ctx: ParseContext<'inp, L>,
/// {
///   let mut parent = input.begin_stacked();
///   let sp_child = {
///     let mut child = parent.begin_stacked();
///     let s = child.savepoint();
///     child.commit();
///     s
///   };
///   parent.rollback_to(sp_child); // error: `parent` is still borrowed by the child id
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SavepointId<'t> {
  /// The savepoint's sequence number, drawn from an input-global counter that is never
  /// reset across the input's transactions. A seq is therefore unique for the whole life
  /// of the input, so an id that crosses transactions on one input (a nested or a
  /// sequential one) can never collide with a live savepoint's seq in another
  /// transaction's stack — the membership scan then panics deterministically as stale.
  seq: u64,
  /// The address of the issuing input's `poison_boundary` field, captured at
  /// [`begin_stacked`](InputRef::begin_stacked). Two simultaneously-live inputs are
  /// distinct structs at distinct addresses, so this separates their transactions. The
  /// lifetime brand — not this address — rules out the address-reuse case where a dropped
  /// input's slot is later reallocated, because a live id keeps its own input's loan open.
  nonce: usize,
  /// Invariant in `'t`, the transaction's borrow of the input. The fn-pointer form (not a
  /// bare reference, which would be covariant and defeat the brand) is what keeps the id
  /// from outliving its loan and makes a parent/child swap under nesting fail to unify.
  _brand: PhantomData<fn(&'t ()) -> &'t ()>,
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
/// holds because the internal stack only ever shrinks from the top. A misused
/// [`SavepointId`] is rejected in layers: a temporally-misused id at compile time via its
/// lifetime brand, a foreign id from another live parser and a stale id both by a runtime
/// check in every build — see [`SavepointId`].
///
/// [`commit`](Self::commit) and [`rollback`](Self::rollback) consume the transaction and
/// are available whatever the drop policy. What an *undecided* transaction does on drop is
/// the compile-time [`DropPolicy`](super::DropPolicy) `P`: the default
/// [`Rollback`](super::Rollback) (from [`begin_stacked`](InputRef::begin_stacked)) rolls
/// back to the begin point, discarding all savepoints — the database default;
/// [`Commit`](super::Commit) (from [`begin_stacked_with`](InputRef::begin_stacked_with))
/// keeps the progress. Cost when unused is low: the transaction's own savepoint `Vec`
/// never allocates until the first [`savepoint`](Self::savepoint), and a begin captures one
/// field address and records its base checkpoint on the input's shared lineage stack (an
/// amortized `Vec` push) — no counter, no atomic.
///
/// # Mixing with raw save/restore, state surgery, and nested transactions
///
/// The guard deref-coerces to [`InputRef`], so raw [`save`](InputRef::save) /
/// [`restore`](InputRef::restore) and the nested backtracking tools are all reachable
/// through it. Two rules govern how they interact with the live savepoints:
///
/// - **A raw restore below a savepoint invalidates it.** Restoring a raw checkpoint taken
///   *before* a savepoint rolls the lineage back past the savepoint's own checkpoint, so the
///   savepoint's checkpoint is no longer on a live lineage, and
///   [`rollback_to`](Self::rollback_to) / [`release`](Self::release) with it **panics as
///   stale in every build** — release and no-`target_has_atomic`-ptr targets included — not
///   just debug witness builds. Restoring the wrong lineage is never silently honored.
/// - **State surgery, nested `attempt` / `try_attempt` / [`Transaction`](super::Transaction),
///   and a LIFO-clean raw save/restore pair taken *above* the savepoints, are all legal and
///   do not disturb the savepoints.** [`set_state`](InputRef::set_state) /
///   [`state_mut`](InputRef::state_mut) re-key the forward-scanning facts but are
///   transactional — a savepoint taken before the surgery stays valid, and
///   [`rollback_to`](Self::rollback_to) it *undoes* the surgery (the regime, boundary,
///   watermark, and position all return). A nested speculation that saves and then restores
///   or commits its own younger checkpoint leaves every savepoint below it untouched.
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
pub struct StackedTransaction<
  'txn,
  'inp,
  'closure,
  L,
  Ctx,
  Lang: ?Sized = (),
  P: DropPolicy = Rollback,
> where
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
  pub(super) saves: SavepointStack<'inp, 'closure, L>,
  /// The address of this input's `poison_boundary` field, stamped into every
  /// [`SavepointId`] this transaction issues. It separates this input's savepoints from
  /// those of another simultaneously-live input, which sits at a distinct address (see
  /// [`SavepointId`]).
  pub(super) nonce: usize,
  /// The drop policy — [`Rollback`](super::Rollback) or [`Commit`](super::Commit) —
  /// carried as a zero-sized typestate. It selects, at compile time and branch-free, what
  /// an undecided guard's `Drop` does: roll back to the begin point, or keep the progress.
  pub(super) _policy: PhantomData<P>,
}

impl<'txn, 'inp, L, Ctx, Lang: ?Sized, P: DropPolicy>
  StackedTransaction<'txn, 'inp, '_, L, Ctx, Lang, P>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Marks the current position as a savepoint and returns its id (SQL `SAVEPOINT`).
  ///
  /// The returned [`SavepointId`] stays usable for [`rollback_to`](Self::rollback_to)
  /// and [`release`](Self::release) until an older savepoint destroys it or it is
  /// released. Its lifetime is branded to this transaction, so it cannot escape the
  /// transaction's scope.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn savepoint(&mut self) -> SavepointId<'txn> {
    // Sequence numbers come from the input, not this transaction, so they never reset:
    // an id can only ever match the one live slot that pushed it (or none, if stale).
    let seq = self.input.next_savepoint_seq();
    let ckp = self.input.save();
    self.saves.push((seq, ckp));
    SavepointId {
      seq,
      nonce: self.nonce,
      _brand: PhantomData,
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
  /// Panics if `sp` was issued by a different, simultaneously-live transaction
  /// (`stacked transaction: savepoint belongs to a different transaction`), was destroyed by
  /// an earlier `rollback_to` / [`release`](Self::release) (`stacked transaction: savepoint
  /// is stale (destroyed by an earlier rollback or release)`), or had its checkpoint
  /// invalidated by a raw restore below it through the transaction (`stacked transaction:
  /// savepoint is stale (invalidated by a raw restore below it)`). All three checks — an
  /// address compare and two short stack scans — run in every build. (Using an id after its
  /// transaction ended is a compile error, not a panic; see [`SavepointId`]. State surgery is
  /// transactional and does *not* invalidate a savepoint — one taken before it stays valid,
  /// and rolling back to it undoes the surgery.)
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn rollback_to(&mut self, sp: SavepointId<'txn>) {
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
  pub fn release(&mut self, sp: SavepointId<'txn>) {
    let idx = self.slot(sp);
    // Forget from the youngest down to `sp` inclusive, so each removed id is the
    // live-stack top when it is forgotten (the `O(1)` fast path). Progress is kept: no
    // checkpoint is restored.
    while self.saves.len() > idx {
      let (_, ckp) = self.saves.pop().expect("len > idx implies a value to pop");
      self.input.forget_checkpoint(ckp.ckp_id);
    }
  }

  /// Validates `sp` and returns its index in `saves`, panicking on a foreign, a destroyed,
  /// or a lineage-invalidated id. An address compare plus two short scans, on a cold path.
  #[cfg_attr(not(tarpaulin), inline)]
  fn slot(&self, sp: SavepointId<'txn>) -> usize {
    assert!(
      sp.nonce == self.nonce,
      "stacked transaction: savepoint belongs to a different transaction"
    );
    let idx = match self.saves.iter().position(|(seq, _)| *seq == sp.seq) {
      Some(idx) => idx,
      None => panic!(
        "stacked transaction: savepoint is stale (destroyed by an earlier rollback or release)"
      ),
    };
    // Lineage validity, in every build: the `seq` is still in `saves`, but the checkpoint
    // that slot marks must still be live on the input's lineage stack. A raw restore through
    // the transaction (via `DerefMut`) to a checkpoint older than this savepoint pops it off
    // that stack without touching `saves` — leaving a stale slot that the nonce + membership
    // check alone would honor, restoring the wrong lineage. (State surgery does NOT reach
    // here: it is transactional and leaves the lineage stack intact, so a savepoint taken
    // before it stays live and rolling back to it undoes the surgery.) This is a plain `Vec`
    // membership scan (no atomics), so it closes the hole on release and
    // no-`target_has_atomic`-ptr targets exactly as in a debug witness build.
    assert!(
      self.input.live_contains(self.saves[idx].1.ckp_id),
      "stacked transaction: savepoint is stale (invalidated by a raw restore below it)"
    );
    idx
  }
}

impl<'inp, L, Ctx, Lang: ?Sized, P: DropPolicy> StackedTransaction<'_, 'inp, '_, L, Ctx, Lang, P>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Commits the whole transaction: keeps every parsed byte and forgets all savepoints
  /// and the begin point without restoring. Available whatever the drop policy.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn commit(mut self) {
    // Forget youngest-first (each is the live-stack top when popped), then the base last
    // (it is the deepest, so it is the top once the savepoints are gone). Taking `base`
    // leaves the `Drop` guard nothing to restore.
    while let Some((_, ckp)) = self.saves.pop() {
      self.input.forget_checkpoint(ckp.ckp_id);
    }
    if let Some(base) = self.base.take() {
      self.input.forget_checkpoint(base.ckp_id);
    }
  }

  /// Rolls the whole transaction back to the begin point, discarding every savepoint and
  /// all parsed progress. Available whatever the drop policy (a [`Commit`](super::Commit)
  /// guard can still be rolled back explicitly).
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn rollback(mut self) {
    // Restoring the base pops the live stack down through it, carrying off every
    // savepoint id in one step; the savepoint checkpoints then just drop with `self`.
    if let Some(base) = self.base.take() {
      self.input.restore(base);
    }
  }
}

impl<'inp, 'closure, L, Ctx, Lang: ?Sized, P: DropPolicy> Deref
  for StackedTransaction<'_, 'inp, 'closure, L, Ctx, Lang, P>
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

impl<'inp, L, Ctx, Lang: ?Sized, P: DropPolicy> DerefMut
  for StackedTransaction<'_, 'inp, '_, L, Ctx, Lang, P>
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

impl<'inp, L, Ctx, Lang: ?Sized, P: DropPolicy> Drop
  for StackedTransaction<'_, 'inp, '_, L, Ctx, Lang, P>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Decides an undecided transaction according to its [`DropPolicy`](super::DropPolicy).
  /// After [`commit`](Self::commit) / [`rollback`](Self::rollback) the base and savepoints
  /// are already taken, so this is a no-op whatever the policy.
  ///
  /// - [`Rollback`](super::Rollback): roll back to the begin point (the database default,
  ///   all savepoints and progress discarded). Restoring the base pops the live stack down
  ///   through it, carrying off every savepoint id in one step.
  /// - [`Commit`](super::Commit): keep the progress, forgetting every savepoint id
  ///   (youngest first) then the base — the same lineage-id hygiene as
  ///   [`commit`](Self::commit).
  ///
  /// `P::ROLLBACK_ON_DROP` is a compile-time constant, so each policy monomorphizes to one
  /// arm with the other eliminated. The rollback arm is silent (unchecked): `Drop` may run
  /// while already unwinding, where `no_std` has no `thread::panicking()` to guard a
  /// drop-bomb; the base is the oldest live checkpoint, so no misuse check is lost, and if
  /// a raw restore below it already invalidated it the lineage pop no-ops.
  #[cfg_attr(not(tarpaulin), inline)]
  fn drop(&mut self) {
    if P::ROLLBACK_ON_DROP {
      if let Some(base) = self.base.take() {
        self.input.restore_unchecked(base);
      }
    } else {
      // Commit-on-drop: progress kept; forget every savepoint id (youngest first, each the
      // live-stack top when popped) then the base, so nothing lingers on the live stack.
      while let Some((_, ckp)) = self.saves.pop() {
        self.input.forget_checkpoint(ckp.ckp_id);
      }
      if let Some(base) = self.base.take() {
        self.input.forget_checkpoint(base.ckp_id);
      }
    }
  }
}

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests;
