use smol_bytes_0_2::{Utf8Bytes, compact, shared};

use crate::utils::{IntoEquivalent, ToEquivalent};

#[test]
fn to_equivalent_smol_bytes_shared() {
  let data: &[u8] = b"hello";
  let result: shared::Bytes = ToEquivalent::<shared::Bytes>::to_equivalent(data);
  assert_eq!(result, data);
}

#[test]
fn to_equivalent_smol_bytes_compact() {
  let data: &[u8] = b"hello";
  let result: compact::Bytes = ToEquivalent::<compact::Bytes>::to_equivalent(data);
  assert_eq!(result, data);
}

#[test]
fn to_equivalent_smol_bytes_utf8() {
  let s: &str = "hello";
  let result: Utf8Bytes = ToEquivalent::<Utf8Bytes>::to_equivalent(s);
  assert_eq!(result, s);
}

#[test]
fn to_equivalent_smol_bytes_compact_utf8() {
  let s: &str = "hello";
  let result: compact::Utf8Bytes = ToEquivalent::<compact::Utf8Bytes>::to_equivalent(s);
  assert_eq!(result, s);
}

#[test]
fn into_equivalent_smol_bytes_shared() {
  let data: &[u8] = b"world";
  let result: shared::Bytes = IntoEquivalent::<shared::Bytes>::into_equivalent(data);
  assert_eq!(result, data);
}

#[test]
fn into_equivalent_smol_bytes_compact() {
  let data: &[u8] = b"world";
  let result: compact::Bytes = IntoEquivalent::<compact::Bytes>::into_equivalent(data);
  assert_eq!(result, data);
}

#[test]
fn into_equivalent_smol_bytes_utf8() {
  let s: &str = "world";
  let result: Utf8Bytes = IntoEquivalent::<Utf8Bytes>::into_equivalent(s);
  assert_eq!(result, s);
}

#[test]
fn into_equivalent_smol_bytes_compact_utf8() {
  let s: &str = "world";
  let result: compact::Utf8Bytes = IntoEquivalent::<compact::Utf8Bytes>::into_equivalent(s);
  assert_eq!(result, s);
}
