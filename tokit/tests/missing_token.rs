#![cfg(feature = "std")]

use tokit::error::token::{Leading, MissingToken, Trailing};
use tokit::utils::CowStr;

#[test]
fn trailing_constructor() {
  let mt: MissingToken<'_, (), usize, Trailing<()>> = MissingToken::trailing(42);
  assert_eq!(*mt.offset_ref(), 42);
  assert!(mt.message().is_none());
  assert!(mt.expected().is_none());
}

#[test]
fn trailing_with_message() {
  let mt: MissingToken<'_, (), usize, Trailing<()>> =
    MissingToken::trailing_with_message(10, CowStr::from_static("expected comma"));
  assert_eq!(*mt.offset_ref(), 10);
  assert_eq!(mt.message().unwrap().as_str(), "expected comma");
}

#[test]
fn leading_constructor() {
  let mt: MissingToken<'_, (), usize, Leading<()>> = MissingToken::leading(5);
  assert_eq!(*mt.offset_ref(), 5);
  assert!(mt.message().is_none());
}

#[test]
fn leading_with_message() {
  let mt: MissingToken<'_, (), usize, Leading<()>> =
    MissingToken::leading_with_message(0, CowStr::from_static("need semicolon"));
  assert_eq!(*mt.offset_ref(), 0);
  assert_eq!(mt.message().unwrap().as_str(), "need semicolon");
}

#[test]
fn new_constructor() {
  let mt: MissingToken<'_, (), usize> = MissingToken::new(99);
  assert_eq!(*mt.offset_ref(), 99);
}

#[test]
fn with_message_builder() {
  let mt: MissingToken<'_, (), usize> =
    MissingToken::new(0).with_message(CowStr::from_static("hello"));
  assert_eq!(mt.message().unwrap().as_str(), "hello");
}

#[test]
fn offset_copy() {
  let mt: MissingToken<'_, (), usize> = MissingToken::new(42);
  assert_eq!(mt.offset(), 42);
}

#[test]
fn offset_mut() {
  let mut mt: MissingToken<'_, (), usize> = MissingToken::new(0);
  *mt.offset_mut() = 100;
  assert_eq!(mt.offset(), 100);
}

#[test]
fn message_mut() {
  let mut mt: MissingToken<'_, (), usize> =
    MissingToken::new(0).with_message(CowStr::from_static("old"));
  if let Some(m) = mt.message_mut() {
    *m = CowStr::from_static("new");
  }
  assert_eq!(mt.message().unwrap().as_str(), "new");
}

#[test]
fn into_components() {
  let mt: MissingToken<'_, (), usize> =
    MissingToken::new(42).with_message(CowStr::from_static("msg"));
  let (off, exp, msg) = mt.into_components();
  assert_eq!(off, 42);
  assert!(exp.is_none());
  assert_eq!(msg.unwrap().as_str(), "msg");
}

#[test]
fn display_formatting() {
  struct Show<'a>(MissingToken<'a, &'a str, usize>);

  impl core::fmt::Display for Show<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      self.0.display_fmt(f)
    }
  }

  let mt: MissingToken<'_, &str, usize> = MissingToken::new(0);
  let _ = format!("{}", Show(mt));
}

#[test]
fn debug_formatting() {
  struct Show<'a>(MissingToken<'a, &'a str, usize>);

  impl core::fmt::Debug for Show<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      self.0.debug_fmt(f)
    }
  }

  let mt: MissingToken<'_, &str, usize> = MissingToken::new(0);
  let _ = format!("{:?}", Show(mt));
}

#[test]
fn from_missing_token_for_unit() {
  let mt: MissingToken<'_, (), usize> = MissingToken::new(0);
  let _: () = mt.into();
}
