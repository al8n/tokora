use core::marker::PhantomData;

use crate::utils::Span;

/// An error indicating too many elements were found.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FullContainer<O: ?Sized, S = Span> {
  span: S,
  nums: usize,
  limit: usize,
  _syn: PhantomData<O>,
}

impl<O: ?Sized, S> FullContainer<O, S> {
  /// Creates a new `FullContainer` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, nums: usize, maximum: usize) -> Self {
    Self::new_in(span, nums, maximum)
  }
}

impl<O: ?Sized, S> FullContainer<O, S> {
  const fn new_in(span: S, nums: usize, limit: usize) -> Self {
    Self {
      span,
      nums,
      limit,
      _syn: PhantomData,
    }
  }

  /// Returns the span associated with this error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> &S {
    &self.span
  }

  /// Returns the number of elements found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn nums(&self) -> usize {
    self.nums
  }

  /// Returns the maximum capacity of the container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn capacity(&self) -> usize {
    self.limit
  }
}

impl<O: ?Sized, S> From<FullContainer<O, S>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: FullContainer<O, S>) -> Self {}
}
