use crate::container::Container;

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
};
