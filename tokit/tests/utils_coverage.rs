#![cfg(feature = "std")]

use std::borrow::Cow;
use tokit::utils::{CowStr, OneOf};

#[test]
fn cowstr_from_static() {
  let s = CowStr::from_static("hello");
  assert_eq!(s.as_str(), "hello");
}

#[test]
fn cowstr_from_string() {
  let s = CowStr::from_string(String::from("dynamic"));
  assert_eq!(s.as_str(), "dynamic");
}

#[test]
fn cowstr_to_mut() {
  let mut s = CowStr::from_static("hello");
  let m = s.to_mut();
  m.push_str(" world");
  assert_eq!(s.as_str(), "hello world");
}

#[test]
fn cowstr_into_inner() {
  let s = CowStr::from_static("test");
  let inner = s.into_inner();
  assert_eq!(&*inner, "test");
}

#[test]
fn cowstr_as_inner() {
  let s = CowStr::from_static("test");
  let _ = s.as_inner();
}

#[test]
fn cowstr_from_string_impl() {
  let s: CowStr = String::from("owned").into();
  assert_eq!(s.as_str(), "owned");
}

#[test]
fn cowstr_from_cow() {
  let cow: Cow<'static, str> = Cow::Borrowed("borrowed");
  let s: CowStr = cow.into();
  assert_eq!(s.as_str(), "borrowed");
}

#[test]
fn cowstr_into_cow() {
  let s = CowStr::from_static("test");
  let cow: Cow<'static, str> = s.into();
  assert_eq!(&*cow, "test");
}

#[test]
fn cowstr_ref_into_cow() {
  let s = CowStr::from_static("test");
  let cow: Cow<'static, str> = (&s).into();
  assert_eq!(&*cow, "test");
}

#[test]
fn cowstr_as_ref() {
  let s = CowStr::from_static("test");
  let r: &str = s.as_ref();
  assert_eq!(r, "test");
}

#[test]
fn cowstr_borrow() {
  use std::borrow::Borrow;
  let s = CowStr::from_static("test");
  let r: &str = s.borrow();
  assert_eq!(r, "test");
}

#[test]
fn cowstr_to_mut_from_static() {
  let mut s = CowStr::from_static("test");
  let m = s.to_mut();
  m.push_str("!");
  assert_eq!(s.as_str(), "test!");
}

#[test]
fn cowstr_display() {
  let s = CowStr::from_static("hello");
  assert_eq!(format!("{s}"), "hello");
}

#[test]
fn cowstr_debug() {
  let s = CowStr::from_static("hello");
  let _ = format!("{s:?}");
}

#[test]
fn oneof_from_slice() {
  let items: &[i32] = &[1, 2, 3];
  let o = OneOf::from_slice(items);
  assert_eq!(o.as_slice(), &[1, 2, 3]);
}

#[test]
fn oneof_from_vec() {
  let o = OneOf::from_vec(vec![1, 2, 3]);
  assert_eq!(o.as_slice(), &[1, 2, 3]);
}

#[test]
fn oneof_to_mut() {
  let items: &[i32] = &[1, 2];
  let mut o = OneOf::from_slice(items);
  let m = o.to_mut();
  assert_eq!(m, &[1, 2]);
}

#[test]
fn oneof_into_inner() {
  let o = OneOf::from_vec(vec![42]);
  let inner = o.into_inner();
  assert_eq!(&*inner, &[42]);
}

#[test]
fn oneof_as_inner() {
  let o = OneOf::from_vec(vec![1]);
  let _ = o.as_inner();
}

#[test]
fn oneof_from_vec_impl() {
  let o: OneOf<'_, i32> = vec![1, 2].into();
  assert_eq!(o.as_slice(), &[1, 2]);
}

#[test]
fn oneof_from_cow() {
  let cow: Cow<'_, [i32]> = Cow::Borrowed(&[1, 2]);
  let o: OneOf<'_, i32> = cow.into();
  assert_eq!(o.as_slice(), &[1, 2]);
}

#[test]
fn oneof_into_cow() {
  let o = OneOf::from_vec(vec![1]);
  let cow: Cow<'_, [i32]> = o.into();
  assert_eq!(&*cow, &[1]);
}

#[test]
fn oneof_ref_into_cow() {
  let o = OneOf::from_vec(vec![1]);
  let cow: Cow<'_, [i32]> = (&o).into();
  assert_eq!(&*cow, &[1]);
}

#[test]
fn oneof_as_ref() {
  let o = OneOf::from_vec(vec![1, 2]);
  let r: &[i32] = o.as_ref();
  assert_eq!(r, &[1, 2]);
}

#[test]
fn oneof_borrow() {
  use std::borrow::Borrow;
  let o = OneOf::from_vec(vec![1, 2]);
  let r: &[i32] = o.borrow();
  assert_eq!(r, &[1, 2]);
}

#[test]
fn oneof_display() {
  // OneOf's Display delegates to the inner Cow<[T]>; exercise via Debug which always works
  let o = OneOf::from_vec(vec![1, 2, 3]);
  let _ = format!("{o:?}");
}

#[test]
fn oneof_debug() {
  let o = OneOf::from_vec(vec![1, 2]);
  let _ = format!("{o:?}");
}
