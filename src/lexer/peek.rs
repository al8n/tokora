use core::{
  mem::{ManuallyDrop, MaybeUninit},
  ops::{Deref, DerefMut},
};

use generic_arraydeque::{ArrayLength, array::GenericArray};
use mayber::{Maybe::Owned, MaybeRef};

use super::*;

/// A peeked buffer of tokens from the lexer.
pub struct Peeked<'p, 'inp, L: Lexer<'inp>, N: ArrayLength> {
  buf: ManuallyDrop<GenericArray<MaybeUninit<MaybeRef<'p, CachedToken<'inp, L>>>, N>>,
  filled: usize,
}

impl<'p, 'inp, L: Lexer<'inp>, N: ArrayLength> Peeked<'p, 'inp, L, N> {
  /// Creates a new `Peeked` buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new() -> Self {
    Self {
      buf: ManuallyDrop::new(GenericArray::uninit()),
      filled: 0,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) unsafe fn set_len(&mut self, len: usize) {
    self.filled = len;
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) fn as_mut_buf(&mut self) -> &mut [MaybeUninit<MaybeRef<'p, CachedToken<'inp, L>>>] {
    &mut self.buf
  }

  /// Returns `true` if the buffer is empty.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_empty(&self) -> bool {
    self.filled == 0
  }

  /// Returns the length of the buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn len(&self) -> usize {
    self.filled
  }

  /// Returns a slice to the initialized portion of the buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_slice(&self) -> &[MaybeRef<'p, CachedToken<'inp, L>>] {
    // SAFETY: We only read initialized elements up to `self.filled`.
    unsafe {
      core::slice::from_raw_parts(
        self.buf.as_ptr() as *const MaybeRef<'p, CachedToken<'inp, L>>,
        self.filled,
      )
    }
  }

  /// Returns a mutable slice to the initialized portion of the buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_mut_slice(&mut self) -> &mut [MaybeRef<'p, CachedToken<'inp, L>>] {
    // SAFETY: We only read initialized elements up to `self.filled`.
    unsafe {
      core::slice::from_raw_parts_mut(
        self.buf.as_mut_ptr() as *mut MaybeRef<'p, CachedToken<'inp, L>>,
        self.filled,
      )
    }
  }
}

impl<'p, 'inp, L: Lexer<'inp>, N: ArrayLength> Deref for Peeked<'p, 'inp, L, N> {
  type Target = [MaybeRef<'p, CachedToken<'inp, L>>];

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<'inp, L: Lexer<'inp>, N: ArrayLength> DerefMut for Peeked<'_, 'inp, L, N> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut_slice()
  }
}

impl<'inp, L: Lexer<'inp>, N: ArrayLength> Drop for Peeked<'_, 'inp, L, N> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn drop(&mut self) {
    for ct in self.iter_mut() {
      if let Owned(cached) = ct {
        // SAFETY: We are dropping the owned cached token.
        unsafe {
          core::ptr::drop_in_place(cached);
        }
      }
    }
  }
}
