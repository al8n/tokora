use super::*;

use std::format;

// --- ParseAttempt tests ---

#[test]
fn accept_is_accepted() {
  let pa = Accept(42);
  assert!(pa.is_accept());
  assert!(!pa.is_decline());
}

#[test]
fn decline_is_declined() {
  let pa: ParseAttempt<i32> = Decline;
  assert!(!pa.is_accept());
  assert!(pa.is_decline());
}

#[test]
fn accept_map() {
  let pa = Accept(10);
  let mapped = pa.map(|v| v + 1);
  assert_eq!(mapped, Accept(11));
}

#[test]
fn decline_map() {
  let pa: ParseAttempt<i32> = Decline;
  let mapped = pa.map(|v: i32| v + 1);
  assert_eq!(mapped, Decline);
}

#[test]
fn accept_as_ref() {
  let pa = Accept(42);
  let r = pa.as_ref();
  assert_eq!(r, Accept(&42));
}

#[test]
fn decline_as_ref() {
  let pa: ParseAttempt<i32> = Decline;
  let r = pa.as_ref();
  assert!(r.is_decline());
}

#[test]
fn accept_as_mut() {
  let mut pa = Accept(42);
  let r = pa.as_mut();
  assert!(r.is_accept());
}

#[test]
fn decline_as_mut() {
  let mut pa: ParseAttempt<i32> = Decline;
  let r = pa.as_mut();
  assert!(r.is_decline());
}

#[test]
fn accept_and_then_ok() {
  let pa = Accept(10);
  let result: Result<ParseAttempt<i32>, &str> = pa.and_then(|v| Ok(v + 1));
  assert_eq!(result, Ok(Accept(11)));
}

#[test]
fn accept_and_then_err() {
  let pa = Accept(10);
  let result: Result<ParseAttempt<i32>, &str> = pa.and_then(|_| Err("fail"));
  assert_eq!(result, Err("fail"));
}

#[test]
fn decline_and_then() {
  let pa: ParseAttempt<i32> = Decline;
  let result: Result<ParseAttempt<i32>, &str> = pa.and_then(|v| Ok(v + 1));
  assert_eq!(result, Ok(Decline));
}

// --- From/Into conversions ---

#[test]
fn from_some_to_accept() {
  let pa: ParseAttempt<i32> = Some(42).into();
  assert_eq!(pa, Accept(42));
}

#[test]
fn from_none_to_decline() {
  let pa: ParseAttempt<i32> = None.into();
  assert_eq!(pa, Decline);
}

#[test]
fn accept_into_some() {
  let opt: Option<i32> = Accept(42).into();
  assert_eq!(opt, Some(42));
}

#[test]
fn decline_into_none() {
  let opt: Option<i32> = Decline.into();
  assert_eq!(opt, None);
}

// --- Derived methods ---

#[test]
fn accept_unwrap() {
  let pa = Accept(42);
  assert_eq!(pa.unwrap_accept_ref(), &42);
}

#[test]
fn accept_try_unwrap() {
  let pa = Accept(42);
  assert_eq!(pa.try_unwrap_accept_ref(), Ok(&42));
}

#[test]
fn decline_try_unwrap() {
  let pa: ParseAttempt<i32> = Decline;
  assert!(pa.try_unwrap_accept_ref().is_err());
}

#[test]
fn accept_clone_eq() {
  let pa = Accept(42);
  let cloned = pa.clone();
  assert_eq!(pa, cloned);
}

#[test]
fn decline_clone_eq() {
  let pa: ParseAttempt<i32> = Decline;
  let cloned = pa.clone();
  assert_eq!(pa, cloned);
}

#[test]
fn accept_debug() {
  let pa = Accept(42);
  let dbg = format!("{:?}", pa);
  assert!(dbg.contains("Accept"));
  assert!(dbg.contains("42"));
}

#[test]
fn decline_debug() {
  let pa: ParseAttempt<i32> = Decline;
  let dbg = format!("{:?}", pa);
  assert!(dbg.contains("Decline"));
}
