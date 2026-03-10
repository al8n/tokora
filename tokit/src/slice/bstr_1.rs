use bstr_1::BStr;

use super::Slice;

impl Slice<'_> for BStr {
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
    <[u8]>::len(self)
  }
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;

  #[test]
  fn bstr_slice_len() {
    let s = BStr::new(b"hello");
    assert_eq!(Slice::len(s), 5);
  }

  #[test]
  fn bstr_slice_iter() {
    let s = BStr::new(b"abc");
    let bytes: std::vec::Vec<u8> = Slice::iter(s).collect();
    assert_eq!(bytes, std::vec![b'a', b'b', b'c']);
  }

  #[test]
  fn bstr_slice_positioned_iter() {
    let s = BStr::new(b"ab");
    let items: std::vec::Vec<(usize, u8)> = Slice::positioned_iter(s).collect();
    assert_eq!(items, std::vec![(0, b'a'), (1, b'b')]);
  }
}
