//! The **session cell** of an [`InputRef`](super::InputRef): the input's lineage memos and the
//! handle's emitter borrow, married to the session-point stack that must release into them.
//!
//! # Why these three live in one cell
//!
//! A [session point](super::InputRef::begin_point) is split across two owners. Its
//! [`Checkpoint`] must live on the *handle* — it carries a mark into the log of the emitter the
//! handle borrows, so it is meaningful only within that borrow (see the contract on
//! [`begin_point`](super::InputRef::begin_point)). Its **pin** and its **live-checkpoint id** live in
//! the [`Lineage`] memos on the *input*, which outlives the handle — and its **emitter mark** is
//! bookkeeping keyed inside the borrowed emitter (a `Sink`'s mark-stack row, for instance),
//! which also outlives the handle. A point abandoned outright — the handle dropped without
//! [`commit_point`](super::InputRef::commit_point) or
//! [`rollback_point`](super::InputRef::rollback_point) — must therefore release bookkeeping it does
//! not own, at a moment it does not control: its own destruction.
//!
//! That is what this cell is for. It owns all three halves — the lineage memos, the emitter
//! borrow, and the checkpoints — so its [`Drop`] can settle an abandoned point exactly as
//! [`commit_point`](super::InputRef::commit_point) would: unpin, release the lineage entry, and
//! [`release`](Emitter::release) the emitter mark, structurally rather than as a rule callers
//! must remember. (The emitter borrow living here is also why the handle's `emitter()` accessor
//! reads through the cell.)
//!
//! # Why it is a cell and not fields on the handle
//!
//! Because the destructor's *reach* is the destructor's *cost*. A `Drop` impl hands its type's
//! address to an opaque function, so every field of that type escapes and must be materialized in
//! memory. Put the `Drop` on [`InputRef`](super::InputRef) itself and the whole handle escapes — the
//! cache, the span, the lexer state — and the scanner's hot loops reload them from
//! memory instead of keeping them in registers (measured: ~40% on the tightest `try_expect` and
//! dispatch benches). Put it on this cell and only these five words escape — and of them only the
//! point stack is ever written after construction (the two borrows are set once), so the handle
//! keeps the layout, and the codegen, it had before session points existed.

use core::marker::PhantomData;

use crate::emitter::Emitter;

use super::{Checkpoint, Lexer, Lineage};

/// The lineage memos and emitter borrow an [`InputRef`](super::InputRef) writes through, together
/// with its live session points — see the [module docs](self) for why they are one cell.
pub(crate) struct Session<'inp, 'closure, L, E, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
{
  /// The input's lineage memos (see [`Lineage`]): the live-checkpoint stack, the pin set, and the
  /// cache-push/checkpoint-id/savepoint counters, reached only through their operations.
  /// [`save`](super::InputRef::save) / [`restore`](super::InputRef::restore) /
  /// [`commit`](super::InputRef::commit) and the transaction guards are the sole writers — plus this
  /// cell's [`Drop`], the one writer that is not a verb.
  pub(super) lineage: &'closure mut Lineage,
  /// The handle's emitter borrow — the ground-truth emission log every diagnostic and event
  /// flows to, rolled back by truncation to a saved mark. It lives in this cell (rather than as
  /// a sibling field on the handle) so the abandoning [`Drop`] can
  /// [`release`](Emitter::release) an open point's emitter mark: mark-keyed emitter bookkeeping
  /// (an event sink's checkpoint stack) would otherwise strand one row per abandoned point, the
  /// pin-leak class one layer up.
  pub(super) emitter: &'closure mut E,
  /// The live **session points**, oldest first: the checkpoints
  /// [`begin_point`](super::InputRef::begin_point) has saved and neither committed nor rolled back.
  /// The vector *is* the last-in, first-out stack — [`commit_point`](super::InputRef::commit_point)
  /// and [`rollback_point`](super::InputRef::rollback_point) pop its back — so nesting is structural
  /// and needs no id validation.
  ///
  /// It is the one **owned** thing on the otherwise all-borrowed handle, and that is the point: a
  /// session point is a value, not a borrow, so opening one leaves nothing borrowed and the consume
  /// surface stays callable — the non-lexical property a [`Transaction`](super::Transaction) guard
  /// cannot have. It never allocates until the first
  /// [`begin_point`](super::InputRef::begin_point). Gated to the allocator builds, exactly like the
  /// guards' savepoint stack.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub(super) points: std::vec::Vec<Checkpoint<'inp, 'closure, L>>,
  /// Allocator-less builds keep no point stack (there is no session-point surface without an
  /// allocator), but the cell still names `'inp` and `L` through the checkpoint type it would hold.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  _points: PhantomData<Checkpoint<'inp, 'closure, L>>,
  /// Ties the cell to `'inp`/`L` in every configuration, so the allocator gate above cannot change
  /// its variance.
  _m: PhantomData<fn(&'inp ()) -> &'inp ()>,
  /// Names `Lang` (the emitter bound's dialect parameter) in the type, exactly as the handle's
  /// own `_marker` does.
  _lang: PhantomData<Lang>,
}

impl<'inp, 'closure, L, E, Lang: ?Sized> Session<'inp, 'closure, L, E, Lang>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
{
  /// A fresh session cell over `lineage` and the handle's `emitter` borrow: no points open, and —
  /// in allocator builds — an unallocated stack. A handle that never opens a session pays for the
  /// zeroed words once, at [`Input::as_ref`](crate::input::Input), and nothing thereafter.
  #[inline(always)]
  pub(crate) const fn new(lineage: &'closure mut Lineage, emitter: &'closure mut E) -> Self {
    Self {
      lineage,
      emitter,
      #[cfg(any(feature = "std", feature = "alloc"))]
      points: std::vec::Vec::new(),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _points: PhantomData,
      _m: PhantomData,
      _lang: PhantomData,
    }
  }

