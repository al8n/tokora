use super::*;

#[test]
fn bstr_slice_len() {
  let s = BStr::new(b"hello");
  assert_eq!(Slice::len(s), 5);
}

#[test]
fn bstr_slice_iter() {
  let s = BStr::new(b"abc");
  let bytes: std::vec::Vec<u8> = Slice::iter(s).collect();
  assert_eq!(bytes, std::vec![b'a', b'b', b'c']);
}

#[test]
fn bstr_slice_positioned_iter() {
  let s = BStr::new(b"ab");
  let items: std::vec::Vec<(usize, u8)> = Slice::positioned_iter(s).collect();
  assert_eq!(items, std::vec![(0, b'a'), (1, b'b')]);
}

#[test]
fn bstr_reference_slice_forwarding_preserves_slice_behavior() {
  let slice = BStr::new(b"ab");
  let slice_ref = &slice;
  let expected = (2, std::vec![b'a', b'b'], std::vec![(0, b'a'), (1, b'b')]);

  assert_eq!(
    (
      <BStr as Slice<'_>>::len(slice),
      <BStr as Slice<'_>>::iter(slice).collect::<std::vec::Vec<_>>(),
      <BStr as Slice<'_>>::positioned_iter(slice).collect::<std::vec::Vec<_>>(),
    ),
    expected
  );
  assert_eq!(
    (
      <&BStr as Slice<'_>>::len(&slice),
      <&BStr as Slice<'_>>::iter(&slice).collect::<std::vec::Vec<_>>(),
      <&BStr as Slice<'_>>::positioned_iter(&slice).collect::<std::vec::Vec<_>>(),
    ),
    expected
  );
  assert_eq!(
    (
      <&&BStr as Slice<'_>>::len(&slice_ref),
      <&&BStr as Slice<'_>>::iter(&slice_ref).collect::<std::vec::Vec<_>>(),
      <&&BStr as Slice<'_>>::positioned_iter(&slice_ref).collect::<std::vec::Vec<_>>(),
    ),
    expected
  );
}
