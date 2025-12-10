use core::marker::PhantomData;

use super::{Lexer, Span};

/// A cursor representing a position in the input source.
///
/// `Cursor` is a lightweight type that wraps a byte offset into the tokenizer's
/// input source. It's used by [`Checkpoint`] to track positions and is returned
/// by [`Input::cursor`] to query the current position.
///
/// The cursor position represents:
/// - The byte offset in the input where the tokenizer will continue lexing
/// - If there are cached tokens, it points to the start of the first cached token
/// - Otherwise, it points to the position where the next token will be lexed from
#[repr(transparent)]
pub struct Cursor<'a, 'closure, L: Lexer<'a>> {
  pub(crate) span: L::Span,
  _phantom: PhantomData<fn(&'closure ()) -> &'closure ()>,
}

impl<'a, L: Lexer<'a>> core::fmt::Debug for Cursor<'a, '_, L> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Cursor({:?})", self.span.end_ref())
  }
}

impl<'a, L: Lexer<'a>> Clone for Cursor<'a, '_, L> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    Self {
      span: self.span.clone(),
      _phantom: PhantomData,
    }
  }
}

impl<'a, L: Lexer<'a>> Copy for Cursor<'a, '_, L> where L::Span: Copy {}

impl<'a, L: Lexer<'a>> Cursor<'a, '_, L> {
  /// Creates a new cursor.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(span: L::Span) -> Self {
    Self {
      span,
      _phantom: PhantomData,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn from_ref(cursor: &L::Span) -> &Self {
    // SAFETY: Cursor is #[repr(transparent)]
    unsafe { &*(cursor as *const L::Span as *const Self) }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn span(&self) -> &L::Span {
    &self.span
  }

  /// Returns a reference to the actual cursor.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_inner(&self) -> &L::Offset {
    self.span.end_ref()
  }

  /// Returns a the actual cursor.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_inner(self) -> L::Offset {
    self.span.into_range().end
  }
}
