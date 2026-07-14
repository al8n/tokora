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
