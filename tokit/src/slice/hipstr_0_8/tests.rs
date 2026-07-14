use super::*;

// ── HipStr Slice tests ─────────────────────────────────────────────

#[test]
fn hipstr_slice_len() {
  let s = HipStr::from("hello");
  assert_eq!(Slice::len(&s), 5);
}

#[test]
fn hipstr_slice_is_empty() {
  let empty = HipStr::from("");
  assert!(Slice::is_empty(&empty));
  let non_empty = HipStr::from("a");
  assert!(!Slice::is_empty(&non_empty));
}

#[test]
fn hipstr_slice_iter() {
  let s = HipStr::from("abc");
  let chars: std::vec::Vec<char> = Slice::iter(&s).collect();
  assert_eq!(chars, std::vec!['a', 'b', 'c']);
}

#[test]
fn hipstr_slice_positioned_iter() {
  let s = HipStr::from("ab");
  let items: std::vec::Vec<(usize, char)> = Slice::positioned_iter(&s).collect();
  assert_eq!(items, std::vec![(0, 'a'), (1, 'b')]);
}

// ── HipByt Slice tests ─────────────────────────────────────────────

#[test]
fn hipbyt_slice_len() {
  let s = HipByt::from(b"hello" as &[u8]);
  assert_eq!(Slice::len(&s), 5);
}

#[test]
fn hipbyt_slice_is_empty() {
  let empty = HipByt::from(b"" as &[u8]);
  assert!(Slice::is_empty(&empty));
  let non_empty = HipByt::from(b"a" as &[u8]);
  assert!(!Slice::is_empty(&non_empty));
}

#[test]
fn hipbyt_slice_iter() {
  let s = HipByt::from(b"abc" as &[u8]);
  let bytes: std::vec::Vec<u8> = Slice::iter(&s).collect();
  assert_eq!(bytes, std::vec![b'a', b'b', b'c']);
}

#[test]
fn hipbyt_slice_positioned_iter() {
  let s = HipByt::from(b"ab" as &[u8]);
  let items: std::vec::Vec<(usize, u8)> = Slice::positioned_iter(&s).collect();
  assert_eq!(items, std::vec![(0, b'a'), (1, b'b')]);
}
