use core::ops::RangeBounds;

use crate::Slice;

#[cfg(feature = "bytes_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
mod bytes_1;

#[cfg(feature = "bstr_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bstr_1")))]
mod bstr_1;

#[cfg(feature = "hipstr_0_8")]
#[cfg_attr(docsrs, doc(cfg(feature = "hipstr_0_8")))]
mod hipstr_0_8;

/// The source trait for lexers
pub trait Source<Cursor>: core::fmt::Debug {
  /// A type this `Source` can be sliced into.
  type Slice<'source>: Slice<'source>
  where
    Self: 'source;

  /// Returns `true` if the source is empty.
  fn is_empty(&self) -> bool;

  /// Length of the source
  fn len(&self) -> Cursor;

  /// Get a slice of the source at given range. This is analogous to
  /// `slice::get(range)`.
  fn slice<'a, R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: RangeBounds<&'a Cursor>,
    Cursor: 'a;

  /// For `&str` sources attempts to find the closest `char` boundary at which source
  /// can be sliced, starting from `index`.
  ///
  /// For binary sources (`&[u8]`) this should just return `index` back.
  #[inline]
  fn find_boundary(&self, index: Cursor) -> Cursor {
    index
  }

  /// Check if `index` is valid for this `Source`, that is:
  ///
  /// + It's not larger than the byte length of the `Source`.
  /// + (`str` only) It doesn't land in the middle of a UTF-8 code point.
  fn is_boundary(&self, index: Cursor) -> bool;
}

impl Source<usize> for [u8] {
  type Slice<'source>
    = &'source [u8]
  where
    Self: 'source;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    <[u8]>::is_empty(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    self.len()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn slice<'a, R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: RangeBounds<&'a usize>,
    usize: 'a,
  {
    self.get((
      range.start_bound().map(|s| **s),
      range.end_bound().map(|s| **s),
    ))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_boundary(&self, index: usize) -> bool {
    index <= self.len()
  }
}

impl Source<usize> for str {
  type Slice<'source>
    = &'source str
  where
    Self: 'source;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    <str>::is_empty(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    <str>::len(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn slice<'a, R>(&self, range: R) -> Option<Self::Slice<'_>>
  where
    R: RangeBounds<&'a usize>,
  {
    self.get((
      range.start_bound().map(|s| **s),
      range.end_bound().map(|s| **s),
    ))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_boundary(&self, index: usize) -> bool {
    self.is_char_boundary(index)
  }
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;

  // --- &[u8] tests ---

  #[test]
  fn u8_slice_is_empty_on_empty() {
    let src: &[u8] = b"";
    assert!(Source::is_empty(src));
  }

  #[test]
  fn u8_slice_is_empty_on_non_empty() {
    let src: &[u8] = b"abc";
    assert!(!Source::is_empty(src));
  }

  #[test]
  fn u8_slice_len() {
    let src: &[u8] = b"hello";
    assert_eq!(Source::len(src), 5);
  }

  #[test]
  fn u8_slice_len_empty() {
    let src: &[u8] = b"";
    assert_eq!(Source::len(src), 0);
  }

  #[test]
  fn u8_slice_slice_full_range() {
    let src: &[u8] = b"abcde";
    let result = Source::slice(src, &0..&5);
    assert_eq!(result, Some(b"abcde".as_slice()));
  }

  #[test]
  fn u8_slice_slice_partial() {
    let src: &[u8] = b"abcde";
    let result = Source::slice(src, &1..&3);
    assert_eq!(result, Some(b"bc".as_slice()));
  }

  #[test]
  fn u8_slice_slice_out_of_bounds() {
    let src: &[u8] = b"abc";
    let result = Source::slice(src, &0..&10);
    assert_eq!(result, None);
  }

  #[test]
  fn u8_slice_is_boundary_valid() {
    let src: &[u8] = b"abc";
    assert!(Source::is_boundary(src, 0));
    assert!(Source::is_boundary(src, 1));
    assert!(Source::is_boundary(src, 3));
  }

  #[test]
  fn u8_slice_is_boundary_beyond_len() {
    let src: &[u8] = b"abc";
    assert!(!Source::is_boundary(src, 4));
  }

  #[test]
  fn u8_slice_find_boundary_returns_index() {
    let src: &[u8] = b"abc";
    assert_eq!(Source::find_boundary(src, 2), 2);
  }

  // --- &str tests ---

  #[test]
  fn str_is_empty_on_empty() {
    let src: &str = "";
    assert!(Source::is_empty(src));
  }

  #[test]
  fn str_is_empty_on_non_empty() {
    let src: &str = "abc";
    assert!(!Source::is_empty(src));
  }

  #[test]
  fn str_len() {
    let src: &str = "hello";
    assert_eq!(Source::len(src), 5);
  }

  #[test]
  fn str_len_multibyte() {
    // Each emoji is 4 bytes in UTF-8
    let src: &str = "\u{1F600}";
    assert_eq!(Source::len(src), 4);
  }

  #[test]
  fn str_slice_full_range() {
    let src: &str = "abcde";
    let result = Source::slice(src, &0..&5);
    assert_eq!(result, Some("abcde"));
  }

  #[test]
  fn str_slice_partial() {
    let src: &str = "abcde";
    let result = Source::slice(src, &1..&3);
    assert_eq!(result, Some("bc"));
  }

  #[test]
  fn str_slice_out_of_bounds() {
    let src: &str = "abc";
    let result = Source::slice(src, &0..&10);
    assert_eq!(result, None);
  }

  #[test]
  fn str_slice_on_non_boundary_returns_none() {
    // 2-byte char: the second byte is not a valid boundary
    let src: &str = "\u{00E9}abc"; // e-acute (2 bytes) + abc
    let result = Source::slice(src, &0..&1);
    assert_eq!(result, None);
  }

  #[test]
  fn str_is_boundary_at_char_boundaries() {
    let src: &str = "\u{00E9}a"; // 2-byte char + 1-byte char
    assert!(Source::is_boundary(src, 0));
    assert!(!Source::is_boundary(src, 1)); // middle of 2-byte char
    assert!(Source::is_boundary(src, 2)); // start of 'a'
    assert!(Source::is_boundary(src, 3)); // end
  }

  #[test]
  fn str_is_boundary_at_end() {
    let src: &str = "abc";
    assert!(Source::is_boundary(src, 3));
  }

  #[test]
  fn str_is_boundary_beyond_len() {
    let src: &str = "abc";
    assert!(!Source::is_boundary(src, 4));
  }

  #[test]
  fn str_find_boundary_returns_index() {
    let src: &str = "abc";
    assert_eq!(Source::find_boundary(src, 1), 1);
  }

  #[test]
  fn str_empty_slice() {
    let src: &str = "abc";
    let result = Source::slice(src, &1..&1);
    assert_eq!(result, Some(""));
  }
}
