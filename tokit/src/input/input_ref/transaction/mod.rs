use core::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

use super::{
  Checkpoint, InputRef, Lexer, ParseContext,
  drop_policy::{DropPolicy, Rollback},
};

/// A scoped backtracking transaction over an [`InputRef`].
///
/// Semantically identical to [`save`](InputRef::save)/[`restore`](InputRef::restore),
/// with the restore discipline enforced by the borrow checker: while a nested
/// transaction is alive, its parent is inaccessible, so out-of-order rollbacks — the
/// one contract violation [`restore`](InputRef::restore) documents — do not compile.
/// Nested transactions behave like database savepoints: rolling back a parent discards
/// everything its children committed.
///
/// [`commit`](Self::commit) and [`rollback`](Self::rollback) both consume the
/// transaction and are available whatever the policy. Zero-cost:
/// [`begin`](InputRef::begin) performs exactly one [`save`](InputRef::save), the guard
/// is two words, and deciding is one branch — there is no journaling, because the input
/// source is immutable and rewinding is a snapshot copy.
///
/// # Drop policy
///
/// The final type parameter `P` is a compile-time [`DropPolicy`](super::DropPolicy) that
/// fixes what an *undecided* guard does on drop:
///
/// - [`Rollback`](super::Rollback) (the default, from [`begin`](InputRef::begin)) — drop
///   restores to the begin point; uncommitted speculative work is discarded.
/// - [`Commit`](super::Commit) (from [`begin_with`](InputRef::begin_with)) — drop keeps
///   the progress, the dual used by commit-by-default loops.
///
/// # When to reach for it
///
/// Use `Transaction` for imperative flows with several exits (loops, `match` arms) —
/// [`begin`](InputRef::begin) for the speculative default, or
/// [`begin_with::<Commit>`](InputRef::begin_with) for a commit-by-default loop that keeps
/// progress on most exits and rolls back explicitly on the few that back out. Reach for
/// [`attempt`](InputRef::attempt)/[`try_attempt`](InputRef::try_attempt) for
/// single-closure speculation, and raw [`save`](InputRef::save)/[`restore`](InputRef::restore)
/// only where no guard shape fits.
///
/// # Compile-time last-in, first-out
///
/// A nested transaction mutably borrows its parent for as long as it is alive, so the
/// non-LIFO shape — deciding a parent while a child is still undecided — is a borrow
/// error, not a runtime panic:
///
/// ```compile_fail
/// use tokit::{InputRef, Lexer, ParseContext};
///
/// fn non_lifo<'inp, 'closure, L, Ctx>(input: &mut InputRef<'inp, 'closure, L, Ctx>)
/// where
///   L: Lexer<'inp>,
///   L::State: Clone,
///   Ctx: ParseContext<'inp, L>,
/// {
///   let mut outer = input.begin();
///   let mut inner = outer.begin();
///   outer.rollback(); // error: `outer` is mutably borrowed by `inner`
///   inner.commit();
/// }
/// ```
///
/// # Mixing with raw save/restore
///
/// The guard deref-coerces to [`InputRef`], so raw [`save`](InputRef::save) /
/// [`restore`](InputRef::restore) are reachable through it. A raw restore to a checkpoint saved
/// *before* the guard began rolls the lineage back past the guard's own begin-point checkpoint,
/// invalidating it. What each decision then does is deterministic in every allocator build: an
/// explicit [`rollback`](Self::rollback) panics as stale (`transaction base is stale`); a
/// rolling-back drop quietly keeps the older restore's state instead of resurrecting the dead
/// base; [`commit`](Self::commit) is unaffected (it never restores). A LIFO-clean raw
/// save/restore pair taken and released entirely *above* the begin point, and state surgery
/// (which is transactional), leave the guard's checkpoint intact. On allocator-less targets
/// there is no lineage stack, so this mixing is unspecified-but-bounded rather than checked.
pub struct Transaction<'txn, 'inp, 'closure, L, Ctx, Lang: ?Sized = (), P: DropPolicy = Rollback>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  pub(super) input: &'txn mut InputRef<'inp, 'closure, L, Ctx, Lang>,
  /// `Some` while the transaction is undecided; `None` once
  /// [`commit`](Self::commit)/[`rollback`](Self::rollback) (or a deciding drop) has
  /// consumed it. Routing every decision through this one `Option::take` is what keeps
  /// `commit`, `rollback`, and `Drop` from ever acting twice.
  pub(super) ckp: Option<Checkpoint<'inp, 'closure, L>>,
  /// The drop policy — [`Rollback`](super::Rollback) or [`Commit`](super::Commit) —
  /// carried as a zero-sized typestate. It selects, at compile time and branch-free, what
  /// an undecided guard's `Drop` does: restore to the begin point, or keep the progress.
  pub(super) _policy: PhantomData<P>,
}

