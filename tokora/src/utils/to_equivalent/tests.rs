use super::*;

#[cfg(feature = "smol_bytes_0_1")]
use smol_bytes_0_1::{Utf8Bytes, compact, shared};

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

#[cfg(feature = "smol_bytes_0_1")]
#[test]
fn to_equivalent_smol_bytes_shared() {
  let data: &[u8] = b"hello";
  let result: shared::Bytes = ToEquivalent::<shared::Bytes>::to_equivalent(data);
  assert_eq!(result, data);
}

#[cfg(feature = "smol_bytes_0_1")]
#[test]
fn to_equivalent_smol_bytes_compact() {
  let data: &[u8] = b"hello";
  let result: compact::Bytes = ToEquivalent::<compact::Bytes>::to_equivalent(data);
  assert_eq!(result, data);
}

#[cfg(feature = "smol_bytes_0_1")]
#[test]
fn to_equivalent_smol_bytes_utf8() {
  let s: &str = "hello";
  let result: Utf8Bytes = ToEquivalent::<Utf8Bytes>::to_equivalent(s);
  assert_eq!(result, s);
}

#[cfg(feature = "smol_bytes_0_1")]
#[test]
fn into_equivalent_smol_bytes_shared() {
  let data: &[u8] = b"world";
  let result: shared::Bytes = IntoEquivalent::<shared::Bytes>::into_equivalent(data);
  assert_eq!(result, data);
}

#[cfg(feature = "smol_bytes_0_1")]
#[test]
fn into_equivalent_smol_bytes_compact() {
  let data: &[u8] = b"world";
  let result: compact::Bytes = IntoEquivalent::<compact::Bytes>::into_equivalent(data);
  assert_eq!(result, data);
}

#[cfg(feature = "smol_bytes_0_1")]
#[test]
fn into_equivalent_smol_bytes_utf8() {
  let s: &str = "world";
  let result: Utf8Bytes = IntoEquivalent::<Utf8Bytes>::into_equivalent(s);
  assert_eq!(result, s);
}
