use core::ops::RangeBounds;

use bytes::Bytes;

use super::{Slice, Source};

impl Slice<'_> for Bytes {
  type Char = u8;

  type Iter<'a>
    = core::iter::Copied<core::slice::Iter<'a, u8>>
  where
    Self: 'a;

  type PositionedIter<'a>
    = core::iter::Enumerate<Self::Iter<'a>>
  where
    Self: 'a;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn iter<'a>(&'a self) -> Self::Iter<'a>
  where
    Self: 'a,
  {
    <[u8]>::iter(self).copied()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn positioned_iter<'a>(&'a self) -> Self::PositionedIter<'a>
  where
    Self: 'a,
  {
    self.iter().enumerate()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    Bytes::len(self)
  }
}

impl Source<usize> for Bytes {
  type Slice<'a>
    = Bytes
  where
    Self: 'a;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    <Bytes>::is_empty(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    self.len()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn slice<'a, R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: RangeBounds<&'a usize>,
  {
    use core::ops::Bound;

    let len = self.len();

    let begin = match range.start_bound() {
      Bound::Included(&&n) => n,
      Bound::Excluded(&&n) => n.checked_add(1)?,
      Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
      Bound::Included(&&n) => n.checked_add(1)?,
      Bound::Excluded(&&n) => n,
      Bound::Unbounded => len,
    };

    if begin > end || end > len {
      return None;
    }

    Some(Bytes::slice(self, begin..end))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_boundary(&self, index: usize) -> bool {
    <[u8]>::is_boundary(self, index)
  }
}
