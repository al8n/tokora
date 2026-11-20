use crate::container::{Container, Stack};

const _: () = {
  use smallvec_1::SmallVec;

  impl<A, T> Container<T> for SmallVec<A>
  where
    A: smallvec_1::Array<Item = T>,
  {
    #[inline(always)]
    fn max_capacity(&self) -> usize {
      usize::MAX
    }

    #[inline(always)]
    fn push(&mut self, item: T) -> Result<(), T> {
      // Refused at `max_capacity` rather than left to `SmallVec::push`'s capacity-overflow panic,
      // as for `Vec` in `container/mod.rs`. A `SmallVec` holds zero-sized items inline up to
      // `usize::MAX`, and sized items run out of memory first.
      if core::mem::size_of::<T>() == 0 && SmallVec::len(self) == usize::MAX {
        return Err(item);
      }
      SmallVec::push(self, item);
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
      SmallVec::len(self)
    }
  }

  impl<A, T> Stack<T> for SmallVec<A>
  where
    A: smallvec_1::Array<Item = T>,
  {
    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
      SmallVec::pop(self)
    }

    #[inline(always)]
    fn last_mut(&mut self) -> Option<&mut T> {
      self.as_mut_slice().last_mut()
    }
  }
};
