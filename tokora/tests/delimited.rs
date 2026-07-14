/// Tests for `utils::Delimited`.
use tokora::{SimpleSpan, utils::Delimited};

// ── Construction and accessors ─────────────────────────────────────────────

#[test]
fn new_and_getters() {
  let d = Delimited::new('(', ')', "hello", SimpleSpan::new(0, 7));
  assert_eq!(d.open(), '(');
  assert_eq!(d.close(), ')');
  assert_eq!(d.span(), SimpleSpan::new(0, 7));
  assert_eq!(*d.data(), "hello");
}

#[test]
fn open_ref() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  assert_eq!(d.open_ref(), &'(');
}

#[test]
fn open_mut() {
  let mut d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  *d.open_mut() = '[';
  assert_eq!(d.open(), '[');
}

#[test]
fn close_ref() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  assert_eq!(d.close_ref(), &')');
}

#[test]
fn close_mut() {
  let mut d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  *d.close_mut() = ']';
  assert_eq!(d.close(), ']');
}

#[test]
fn span_ref() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(5, 10));
  assert_eq!(d.span_ref(), &SimpleSpan::new(5, 10));
}

#[test]
fn span_mut() {
  let mut d = Delimited::new('(', ')', 42i32, SimpleSpan::new(5, 10));
  d.span_mut().set_end(15);
  assert_eq!(d.span(), SimpleSpan::new(5, 15));
}

#[test]
fn data_ref() {
  let d = Delimited::new('(', ')', 100i32, SimpleSpan::new(0, 5));
  assert_eq!(*d.data(), 100);
}

#[test]
fn data_mut() {
  let mut d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  *d.data_mut() = 99;
  assert_eq!(*d.data(), 99);
}

// ── Deref / DerefMut ──────────────────────────────────────────────────────

#[test]
fn deref() {
  let d = Delimited::new('"', '"', "abc", SimpleSpan::new(0, 5));
  assert_eq!(d.len(), 3); // str::len via Deref
}

#[test]
fn deref_mut() {
  let mut d = Delimited::new('(', ')', 10i32, SimpleSpan::new(0, 3));
  *d += 5;
  assert_eq!(*d, 15);
}

// ── Display ────────────────────────────────────────────────────────────────

#[test]
fn display() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  assert_eq!(format!("{d}"), "42");
}

// ── AsRef<S> / AsSpan / IntoSpan ──────────────────────────────────────────

#[test]
fn as_ref_span() {
  use tokora::span::AsSpan;
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(5, 10));
  let s: &SimpleSpan = d.as_span();
  assert_eq!(*s, SimpleSpan::new(5, 10));
}

#[test]
fn into_span_trait() {
  use tokora::span::IntoSpan;
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(5, 10));
  let s: SimpleSpan = IntoSpan::into_span(d);
  assert_eq!(s, SimpleSpan::new(5, 10));
}

#[test]
fn asref_span_via_asref_trait() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(5, 10));
  let s: &SimpleSpan = AsRef::as_ref(&d);
  assert_eq!(*s, SimpleSpan::new(5, 10));
}

// ── Consuming into components ──────────────────────────────────────────────

#[test]
fn into_open() {
  let d = Delimited::new('[', ']', 0i32, SimpleSpan::new(0, 3));
  assert_eq!(d.into_open(), '[');
}

#[test]
fn into_close() {
  let d = Delimited::new('[', ']', 0i32, SimpleSpan::new(0, 3));
  assert_eq!(d.into_close(), ']');
}

#[test]
fn into_data() {
  let d = Delimited::new('[', ']', 42i32, SimpleSpan::new(0, 3));
  assert_eq!(d.into_data(), 42);
}

#[test]
fn into_span() {
  let d = Delimited::new('[', ']', 0i32, SimpleSpan::new(7, 12));
  assert_eq!(d.into_span(), SimpleSpan::new(7, 12));
}

#[test]
fn into_components() {
  let d = Delimited::new('[', ']', 42i32, SimpleSpan::new(5, 8));
  let (span, open, close, value) = d.into_components();
  assert_eq!(span, SimpleSpan::new(5, 8));
  assert_eq!(open, '[');
  assert_eq!(close, ']');
  assert_eq!(value, 42);
}

