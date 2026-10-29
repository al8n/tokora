use crate::container::Container;

const _: () = {
  use heapless_0_9::{Deque, LenType, Vec};

  impl<T, LenT, const N: usize> Container<T> for Vec<T, N, LenT>
  where
    LenT: LenType,
  {
    #[inline(always)]
    fn max_capacity(&self) -> usize {
      N
    }

    #[inline(always)]
    fn push(&mut self, item: T) -> Result<(), T> {
      Vec::push(self, item)
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
      self.as_slice().len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
      Vec::is_empty(self)
    }
  }

  impl<T, const N: usize> Container<T> for Deque<T, N> {
    #[inline(always)]
    fn max_capacity(&self) -> usize {
      N
    }

    #[inline(always)]
    fn push(&mut self, item: T) -> Result<(), T> {
      Deque::push_back(self, item)
    }

    #[inline(always)]
    fn first(&self) -> Option<&T> {
      self.front()
    }

    #[inline(always)]
    fn last(&self) -> Option<&T> {
      self.back()
    }

    #[inline(always)]
    fn len(&self) -> usize {
      Deque::len(self)
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
      Deque::is_empty(self)
    }
  }
};
