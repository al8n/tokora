use super::*;

// --- &[u8] tests ---

#[test]
fn u8_slice_is_empty_on_empty() {
  let src: &[u8] = b"";
  assert!(Source::is_empty(src));
}

#[test]
fn u8_slice_is_empty_on_non_empty() {
  let src: &[u8] = b"abc";
  assert!(!Source::is_empty(src));
}

#[test]
fn u8_slice_len() {
  let src: &[u8] = b"hello";
  assert_eq!(Source::len(src), 5);
}

#[test]
fn u8_slice_len_empty() {
  let src: &[u8] = b"";
  assert_eq!(Source::len(src), 0);
}

#[test]
fn u8_slice_slice_full_range() {
  let src: &[u8] = b"abcde";
  let result = Source::slice(src, 0..5);
  assert_eq!(result, Some(b"abcde".as_slice()));
}

#[test]
fn u8_slice_slice_partial() {
  let src: &[u8] = b"abcde";
  let result = Source::slice(src, 1..3);
  assert_eq!(result, Some(b"bc".as_slice()));
}

#[test]
fn u8_slice_slice_out_of_bounds() {
  let src: &[u8] = b"abc";
  let result = Source::slice(src, 0..10);
  assert_eq!(result, None);
}

#[test]
fn u8_slice_is_boundary_valid() {
  let src: &[u8] = b"abc";
  assert!(Source::is_boundary(src, 0));
  assert!(Source::is_boundary(src, 1));
  assert!(Source::is_boundary(src, 3));
}

#[test]
fn u8_slice_is_boundary_beyond_len() {
  let src: &[u8] = b"abc";
  assert!(!Source::is_boundary(src, 4));
}

#[test]
fn u8_slice_find_boundary_returns_index() {
  let src: &[u8] = b"abc";
  assert_eq!(Source::find_boundary(src, 2), 2);
}

// --- &str tests ---

#[test]
fn str_is_empty_on_empty() {
  let src: &str = "";
  assert!(Source::is_empty(src));
}

#[test]
fn str_is_empty_on_non_empty() {
  let src: &str = "abc";
  assert!(!Source::is_empty(src));
}

#[test]
fn str_len() {
  let src: &str = "hello";
  assert_eq!(Source::len(src), 5);
}

#[test]
fn str_len_multibyte() {
  // Each emoji is 4 bytes in UTF-8
  let src: &str = "\u{1F600}";
  assert_eq!(Source::len(src), 4);
}

#[test]
fn str_slice_full_range() {
  let src: &str = "abcde";
  let result = Source::slice(src, 0..5);
  assert_eq!(result, Some("abcde"));
}

#[test]
fn str_slice_partial() {
  let src: &str = "abcde";
  let result = Source::slice(src, 1..3);
  assert_eq!(result, Some("bc"));
}

#[test]
fn str_slice_out_of_bounds() {
  let src: &str = "abc";
  let result = Source::slice(src, 0..10);
  assert_eq!(result, None);
}

#[test]
fn str_slice_on_non_boundary_returns_none() {
  // 2-byte char: the second byte is not a valid boundary
  let src: &str = "\u{00E9}abc"; // e-acute (2 bytes) + abc
  let result = Source::slice(src, 0..1);
  assert_eq!(result, None);
}

#[test]
fn str_is_boundary_at_char_boundaries() {
  let src: &str = "\u{00E9}a"; // 2-byte char + 1-byte char
  assert!(Source::is_boundary(src, 0));
  assert!(!Source::is_boundary(src, 1)); // middle of 2-byte char
  assert!(Source::is_boundary(src, 2)); // start of 'a'
  assert!(Source::is_boundary(src, 3)); // end
}

#[test]
fn str_is_boundary_at_end() {
  let src: &str = "abc";
  assert!(Source::is_boundary(src, 3));
}

#[test]
fn str_is_boundary_beyond_len() {
  let src: &str = "abc";
  assert!(!Source::is_boundary(src, 4));
}

#[test]
fn str_find_boundary_returns_index() {
  let src: &str = "abc";
  assert_eq!(Source::find_boundary(src, 1), 1);
}

#[test]
fn str_find_boundary_rounds_down_multibyte() {
  // "é" is a single 2-byte code point occupying 0..2.
  let src: &str = "\u{00E9}";
  assert_eq!(Source::find_boundary(src, 1), 0);
}

#[test]
fn str_find_boundary_rounds_down_after_ascii() {
  // 'a' at 0, "é" at 1..3.
  let src: &str = "a\u{00E9}";
  assert_eq!(Source::find_boundary(src, 2), 1);
}

#[test]
fn str_find_boundary_passes_through_boundaries() {
  let src: &str = "a\u{00E9}"; // boundaries at 0, 1, and 3 (== len)
  assert_eq!(Source::find_boundary(src, 0), 0);
  assert_eq!(Source::find_boundary(src, 1), 1);
  assert_eq!(Source::find_boundary(src, 3), 3);
}

#[test]
fn str_find_boundary_at_and_beyond_len() {
  // index >= len is returned unchanged, symmetric with the byte sources.
  let src: &str = "a\u{00E9}"; // len 3
  assert_eq!(Source::find_boundary(src, 3), 3);
  assert_eq!(Source::find_boundary(src, 10), 10);
}

#[test]
fn str_empty_slice() {
  let src: &str = "abc";
  let result = Source::slice(src, 1..1);
  assert_eq!(result, Some(""));
}
