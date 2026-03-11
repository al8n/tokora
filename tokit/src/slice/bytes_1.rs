use bytes_1::Bytes;

use super::Slice;

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

#[cfg(test)]
#[allow(warnings)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;

  #[test]
  fn bytes_slice_len() {
    let b = Bytes::from_static(b"hello");
    assert_eq!(Slice::len(&b), 5);
  }

  #[test]
  fn bytes_slice_iter() {
    let b = Bytes::from_static(b"abc");
    let bytes: std::vec::Vec<u8> = Slice::iter(&b).collect();
    assert_eq!(bytes, std::vec![b'a', b'b', b'c']);
  }

  #[test]
  fn bytes_slice_positioned_iter() {
    let b = Bytes::from_static(b"ab");
    let items: std::vec::Vec<(usize, u8)> = Slice::positioned_iter(&b).collect();
    assert_eq!(items, std::vec![(0, b'a'), (1, b'b')]);
  }
}
