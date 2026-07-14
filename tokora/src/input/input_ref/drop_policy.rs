//! Compile-time drop policy for the transaction guards.
//!
//! An undecided [`Transaction`](super::Transaction) /
//! [`StackedTransaction`](super::StackedTransaction) needs a rule for what its `Drop`
//! does. That rule is a **typestate**, not a runtime field: the guard carries a
//! zero-sized policy marker as a type parameter, so the choice is fixed at the type
//! level (it cannot be forgotten or mutated after the guard is built) and each flavour
//! monomorphizes to a branch-free drop.
//!
//! - [`Rollback`] — the speculative default: dropping an undecided guard restores the
//!   input to the begin point, exactly as an explicit `rollback` would. Uncommitted
//!   speculative work is discarded, the database default.
//! - [`Commit`] — commit-by-default: dropping an undecided guard *keeps* the progress,
//!   identical to dropping a raw [`Checkpoint`](crate::input::Checkpoint) — including
//!   when an error propagates out of the guard through `?` under a fail-fast emitter, in
//!   which case the drop keeps the progress consumed up to the error rather than rolling
//!   it back.
//!
//! Each guard has a default constructor that selects [`Rollback`]
//! ([`begin`](super::InputRef::begin), [`begin_stacked`](super::InputRef::begin_stacked))
//! and a generic one that selects any policy
//! ([`begin_with`](super::InputRef::begin_with),
//! [`begin_stacked_with`](super::InputRef::begin_stacked_with)). With both policies the
//! guard family is capability-complete over every legal (last-in, first-out) flow: the
//! speculative shape and the commit-by-default shape (the dual exercised by the Pratt
//! operator loop) each have a guard, so no legal flow is forced back to raw
//! `save`/`restore`.

pub(crate) mod sealed {
  /// Seals [`DropPolicy`](super::DropPolicy) and carries its drop-behaviour selector.
  ///
  /// Only nameable inside this crate, so no downstream type can implement it — and hence
  /// none can implement [`DropPolicy`](super::DropPolicy). The set of policies is closed
  /// to exactly the two markers defined here.
  pub trait Sealed {
    /// Whether an *undecided* guard of this policy restores (rolls back) on drop.
    ///
    /// The guards' single generic `Drop` impl branches on this `const`; because it is a
    /// compile-time constant, each policy monomorphizes to a straight-line drop with the
    /// other arm eliminated.
    const ROLLBACK_ON_DROP: bool;
  }
}

/// The drop policy of a transaction guard — what an *undecided* guard does when dropped.
///
/// A closed set of two zero-sized markers, [`Rollback`] and [`Commit`], chosen as a type
/// parameter on [`Transaction`](super::Transaction) and
/// [`StackedTransaction`](super::StackedTransaction). The trait is **sealed**: exactly
/// these two policies exist, and the choice is a compile-time typestate rather than a
/// runtime flag. Each marker's documentation says when to reach for it; the constructors
/// that select one are [`begin`](super::InputRef::begin) /
/// [`begin_with`](super::InputRef::begin_with) and their stacked counterparts.
pub trait DropPolicy: sealed::Sealed {}

/// The speculative, rollback-on-drop policy — the default.
///
/// An undecided [`Transaction`](super::Transaction) /
/// [`StackedTransaction`](super::StackedTransaction) with this policy restores the input
/// to its begin point when dropped, exactly as an explicit
/// [`rollback`](super::Transaction::rollback) would: uncommitted speculative work is
/// discarded, the database default. This is the policy selected by
/// [`begin`](super::InputRef::begin) and
/// [`begin_stacked`](super::InputRef::begin_stacked).
#[derive(Debug)]
pub struct Rollback;

/// The commit-by-default, keep-on-drop policy.
///
/// An undecided guard with this policy *keeps* its progress when dropped — identical to
/// dropping a raw [`Checkpoint`](crate::input::Checkpoint), including when an error
/// propagates out of the guard through `?` under a fail-fast emitter (the drop keeps the
/// progress consumed up to the error, never rolling it back). It is the dual of the
/// speculative default and the shape a commit-by-default loop wants — the Pratt operator
/// loop keeps progress on every success and every `?`-propagation and rolls back only on
/// its two "operator isn't ours" exits. Select it with
/// [`begin_with`](super::InputRef::begin_with) /
/// [`begin_stacked_with`](super::InputRef::begin_stacked_with).
#[derive(Debug)]
pub struct Commit;

impl sealed::Sealed for Rollback {
  const ROLLBACK_ON_DROP: bool = true;
}
impl DropPolicy for Rollback {}

impl sealed::Sealed for Commit {
  const ROLLBACK_ON_DROP: bool = false;
}
impl DropPolicy for Commit {}
