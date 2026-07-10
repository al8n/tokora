use core::ops::{Deref, DerefMut};

use super::{Checkpoint, InputRef, Lexer, ParseContext};

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
/// transaction; an undecided transaction rolls back on drop. Zero-cost:
/// [`begin`](InputRef::begin) performs exactly one [`save`](InputRef::save), the guard
/// is two words, and deciding is one branch — there is no journaling, because the input
/// source is immutable and rewinding is a snapshot copy.
///
/// Use `Transaction` for imperative flows with several exits (loops, `match` arms);
/// [`attempt`](InputRef::attempt)/[`try_attempt`](InputRef::try_attempt) for
/// single-closure speculation; raw [`save`](InputRef::save)/[`restore`](InputRef::restore)
/// only where neither shape fits.
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
pub struct Transaction<'txn, 'inp, 'closure, L, Ctx, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  pub(super) input: &'txn mut InputRef<'inp, 'closure, L, Ctx, Lang>,
  /// `Some` while the transaction is undecided; `None` once
  /// [`commit`](Self::commit)/[`rollback`](Self::rollback) (or a rolling-back drop)
  /// has consumed it. Routing every decision through this one `Option::take` is what
  /// keeps `commit`, `rollback`, and `Drop` from ever restoring twice.
  pub(super) ckp: Option<Checkpoint<'inp, 'closure, L>>,
}

impl<'inp, L, Ctx, Lang: ?Sized> Transaction<'_, 'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Commits the transaction: keeps the progress parsed through the guard and drops the
  /// begin-point checkpoint without restoring.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn commit(mut self) {
    // Take the checkpoint so the `Drop` guard below sees `None` and does not roll back.
    if let Some(ckp) = self.ckp.take() {
      // Kept, not restored: drop its debug-witness id so it does not linger on the
      // live stack across commit-heavy loops.
      #[cfg(all(
        debug_assertions,
        any(feature = "std", feature = "alloc"),
        target_has_atomic = "ptr"
      ))]
      self.input.forget_checkpoint(ckp.ckp_id);
      #[cfg(not(all(
        debug_assertions,
        any(feature = "std", feature = "alloc"),
        target_has_atomic = "ptr"
      )))]
      let _ = ckp;
    }
  }

  /// Rolls the transaction back: returns the input to the begin point — position, span,
  /// lexer state, emission log, dedup watermark, and poison boundary all restored.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn rollback(mut self) {
    if let Some(ckp) = self.ckp.take() {
      self.input.restore(ckp);
    }
  }
}

impl<'inp, 'closure, L, Ctx, Lang: ?Sized> Deref for Transaction<'_, 'inp, 'closure, L, Ctx, Lang>
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

impl<'inp, L, Ctx, Lang: ?Sized> DerefMut for Transaction<'_, 'inp, '_, L, Ctx, Lang>
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

impl<'inp, L, Ctx, Lang: ?Sized> Drop for Transaction<'_, 'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Rolls back an undecided transaction (database default: uncommitted work is
  /// discarded). After [`commit`](Self::commit)/[`rollback`](Self::rollback) the
  /// checkpoint is already taken, so this is a no-op. Silent rather than a panicking
  /// drop-bomb: tokit is `no_std`, where `thread::panicking()` is unavailable, so a
  /// panic here could double-panic during unwinding.
  #[cfg_attr(not(tarpaulin), inline)]
  fn drop(&mut self) {
    if let Some(ckp) = self.ckp.take() {
      self.input.restore(ckp);
    }
  }
}

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests;
