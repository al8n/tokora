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
