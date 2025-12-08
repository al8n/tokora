use core::{
  mem::{ManuallyDrop, MaybeUninit},
  ops::{Deref, DerefMut},
};

use generic_arraydeque::{array::GenericArray, typenum::U1};
use mayber::{Maybe::Owned, MaybeRef};

use crate::Window;

use super::*;

/// A peeked buffer of tokens from the lexer.
pub struct Peeked<'p, 'inp, L: Lexer<'inp>, W: Window> {
  buf: ManuallyDrop<GenericArray<MaybeUninit<MaybeRefCachedTokenOf<'p, 'inp, L>>, W::CAPACITY>>,
  filled: usize,
  head: usize,
}

impl<'p, 'inp, L: Lexer<'inp>> From<Peeked<'p, 'inp, L, U1>>
  for Option<MaybeRefCachedTokenOf<'p, 'inp, L>>
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(peeked: Peeked<'p, 'inp, L, U1>) -> Self {
    peeked.into_option()
  }
}

impl<'p, 'inp, L: Lexer<'inp>> Peeked<'p, 'inp, L, U1> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) fn into_option(mut self) -> Option<MaybeRefCachedTokenOf<'p, 'inp, L>> {
    if self.is_empty() {
      None
    } else {
      // SAFETY: We checked that the buffer is not empty.
      self.filled = 0; // Prevent double drop
      Some(unsafe {
        core::mem::replace(&mut self.buf[0], MaybeUninit::uninit()).assume_init_read()
      })
    }
  }
}

impl<'p, 'inp, L: Lexer<'inp>, W: Window> Peeked<'p, 'inp, L, W> {
  /// Creates a new `Peeked` buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new() -> Self {
    Self {
      buf: ManuallyDrop::new(GenericArray::uninit()),
      filled: 0,
      head: 0,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) unsafe fn set_len(&mut self, len: usize) {
    self.filled = len;
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) fn as_mut_buf(&mut self) -> &mut [MaybeUninit<MaybeRefCachedTokenOf<'p, 'inp, L>>] {
    &mut self.buf
  }

  /// Returns `true` if the buffer is empty.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_empty(&self) -> bool {
    self.filled.saturating_sub(self.head) == 0
  }

  /// Returns the length of the buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn len(&self) -> usize {
    self.filled.saturating_sub(self.head)
  }

  /// Pops a token from the front of the buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn pop(&mut self) -> Option<MaybeRefCachedTokenOf<'p, 'inp, L>> {
    if self.head < self.filled {
      // SAFETY: We checked that head is less than filled.
      let ct = unsafe {
        self.buf[self.head].assume_init_read()
      };
      self.head += 1;
      Some(ct)
    } else {
      None
    }
  }

  /// Returns a slice to the initialized portion of the buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_slice(&self) -> &[MaybeRef<'p, CachedTokenOf<'inp, L>>] {
    // SAFETY: We only read initialized elements up to `self.filled`.
    unsafe {
      core::slice::from_raw_parts(
        self.buf.as_ptr().add(self.head) as *const MaybeRef<'p, CachedTokenOf<'inp, L>>,
        self.filled.saturating_sub(self.head),
      )
    }
  }

  /// Returns a mutable slice to the initialized portion of the buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_mut_slice(&mut self) -> &mut [MaybeRef<'p, CachedTokenOf<'inp, L>>] {
    // SAFETY: We only read initialized elements up to `self.filled`.
    unsafe {
      core::slice::from_raw_parts_mut(
        self.buf.as_mut_ptr().add(self.head) as *mut MaybeRef<'p, CachedTokenOf<'inp, L>>,
        self.filled.saturating_sub(self.head),
      )
    }
  }
}

impl<'p, 'inp, L: Lexer<'inp>, W: Window> Deref for Peeked<'p, 'inp, L, W> {
  type Target = [MaybeRef<'p, CachedTokenOf<'inp, L>>];

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<'inp, L: Lexer<'inp>, W: Window> DerefMut for Peeked<'_, 'inp, L, W> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut_slice()
  }
}

impl<'inp, L: Lexer<'inp>, W: Window> Drop for Peeked<'_, 'inp, L, W> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn drop(&mut self) {
    let head = self.head;
    for ct in self.iter_mut().skip(head) {
      if let Owned(cached) = ct {
        // SAFETY: We are dropping the owned cached token.
        unsafe {
          core::ptr::drop_in_place(cached);
        }
      }
    }
  }
}
