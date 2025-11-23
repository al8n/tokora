use core::mem::MaybeUninit;

use mayber::MaybeRef;

use super::{super::BlackHole, Cache, CachedToken, Checkpoint, Lexer, Token};

impl<'a, T, L> Cache<'a, T, L> for BlackHole
where
  T: Token<'a>,
  L: Lexer<'a, T> + 'a,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    0
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn remaining(&self) -> usize {
    0
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, _: &Checkpoint<'a, '_, T, L>) {}

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_back(
    &mut self,
    tok: CachedToken<'a, T, L>,
  ) -> Result<&CachedToken<'a, T, L>, CachedToken<'a, T, L>> {
    Err(tok)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn pop_front(&mut self) -> Option<CachedToken<'a, T, L>> {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn pop_back(&mut self) -> Option<CachedToken<'a, T, L>> {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clear(&mut self) {}

  #[cfg_attr(not(tarpaulin), inline(always))]
  unsafe fn peek(
    &self,
    buf: &mut [MaybeUninit<MaybeRef<'_, CachedToken<'a, T, L>>>],
  ) -> &mut [MaybeRef<'_, CachedToken<'a, T, L>>] {
    // SAFETY: We never initialize any element in the buffer, so the returned slice is always empty.
    unsafe {
      core::slice::from_raw_parts_mut(
        buf.as_mut_ptr() as *mut MaybeRef<'_, CachedToken<'a, T, L>>,
        0,
      )
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn first(&self) -> Option<&CachedToken<'a, T, L>> {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn last(&self) -> Option<&CachedToken<'a, T, L>> {
    None
  }
}