#[test]
fn into_components_via_trait() {
  use tokora::utils::IntoComponents;
  let d = Delimited::new('{', '}', "test", SimpleSpan::new(0, 6));
  let (span, open, close, data) = IntoComponents::into_components(d);
  assert_eq!(open, '{');
  assert_eq!(close, '}');
  assert_eq!(data, "test");
  assert_eq!(span, SimpleSpan::new(0, 6));
}

// ── Mapping ────────────────────────────────────────────────────────────────

#[test]
fn map_data() {
  let d = Delimited::new('"', '"', "42", SimpleSpan::new(0, 4));
  let parsed: Delimited<char, char, i32> = d.map_data(|s| s.parse().unwrap());
  assert_eq!(*parsed, 42);
  assert_eq!(parsed.open(), '"');
  assert_eq!(parsed.close(), '"');
  assert_eq!(parsed.span(), SimpleSpan::new(0, 4));
}

#[test]
fn map_open() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  let d2 = d.map_open(|c| c as u8);
  assert_eq!(d2.open(), b'(');
  assert_eq!(d2.close(), ')');
}

#[test]
fn map_close() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  let d2 = d.map_close(|c| c as u8);
  assert_eq!(d2.open(), '(');
  assert_eq!(d2.close(), b')');
}

#[test]
fn map_span() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  let d2 = d.map_span(|s| s.start());
  assert_eq!(d2.span(), 0usize);
  assert_eq!(*d2.data(), 42);
}

#[test]
fn map_all_components() {
  let d = Delimited::new('(', ')', 10i32, SimpleSpan::new(0, 4));
  let d2 = d.map(|o| o as u8, |c| c as u8, |v| v * 2, |s: SimpleSpan| s.end());
  assert_eq!(d2.open(), b'(');
  assert_eq!(d2.close(), b')');
  assert_eq!(*d2.data(), 20);
  assert_eq!(d2.span(), 4usize);
}

// ── Borrowed views ─────────────────────────────────────────────────────────

#[test]
fn as_ref_view() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  let borrowed = d.as_ref();
  assert_eq!(borrowed.open(), &'(');
  assert_eq!(borrowed.close(), &')');
  assert_eq!(borrowed.data(), &&42i32);
}

#[test]
fn as_mut_view() {
  let mut d = Delimited::new('(', ')', String::from("hello"), SimpleSpan::new(0, 7));
  {
    let mut borrowed = d.as_mut();
    borrowed.data_mut().push_str(" world");
  }
  assert_eq!(d.data(), &"hello world");
}

// ── Copied / Cloned ────────────────────────────────────────────────────────

#[test]
fn copied_from_ref_view() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  let borrowed = d.as_ref();
  let owned = borrowed.copied();
  assert_eq!(owned.open(), '(');
  assert_eq!(owned.close(), ')');
  assert_eq!(*owned.data(), 42);
}

#[test]
fn cloned_from_ref_view() {
  let d = Delimited::new('(', ')', String::from("hi"), SimpleSpan::new(0, 4));
  let borrowed = d.as_ref();
  let owned = borrowed.cloned();
  assert_eq!(owned.open(), '(');
  assert_eq!(owned.data(), &"hi");
}

// ── Ordering / Hash ────────────────────────────────────────────────────────

#[test]
fn ordering() {
  let d1 = Delimited::new('(', ')', 1i32, SimpleSpan::new(0, 3));
  let d2 = Delimited::new('(', ')', 2i32, SimpleSpan::new(0, 3));
  assert!(d1 < d2);
  assert!(d2 > d1);
  assert_eq!(d1, d1);
}

#[test]
fn hash() {
  use std::collections::HashSet;
  let mut set = HashSet::new();
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  set.insert(d);
  assert_eq!(set.len(), 1);
}

// ── Debug ─────────────────────────────────────────────────────────────────

#[test]
fn debug() {
  let d = Delimited::new('(', ')', 42i32, SimpleSpan::new(0, 4));
  let s = format!("{d:?}");
  assert!(s.contains("42"));
}
