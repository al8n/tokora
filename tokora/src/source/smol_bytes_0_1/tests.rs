use super::*;

// ── Byte-shaped sources (shared::Bytes, compact::Bytes) ────────────
//
// Both mirror the `bytes_1::Bytes` source matrix: the bound-checked
// `slice` body returns `None` for out-of-bounds ranges, and
// `is_boundary` uses byte (not char) semantics.

macro_rules! byte_source_tests {
  ($modname:ident, $ty:ty) => {
    mod $modname {
      use super::*;

      #[test]
      fn is_empty_on_empty() {
        let src = <$ty>::new();
        assert!(Source::is_empty(&src));
      }

      #[test]
      fn is_empty_on_non_empty() {
        let src = <$ty>::from_static(b"abc");
        assert!(!Source::is_empty(&src));
      }

      #[test]
      fn len() {
        let src = <$ty>::from_static(b"hello");
        assert_eq!(Source::len(&src), 5);
      }

      #[test]
      fn len_empty() {
        let src = <$ty>::new();
        assert_eq!(Source::len(&src), 0);
      }

      #[test]
      fn slice_full_range() {
        let src = <$ty>::from_static(b"abcde");
        let result = Source::slice(&src, 0..5);
        assert_eq!(result.as_deref(), Some(b"abcde".as_slice()));
      }

      #[test]
      fn slice_partial() {
        let src = <$ty>::from_static(b"abcde");
        let result = Source::slice(&src, 1..3);
        assert_eq!(result.as_deref(), Some(b"bc".as_slice()));
      }

      #[test]
      fn slice_empty_range() {
        let src = <$ty>::from_static(b"abc");
        let result = Source::slice(&src, 1..1);
        assert_eq!(result.as_deref(), Some(b"".as_slice()));
      }

      #[test]
      fn slice_out_of_bounds() {
        let src = <$ty>::from_static(b"abc");
        let result = Source::slice(&src, 0..10);
        assert!(result.is_none());
      }

      #[test]
      fn slice_inclusive_range() {
        let src = <$ty>::from_static(b"abcde");
        let result = Source::slice(&src, 1..=3);
        assert_eq!(result.as_deref(), Some(b"bcd".as_slice()));
      }

      #[test]
      fn slice_unbounded_start() {
        let src = <$ty>::from_static(b"abcde");
        let result = Source::slice(&src, ..3);
        assert_eq!(result.as_deref(), Some(b"abc".as_slice()));
      }

      #[test]
      fn slice_unbounded_end() {
        let src = <$ty>::from_static(b"abcde");
        let result = Source::slice(&src, 2..);
        assert_eq!(result.as_deref(), Some(b"cde".as_slice()));
      }

      #[test]
      fn slice_fully_unbounded() {
        let src = <$ty>::from_static(b"abcde");
        let result = Source::slice(&src, ..);
        assert_eq!(result.as_deref(), Some(b"abcde".as_slice()));
      }

      #[test]
      fn slice_empty_source() {
        let src = <$ty>::new();
        let result = Source::slice(&src, 0..0);
        assert_eq!(result.as_deref(), Some(b"".as_slice()));
      }

      #[test]
      fn slice_empty_source_out_of_range() {
        let src = <$ty>::new();
        let result = Source::slice(&src, 0..1);
        assert!(result.is_none());
      }

      #[test]
      fn is_boundary_valid() {
        let src = <$ty>::from_static(b"abc");
        assert!(Source::is_boundary(&src, 0));
        assert!(Source::is_boundary(&src, 1));
        assert!(Source::is_boundary(&src, 3));
      }

      #[test]
      fn is_boundary_beyond_len() {
        let src = <$ty>::from_static(b"abc");
        assert!(!Source::is_boundary(&src, 4));
      }

      #[test]
      fn is_boundary_empty() {
        let src = <$ty>::new();
        assert!(Source::is_boundary(&src, 0));
        assert!(!Source::is_boundary(&src, 1));
      }

      #[test]
      fn find_boundary_returns_index() {
        // Byte sources leave the index unchanged.
        let src = <$ty>::from_static(b"abc");
        assert_eq!(Source::find_boundary(&src, 2), 2);
      }
    }
  };
}

byte_source_tests!(shared_bytes, shared::Bytes);
byte_source_tests!(compact_bytes, compact::Bytes);

