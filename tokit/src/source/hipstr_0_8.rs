use hipstr_0_8::{HipByt, HipStr};

use super::Source;

impl<'h> Source<usize> for HipStr<'h> {
  type Slice<'a>
    = HipStr<'a>
  where
    Self: 'a;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    <HipStr<'h>>::is_empty(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    <HipStr<'h>>::len(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn slice<'a, R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: core::ops::RangeBounds<&'a usize>,
    usize: 'a,
  {
    self
      .try_slice((
        range.start_bound().map(|s| **s),
        range.end_bound().map(|s| **s),
      ))
      .ok()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_boundary(&self, index: usize) -> bool {
    self.is_char_boundary(index)
  }
}

impl Source<usize> for HipByt<'_> {
  type Slice<'a>
    = HipByt<'a>
  where
    Self: 'a;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    <HipByt<'_>>::is_empty(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    <HipByt<'_>>::len(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn slice<'a, R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: core::ops::RangeBounds<&'a usize>,
    usize: 'a,
  {
    self
      .try_slice((
        range.start_bound().map(|s| **s),
        range.end_bound().map(|s| **s),
      ))
      .ok()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_boundary(&self, index: usize) -> bool {
    <[u8]>::is_boundary(self, index)
  }
}
