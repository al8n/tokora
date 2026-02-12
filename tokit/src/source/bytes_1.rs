use core::ops::RangeBounds;

use bytes_1::Bytes;

use super::Source;

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
