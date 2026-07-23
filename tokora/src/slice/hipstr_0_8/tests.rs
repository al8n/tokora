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

#[test]
fn hipstr_and_hipbyt_borrowed_slice_forwarding() {
  let text_data = std::string::String::from("\u{00E9}a");
  let text = HipStr::from(text_data.as_str());
  let text_ref = &text;

  assert_eq!(<&HipStr<'_> as Slice<'_>>::len(&text_ref), 3);
  assert_eq!(
    <&HipStr<'_> as Slice<'_>>::iter(&text_ref).collect::<std::vec::Vec<_>>(),
    std::vec!['\u{00E9}', 'a']
  );

  let byte_data = std::vec![b'a', b'b'];
  let bytes = HipByt::from(byte_data.as_slice());
  let bytes_ref = &bytes;

  assert_eq!(<&HipByt<'_> as Slice<'_>>::len(&bytes_ref), 2);
  assert_eq!(
    <&HipByt<'_> as Slice<'_>>::positioned_iter(&bytes_ref).collect::<std::vec::Vec<_>>(),
    std::vec![(0, b'a'), (1, b'b')]
  );
}
