use crate::container::{Container, Stack};

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

  impl<T, LenT, const N: usize> Stack<T> for Vec<T, N, LenT>
  where
    LenT: LenType,
  {
    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
      Vec::pop(self)
    }

    #[inline(always)]
    fn last_mut(&mut self) -> Option<&mut T> {
      self.as_mut_slice().last_mut()
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
      // The two halves of the ring, summed. They are disjoint runs of the buffer, so the sum is
      // at most `N`. `Deque::len` computes a wrapped length as `back + N - front` instead, and
      // that overflows when `N` is near `usize::MAX`, which a deque of zero-sized items can be.
      let (front, back) = self.as_slices();
      front.len() + back.len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
      Deque::is_empty(self)
    }
  }

  impl<T, const N: usize> Stack<T> for Deque<T, N> {
    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
      Deque::pop_back(self)
    }

    #[inline(always)]
    fn last_mut(&mut self) -> Option<&mut T> {
      Deque::back_mut(self)
    }
  }
};
