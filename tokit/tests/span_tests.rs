/// Tests for span.rs — SimpleSpan and Spanned types.
use tokit::SimpleSpan;
use tokit::span::{Span, Spanned};

// ── SimpleSpan::try_new ────────────────────────────────────────────────────

#[test]
fn simple_span_try_new_valid() {
  assert_eq!(SimpleSpan::try_new(5, 10), Some(SimpleSpan::new(5, 10)));
}

#[test]
fn simple_span_try_new_equal() {
  assert_eq!(SimpleSpan::try_new(5, 5), Some(SimpleSpan::new(5, 5)));
}

#[test]
fn simple_span_try_new_invalid() {
  assert_eq!(SimpleSpan::try_new(10usize, 5usize), None);
}

// ── SimpleSpan::const_new ─────────────────────────────────────────────────

#[test]
fn simple_span_const_new() {
  let s = SimpleSpan::const_new(3, 8);
  assert_eq!(s.start(), 3);
  assert_eq!(s.end(), 8);
}

#[test]
fn simple_span_try_const_new_valid() {
  assert_eq!(SimpleSpan::try_const_new(3, 8), Some(SimpleSpan::new(3, 8)));
}

#[test]
fn simple_span_try_const_new_invalid() {
  assert_eq!(SimpleSpan::try_const_new(8, 3), None);
}

// ── SimpleSpan::bump_start / bump_end / bump ──────────────────────────────

#[test]
fn simple_span_bump_start() {
  let mut s = SimpleSpan::new(5, 15);
  s.bump_start(3);
  assert_eq!(s, SimpleSpan::new(8, 15));
}

#[test]
fn simple_span_bump_end() {
  let mut s = SimpleSpan::new(5, 15);
  s.bump_end(5usize);
  assert_eq!(s, SimpleSpan::new(5, 20));
}

#[test]
fn simple_span_bump() {
  let mut s = SimpleSpan::new(5, 15);
  s.bump(&10usize);
  assert_eq!(s, SimpleSpan::new(15, 25));
}

// ── SimpleSpan const bump variants ───────────────────────────────────────

#[test]
fn simple_span_bump_start_const() {
  let mut s = SimpleSpan::new(5, 15);
  s.bump_start_const(3);
  assert_eq!(s, SimpleSpan::new(8, 15));
}

#[test]
fn simple_span_bump_end_const() {
  let mut s = SimpleSpan::new(5, 15);
  s.bump_end_const(5);
  assert_eq!(s, SimpleSpan::new(5, 20));
}

#[test]
fn simple_span_bump_const() {
  let mut s = SimpleSpan::new(5, 15);
  s.bump_const(10);
  assert_eq!(s, SimpleSpan::new(15, 25));
}

// ── SimpleSpan::set_start / set_end ──────────────────────────────────────

#[test]
fn simple_span_set_start() {
  let mut s = SimpleSpan::new(5, 15);
  s.set_start(10);
  assert_eq!(s, SimpleSpan::new(10, 15));
}

#[test]
fn simple_span_set_end() {
  let mut s = SimpleSpan::new(5, 15);
  s.set_end(20);
  assert_eq!(s, SimpleSpan::new(5, 20));
}

#[test]
fn simple_span_set_start_const() {
  let mut s = SimpleSpan::new(5, 15);
  s.set_start_const(10);
  assert_eq!(s, SimpleSpan::new(10, 15));
}

#[test]
fn simple_span_set_end_const() {
  let mut s = SimpleSpan::new(5, 15);
  s.set_end_const(20);
  assert_eq!(s, SimpleSpan::new(5, 20));
}

// ── SimpleSpan::with_start / with_end ────────────────────────────────────

#[test]
fn simple_span_with_start() {
  let s = SimpleSpan::new(5, 15).with_start(10);
  assert_eq!(s, SimpleSpan::new(10, 15));
}

#[test]
fn simple_span_with_end() {
  let s = SimpleSpan::new(5, 15).with_end(20);
  assert_eq!(s, SimpleSpan::new(5, 20));
}

#[test]
fn simple_span_with_start_const() {
  let s = SimpleSpan::new(5, 15).with_start_const(10);
  assert_eq!(s, SimpleSpan::new(10, 15));
}

#[test]
fn simple_span_with_end_const() {
  let s = SimpleSpan::new(5, 15).with_end_const(20);
  assert_eq!(s, SimpleSpan::new(5, 20));
}

// ── SimpleSpan refs ──────────────────────────────────────────────────────

#[test]
fn simple_span_start_ref() {
  let s = SimpleSpan::new(5, 15);
  assert_eq!(*s.start_ref(), 5);
}

#[test]
fn simple_span_end_ref() {
  let s = SimpleSpan::new(5, 15);
  assert_eq!(*s.end_ref(), 15);
}

#[test]
fn simple_span_start_mut() {
  let mut s = SimpleSpan::new(5, 15);
  *s.start_mut() = 8;
  assert_eq!(s.start(), 8);
}

