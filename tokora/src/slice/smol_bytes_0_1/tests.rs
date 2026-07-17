use super::*;

// ── shared::Bytes Slice tests ──────────────────────────────────────

#[test]
fn shared_bytes_slice_len() {
  let b = shared::Bytes::from_static(b"hello");
  assert_eq!(Slice::len(&b), 5);
}

#[test]
fn shared_bytes_slice_is_empty() {
  let empty = shared::Bytes::new();
  assert!(Slice::is_empty(&empty));
  let non_empty = shared::Bytes::from_static(b"a");
  assert!(!Slice::is_empty(&non_empty));
}

#[test]
fn shared_bytes_slice_iter() {
  let b = shared::Bytes::from_static(b"abc");
  let bytes: std::vec::Vec<u8> = Slice::iter(&b).collect();
  assert_eq!(bytes, std::vec![b'a', b'b', b'c']);
}

#[test]
fn shared_bytes_slice_positioned_iter() {
  let b = shared::Bytes::from_static(b"ab");
  let items: std::vec::Vec<(usize, u8)> = Slice::positioned_iter(&b).collect();
  assert_eq!(items, std::vec![(0, b'a'), (1, b'b')]);
}

// ── compact::Bytes Slice tests ─────────────────────────────────────

#[test]
fn compact_bytes_slice_len() {
  let b = compact::Bytes::from_static(b"hello");
  assert_eq!(Slice::len(&b), 5);
}

#[test]
fn compact_bytes_slice_is_empty() {
  let empty = compact::Bytes::new();
  assert!(Slice::is_empty(&empty));
  let non_empty = compact::Bytes::from_static(b"a");
  assert!(!Slice::is_empty(&non_empty));
}

#[test]
fn compact_bytes_slice_iter() {
  let b = compact::Bytes::from_static(b"abc");
  let bytes: std::vec::Vec<u8> = Slice::iter(&b).collect();
  assert_eq!(bytes, std::vec![b'a', b'b', b'c']);
}

#[test]
fn compact_bytes_slice_positioned_iter() {
  let b = compact::Bytes::from_static(b"ab");
  let items: std::vec::Vec<(usize, u8)> = Slice::positioned_iter(&b).collect();
  assert_eq!(items, std::vec![(0, b'a'), (1, b'b')]);
}

// ── Utf8Bytes Slice tests ──────────────────────────────────────────

#[test]
fn utf8_bytes_slice_len() {
  let s = Utf8Bytes::from_static("hello");
  assert_eq!(Slice::len(&s), 5);
}

#[test]
fn utf8_bytes_slice_is_empty() {
  let empty = Utf8Bytes::new();
  assert!(Slice::is_empty(&empty));
  let non_empty = Utf8Bytes::from_static("a");
  assert!(!Slice::is_empty(&non_empty));
}

#[test]
fn utf8_bytes_slice_iter() {
  let s = Utf8Bytes::from_static("abc");
  let chars: std::vec::Vec<char> = Slice::iter(&s).collect();
  assert_eq!(chars, std::vec!['a', 'b', 'c']);
}

#[test]
fn utf8_bytes_slice_positioned_iter() {
  let s = Utf8Bytes::from_static("ab");
  let items: std::vec::Vec<(usize, char)> = Slice::positioned_iter(&s).collect();
  assert_eq!(items, std::vec![(0, 'a'), (1, 'b')]);
}
