use hipstr_0_8::{HipByt, HipStr};

use super::Source;

impl<'h> Source<usize> for HipStr<'h> {
  type Slice<'a>
    = HipStr<'a>
  where
    Self: 'a;

  #[inline(always)]
  fn is_empty(&self) -> bool {
    <HipStr<'h>>::is_empty(self)
  }

  #[inline(always)]
  fn len(&self) -> usize {
    <HipStr<'h>>::len(self)
  }

  #[inline(always)]
  fn as_slice(&self) -> Self::Slice<'_> {
    self.clone()
  }

  #[inline(always)]
  fn slice<R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: core::ops::RangeBounds<usize>,
  {
    self
      .try_slice((
        range.start_bound().map(|s| *s),
        range.end_bound().map(|s| *s),
      ))
      .ok()
  }

  /// Rounds `index` DOWN to the nearest UTF-8 code point boundary so the result
  /// is always a valid slice position. Indices at or beyond the end are returned
  /// unchanged, matching the byte sources.
  #[inline(always)]
  fn find_boundary(&self, index: usize) -> usize {
    if index >= <HipStr<'h>>::len(self) {
      return index;
    }
    let mut index = index;
    while !self.is_char_boundary(index) {
      index -= 1;
    }
    index
  }

  #[inline(always)]
  fn is_boundary(&self, index: usize) -> bool {
    self.is_char_boundary(index)
  }
}

impl Source<usize> for HipByt<'_> {
  type Slice<'a>
    = HipByt<'a>
  where
    Self: 'a;

  #[inline(always)]
  fn is_empty(&self) -> bool {
    <HipByt<'_>>::is_empty(self)
  }

  #[inline(always)]
  fn len(&self) -> usize {
    <HipByt<'_>>::len(self)
  }

  #[inline(always)]
  fn as_slice(&self) -> Self::Slice<'_> {
    self.clone()
  }

  #[inline(always)]
  fn slice<R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: core::ops::RangeBounds<usize>,
  {
    self
      .try_slice((
        range.start_bound().map(|s| *s),
        range.end_bound().map(|s| *s),
      ))
      .ok()
  }

  #[inline(always)]
  fn is_boundary(&self, index: usize) -> bool {
    <[u8]>::is_boundary(self, index)
  }
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
#[allow(warnings)]
mod tests;
