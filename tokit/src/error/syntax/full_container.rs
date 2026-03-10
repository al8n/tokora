use core::marker::PhantomData;

use crate::span::{SimpleSpan, Span};

/// An error indicating too many elements were found.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FullContainer<S = SimpleSpan, Lang: ?Sized = ()> {
  span: S,
  nums: usize,
  limit: usize,
  _lang: PhantomData<Lang>,
}

impl<S> FullContainer<S> {
  /// Creates a new `FullContainer` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, nums: usize, maximum: usize) -> Self {
    Self::of(span, nums, maximum)
  }
}

impl<S, Lang: ?Sized> FullContainer<S, Lang> {
  /// Creates a new `FullContainer` error for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(span: S, nums: usize, maximum: usize) -> Self {
    Self::new_in(span, nums, maximum)
  }
}

impl<S, Lang: ?Sized> FullContainer<S, Lang> {
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
  pub const fn span(&self) -> &S {
    &self.span
  }

  /// Returns the number of elements found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn nums(&self) -> usize {
    self.nums
  }

  /// Bumps the span by the given offset.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, by: &S::Offset) -> &mut Self
  where
    S: Span,
  {
    self.span.bump(by);
    self
  }

  /// Returns the maximum capacity of the container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn capacity(&self) -> usize {
    self.limit
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: FullContainer<S, Lang>) -> Self {}
}

impl<S, Lang> FullContainer<S, Lang>
where
  Lang: ?Sized,
{
  /// Formats the error message for this error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn display_fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "found {} elements, which exceeds the maximum capacity of {}",
      self.nums, self.limit
    )
  }
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;
  use crate::span::SimpleSpan;

  use std::format;

  #[test]
  fn full_container_new() {
    let err = FullContainer::new(SimpleSpan::new(0, 5), 10, 5);
    assert_eq!(*err.span(), SimpleSpan::new(0, 5));
    assert_eq!(err.nums(), 10);
    assert_eq!(err.capacity(), 5);
  }

  #[test]
  fn full_container_of_with_lang() {
    struct MyLang;
    let err = FullContainer::<SimpleSpan, MyLang>::of(SimpleSpan::new(0, 5), 10, 5);
    assert_eq!(err.nums(), 10);
    assert_eq!(err.capacity(), 5);
  }

  #[test]
  fn full_container_bump() {
    let mut err = FullContainer::new(SimpleSpan::new(0, 5), 10, 5);
    err.bump(&10);
    assert_eq!(*err.span(), SimpleSpan::new(10, 15));
  }

  #[test]
  fn full_container_into_unit() {
    let err = FullContainer::new(SimpleSpan::new(0, 5), 10, 5);
    let _: () = err.into();
  }

  #[test]
  fn full_container_display_fmt() {
    let err = FullContainer::new(SimpleSpan::new(2, 8), 10, 5);
    let msg = format!("{}", DisplayWrapper(&err));
    assert!(msg.contains("10"));
    assert!(msg.contains("5"));
    assert!(msg.contains("exceeds the maximum capacity"));
  }

  struct DisplayWrapper<'a>(&'a FullContainer<SimpleSpan>);
  impl core::fmt::Display for DisplayWrapper<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      self.0.display_fmt(f)
    }
  }
}
