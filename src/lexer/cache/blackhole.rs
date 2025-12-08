use super::{
  super::BlackHole, Cache, CachedTokenOf, CachedTokenRefOf, Checkpoint, Lexer,
  MaybeRefCachedTokenOf, GenericArrayDeque,
};

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
        tok: CachedTokenOf<'a, L>,
      ) -> Result<CachedTokenRefOf<'_, 'a, L>, CachedTokenOf<'a, L>> {
        Err(tok)
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn pop_front(&mut self) -> Option<CachedTokenOf<'a, L>> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn pop_back(&mut self) -> Option<CachedTokenOf<'a, L>> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn clear(&mut self) {}

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn peek<'p, W>(
        &'p self,
        _: &mut GenericArrayDeque<MaybeRefCachedTokenOf<'p, 'a, L>, W::CAPACITY>,
      )
      where
        W: crate::Window,
      {}

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn first(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn last(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
        None
      }
    }
  };
}

blackhole!(BlackHole);
blackhole!(());
