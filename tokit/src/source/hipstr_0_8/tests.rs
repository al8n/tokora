use super::*;
use hipstr_0_8::{HipByt, HipStr};

// --- HipStr tests ---

#[test]
fn hipstr_is_empty_on_empty() {
  let src = HipStr::from("");
  assert!(Source::is_empty(&src));
}

#[test]
fn hipstr_is_empty_on_non_empty() {
  let src = HipStr::from("abc");
  assert!(!Source::is_empty(&src));
}

#[test]
fn hipstr_len() {
  let src = HipStr::from("hello");
  assert_eq!(Source::len(&src), 5);
}

#[test]
fn hipstr_len_empty() {
  let src = HipStr::from("");
  assert_eq!(Source::len(&src), 0);
}

#[test]
fn hipstr_len_multibyte() {
  let src = HipStr::from("\u{1F600}");
  assert_eq!(Source::len(&src), 4);
}

#[test]
fn hipstr_slice_full_range() {
  let src = HipStr::from("abcde");
  let result = Source::slice(&src, &0..&5);
  assert_eq!(result.as_deref(), Some("abcde"));
}

#[test]
fn hipstr_slice_partial() {
  let src = HipStr::from("abcde");
  let result = Source::slice(&src, &1..&3);
  assert_eq!(result.as_deref(), Some("bc"));
}

#[test]
fn hipstr_slice_empty() {
  let src = HipStr::from("abc");
  let result = Source::slice(&src, &1..&1);
  assert_eq!(result.as_deref(), Some(""));
}

#[test]
fn hipstr_slice_out_of_bounds() {
  let src = HipStr::from("abc");
  let result = Source::slice(&src, &0..&10);
  assert!(result.is_none());
}

#[test]
fn hipstr_slice_non_boundary() {
  let src = HipStr::from("\u{00E9}abc");
  let result = Source::slice(&src, &0..&1);
  assert!(result.is_none());
}

#[test]
fn hipstr_is_boundary_at_char_boundaries() {
  let src = HipStr::from("\u{00E9}a");
  assert!(Source::is_boundary(&src, 0));
  assert!(!Source::is_boundary(&src, 1));
  assert!(Source::is_boundary(&src, 2));
  assert!(Source::is_boundary(&src, 3));
}

#[test]
fn hipstr_is_boundary_beyond_len() {
  let src = HipStr::from("abc");
  assert!(!Source::is_boundary(&src, 4));
}

#[test]
fn hipstr_is_boundary_at_end() {
  let src = HipStr::from("abc");
  assert!(Source::is_boundary(&src, 3));
}

#[test]
fn hipstr_find_boundary_rounds_down_multibyte() {
  let src = HipStr::from("\u{00E9}"); // 2-byte code point at 0..2
  assert_eq!(Source::find_boundary(&src, 1), 0);
}

#[test]
fn hipstr_find_boundary_rounds_down_after_ascii() {
  let src = HipStr::from("a\u{00E9}"); // 'a' at 0, "é" at 1..3
  assert_eq!(Source::find_boundary(&src, 2), 1);
}

#[test]
fn hipstr_find_boundary_passes_through_boundaries() {
  let src = HipStr::from("a\u{00E9}"); // boundaries at 0, 1, and 3 (== len)
  assert_eq!(Source::find_boundary(&src, 0), 0);
  assert_eq!(Source::find_boundary(&src, 1), 1);
  assert_eq!(Source::find_boundary(&src, 3), 3);
}

#[test]
fn hipstr_find_boundary_at_and_beyond_len() {
  let src = HipStr::from("a\u{00E9}"); // len 3
  assert_eq!(Source::find_boundary(&src, 3), 3);
  assert_eq!(Source::find_boundary(&src, 10), 10);
}

// --- HipByt tests ---

#[test]
fn hipbyt_is_empty_on_empty() {
  let src = HipByt::from(b"" as &[u8]);
  assert!(Source::is_empty(&src));
}

#[test]
fn hipbyt_is_empty_on_non_empty() {
  let src = HipByt::from(b"abc" as &[u8]);
  assert!(!Source::is_empty(&src));
}

#[test]
fn hipbyt_len() {
  let src = HipByt::from(b"hello" as &[u8]);
  assert_eq!(Source::len(&src), 5);
}

#[test]
fn hipbyt_len_empty() {
  let src = HipByt::from(b"" as &[u8]);
  assert_eq!(Source::len(&src), 0);
}

#[test]
fn hipbyt_slice_full_range() {
  let src = HipByt::from(b"abcde" as &[u8]);
  let result = Source::slice(&src, &0..&5);
  assert_eq!(result.as_deref(), Some(b"abcde".as_slice()));
}

#[test]
fn hipbyt_slice_partial() {
  let src = HipByt::from(b"abcde" as &[u8]);
  let result = Source::slice(&src, &1..&3);
  assert_eq!(result.as_deref(), Some(b"bc".as_slice()));
}

#[test]
fn hipbyt_slice_empty() {
  let src = HipByt::from(b"abc" as &[u8]);
  let result = Source::slice(&src, &1..&1);
  assert_eq!(result.as_deref(), Some(b"".as_slice()));
}

#[test]
fn hipbyt_slice_out_of_bounds() {
  let src = HipByt::from(b"abc" as &[u8]);
  let result = Source::slice(&src, &0..&10);
  assert!(result.is_none());
}

#[test]
fn hipbyt_is_boundary_valid() {
  let src = HipByt::from(b"abc" as &[u8]);
  assert!(Source::is_boundary(&src, 0));
  assert!(Source::is_boundary(&src, 1));
  assert!(Source::is_boundary(&src, 3));
}

#[test]
fn hipbyt_is_boundary_beyond_len() {
  let src = HipByt::from(b"abc" as &[u8]);
  assert!(!Source::is_boundary(&src, 4));
}
