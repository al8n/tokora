use super::*;
use core::hash::Hash;

#[test]
fn new_without_knowledge() {
  let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
  assert_eq!(e.span(), SimpleSpan::new(0, 5));
  assert_eq!(e.knowledge(), None);
}

#[test]
fn with_knowledge_test() {
  let e: IncompleteToken<&str> = IncompleteToken::with_knowledge(SimpleSpan::new(0, 5), "int");
  assert_eq!(e.span(), SimpleSpan::new(0, 5));
  assert_eq!(e.knowledge(), Some(&"int"));
}

#[test]
fn into_components_no_knowledge() {
  let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(2, 8));
  let (span, knowledge) = e.into_components();
  assert_eq!(span, SimpleSpan::new(2, 8));
  assert_eq!(knowledge, None);
}

#[test]
fn into_components_with_knowledge() {
  let e: IncompleteToken<&str> = IncompleteToken::with_knowledge(SimpleSpan::new(2, 8), "float");
  let (span, knowledge) = e.into_components();
  assert_eq!(span, SimpleSpan::new(2, 8));
  assert_eq!(knowledge, Some("float"));
}

#[test]
fn bump_adjusts_span() {
  let mut e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
  e.bump(&10);
  assert_eq!(e.span(), SimpleSpan::new(10, 15));
}

#[test]
fn display_no_knowledge() {
  extern crate alloc;
  let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(3, 7));
  let s = alloc::format!("{e}");
  assert!(s.contains("incomplete token at"));
}

#[test]
fn display_with_knowledge() {
  extern crate alloc;
  use crate::utils::knowledge::IntLiteral;
  let e: IncompleteToken<IntLiteral> =
    IncompleteToken::with_knowledge(SimpleSpan::new(3, 7), IntLiteral(()));
  let s = alloc::format!("{e}");
  assert!(s.contains("incomplete"));
  assert!(s.contains("token at"));
}

#[test]
fn clone_and_eq() {
  let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
  assert_eq!(e, e.clone());
}

#[test]
fn debug_impl() {
  extern crate alloc;
  let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
  let s = alloc::format!("{e:?}");
  assert!(s.contains("IncompleteToken"));
}

#[test]
fn hash_impl() {
  let e: IncompleteToken<()> = IncompleteToken::new(SimpleSpan::new(0, 5));
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  e.hash(&mut hasher);
}
