//! A machine's storage on the native stack, a segment at a time: how a default stack grows past
//! its first frames where there is no heap to grow on.
//!
//! A segment holds `K` frames in the native frame of the loop that made it, over the full segment
//! below it. It is a [`Stack`], so a machine runs over it as over any storage, and no rule of the
//! machine knows that it exists. Its `push` refuses once its own `K` frames are in use, and its
//! `pop` takes from its own. Its `last` and `last_mut` reach the frame on top of what is below
//! when it holds none, which breaks one of `Stack`'s written obligations on purpose: with its own
//! frames gone it answers `pop` with `None` and `last` with the frame below. Two things cross that
//! link: the latch an infix reduction writes on the frame now on top, when that reduction emptied
//! the segment, and a read of that frame, when a pop, a reduction's or a `retract`'s, has emptied
//! the segment, for the floor it gives its operand and the latch it holds, which the machine keeps
//! a copy of. A debug build reads it too, to check the floor kept, where the loop feeds an
//! operator again on a new, empty segment.
//!
//! The loop that runs a machine over a segment keeps two rules, which the segment cannot keep for
//! it:
//!
//! - **Growing.** When a push is refused below the machine's cap, the loop makes the next segment
//!   over this one in a deeper native frame, re-seats the machine on it through `into_parts` and
//!   `from_parts`, and feeds the event again, which the empty segment takes.
//! - **Returning.** Once its own frames are gone with frames below, the loop returns to the frame
//!   below before the machine next reads its stack. The loop tests that as
//!   `UPPER && machine.storage().is_empty()`, where `UPPER` marks a segment that has frames below;
//!   [`drained`](Segment::drained) is the same test on the segment alone. The frame below
//!   re-seats the machine on its own segment, which is as it was left: full. So no frame is ever
//!   popped through a link, every segment below the deepest is full, and wherever the loop reads
//!   its stack the native stack holds `ceil(depth / K)` segments: a long expression that is not
//!   deep cannot grow it.
//!
//! A segment reports its own `K` as its [`max_capacity`](Container::max_capacity), as `Container`
//! asks of a storage that refuses there, while the machine's count spans every segment. So the
//! machine's `capacity` reads `K` over a segment, and nothing may read it as a bound: the cap is
//! the bound, and the machine never consults `capacity`.

use hybrid_arraydeque::{ArrayDeque, ArraySize};

use crate::container::{Container, Stack};

/// What a segment reads and writes of the storage below it: the frame on top. Object-safe, so one
/// segment type links to any storage, and the type of what is below does not appear in its own.
trait Lower<T> {
  /// The frame on top of the storage below.
  fn lower_last(&self) -> Option<&T>;

  /// The frame on top of the storage below, mutably.
  fn lower_last_mut(&mut self) -> Option<&mut T>;
}

impl<T, S: Stack<T>> Lower<T> for S {
  #[inline]
  fn lower_last(&self) -> Option<&T> {
    self.last()
  }

  #[inline]
  fn lower_last_mut(&mut self) -> Option<&mut T> {
    self.last_mut()
  }
}

/// `K` frames of a machine's stack in one native frame, over the full storage below them.
pub(crate) struct Segment<'below, T, K: ArraySize> {
  /// This segment's own frames, bottom first.
  own: ArrayDeque<T, K>,
  /// The storage below, full when this segment was made; `None` for the first segment.
  below: Option<&'below mut dyn Lower<T>>,
}

impl<'below, T, K: ArraySize> Segment<'below, T, K> {
  /// The first segment, with nothing below it.
  #[inline]
  pub(crate) fn root() -> Self {
    Self {
      own: ArrayDeque::new(),
      below: None,
    }
  }

  /// A segment over `below`, which a push has just found full.
  #[inline]
  pub(crate) fn over<S: Stack<T>>(below: &'below mut S) -> Self {
    Self {
      own: ArrayDeque::new(),
      below: Some(below),
    }
  }

  /// Whether its own frames are gone, and frames remain below. The loop that runs a machine over a
  /// segment spells this test `UPPER && machine.storage().is_empty()`.
  #[inline]
  pub(crate) fn drained(&self) -> bool {
    self.own.is_empty() && self.below.is_some()
  }
}

impl<T, K: ArraySize> Container<T> for Segment<'_, T, K> {
  /// Refused once its own `K` frames are in use: it never takes a frame for the storage below.
  #[inline]
  fn push(&mut self, item: T) -> Result<(), T> {
    match self.own.push_back(item) {
      None => Ok(()),
      Some(refused) => Err(refused),
    }
  }

  /// Its own bottom frame.
  #[inline]
  fn first(&self) -> Option<&T> {
    self.own.front()
  }

  /// Its own top frame, or the top of the storage below when it holds none.
  #[inline]
  fn last(&self) -> Option<&T> {
    match self.own.back() {
      Some(item) => Some(item),
      None => self.below.as_ref().and_then(|below| below.lower_last()),
    }
  }

  /// Its own frames: those it has taken and not given back.
  #[inline]
  fn len(&self) -> usize {
    self.own.len()
  }

  /// `K`, the bound its `push` refuses at.
  #[inline]
  fn max_capacity(&self) -> usize {
    K::USIZE
  }
}

impl<T, K: ArraySize> Stack<T> for Segment<'_, T, K> {
  /// Its own top frame, and never one through the link: the loop has returned to the frame below
  /// before a pop could reach it.
  #[inline]
  fn pop(&mut self) -> Option<T> {
    self.own.pop_back()
  }

  /// Its own top frame, or the top of the storage below when it holds none: the one write that
  /// crosses the link is the latch a reduction leaves there.
  #[inline]
  fn last_mut(&mut self) -> Option<&mut T> {
    match self.own.back_mut() {
      Some(item) => Some(item),
      None => self.below.as_mut().and_then(|below| below.lower_last_mut()),
    }
  }
}

#[cfg(test)]
mod tests;
