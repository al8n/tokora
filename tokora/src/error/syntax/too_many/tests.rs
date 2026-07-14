use super::*;
use crate::span::SimpleSpan;

use std::format;

#[test]
fn too_many_new() {
  let err = TooMany::new(SimpleSpan::new(0, 5), 10, 5);
  assert_eq!(*err.span_ref(), SimpleSpan::new(0, 5));
  assert_eq!(err.nums(), 10);
  assert_eq!(err.limit(), 5);
}

#[test]
fn too_many_span_copy() {
  let err = TooMany::new(SimpleSpan::new(1, 3), 5, 3);
  assert_eq!(err.span(), SimpleSpan::new(1, 3));
}

#[test]
fn too_many_span_mut() {
  let mut err = TooMany::new(SimpleSpan::new(0, 5), 10, 5);
  *err.span_mut() = SimpleSpan::new(10, 15);
  assert_eq!(err.span(), SimpleSpan::new(10, 15));
}

#[test]
fn too_many_bump() {
  let mut err = TooMany::new(SimpleSpan::new(0, 5), 10, 5);
  err.bump(&10);
  assert_eq!(err.span(), SimpleSpan::new(10, 15));
}

#[test]
fn too_many_of_with_lang() {
  struct MyLang;
  let err = TooMany::<SimpleSpan, MyLang>::of(SimpleSpan::new(0, 5), 10, 5);
  assert_eq!(err.nums(), 10);
  assert_eq!(err.limit(), 5);
}

#[test]
fn too_many_into_unit() {
  let err = TooMany::new(SimpleSpan::new(0, 5), 10, 5);
  let _: () = err.into();
}

#[test]
fn too_many_display() {
  let err = TooMany::new(SimpleSpan::new(2, 8), 10, 5);
  let msg = format!("{err}");
  assert!(msg.contains("too many elements"));
  assert!(msg.contains("10"));
  assert!(msg.contains("5"));
}
