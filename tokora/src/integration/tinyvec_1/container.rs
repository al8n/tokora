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
      if A::CAPACITY > u16::MAX as usize {
        // tinyvec's own capacity, which it clamps to the `u16` the length is stored in.
        ArrayVec::capacity(self)
      } else {
        A::CAPACITY
      }
    }

    #[inline(always)]
    fn push(&mut self, item: T) -> Result<(), T> {
      // tinyvec stores the length in a `u16`, and `try_push` tests only the backing array: past
      // `u16::MAX` items it writes the item and then overflows the length, which panics under
      // overflow checks and wraps to zero without them. So a backing array longer than that is
      // refused at tinyvec's own clamped `capacity`, before the write. The test is on a constant
      // first, so an array of an ordinary size keeps the plain path.
      if A::CAPACITY > u16::MAX as usize && ArrayVec::len(self) >= ArrayVec::capacity(self) {
        return Err(item);
      }
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
        // Refused at `max_capacity` rather than left to the heap `Vec`'s capacity-overflow panic,
        // as for `Vec` in `container/mod.rs`: only zero-sized items reach `usize::MAX`.
        if core::mem::size_of::<T>() == 0 && TinyVec::len(self) == usize::MAX {
          return Err(item);
        }
        // tinyvec's inline push stops at the end of the backing array, not at the `u16` its length
        // is stored in, so over an array longer than `u16::MAX` it would write the item and
        // overflow the length instead of moving to the heap. Move first, while every item is still
        // held, in order. The test is on a constant first, so an array of an ordinary size keeps
        // the plain path.
        if A::CAPACITY > u16::MAX as usize
          && self.is_inline()
          && TinyVec::len(self) >= TinyVec::capacity(self)
        {
          self.move_to_the_heap();
        }
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
