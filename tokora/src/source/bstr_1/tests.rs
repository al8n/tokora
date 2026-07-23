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
  let sliced = Source::slice(s, 1..3);
  assert_eq!(sliced, Some(BStr::new(b"el")));
}

#[test]
fn bstr_is_boundary() {
  let s = BStr::new(b"abc");
  assert!(Source::is_boundary(s, 0));
  assert!(Source::is_boundary(s, 3));
  assert!(!Source::is_boundary(s, 4));
}

#[test]
fn borrowed_bstr_source_preserves_behavior_and_data_lifetime() {
  fn as_slice<'data>(source: &'data BStr) -> &'data BStr {
    <&'data BStr as Source<usize>>::as_slice(&source)
  }

  fn slice<'data>(source: &'data BStr) -> Option<&'data BStr> {
    <&'data BStr as Source<usize>>::slice(&source, 1..3)
  }

  let source = BStr::new(b"hello");
  let expected = (5, Some(BStr::new(b"el")), source, true, false);

  assert_eq!(
    (
      <&BStr as Source<usize>>::len(&source),
      slice(source),
      as_slice(source),
      <&BStr as Source<usize>>::is_boundary(&source, 5),
      <&BStr as Source<usize>>::is_boundary(&source, 6),
    ),
    expected
  );
}
