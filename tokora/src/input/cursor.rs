use core::marker::PhantomData;

use super::Lexer;

/// A cursor representing a position in the input source.
///
/// `Cursor` is a lightweight type that wraps a byte offset into the lexer's
/// input source. It's used by [`Checkpoint`](crate::input::Checkpoint) to track positions and represents
/// the current position in the input stream.
///
/// The cursor position represents:
/// - The byte offset in the input where the tokenizer will continue lexing
/// - If there are cached tokens, it points to the start of the first cached token
/// - Otherwise, it points to the position where the next token will be lexed from
#[repr(transparent)]
pub struct Cursor<'inp, 'closure, L: Lexer<'inp>> {
  pub(crate) offset: L::Offset,
  _phantom: PhantomData<fn(&'closure ()) -> &'closure ()>,
}

impl<'a, L: Lexer<'a>> core::fmt::Debug for Cursor<'a, '_, L> {
  #[inline(always)]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Cursor({:?})", self.offset)
  }
}

impl<'a, L: Lexer<'a>> Clone for Cursor<'a, '_, L> {
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      offset: self.offset.clone(),
      _phantom: PhantomData,
    }
  }
}

impl<'a, L: Lexer<'a>> Copy for Cursor<'a, '_, L> where L::Offset: Copy {}

impl<'a, L: Lexer<'a>> Cursor<'a, '_, L> {
  // `pub(crate)` (not `pub(super)`): the CST sink's unit tests drive the
  // `Emitter::rewind` contract directly (clamp shapes and journal replays that the
  // disciplined public surface cannot reach), and a rewind takes `&Cursor`.
  #[inline(always)]
  pub(crate) const fn from_ref(offset: &L::Offset) -> &Self {
    // SAFETY: Cursor is #[repr(transparent)] over `L::Offset`.
    unsafe { &*(offset as *const L::Offset as *const Self) }
  }

  /// Returns a reference to the actual cursor.
  #[inline(always)]
  pub fn as_inner(&self) -> &L::Offset {
    &self.offset
  }

  /// Returns the actual cursor.
  #[inline(always)]
  pub fn into_inner(self) -> L::Offset {
    self.offset
  }
}
