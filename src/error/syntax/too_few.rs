use core::marker::PhantomData;

use crate::utils::SimpleSpan;

/// An error indicating too few elements were found.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TooFew<O: ?Sized, S = SimpleSpan, Lang: ?Sized = ()> {
  span: S,
  nums: usize,
  limit: usize,
  _syn: PhantomData<O>,
  _lang: PhantomData<Lang>,
}

impl<O: ?Sized, S> TooFew<O, S> {
  /// Creates a new `TooFew` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, nums: usize, minimum: usize) -> Self {
    Self::of(span, nums, minimum)
  }
}

impl<O: ?Sized, S, Lang: ?Sized> TooFew<O, S, Lang> {
  /// Creates a new `TooFew` error for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(span: S, nums: usize, minimum: usize) -> Self {
    Self::new_in(span, nums, minimum)
  }
}

impl<O: ?Sized, S, Lang: ?Sized> TooFew<O, S, Lang> {
  const fn new_in(span: S, nums: usize, limit: usize) -> Self {
    Self {
      span,
      nums,
      limit,
      _syn: PhantomData,
      _lang: PhantomData,
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

impl<O: ?Sized, S, Lang: ?Sized> From<TooFew<O, S, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: TooFew<O, S, Lang>) -> Self {}
}

impl<O: ?Sized, S, Lang> TooFew<O, S, Lang>
where
  Lang: ?Sized,
{
  /// Formats this error for display purposes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn display_fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
  where
    S: core::fmt::Display,
  {
    write!(
      f,
      "too few elements: found {}, but minimum is {} at {}",
      self.nums, self.limit, self.span
    )
  }
}
