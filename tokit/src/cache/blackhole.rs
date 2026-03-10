use super::{
  Cache, CachedTokenOf, CachedTokenRefOf, Checkpoint, GenericArrayDeque, Lexer,
  MaybeRefCachedTokenOf,
};

macro_rules! blackhole {
  ($ty:ty) => {
    impl<'a, L, Lang: ?Sized> Cache<'a, L, Lang> for $ty
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
      fn push_front(
        &mut self,
        tok: CachedTokenOf<'a, L>,
      ) -> Result<CachedTokenRefOf<'_, 'a, L>, CachedTokenOf<'a, L>> {
        Err(tok)
      }

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
      ) where
        W: crate::Window,
      {
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn front(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn back(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
        None
      }
    }
  };
}

blackhole!(());

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unit_cache_len_is_zero() {
    let cache: () = ();
    assert_eq!(<() as Cache<'_, crate::lexer::DummyLexer>>::len(&cache), 0);
  }

  #[test]
  fn unit_cache_remaining_is_zero() {
    let cache: () = ();
    assert_eq!(
      <() as Cache<'_, crate::lexer::DummyLexer>>::remaining(&cache),
      0
    );
  }

  #[test]
  fn unit_cache_front_is_none() {
    let cache: () = ();
    assert!(<() as Cache<'_, crate::lexer::DummyLexer>>::front(&cache).is_none());
  }

  #[test]
  fn unit_cache_back_is_none() {
    let cache: () = ();
    assert!(<() as Cache<'_, crate::lexer::DummyLexer>>::back(&cache).is_none());
  }

  #[test]
  fn unit_cache_pop_front_is_none() {
    let mut cache: () = ();
    assert!(<() as Cache<'_, crate::lexer::DummyLexer>>::pop_front(&mut cache).is_none());
  }

  #[test]
  fn unit_cache_pop_back_is_none() {
    let mut cache: () = ();
    assert!(<() as Cache<'_, crate::lexer::DummyLexer>>::pop_back(&mut cache).is_none());
  }

  #[test]
  fn unit_cache_clear_is_noop() {
    let mut cache: () = ();
    <() as Cache<'_, crate::lexer::DummyLexer>>::clear(&mut cache);
  }

  #[test]
  fn unit_cache_new() {
    let cache = <() as Cache<'_, crate::lexer::DummyLexer>>::new();
    assert_eq!(cache, ());
  }

  #[test]
  fn unit_cache_with_options() {
    let cache = <() as Cache<'_, crate::lexer::DummyLexer>>::with_options(());
    assert_eq!(cache, ());
  }
}
