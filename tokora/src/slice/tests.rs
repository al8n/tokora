use super::*;
use std::{format, string::String, vec, vec::Vec};

// --- Slice trait tests for &[u8] ---

#[test]
fn u8_slice_len() {
  let s: &[u8] = b"hello";
  assert_eq!(Slice::len(&s), 5);
}

#[test]
fn u8_slice_is_empty() {
  let empty: &[u8] = b"";
  assert!(Slice::is_empty(&empty));
  let non_empty: &[u8] = b"a";
  assert!(!Slice::is_empty(&non_empty));
}

#[test]
fn u8_slice_iter() {
  let s: &[u8] = b"abc";
  let chars: Vec<u8> = Slice::iter(&s).collect();
  assert_eq!(chars, vec![b'a', b'b', b'c']);
}

#[test]
fn u8_slice_positioned_iter() {
  let s: &[u8] = b"ab";
  let items: Vec<(usize, u8)> = Slice::positioned_iter(&s).collect();
  assert_eq!(items, vec![(0, b'a'), (1, b'b')]);
}

// --- Slice trait tests for &str ---

#[test]
fn str_slice_len() {
  let s: &str = "hello";
  assert_eq!(Slice::len(&s), 5);
}

#[test]
fn str_slice_len_multibyte() {
  let s: &str = "\u{00E9}"; // 2-byte char
  assert_eq!(Slice::len(&s), 2);
}

#[test]
fn str_slice_is_empty() {
  let empty: &str = "";
  assert!(Slice::is_empty(&empty));
  let non_empty: &str = "a";
  assert!(!Slice::is_empty(&non_empty));
}

#[test]
fn str_slice_iter() {
  let s: &str = "abc";
  let chars: Vec<char> = Slice::iter(&s).collect();
  assert_eq!(chars, vec!['a', 'b', 'c']);
}

#[test]
fn str_slice_iter_multibyte() {
  let s: &str = "\u{00E9}x";
  let chars: Vec<char> = Slice::iter(&s).collect();
  assert_eq!(chars, vec!['\u{00E9}', 'x']);
}

#[test]
fn str_slice_positioned_iter() {
  let s: &str = "ab";
  let items: Vec<(usize, char)> = Slice::positioned_iter(&s).collect();
  assert_eq!(items, vec![(0, 'a'), (1, 'b')]);
}

#[test]
fn str_slice_positioned_iter_multibyte() {
  let s: &str = "\u{00E9}a"; // 2 bytes + 1 byte
  let items: Vec<(usize, char)> = Slice::positioned_iter(&s).collect();
  assert_eq!(items, vec![(0, '\u{00E9}'), (2, 'a')]);
}

#[test]
fn slice_reference_forwarding_preserves_core_slice_behavior() {
  let text: &str = "\u{00E9}a";
  let text_ref = &text;
  let expected_text = (3, vec!['\u{00E9}', 'a'], vec![(0, '\u{00E9}'), (2, 'a')]);

  assert_eq!(
    (
      <str as Slice<'_>>::len(text),
      <str as Slice<'_>>::iter(text).collect::<Vec<_>>(),
      <str as Slice<'_>>::positioned_iter(text).collect::<Vec<_>>(),
    ),
    expected_text
  );
  assert_eq!(
    (
      <&str as Slice<'_>>::len(&text),
      <&str as Slice<'_>>::iter(&text).collect::<Vec<_>>(),
      <&str as Slice<'_>>::positioned_iter(&text).collect::<Vec<_>>(),
    ),
    expected_text
  );
  assert_eq!(
    (
      <&&str as Slice<'_>>::len(&text_ref),
      <&&str as Slice<'_>>::iter(&text_ref).collect::<Vec<_>>(),
      <&&str as Slice<'_>>::positioned_iter(&text_ref).collect::<Vec<_>>(),
    ),
    expected_text
  );

  let bytes: &[u8] = b"ab";
  let bytes_ref = &bytes;
  let expected_bytes = (2, vec![b'a', b'b'], vec![(0, b'a'), (1, b'b')]);

  assert_eq!(
    (
      <[u8] as Slice<'_>>::len(bytes),
      <[u8] as Slice<'_>>::iter(bytes).collect::<Vec<_>>(),
      <[u8] as Slice<'_>>::positioned_iter(bytes).collect::<Vec<_>>(),
    ),
    expected_bytes
  );
  assert_eq!(
    (
      <&[u8] as Slice<'_>>::len(&bytes),
      <&[u8] as Slice<'_>>::iter(&bytes).collect::<Vec<_>>(),
      <&[u8] as Slice<'_>>::positioned_iter(&bytes).collect::<Vec<_>>(),
    ),
    expected_bytes
  );
  assert_eq!(
    (
      <&&[u8] as Slice<'_>>::len(&bytes_ref),
      <&&[u8] as Slice<'_>>::iter(&bytes_ref).collect::<Vec<_>>(),
      <&&[u8] as Slice<'_>>::positioned_iter(&bytes_ref).collect::<Vec<_>>(),
    ),
    expected_bytes
  );
}

