use super::*;
use core::hash::Hash;

type UpError = UnexpectedPrefix<u8, ()>;

fn make_char_error() -> UpError {
  UnexpectedPrefix::new(
    SimpleSpan::new(1, 5),
    Lexeme::Char(PositionedChar::with_position(b'x', 0)),
  )
}

fn make_range_error() -> UpError {
  UnexpectedPrefix::from_prefix(SimpleSpan::new(6, 10), SimpleSpan::new(0, 6))
}

#[test]
fn new_creates_error() {
  let e = make_char_error();
  assert_eq!(e.token(), SimpleSpan::new(1, 5));
}

#[test]
fn from_char_creates_error() {
  let e: UpError = UnexpectedPrefix::from_char(SimpleSpan::new(1, 5), 0, b'x');
  assert_eq!(e.token(), SimpleSpan::new(1, 5));
}

#[test]
fn from_positioned_char_creates_error() {
  let e: UpError = UnexpectedPrefix::from_positioned_char(
    SimpleSpan::new(1, 5),
    PositionedChar::with_position(b'x', 0),
  );
  assert_eq!(e.token(), SimpleSpan::new(1, 5));
}

#[test]
fn from_prefix_creates_error() {
  let e = make_range_error();
  assert_eq!(e.token(), SimpleSpan::new(6, 10));
}

#[test]
fn with_knowledge_method() {
  let e = make_char_error().with_knowledge(());
  assert_eq!(e.token(), SimpleSpan::new(1, 5));
}

#[test]
fn with_knowledge_const_method() {
  let e = make_char_error().with_knowledge_const(());
  assert_eq!(e.token(), SimpleSpan::new(1, 5));
}

#[test]
fn span_char_variant() {
  let e = make_char_error();
  assert_eq!(e.span(), SimpleSpan::new(0, 5));
}

#[test]
fn span_range_variant() {
  let e = make_range_error();
  assert_eq!(e.span(), SimpleSpan::new(0, 10));
}

#[test]
fn prefix_accessor() {
  let e = make_char_error();
  assert_eq!(
    e.prefix(),
    &Lexeme::Char(PositionedChar::with_position(b'x', 0))
  );
}

#[test]
fn into_components_test() {
  let e = make_char_error();
  let (token, prefix) = e.into_components();
  assert_eq!(token, SimpleSpan::new(1, 5));
  assert_eq!(prefix, Lexeme::Char(PositionedChar::with_position(b'x', 0)));
}

#[test]
fn bump_test() {
  let mut e = make_char_error();
  e.bump(&10);
  assert_eq!(e.token(), SimpleSpan::new(11, 15));
}

#[test]
fn display_char_no_knowledge() {
  extern crate alloc;
  let e = make_char_error();
  let s = alloc::format!("{e}");
  assert!(s.contains("unexpected prefix"));
  assert!(s.contains("position 0"));
}

#[test]
fn display_char_with_knowledge() {
  extern crate alloc;
  let e = make_char_error().with_knowledge(());
  let s = alloc::format!("{e}");
  assert!(s.contains("unexpected prefix"));
}

#[test]
fn display_range_no_knowledge() {
  extern crate alloc;
  let e = make_range_error();
  let s = alloc::format!("{e}");
  assert!(s.contains("unexpected prefix at"));
}

#[test]
fn display_range_with_knowledge() {
  extern crate alloc;
  let e = make_range_error().with_knowledge(());
  let s = alloc::format!("{e}");
  assert!(s.contains("unexpected prefix at"));
}

#[test]
fn clone_and_eq() {
  let e = make_char_error();
  assert_eq!(e, e.clone());
}

#[test]
fn debug_impl() {
  extern crate alloc;
  let e = make_char_error();
  let s = alloc::format!("{e:?}");
  assert!(s.contains("UnexpectedPrefix"));
}

#[test]
fn hash_impl() {
  let e = make_char_error();
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  e.hash(&mut hasher);
}

#[test]
fn leading_zero() {
  use crate::utils::knowledge::IntLiteral;
  let e: UnexpectedPrefix<u8, IntLiteral> =
    UnexpectedPrefix::leading_zero(SimpleSpan::new(1, 5), 0, b'0');
  assert_eq!(e.token(), SimpleSpan::new(1, 5));
}

#[test]
fn leading_zeros() {
  use crate::utils::knowledge::IntLiteral;
  let e: UnexpectedPrefix<u8, IntLiteral> =
    UnexpectedPrefix::leading_zeros(SimpleSpan::new(6, 10), SimpleSpan::new(0, 6));
  assert_eq!(e.token(), SimpleSpan::new(6, 10));
}