// ── Str-shaped source (Utf8Bytes) ──────────────────────────────────
//
// Mirrors the `&str` / `HipStr` source matrix: `slice` returns `None`
// for out-of-bounds ranges AND for ranges that split a code point, and
// `is_boundary` / `find_boundary` use char-boundary semantics.

#[test]
fn utf8_is_empty_on_empty() {
  let src = Utf8Bytes::new();
  assert!(Source::is_empty(&src));
}

#[test]
fn utf8_is_empty_on_non_empty() {
  let src = Utf8Bytes::from_static("abc");
  assert!(!Source::is_empty(&src));
}

#[test]
fn utf8_len() {
  let src = Utf8Bytes::from_static("hello");
  assert_eq!(Source::len(&src), 5);
}

#[test]
fn utf8_len_empty() {
  let src = Utf8Bytes::new();
  assert_eq!(Source::len(&src), 0);
}

#[test]
fn utf8_len_multibyte() {
  // Each emoji is 4 bytes in UTF-8.
  let src = Utf8Bytes::from_static("\u{1F600}");
  assert_eq!(Source::len(&src), 4);
}

#[test]
fn utf8_slice_full_range() {
  let src = Utf8Bytes::from_static("abcde");
  let result = Source::slice(&src, 0..5);
  assert_eq!(result.as_deref(), Some("abcde"));
}

#[test]
fn utf8_slice_partial() {
  let src = Utf8Bytes::from_static("abcde");
  let result = Source::slice(&src, 1..3);
  assert_eq!(result.as_deref(), Some("bc"));
}

#[test]
fn utf8_slice_empty() {
  let src = Utf8Bytes::from_static("abc");
  let result = Source::slice(&src, 1..1);
  assert_eq!(result.as_deref(), Some(""));
}

#[test]
fn utf8_slice_out_of_bounds() {
  let src = Utf8Bytes::from_static("abc");
  let result = Source::slice(&src, 0..10);
  assert!(result.is_none());
}

#[test]
fn utf8_slice_on_non_boundary_returns_none() {
  // 2-byte char: the second byte is not a valid boundary.
  let src = Utf8Bytes::from_static("\u{00E9}abc"); // e-acute (2 bytes) + abc
  let result = Source::slice(&src, 0..1);
  assert!(result.is_none());
}

#[test]
fn utf8_is_boundary_at_char_boundaries() {
  let src = Utf8Bytes::from_static("\u{00E9}a"); // 2-byte char + 1-byte char
  assert!(Source::is_boundary(&src, 0));
  assert!(!Source::is_boundary(&src, 1)); // middle of 2-byte char
  assert!(Source::is_boundary(&src, 2)); // start of 'a'
  assert!(Source::is_boundary(&src, 3)); // end
}

#[test]
fn utf8_is_boundary_at_end() {
  let src = Utf8Bytes::from_static("abc");
  assert!(Source::is_boundary(&src, 3));
}

#[test]
fn utf8_is_boundary_beyond_len() {
  let src = Utf8Bytes::from_static("abc");
  assert!(!Source::is_boundary(&src, 4));
}

#[test]
fn utf8_find_boundary_rounds_down_multibyte() {
  // "é" is a single 2-byte code point occupying 0..2.
  let src = Utf8Bytes::from_static("\u{00E9}");
  assert_eq!(Source::find_boundary(&src, 1), 0);
}

#[test]
fn utf8_find_boundary_rounds_down_after_ascii() {
  // 'a' at 0, "é" at 1..3.
  let src = Utf8Bytes::from_static("a\u{00E9}");
  assert_eq!(Source::find_boundary(&src, 2), 1);
}

#[test]
fn utf8_find_boundary_passes_through_boundaries() {
  let src = Utf8Bytes::from_static("a\u{00E9}"); // boundaries at 0, 1, and 3 (== len)
  assert_eq!(Source::find_boundary(&src, 0), 0);
  assert_eq!(Source::find_boundary(&src, 1), 1);
  assert_eq!(Source::find_boundary(&src, 3), 3);
}

#[test]
fn utf8_find_boundary_at_and_beyond_len() {
  // index >= len is returned unchanged, symmetric with the byte sources.
  let src = Utf8Bytes::from_static("a\u{00E9}"); // len 3
  assert_eq!(Source::find_boundary(&src, 3), 3);
  assert_eq!(Source::find_boundary(&src, 10), 10);
}