// --- Sliced tests ---

#[test]
fn sliced_new_and_accessors() {
  let s = Sliced::new("file.rs", 42);
  assert_eq!(s.slice(), "file.rs");
  assert_eq!(*s.data(), 42);
}

#[test]
fn sliced_slice_ref() {
  let s = Sliced::new("file.rs", 42);
  assert_eq!(s.slice_ref(), &"file.rs");
}

#[test]
fn sliced_slice_mut() {
  let mut s = Sliced::new("old.rs", 42);
  *s.slice_mut() = "new.rs";
  assert_eq!(s.slice(), "new.rs");
}

#[test]
fn sliced_data_mut() {
  let mut s = Sliced::new("file.rs", 42);
  *s.data_mut() = 100;
  assert_eq!(*s.data(), 100);
}

#[test]
fn sliced_deref() {
  let s = Sliced::new("file.rs", 42i32);
  let val: &i32 = &s;
  assert_eq!(*val, 42);
}

#[test]
fn sliced_deref_mut() {
  let mut s = Sliced::new("file.rs", 42i32);
  *s = 100;
  assert_eq!(*s, 100);
}

#[test]
fn sliced_display() {
  let s = Sliced::new("file.rs", "hello");
  assert_eq!(format!("{s}"), "hello");
}

#[test]
fn sliced_as_ref_borrowed() {
  let s = Sliced::new(String::from("file.rs"), String::from("data"));
  let borrowed = s.as_ref();
  assert_eq!(borrowed.data(), &&String::from("data"));
}

#[test]
fn sliced_as_mut_borrowed() {
  let mut s = Sliced::new("file.rs", 42i32);
  {
    let m = s.as_mut();
    *m.data = 100;
  }
  assert_eq!(*s.data(), 100);
}

#[test]
fn sliced_into_slice() {
  let s = Sliced::new("file.rs", 42);
  assert_eq!(s.into_slice(), "file.rs");
}

#[test]
fn sliced_into_data() {
  let s = Sliced::new("file.rs", 42);
  assert_eq!(s.into_data(), 42);
}

#[test]
fn sliced_into_components() {
  let s = Sliced::new("file.rs", 42);
  let (slice, data) = s.into_components();
  assert_eq!(slice, "file.rs");
  assert_eq!(data, 42);
}

#[test]
fn sliced_into_components_trait() {
  let s = Sliced::new("file.rs", 42);
  let (slice, data) = IntoComponents::into_components(s);
  assert_eq!(slice, "file.rs");
  assert_eq!(data, 42);
}

#[test]
fn sliced_map_data() {
  let s = Sliced::new("file.rs", "42");
  let mapped = s.map_data(|d| d.parse::<i32>().unwrap());
  assert_eq!(*mapped.data(), 42);
  assert_eq!(mapped.slice(), "file.rs");
}

#[test]
fn sliced_as_ref_trait() {
  let s: Sliced<i32, &str> = Sliced::new("file.rs", 42);
  let r: &&str = <Sliced<i32, &str> as AsRef<&str>>::as_ref(&s);
  assert_eq!(r, &"file.rs");
}