#[test]
fn simple_span_end_mut() {
  let mut s = SimpleSpan::new(5, 15);
  *s.end_mut() = 20;
  assert_eq!(s.end(), 20);
}

// ── SimpleSpan::range ────────────────────────────────────────────────────

#[test]
fn simple_span_range() {
  let s = SimpleSpan::new(5, 15);
  let r = s.range();
  assert_eq!(*r.start, 5);
  assert_eq!(*r.end, 15);
}

// ── SimpleSpan::as_ref / as_mut ───────────────────────────────────────────

#[test]
fn simple_span_as_ref() {
  let s = SimpleSpan::new(5, 15);
  let r = s.as_ref();
  assert_eq!(**r.start_ref(), 5);
  assert_eq!(**r.end_ref(), 15);
}

#[test]
fn simple_span_as_ref_cloned() {
  let s = SimpleSpan::new(5, 15);
  let r = s.as_ref();
  let cloned: SimpleSpan<usize> = r.cloned();
  assert_eq!(cloned, s);
}

#[test]
fn simple_span_as_mut() {
  let mut s = SimpleSpan::new(5, 15);
  {
    let mut m = s.as_mut();
    **m.start_mut() = 10;
    **m.end_mut() = 20;
  }
  assert_eq!(s, SimpleSpan::new(10, 20));
}

// ── SimpleSpan From conversions ───────────────────────────────────────────

#[test]
fn simple_span_from_range() {
  let s: SimpleSpan = (5..15).into();
  assert_eq!(s, SimpleSpan::new(5, 15));
}

#[test]
fn simple_span_into_range() {
  let s = SimpleSpan::new(5, 15);
  let r: core::ops::Range<usize> = s.into();
  assert_eq!(r, 5..15);
}

#[test]
fn simple_span_from_tuple() {
  let s: SimpleSpan = (5usize, 15usize).into();
  assert_eq!(s, SimpleSpan::new(5, 15));
}

#[test]
fn simple_span_into_tuple() {
  let s = SimpleSpan::new(5, 15);
  let t: (usize, usize) = s.into();
  assert_eq!(t, (5, 15));
}

// ── Span trait impl for Range<usize> ─────────────────────────────────────

#[test]
fn range_span_new() {
  let r = <core::ops::Range<usize> as Span>::new(3, 8);
  assert_eq!(r, 3..8);
}

#[test]
fn range_span_start_ref() {
  let r = 3usize..8;
  assert_eq!(*r.start_ref(), 3);
}

#[test]
fn range_span_end_ref() {
  let r = 3usize..8;
  assert_eq!(*r.end_ref(), 8);
}

#[test]
fn range_span_start_mut() {
  let mut r = 3usize..8;
  *r.start_mut() = 5;
  assert_eq!(r.start, 5);
}

#[test]
fn range_span_end_mut() {
  let mut r = 3usize..8;
  *r.end_mut() = 10;
  assert_eq!(r.end, 10);
}

#[test]
fn range_span_into_start() {
  let r = 3usize..8;
  assert_eq!(r.into_start(), 3);
}

#[test]
fn range_span_into_end() {
  let r = 3usize..8;
  assert_eq!(r.into_end(), 8);
}

#[test]
fn range_span_bump() {
  let mut r = 3usize..8;
  r.bump(&5);
  assert_eq!(r.end, 13);
}

#[test]
fn range_span_into_range() {
  let r = 3usize..8;
  let r2 = r.clone().into_range();
  assert_eq!(r2, r);
}

#[test]
fn range_span_start_end() {
  let r = 3usize..8;
  assert_eq!(r.start(), 3);
  assert_eq!(r.end(), 8);
}

// ── Span trait impl for SimpleSpan ───────────────────────────────────────

#[test]
fn simple_span_trait_into_range() {
  let s = SimpleSpan::new(5, 15);
  let r = <SimpleSpan as Span>::into_range(s);
  assert_eq!(r, 5..15);
}

#[test]
fn simple_span_trait_into_start() {
  let s = SimpleSpan::new(5, 15);
  assert_eq!(<SimpleSpan as Span>::into_start(s), 5);
}

#[test]
fn simple_span_trait_into_end() {
  let s = SimpleSpan::new(5, 15);
  assert_eq!(<SimpleSpan as Span>::into_end(s), 15);
}

#[test]
fn simple_span_trait_bump() {
  let mut s = SimpleSpan::new(5, 15);
  <SimpleSpan as Span>::bump(&mut s, &10);
  assert_eq!(s, SimpleSpan::new(15, 25));
}

#[test]
fn simple_span_trait_start_mut() {
  let mut s = SimpleSpan::new(5, 15);
  *<SimpleSpan as Span>::start_mut(&mut s) = 8;
  assert_eq!(s.start(), 8);
}

