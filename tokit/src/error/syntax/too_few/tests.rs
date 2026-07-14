use super::*;
use crate::span::SimpleSpan;

use std::format;

#[test]
fn too_few_new() {
  let err = TooFew::new(SimpleSpan::new(0, 5), 2, 5);
  assert_eq!(*err.span_ref(), SimpleSpan::new(0, 5));
  assert_eq!(err.nums(), 2);
  assert_eq!(err.limit(), 5);
}

#[test]
fn too_few_span_copy() {
  let err = TooFew::new(SimpleSpan::new(1, 3), 0, 1);
  assert_eq!(err.span(), SimpleSpan::new(1, 3));
}

#[test]
fn too_few_span_mut() {
  let mut err = TooFew::new(SimpleSpan::new(0, 5), 1, 3);
  *err.span_mut() = SimpleSpan::new(10, 15);
  assert_eq!(err.span(), SimpleSpan::new(10, 15));
}

#[test]
fn too_few_bump() {
  let mut err = TooFew::new(SimpleSpan::new(0, 5), 1, 3);
  err.bump(&10);
  assert_eq!(err.span(), SimpleSpan::new(10, 15));
}

#[test]
fn too_few_of_with_lang() {
  struct MyLang;
  let err = TooFew::<SimpleSpan, MyLang>::of(SimpleSpan::new(0, 5), 2, 10);
  assert_eq!(err.nums(), 2);
  assert_eq!(err.limit(), 10);
}

#[test]
fn too_few_into_unit() {
  let err = TooFew::new(SimpleSpan::new(0, 5), 1, 3);
  let _: () = err.into();
}

#[test]
fn too_few_display() {
  let err = TooFew::new(SimpleSpan::new(2, 8), 1, 3);
  let msg = format!("{err}");
  assert!(msg.contains("too few elements"));
  assert!(msg.contains("1"));
  assert!(msg.contains("3"));
}
