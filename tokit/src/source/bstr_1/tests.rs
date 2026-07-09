use super::*;

#[test]
fn bstr_is_empty() {
  let empty = BStr::new(b"");
  assert!(Source::is_empty(empty));
  let non_empty = BStr::new(b"abc");
  assert!(!Source::is_empty(non_empty));
}

#[test]
fn bstr_len() {
  let s = BStr::new(b"hello");
  assert_eq!(Source::len(s), 5);
}

#[test]
fn bstr_slice() {
  let s = BStr::new(b"hello");
  let sliced = Source::slice(s, &1..&3);
  assert_eq!(sliced, Some(&b"el"[..]));
}

#[test]
fn bstr_is_boundary() {
  let s = BStr::new(b"abc");
  assert!(Source::is_boundary(s, 0));
  assert!(Source::is_boundary(s, 3));
  assert!(!Source::is_boundary(s, 4));
}
