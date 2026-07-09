use super::*;
use core::hash::Hash;

type UsSuffix = UnexpectedSuffix<u8, ()>;

fn make_char_error() -> UsSuffix {
  UnexpectedSuffix::new(
    SimpleSpan::new(0, 5),
    Lexeme::Char(PositionedChar::with_position(b'x', 5)),
  )
}

fn make_range_error() -> UsSuffix {
  UnexpectedSuffix::from_suffix(SimpleSpan::new(0, 5), SimpleSpan::new(5, 10))
}

#[test]
fn new_creates_error() {
  let e = make_char_error();
  assert_eq!(e.token(), SimpleSpan::new(0, 5));
}

#[test]
fn from_char_creates_error() {
  let e: UsSuffix = UnexpectedSuffix::from_char(SimpleSpan::new(0, 5), 5, b'x');
  assert_eq!(e.token(), SimpleSpan::new(0, 5));
}

#[test]
fn from_positioned_char_creates_error() {
  let e: UsSuffix = UnexpectedSuffix::from_positioned_char(
    SimpleSpan::new(0, 5),
    PositionedChar::with_position(b'x', 5),
  );
  assert_eq!(e.token(), SimpleSpan::new(0, 5));
}

#[test]
fn from_suffix_creates_error() {
  let e = make_range_error();
  assert_eq!(e.token(), SimpleSpan::new(0, 5));
}

#[test]
fn with_knowledge_method() {
  let e = make_char_error().with_knowledge(());
  assert_eq!(e.token(), SimpleSpan::new(0, 5));
}

#[test]
fn with_knowledge_const_method() {
  let e = make_char_error().with_knowledge_const(());
  assert_eq!(e.token(), SimpleSpan::new(0, 5));
}

#[test]
fn span_char_variant() {
  let e = make_char_error();
  assert_eq!(e.span(), SimpleSpan::new(0, 6));
}

#[test]
fn span_range_variant() {
  let e = make_range_error();
  assert_eq!(e.span(), SimpleSpan::new(0, 10));
}

#[test]
fn token_ref_method() {
  let e = make_char_error();
  assert_eq!(e.token_ref(), SimpleSpan::new(&0, &5));
}

#[test]
fn suffix_accessor() {
  let e = make_char_error();
  assert_eq!(
    e.suffix(),
    &Lexeme::Char(PositionedChar::with_position(b'x', 5))
  );
}

#[test]
fn into_components_test() {
  let e = make_char_error();
  let (token, suffix) = e.into_components();
  assert_eq!(token, SimpleSpan::new(0, 5));
  assert_eq!(suffix, Lexeme::Char(PositionedChar::with_position(b'x', 5)));
}

#[test]
fn bump_test() {
  let mut e = make_char_error();
  e.bump(&10);
  assert_eq!(e.token(), SimpleSpan::new(10, 15));
}

#[test]
fn display_char_no_knowledge() {
  extern crate alloc;
  let e = make_char_error();
  let s = alloc::format!("{e}");
  assert!(s.contains("unexpected suffix"));
  assert!(s.contains("position 5"));
}

#[test]
fn display_char_with_knowledge() {
  extern crate alloc;
  let e = make_char_error().with_knowledge(());
  let s = alloc::format!("{e}");
  assert!(s.contains("unexpected suffix"));
}

#[test]
fn display_range_no_knowledge() {
  extern crate alloc;
  let e = make_range_error();
  let s = alloc::format!("{e}");
  assert!(s.contains("unexpected suffix at"));
}

#[test]
fn display_range_with_knowledge() {
  extern crate alloc;
  let e = make_range_error().with_knowledge(());
  let s = alloc::format!("{e}");
  assert!(s.contains("unexpected suffix at"));
}

#[test]
fn clone_and_eq() {
  let e = make_char_error();
  let e2 = e.clone();
  assert_eq!(e, e2);
}

#[test]
fn debug_impl() {
  extern crate alloc;
  let e = make_char_error();
  let s = alloc::format!("{e:?}");
  assert!(s.contains("UnexpectedSuffix"));
}

#[test]
fn hash_impl() {
  let e = make_char_error();
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  e.hash(&mut hasher);
}
