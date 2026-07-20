use bstr_1::BStr;

use super::Source;

impl Source<usize> for BStr {
  type Slice<'a>
    = &'a [u8]
  where
    Self: 'a;

  #[inline(always)]
  fn is_empty(&self) -> bool {
    <[u8]>::is_empty(self)
  }

  #[inline(always)]
  fn len(&self) -> usize {
    <[u8]>::len(self)
  }

  #[inline(always)]
  fn as_slice(&self) -> Self::Slice<'_>
  where
    Self: Sized,
  {
    BStr::as_ref(self)
  }

  #[inline(always)]
  fn slice<R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: core::ops::RangeBounds<usize>,
  {
    <[u8]>::slice(self, range)
  }

  #[inline(always)]
  fn is_boundary(&self, index: usize) -> bool {
    <[u8]>::is_boundary(self, index)
  }
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests;
