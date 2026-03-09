use core::marker::PhantomData;

use crate::span::{SimpleSpan, Span};

/// An error indicating too few elements were found.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TooFew<S = SimpleSpan, Lang: ?Sized = ()> {
  span: S,
  nums: usize,
  limit: usize,
  _lang: PhantomData<Lang>,
}

impl<S> TooFew<S> {
  /// Creates a new `TooFew` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, nums: usize, minimum: usize) -> Self {
    Self::of(span, nums, minimum)
  }
}

impl<S, Lang: ?Sized> TooFew<S, Lang> {
  /// Creates a new `TooFew` error for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(span: S, nums: usize, minimum: usize) -> Self {
    Self::new_in(span, nums, minimum)
  }
}

impl<S, Lang: ?Sized> TooFew<S, Lang> {
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

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: TooFew<S, Lang>) -> Self {}
}

impl<S, Lang> TooFew<S, Lang>
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::span::SimpleSpan;

  #[test]
  fn too_few_new() {
    let err = TooFew::new(SimpleSpan::new(0, 5), 2, 5);
    assert_eq!(*err.span_ref(), SimpleSpan::new(0, 5));
    assert_eq!(err.nums(), 2);
    assert_eq!(err.limit(), 5);
  }

  #[test]
  fn too_few_span_copy() {
    let err = TooFew::new(SimpleSpan::new(1, 3), 0, 1);
    assert_eq!(err.span(), SimpleSpan::new(1, 3));
  }

  #[test]
  fn too_few_span_mut() {
    let mut err = TooFew::new(SimpleSpan::new(0, 5), 1, 3);
    *err.span_mut() = SimpleSpan::new(10, 15);
    assert_eq!(err.span(), SimpleSpan::new(10, 15));
  }

  #[test]
  fn too_few_bump() {
    let mut err = TooFew::new(SimpleSpan::new(0, 5), 1, 3);
    err.bump(&10);
    assert_eq!(err.span(), SimpleSpan::new(10, 15));
  }

  #[test]
  fn too_few_of_with_lang() {
    struct MyLang;
    let err = TooFew::<SimpleSpan, MyLang>::of(SimpleSpan::new(0, 5), 2, 10);
    assert_eq!(err.nums(), 2);
    assert_eq!(err.limit(), 10);
  }

  #[test]
  fn too_few_into_unit() {
    let err = TooFew::new(SimpleSpan::new(0, 5), 1, 3);
    let _: () = err.into();
  }

  #[test]
  fn too_few_display_fmt() {
    let err = TooFew::new(SimpleSpan::new(2, 8), 1, 3);
    let msg = format!("{}", DisplayWrapper(&err));
    assert!(msg.contains("too few elements"));
    assert!(msg.contains("1"));
    assert!(msg.contains("3"));
  }

  struct DisplayWrapper<'a>(&'a TooFew<SimpleSpan>);
  impl core::fmt::Display for DisplayWrapper<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      self.0.display_fmt(f)
    }
  }
}
