use super::*;
use crate::span::SimpleSpan;

use std::format;

#[test]
fn full_container_new() {
  let err = FullContainer::new(SimpleSpan::new(0, 5), 10, 5);
  assert_eq!(*err.span(), SimpleSpan::new(0, 5));
  assert_eq!(err.nums(), 10);
  assert_eq!(err.capacity(), 5);
}

#[test]
fn full_container_of_with_lang() {
  struct MyLang;
  let err = FullContainer::<SimpleSpan, MyLang>::of(SimpleSpan::new(0, 5), 10, 5);
  assert_eq!(err.nums(), 10);
  assert_eq!(err.capacity(), 5);
}

#[test]
fn full_container_bump() {
  let mut err = FullContainer::new(SimpleSpan::new(0, 5), 10, 5);
  err.bump(&10);
  assert_eq!(*err.span(), SimpleSpan::new(10, 15));
}

#[test]
fn full_container_into_unit() {
  let err = FullContainer::new(SimpleSpan::new(0, 5), 10, 5);
  let _: () = err.into();
}

#[test]
fn full_container_display() {
  let err = FullContainer::new(SimpleSpan::new(2, 8), 10, 5);
  let msg = format!("{err}");
  assert!(msg.contains("10"));
  assert!(msg.contains("5"));
  assert!(msg.contains("exceeds the maximum capacity"));
}
