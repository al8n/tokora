use super::*;
use bytes_1::Bytes;

#[test]
fn bytes_is_empty_on_empty() {
  let src = Bytes::new();
  assert!(Source::is_empty(&src));
}

#[test]
fn bytes_is_empty_on_non_empty() {
  let src = Bytes::from_static(b"abc");
  assert!(!Source::is_empty(&src));
}

#[test]
fn bytes_len() {
  let src = Bytes::from_static(b"hello");
  assert_eq!(Source::len(&src), 5);
}

#[test]
fn bytes_len_empty() {
  let src = Bytes::new();
  assert_eq!(Source::len(&src), 0);
}

#[test]
fn bytes_slice_full_range() {
  let src = Bytes::from_static(b"abcde");
  let result = Source::slice(&src, 0..5);
  assert_eq!(result.as_deref(), Some(b"abcde".as_slice()));
}

#[test]
fn bytes_slice_partial() {
  let src = Bytes::from_static(b"abcde");
  let result = Source::slice(&src, 1..3);
  assert_eq!(result.as_deref(), Some(b"bc".as_slice()));
}

#[test]
fn bytes_slice_empty_range() {
  let src = Bytes::from_static(b"abc");
  let result = Source::slice(&src, 1..1);
  assert_eq!(result.as_deref(), Some(b"".as_slice()));
}

#[test]
fn bytes_slice_out_of_bounds() {
  let src = Bytes::from_static(b"abc");
  let result = Source::slice(&src, 0..10);
  assert!(result.is_none());
}

#[test]
fn bytes_slice_inclusive_range() {
  let src = Bytes::from_static(b"abcde");
  let result = Source::slice(&src, 1..=3);
  assert_eq!(result.as_deref(), Some(b"bcd".as_slice()));
}

#[test]
fn bytes_slice_unbounded_start() {
  let src = Bytes::from_static(b"abcde");
  let result = Source::slice(&src, ..3);
  assert_eq!(result.as_deref(), Some(b"abc".as_slice()));
}

#[test]
fn bytes_slice_unbounded_end() {
  let src = Bytes::from_static(b"abcde");
  let result = Source::slice(&src, 2..);
  assert_eq!(result.as_deref(), Some(b"cde".as_slice()));
}

#[test]
fn bytes_slice_fully_unbounded() {
  let src = Bytes::from_static(b"abcde");
  let result = Source::slice(&src, ..);
  assert_eq!(result.as_deref(), Some(b"abcde".as_slice()));
}

#[test]
fn bytes_slice_empty_source() {
  let src = Bytes::new();
  let result = Source::slice(&src, 0..0);
  assert_eq!(result.as_deref(), Some(b"".as_slice()));
}

#[test]
fn bytes_slice_empty_source_out_of_range() {
  let src = Bytes::new();
  let result = Source::slice(&src, 0..1);
  assert!(result.is_none());
}

#[test]
fn bytes_is_boundary_valid() {
  let src = Bytes::from_static(b"abc");
  assert!(Source::is_boundary(&src, 0));
  assert!(Source::is_boundary(&src, 1));
  assert!(Source::is_boundary(&src, 3));
}

#[test]
fn bytes_is_boundary_beyond_len() {
  let src = Bytes::from_static(b"abc");
  assert!(!Source::is_boundary(&src, 4));
}

#[test]
fn bytes_is_boundary_empty() {
  let src = Bytes::new();
  assert!(Source::is_boundary(&src, 0));
  assert!(!Source::is_boundary(&src, 1));
}
