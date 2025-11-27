use core::mem::MaybeUninit;

use mayber::MaybeRef;

use super::{super::BlackHole, Cache, CachedToken, Checkpoint, Lexer};

macro_rules! blackhole {
  ($ty:ty) => {
    impl<'a, L> Cache<'a, L> for $ty
    where
      L: Lexer<'a> + 'a,
    {
      type Options = ();

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn new() -> Self {
        Default::default()
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn with_options(_: Self::Options) -> Self
      where
        Self: Sized,
      {
        Default::default()
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn len(&self) -> usize {
        0
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn remaining(&self) -> usize {
        0
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn rewind(&mut self, _: &Checkpoint<'a, '_, L>) {}

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push_back(
        &mut self,
        tok: CachedToken<'a, L>,
      ) -> Result<&CachedToken<'a, L>, CachedToken<'a, L>> {
        Err(tok)
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn pop_front(&mut self) -> Option<CachedToken<'a, L>> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn pop_back(&mut self) -> Option<CachedToken<'a, L>> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn clear(&mut self) {}

      #[cfg_attr(not(tarpaulin), inline(always))]
      unsafe fn peek(
        &self,
        buf: &mut [MaybeUninit<MaybeRef<'_, CachedToken<'a, L>>>],
      ) -> &mut [MaybeRef<'_, CachedToken<'a, L>>] {
        // SAFETY: We never initialize any element in the buffer, so the returned slice is always empty.
        unsafe {
          core::slice::from_raw_parts_mut(
            buf.as_mut_ptr() as *mut MaybeRef<'_, CachedToken<'a, L>>,
            0,
          )
        }
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn first(&self) -> Option<&CachedToken<'a, L>> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn last(&self) -> Option<&CachedToken<'a, L>> {
        None
      }
    }
  };
}

blackhole!(BlackHole);
blackhole!(());