#[test]
fn simple_span_trait_end_mut() {
  let mut s = SimpleSpan::new(5, 15);
  *<SimpleSpan as Span>::end_mut(&mut s) = 20;
  assert_eq!(s.end(), 20);
}

// ── Spanned ───────────────────────────────────────────────────────────────

#[test]
fn spanned_new() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  assert_eq!(sp.span(), SimpleSpan::new(5, 10));
  assert_eq!(*sp.data(), 42);
}

#[test]
fn spanned_span_ref() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), "hello");
  assert_eq!(sp.span_ref(), &SimpleSpan::new(5, 10));
}

#[test]
fn spanned_span_mut() {
  let mut sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  sp.span_mut().set_end(20);
  assert_eq!(sp.span().end(), 20);
}

#[test]
fn spanned_data_mut() {
  let mut sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  *sp.data_mut() = 100;
  assert_eq!(*sp.data(), 100);
}

#[test]
fn spanned_into_span() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  assert_eq!(sp.into_span(), SimpleSpan::new(5, 10));
}

#[test]
fn spanned_into_data() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  assert_eq!(sp.into_data(), 42);
}

#[test]
fn spanned_into_components() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  let (span, data) = sp.into_components();
  assert_eq!(span, SimpleSpan::new(5, 10));
  assert_eq!(data, 42);
}

#[test]
fn spanned_map_data() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  let sp2 = sp.map_data(|x| x * 2);
  assert_eq!(*sp2.data(), 84);
  assert_eq!(sp2.span(), SimpleSpan::new(5, 10));
}

#[test]
fn spanned_map_span() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  let sp2 = sp.map_span(|s| SimpleSpan::new(s.start() + 1, s.end() + 1));
  assert_eq!(sp2.span(), SimpleSpan::new(6, 11));
  assert_eq!(*sp2.data(), 42);
}

#[test]
fn spanned_map() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  let sp2 = sp.map(|s| SimpleSpan::new(s.start() * 2, s.end() * 2), |x| x + 1);
  assert_eq!(sp2.span(), SimpleSpan::new(10, 20));
  assert_eq!(*sp2.data(), 43);
}

#[test]
fn spanned_deref() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), "hello");
  assert_eq!(sp.len(), 5); // calls str::len via Deref
}

#[test]
fn spanned_deref_mut() {
  let mut sp = Spanned::new(SimpleSpan::new(0, 1), 10i32);
  *sp += 5;
  assert_eq!(*sp, 15);
}

#[test]
fn spanned_display() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  assert_eq!(sp.to_string(), "42");
}

#[test]
fn spanned_as_ref() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), "hello");
  let r = sp.as_ref();
  assert_eq!(*r.data(), &"hello");
  assert_eq!(*r.span_ref(), &SimpleSpan::new(5, 10));
}

#[test]
fn spanned_as_mut() {
  let mut sp = Spanned::new(SimpleSpan::new(5, 10), String::from("hello"));
  {
    let mut m = sp.as_mut();
    m.data.push_str(" world");
  }
  assert_eq!(sp.data(), &"hello world");
}

#[test]
fn spanned_as_ref_cloned() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42i32);
  let r: Spanned<&i32, &SimpleSpan> = sp.as_ref();
  let cloned = r.cloned();
  assert_eq!(cloned.span(), SimpleSpan::new(5, 10));
  assert_eq!(*cloned.data(), 42);
}

#[test]
fn spanned_as_ref_copied() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42i32);
  let r: Spanned<&i32, &SimpleSpan> = sp.as_ref();
  let copied = r.copied();
  assert_eq!(copied.span(), SimpleSpan::new(5, 10));
  assert_eq!(*copied.data(), 42);
}

#[test]
fn spanned_as_span_trait() {
  use tokit::span::AsSpan;
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  assert_eq!(sp.as_span(), &SimpleSpan::new(5, 10));
}

#[test]
fn spanned_into_span_trait() {
  use tokit::span::IntoSpan;
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  assert_eq!(sp.into_span(), SimpleSpan::new(5, 10));
}

#[test]
fn spanned_as_ref_trait() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  let r: &SimpleSpan = AsRef::as_ref(&sp);
  assert_eq!(*r, SimpleSpan::new(5, 10));
}

#[test]
fn spanned_from_into_unit() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  let _unit: () = sp.into();
}

#[test]
fn spanned_debug() {
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  let s = format!("{sp:?}");
  assert!(!s.is_empty());
}

#[test]
fn spanned_ordering() {
  let a = Spanned::new(SimpleSpan::new(1, 5), 10);
  let b = Spanned::new(SimpleSpan::new(5, 10), 20);
  assert!(a < b);
}

#[test]
fn spanned_hash() {
  use std::collections::HashSet;
  let sp = Spanned::new(SimpleSpan::new(5, 10), 42);
  let mut set = HashSet::new();
  set.insert(sp);
  assert!(set.contains(&sp));
}
