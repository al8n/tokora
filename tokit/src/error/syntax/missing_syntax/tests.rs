use super::*;

use std::format;

#[test]
fn missing_syntax_new() {
  let err = MissingSyntax::new(10usize);
  assert_eq!(err.offset(), 10);
  assert!(err.message().is_none());
}

#[test]
fn missing_syntax_with_message() {
  let err = MissingSyntax::with_message(20, CowStr::from_static("expected expression"));
  assert_eq!(err.offset(), 20);
  assert_eq!(err.message().unwrap().as_str(), "expected expression");
}

#[test]
fn missing_syntax_offset_ref() {
  let err = MissingSyntax::new(15usize);
  assert_eq!(err.offset_ref(), &15);
}

#[test]
fn missing_syntax_offset_mut() {
  let mut err = MissingSyntax::new(10usize);
  *err.offset_mut() = 20;
  assert_eq!(err.offset(), 20);
}

#[test]
fn missing_syntax_message_mut() {
  let mut err = MissingSyntax::with_message(10, CowStr::from_static("original"));
  if let Some(msg) = err.message_mut() {
    *msg = CowStr::from_static("updated");
  }
  assert_eq!(err.message().unwrap().as_str(), "updated");
}

#[test]
fn missing_syntax_bump() {
  let mut err = MissingSyntax::new(10usize);
  err.bump(&5);
  assert_eq!(err.offset(), 15);
}

#[test]
fn missing_syntax_into_components() {
  let err = MissingSyntax::new(10usize);
  let (offset, msg) = err.into_components();
  assert_eq!(offset, 10);
  assert!(msg.is_none());
}

#[test]
fn missing_syntax_into_components_with_message() {
  let err = MissingSyntax::with_message(20, CowStr::from_static("test"));
  let (offset, msg) = err.into_components();
  assert_eq!(offset, 20);
  assert_eq!(msg.unwrap().as_str(), "test");
}

#[test]
fn missing_syntax_into_unit() {
  let err = MissingSyntax::new(10usize);
  let _: () = err.into();
}

#[test]
fn missing_syntax_of_with_lang() {
  struct MyLang;
  let err = MissingSyntax::<usize, MyLang>::of(10);
  assert_eq!(err.offset(), 10);
}

#[test]
fn missing_syntax_display_no_message() {
  let err = MissingSyntax::new(10usize);
  let msg = format!("{err}");
  assert_eq!(msg, "missing syntax at offset 10");
}

#[test]
fn missing_syntax_display_with_message() {
  let err = MissingSyntax::with_message(20usize, CowStr::from_static("expected ident"));
  let msg = format!("{err}");
  assert_eq!(msg, "missing syntax at offset 20: expected ident");
}

#[test]
fn missing_syntax_debug() {
  let err = MissingSyntax::new(10usize);
  let msg = format!("{err:?}");
  assert!(msg.contains("MissingSyntax"));
  assert!(msg.contains("10"));
}
