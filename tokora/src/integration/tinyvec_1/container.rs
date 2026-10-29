use crate::container::Container;

const _: () = {
  use tinyvec_1::{Array, ArrayVec, SliceVec};

  impl<T> Container<T> for SliceVec<'_, T> {
    #[inline(always)]
    fn max_capacity(&self) -> usize {
      self.capacity()
    }

    #[inline(always)]
    fn push(&mut self, item: T) -> Result<(), T> {
      // `SliceVec::push` panics on capacity overflow and `tinyvec` publishes no fallible twin
      // of it, so the bound is tested here: the backing slice cannot grow, and an element count
      // the input chooses would otherwise unwind the parser instead of reaching the refusal
      // channel every other fixed-capacity adapter uses.
      if SliceVec::len(self) < self.capacity() {
        SliceVec::push(self, item);
        Ok(())
      } else {
        Err(item)
      }
    }

    #[inline(always)]
    fn first(&self) -> Option<&T> {
      self.as_slice().first()
    }

    #[inline(always)]
    fn last(&self) -> Option<&T> {
      self.as_slice().last()
    }

    #[inline(always)]
    fn len(&self) -> usize {
      SliceVec::len(self)
    }
  }

  impl<A, T> Container<T> for ArrayVec<A>
  where
    A: Array<Item = T>,
  {
    #[inline(always)]
    fn max_capacity(&self) -> usize {
      A::CAPACITY
    }

    #[inline(always)]
    fn push(&mut self, item: T) -> Result<(), T> {
      match self.try_push(item) {
        Some(t) => Err(t),
        None => Ok(()),
      }
    }

    #[inline(always)]
    fn first(&self) -> Option<&T> {
      self.as_slice().first()
    }

    #[inline(always)]
    fn last(&self) -> Option<&T> {
      self.as_slice().last()
    }

    #[inline(always)]
    fn len(&self) -> usize {
      ArrayVec::len(self)
    }
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  const _: () = {
    use tinyvec_1::TinyVec;

    impl<A, T> Container<T> for TinyVec<A>
    where
      A: Array<Item = T>,
    {
      #[inline(always)]
      fn max_capacity(&self) -> usize {
        usize::MAX
      }

      #[inline(always)]
      fn push(&mut self, item: T) -> Result<(), T> {
        TinyVec::push(self, item);
        Ok(())
      }

      #[inline(always)]
      fn first(&self) -> Option<&T> {
        self.as_slice().first()
      }

      #[inline(always)]
      fn last(&self) -> Option<&T> {
        self.as_slice().last()
      }

      #[inline(always)]
      fn len(&self) -> usize {
        TinyVec::len(self)
      }
    }
  };
};