  /// CELL_CENSUS — this cell's half of the [taxonomy](super::super::lineage) tripwire, kept here
  /// because these fields are private to this module and the main census cannot see them.
  ///
  /// Same discipline, same reason: the exhaustive destructure — no `..` — makes a new field on this
  /// cell a **compile error**, so it cannot be added without deciding its class and its restore
  /// semantics. Generic and never instantiated: type-checked in every build, zero bytes of code.
  #[allow(dead_code)]
  fn census(&self) {
    let Self {
      // — lineage memos (borrowed): the live-checkpoint stack, the pin set, and the counters.
      lineage: _,
      // — ground truth (borrowed): the emission log, rolled back by truncation to the saved
      //   mark. Held in this cell so the abandoning drop can release an open point's mark.
      emitter: _,
      // — lineage memo, handle-local: the open session points. A restore does NOT rewind this
      //   stack; a rewind reaching *below* an open point is refused outright by that point's pin
      //   (`Lineage::assert_restore_preserves_pins`), so the stack cannot be left describing a
      //   lineage that no longer exists.
      #[cfg(any(feature = "std", feature = "alloc"))]
        points: _,
      #[cfg(not(any(feature = "std", feature = "alloc")))]
        _points: _,
      // — ZSTs.
      _m: _,
      _lang: _,
    } = self;
  }

  /// Releases every point still open — the whole body of the abandoning [`Drop`], deliberately
  /// **outlined**.
  ///
  /// Reached only by a handle that abandons open points, which no correct driver does on a hot path,
  /// so `#[cold]` + `#[inline(never)]` keeps it out of the caller entirely and leaves the drop itself
  /// a single `is_empty` branch. That is not cosmetic: [`Lineage::unpin`] and [`Lineage::forget`] are
  /// `inline(always)` stack scans, and a destructor is emitted at *every* unwind edge of its owner —
  /// so inlining this loop would paste two `SmallVec` searches and the checkpoint drop glue into
  /// every landing pad of the scanner's tightest loops. Measured, that cost the `try_expect` hit path
  /// ~27% and the fused dispatch path ~45%; outlined, both are back at parity.
  ///
  /// Newest-first, so [`Lineage::unpin`] and [`Lineage::forget`] each take their `O(1)` stack-top
  /// path and each emitter mark is released newest-first (the settle order the kept-checkpoint
  /// funnel guarantees everywhere else). Silent: no assert, no panic — a `Drop` may run while
  /// already unwinding, and [`release`](Emitter::release) is advisory-and-observably-pure by its
  /// contract.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cold]
  #[inline(never)]
  fn release_abandoned_points(&mut self) {
    // The popped `Checkpoint` is dropped WITHOUT restoring — that is exactly what keeps the
    // progress. `unpin`/`forget` are the assert-free `Lineage` primitives, and the emitter
    // release is the same pin-then-lineage-then-mark settle `commit_point` performs through
    // `forget_kept_checkpoint`, minus the asserts a mid-unwind drop must not make.
    while let Some(ckp) = self.points.pop() {
      self.lineage.unpin(ckp.ckp_id);
      self.lineage.forget(ckp.ckp_id);
      self.emitter.release(ckp.emitter_checkpoint);
    }
  }
}

/// Releases the **session points** still open when the handle dies — the drop half of the
/// [`begin_point`](super::InputRef::begin_point) contract.
///
/// Without this, the halves of an abandoned point would part ways at the handle's death: the
/// checkpoint would go, but the input would keep its pin and its lineage entry — and the emitter
/// its mark-keyed bookkeeping — for a point that no one can ever settle. The pin set's own
/// invariant ("it holds exactly the live begin points") would quietly become false, and the memos
/// would grow without bound across a driver that takes handle after handle from one input; a
/// mark-keyed emitter (an event sink's checkpoint stack) would strand one row per abandoned
/// point the same way.
///
/// So an abandoned point settles here exactly as [`commit_point`](super::InputRef::commit_point)
/// settles a kept one — **unpin, release the lineage entry, release the emitter mark** — and,
/// like it, rewinds nothing: the progress made through the point stays (the no-rollback-on-drop
/// law on [`begin_point`](super::InputRef::begin_point)). The checkpoints are plain data and go
/// with the stack. Newest-first, so the releases take their `O(1)` stack-top paths.
///
/// Silent by construction — no assert, no panic, and deliberately *not* routed through the raw
/// checkpoint-commit path (whose debug foreign-input assert would abort if this ran while already
/// unwinding), for the same reason the guards' rolling-back drop consults liveness rather than
/// asserting it. [`Emitter::release`] is safe to call here by its own contract: advisory,
/// observably pure, at most once per mark.
///
/// **This body is one branch.** Everything it does when the branch is taken lives out of line in
/// [`release_abandoned_points`](Session::release_abandoned_points) — see there for why that is load
/// bearing rather than tidy. The common case, a handle that never opened a session, is a length check
/// on a `Vec` that never allocated.
#[cfg(any(feature = "std", feature = "alloc"))]
impl<'inp, L, E, Lang: ?Sized> Drop for Session<'inp, '_, L, E, Lang>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
{
  #[inline(always)]
  fn drop(&mut self) {
    if !self.points.is_empty() {
      self.release_abandoned_points();
    }
  }
}
