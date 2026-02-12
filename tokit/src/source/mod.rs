use core::ops::RangeBounds;

use crate::Slice;

#[cfg(feature = "bytes_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
mod bytes_1;

#[cfg(feature = "bstr_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bstr_1")))]
mod bstr_1;

#[cfg(feature = "hipstr")]
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
