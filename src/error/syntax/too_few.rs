use core::marker::PhantomData;

use crate::utils::Span;

/// An error indicating too few elements were found.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TooFew<O: ?Sized, S = Span> {
  span: S,
  nums: usize,
  limit: usize,
  _syn: PhantomData<O>,
}

impl<O: ?Sized, S> TooFew<O, S> {
  /// Creates a new `TooFew` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, nums: usize, minimum: usize) -> Self {
    Self::new_in(span, nums, minimum)
  }
}

impl<O: ?Sized, S> TooFew<O, S> {
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

  /// Returns the limit that was violated.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn limit(&self) -> usize {
    self.limit
  }
}
