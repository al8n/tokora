use super::*;

#[test]
fn to_equivalent_str_ref() {
  let s: &str = "hello";
  let result: &str = ToEquivalent::<&str>::to_equivalent(&s);
  assert_eq!(result, "hello");
}

#[test]
fn to_equivalent_bytes_ref() {
  let b: &[u8] = b"hello";
  let result: &[u8] = ToEquivalent::<&[u8]>::to_equivalent(&b);
  assert_eq!(result, b"hello");
}

#[test]
fn to_equivalent_via_double_ref() {
  let s: &str = "world";
  let r: &&str = &s;
  let result: &str = ToEquivalent::<&str>::to_equivalent(r);
  assert_eq!(result, "world");
}

#[test]
fn into_equivalent_str() {
  let s: &str = "test";
  let result: &str = IntoEquivalent::<&str>::into_equivalent(s);
  assert_eq!(result, "test");
}

#[test]
fn into_equivalent_bytes() {
  let b: &[u8] = b"test";
  let result: &[u8] = IntoEquivalent::<&[u8]>::into_equivalent(b);
  assert_eq!(result, b"test");
}