impl<'inp, L, Ctx, Lang: ?Sized, P: DropPolicy> Transaction<'_, 'inp, '_, L, Ctx, Lang, P>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Commits the transaction: keeps the progress parsed through the guard and drops the
  /// begin-point checkpoint without restoring. Available whatever the drop policy.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn commit(mut self) {
    // Take the checkpoint so the `Drop` guard below sees `None` and does not roll back.
    if let Some(ckp) = self.ckp.take() {
      // Kept, not restored: drop its lineage id so it does not linger on the live stack
      // across commit-heavy loops.
      #[cfg(any(feature = "std", feature = "alloc"))]
      self.input.forget_checkpoint(ckp.ckp_id);
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      let _ = ckp;
    }
  }

  /// Rolls the transaction back: returns the input to the begin point — position, span,
  /// lexer state, emission log, dedup watermark, and poison boundary all restored.
  /// Available whatever the drop policy (a [`Commit`](super::Commit) guard can still be
  /// rolled back explicitly).
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn rollback(mut self) {
    if let Some(ckp) = self.ckp.take() {
      // A raw restore below the begin point (through this guard's `DerefMut`) pops the base
      // off the live lineage. An explicit rollback to an invalidated base cannot be honored —
      // it would resurrect an abandoned lineage — so refuse it loudly in every allocator build,
      // matching the [`StackedTransaction`] and savepoint precedent. (A rolling-back drop, which
      // may run mid-unwind, quietly skips the restore instead.)
      #[cfg(any(feature = "std", feature = "alloc"))]
      assert!(
        self.input.live_contains(ckp.ckp_id),
        "transaction base is stale (invalidated by an earlier restore)"
      );
      self.input.restore(ckp);
    }
  }
}

impl<'inp, 'closure, L, Ctx, Lang: ?Sized, P: DropPolicy> Deref
  for Transaction<'_, 'inp, 'closure, L, Ctx, Lang, P>
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
  for Transaction<'_, 'inp, '_, L, Ctx, Lang, P>
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

impl<'inp, L, Ctx, Lang: ?Sized, P: DropPolicy> Drop for Transaction<'_, 'inp, '_, L, Ctx, Lang, P>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Decides an undecided transaction according to its [`DropPolicy`](super::DropPolicy).
  /// After [`commit`](Self::commit)/[`rollback`](Self::rollback) the checkpoint is
  /// already taken, so this is a no-op whatever the policy.
  ///
  /// - [`Rollback`](super::Rollback): restore to the begin point (the database default,
  ///   uncommitted work discarded).
  /// - [`Commit`](super::Commit): keep the progress, only forgetting the checkpoint's
  ///   lineage id — identical to dropping a raw [`Checkpoint`], including during an error
  ///   `?`-propagation under a fail-fast emitter.
  ///
  /// `P::ROLLBACK_ON_DROP` is a compile-time constant, so each policy monomorphizes to
  /// one arm with the other eliminated. Either arm is silent (no debug raw-misuse panic):
  /// `Drop` may run while already unwinding, where `no_std` has no `thread::panicking()`
  /// to guard a drop-bomb. The begin point is usually the oldest live checkpoint; if a raw
  /// restore below it (through this guard) has invalidated it first, the rollback arm skips
  /// the rewind rather than resurrecting the dead base (the input already sits where that
  /// older restore left it), so nothing silent is ever restored to a stale lineage.
  #[cfg_attr(not(tarpaulin), inline)]
  fn drop(&mut self) {
    if let Some(ckp) = self.ckp.take() {
      if P::ROLLBACK_ON_DROP {
        // Skip the rewind if a raw restore below the base already invalidated it: the input
        // sits where that older restore left it, and resurrecting the dead base would be wrong
        // (and this may run mid-unwind, where panicking is forbidden). An explicit `rollback`
        // reports that stale case loudly instead; here we stay silent and truthful.
        self.input.restore_unchecked_if_live(ckp);
      } else {
        // Commit-on-drop: progress kept; only tidy the committed checkpoint's lineage id
        // so it does not linger on the live stack across commit-heavy loops (as
        // `commit` does).
        #[cfg(any(feature = "std", feature = "alloc"))]
        self.input.forget_checkpoint(ckp.ckp_id);
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        let _ = ckp;
      }
    }
  }
}

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests;
