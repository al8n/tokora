use core::marker::PhantomData;

use crate::span::{SimpleSpan, Span};

/// An error indicating that unexpected repeated tokens were found during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnexpectedRepeatedToken<T, S = SimpleSpan, Lang: ?Sized = ()> {
  span: S,
  count: usize,
  _t: PhantomData<T>,
  _lang: PhantomData<Lang>,
}

impl<T, S> UnexpectedRepeatedToken<T, S> {
  /// Creates a new `UnexpectedRepeatedToken` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, count: usize) -> Self {
    Self::of(span, count)
  }
}

impl<T, S, Lang: ?Sized> UnexpectedRepeatedToken<T, S, Lang> {
  /// Creates a new `UnexpectedRepeatedToken` error for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(span: S, count: usize) -> Self {
    Self {
      span,
      count,
      _lang: PhantomData,
      _t: PhantomData,
    }
  }

  /// Returns the reference to the span covering the repeated tokens.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &S {
    &self.span
  }

  /// Returns the mutable reference to the span covering the repeated tokens.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut S {
    &mut self.span
  }

  /// Returns the span covering the repeated tokens.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    self.span
  }

  /// Returns the number of repeated tokens found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn count(&self) -> usize {
    self.count
  }

  /// Expands the span covering the repeated tokens.
  ///
  /// The `span` parameter is used to extend the end of the current span,
  /// and the `count` parameter indicates how many additional repeated tokens were found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn expand(&mut self, span: S, count: usize) -> &mut Self
  where
    S: Span,
  {
    *self.span.end_mut() = span.end();
    self.count += count;
    self
  }
}

impl<T, S, Lang: ?Sized> From<UnexpectedRepeatedToken<T, S, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: UnexpectedRepeatedToken<T, S, Lang>) -> Self {}
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(feature = "std")]
mod tests {
  use super::*;
  use crate::span::SimpleSpan;

  type Urt = UnexpectedRepeatedToken<u8, SimpleSpan>;

  #[test]
  fn new_and_accessors() {
    let span = SimpleSpan::const_new(0, 5);
    let err = Urt::new(span, 3);
    assert_eq!(err.span(), span);
    assert_eq!(err.count(), 3);
  }

  #[test]
  fn of_with_lang() {
    let span = SimpleSpan::const_new(1, 4);
    let err = UnexpectedRepeatedToken::<u8, SimpleSpan, ()>::of(span, 2);
    assert_eq!(err.span(), span);
    assert_eq!(err.count(), 2);
  }

  #[test]
  fn span_ref_and_span_mut() {
    let span = SimpleSpan::const_new(0, 5);
    let mut err = Urt::new(span, 1);
    assert_eq!(*err.span_ref(), span);
    *err.span_mut() = SimpleSpan::const_new(1, 6);
    assert_eq!(err.span(), SimpleSpan::const_new(1, 6));
  }

  #[test]
  fn expand() {
    let mut err = Urt::new(SimpleSpan::const_new(0, 3), 1);
    err.expand(SimpleSpan::const_new(3, 7), 2);
    assert_eq!(err.span(), SimpleSpan::const_new(0, 7));
    assert_eq!(err.count(), 3);
  }

  #[test]
  fn derive_traits() {
    let err = Urt::new(SimpleSpan::const_new(0, 1), 1);
    let err2 = err.clone();
    assert_eq!(err, err2);
    let _ = format!("{:?}", err);
  }

  #[test]
  fn from_into_unit() {
    let err = Urt::new(SimpleSpan::const_new(0, 1), 1);
    let _: () = err.into();
  }
}
