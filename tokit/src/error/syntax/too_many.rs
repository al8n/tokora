use core::marker::PhantomData;

use crate::span::{SimpleSpan, Span};

/// An error indicating too many elements were found.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TooMany<S = SimpleSpan, Lang: ?Sized = ()> {
  span: S,
  nums: usize,
  limit: usize,
  _lang: PhantomData<Lang>,
}

impl<S> TooMany<S> {
  /// Creates a new `TooMany` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, nums: usize, maximum: usize) -> Self {
    Self::of(span, nums, maximum)
  }
}

impl<S, Lang: ?Sized> TooMany<S, Lang> {
  /// Creates a new `TooMany` error for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(span: S, nums: usize, maximum: usize) -> Self {
    Self::new_in(span, nums, maximum)
  }
}

impl<S, Lang: ?Sized> TooMany<S, Lang> {
  const fn new_in(span: S, nums: usize, limit: usize) -> Self {
    Self {
      span,
      nums,
      limit,
      _lang: PhantomData,
    }
  }

  /// Returns the span associated with this error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &S {
    &self.span
  }

  /// Returns the span associated with this error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(self) -> S
  where
    S: Copy,
  {
    self.span
  }

  /// Returns the mutable reference to the span associated with this error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut S {
    &mut self.span
  }

  /// Bumps the span by n offsets.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, by: &S::Offset) -> &mut Self
  where
    S: Span,
  {
    self.span.bump(by);
    self
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

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: TooMany<S, Lang>) -> Self {}
}

impl<S, Lang> TooMany<S, Lang>
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
      "too many elements: found {}, but maximum is {} at {}",
      self.nums, self.limit, self.span
    )
  }
}
