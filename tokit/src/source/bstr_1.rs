use bstr_1::BStr;

use super::Source;

impl Source<usize> for BStr {
  type Slice<'a>
    = &'a [u8]
  where
    Self: 'a;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    <[u8]>::is_empty(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    <[u8]>::len(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn slice<'a, R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: core::ops::RangeBounds<&'a usize>,
    usize: 'a,
  {
    <[u8]>::slice(self, range)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_boundary(&self, index: usize) -> bool {
    <[u8]>::is_boundary(self, index)
  }
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;

  #[test]
  fn bstr_is_empty() {
    let empty = BStr::new(b"");
    assert!(Source::is_empty(empty));
    let non_empty = BStr::new(b"abc");
    assert!(!Source::is_empty(non_empty));
  }

  #[test]
  fn bstr_len() {
    let s = BStr::new(b"hello");
    assert_eq!(Source::len(s), 5);
  }

  #[test]
  fn bstr_slice() {
    let s = BStr::new(b"hello");
    let sliced = Source::slice(s, &1..&3);
    assert_eq!(sliced, Some(&b"el"[..]));
  }

  #[test]
  fn bstr_is_boundary() {
    let s = BStr::new(b"abc");
    assert!(Source::is_boundary(s, 0));
    assert!(Source::is_boundary(s, 3));
    assert!(!Source::is_boundary(s, 4));
  }
}
