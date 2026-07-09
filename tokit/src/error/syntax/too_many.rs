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
  pub const fn span(&self) -> S
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

impl<S, Lang: ?Sized> core::fmt::Display for TooMany<S, Lang>
where
  S: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "too many elements: found {}, but maximum is {} at {}",
      self.nums, self.limit, self.span
    )
  }
}

impl<S, Lang: ?Sized> core::error::Error for TooMany<S, Lang>
where
  S: core::fmt::Display + core::fmt::Debug,
  Lang: core::fmt::Debug,
{
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;
  use crate::span::SimpleSpan;

  use std::format;

  #[test]
  fn too_many_new() {
    let err = TooMany::new(SimpleSpan::new(0, 5), 10, 5);
    assert_eq!(*err.span_ref(), SimpleSpan::new(0, 5));
    assert_eq!(err.nums(), 10);
    assert_eq!(err.limit(), 5);
  }

  #[test]
  fn too_many_span_copy() {
    let err = TooMany::new(SimpleSpan::new(1, 3), 5, 3);
    assert_eq!(err.span(), SimpleSpan::new(1, 3));
  }

  #[test]
  fn too_many_span_mut() {
    let mut err = TooMany::new(SimpleSpan::new(0, 5), 10, 5);
    *err.span_mut() = SimpleSpan::new(10, 15);
    assert_eq!(err.span(), SimpleSpan::new(10, 15));
  }

  #[test]
  fn too_many_bump() {
    let mut err = TooMany::new(SimpleSpan::new(0, 5), 10, 5);
    err.bump(&10);
    assert_eq!(err.span(), SimpleSpan::new(10, 15));
  }

  #[test]
  fn too_many_of_with_lang() {
    struct MyLang;
    let err = TooMany::<SimpleSpan, MyLang>::of(SimpleSpan::new(0, 5), 10, 5);
    assert_eq!(err.nums(), 10);
    assert_eq!(err.limit(), 5);
  }

  #[test]
  fn too_many_into_unit() {
    let err = TooMany::new(SimpleSpan::new(0, 5), 10, 5);
    let _: () = err.into();
  }

  #[test]
  fn too_many_display() {
    let err = TooMany::new(SimpleSpan::new(2, 8), 10, 5);
    let msg = format!("{err}");
    assert!(msg.contains("too many elements"));
    assert!(msg.contains("10"));
    assert!(msg.contains("5"));
  }
}
