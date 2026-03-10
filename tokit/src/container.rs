use generic_arraydeque::{ArrayLength, GenericArrayDeque};

/// Trait for container types used in parsers.
pub trait Container<T> {
  /// Push an item into the container.
  fn push(&mut self, item: T) -> Result<(), T>;

  /// Returns the first item in the container, if any.
  fn first(&self) -> Option<&T>;

  /// Returns the last item in the container, if any.
  fn last(&self) -> Option<&T>;

  /// Returns the number of items in the container.
  fn len(&self) -> usize;

  /// Returns `true` if the container is empty.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Returns the maximum capacity of the container.
  ///
  /// If the container has no fixed maximum capacity, returns `usize::MAX`, e.g., for `Vec<T>`.
  /// Otherwise, returns the actual maximum capacity.
  fn max_capacity(&self) -> usize;
}

impl<T, U> Container<T> for &mut U
where
  U: Container<T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn max_capacity(&self) -> usize {
    U::max_capacity(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push(&mut self, item: T) -> Result<(), T> {
    U::push(self, item)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn first(&self) -> Option<&T> {
    U::first(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn last(&self) -> Option<&T> {
    U::last(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    U::len(self)
  }
}

macro_rules! blackhole {
  ($ty:ty) => {
    impl<T> Container<T> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn max_capacity(&self) -> usize {
        usize::MAX
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push(&mut self, _: T) -> Result<(), T> {
        Ok(())
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn first(&self) -> Option<&T> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn last(&self) -> Option<&T> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn len(&self) -> usize {
        0
      }
    }
  };
}

blackhole!(());
blackhole!(core::marker::PhantomData<T>);
blackhole!(crate::utils::marker::Ignored<T>);

impl<T> Container<T> for Option<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn max_capacity(&self) -> usize {
    1
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push(&mut self, item: T) -> Result<(), T> {
    if self.is_none() {
      *self = Some(item);
      Ok(())
    } else {
      Err(item)
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn first(&self) -> Option<&T> {
    self.as_ref()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn last(&self) -> Option<&T> {
    self.as_ref()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    if self.is_some() { 1 } else { 0 }
  }
}

impl<T, N> Container<T> for GenericArrayDeque<T, N>
where
  N: ArrayLength,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn max_capacity(&self) -> usize {
    N::to_usize()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push(&mut self, item: T) -> Result<(), T> {
    match GenericArrayDeque::push_back(self, item) {
      None => Ok(()),
      Some(e) => Err(e),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn first(&self) -> Option<&T> {
    GenericArrayDeque::front(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn last(&self) -> Option<&T> {
    GenericArrayDeque::back(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    GenericArrayDeque::len(self)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
const _: () = {
  use std::{collections::VecDeque, vec::Vec};

  impl<T> Container<T> for Vec<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn max_capacity(&self) -> usize {
      usize::MAX
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) -> Result<(), T> {
      Vec::push(self, item);
      Ok(())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn first(&self) -> Option<&T> {
      self.as_slice().first()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn last(&self) -> Option<&T> {
      self.as_slice().last()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
      Vec::len(self)
    }
  }

  impl<T> Container<T> for VecDeque<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn max_capacity(&self) -> usize {
      usize::MAX
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) -> Result<(), T> {
      VecDeque::push_back(self, item);
      Ok(())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn first(&self) -> Option<&T> {
      self.front()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn last(&self) -> Option<&T> {
      self.back()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
      VecDeque::len(self)
    }
  }
};

#[cfg(feature = "smallvec_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "smallvec_1")))]
const _: () = {
  use smallvec_1::SmallVec;

  impl<A, T> Container<T> for SmallVec<A>
  where
    A: smallvec_1::Array<Item = T>,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn max_capacity(&self) -> usize {
      usize::MAX
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) -> Result<(), T> {
      SmallVec::push(self, item);
      Ok(())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn first(&self) -> Option<&T> {
      self.as_slice().first()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn last(&self) -> Option<&T> {
      self.as_slice().last()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
      SmallVec::len(self)
    }
  }
};

#[cfg(feature = "tinyvec_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "tinyvec_1")))]
const _: () = {
  use tinyvec_1::{Array, ArrayVec, SliceVec};

  impl<T> Container<T> for SliceVec<'_, T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn max_capacity(&self) -> usize {
      self.capacity()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) -> Result<(), T> {
      SliceVec::push(self, item);
      Ok(())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn first(&self) -> Option<&T> {
      self.as_slice().first()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn last(&self) -> Option<&T> {
      self.as_slice().last()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
      SliceVec::len(self)
    }
  }

  impl<A, T> Container<T> for ArrayVec<A>
  where
    A: Array<Item = T>,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn max_capacity(&self) -> usize {
      A::CAPACITY
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) -> Result<(), T> {
      match self.try_push(item) {
        Some(t) => Err(t),
        None => Ok(()),
      }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn first(&self) -> Option<&T> {
      self.as_slice().first()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn last(&self) -> Option<&T> {
      self.as_slice().last()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
      ArrayVec::len(self)
    }
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  {
    use tinyvec_1::TinyVec;

    impl<A, T> Container<T> for TinyVec<A>
    where
      A: Array<Item = T>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn max_capacity(&self) -> usize {
        usize::MAX
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push(&mut self, item: T) -> Result<(), T> {
        TinyVec::push(self, item);
        Ok(())
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn first(&self) -> Option<&T> {
        self.as_slice().first()
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn last(&self) -> Option<&T> {
        self.as_slice().last()
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn len(&self) -> usize {
        TinyVec::len(self)
      }
    }
  }
};

#[cfg(feature = "heapless_0_9")]
#[cfg_attr(docsrs, doc(cfg(feature = "heapless_0_9")))]
const _: () = {
  use heapless_0_9::{Deque, LenType, Vec};

  impl<T, LenT, const N: usize> Container<T> for Vec<T, N, LenT>
  where
    LenT: LenType,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn max_capacity(&self) -> usize {
      N
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) -> Result<(), T> {
      Vec::push(self, item)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn first(&self) -> Option<&T> {
      self.as_slice().first()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn last(&self) -> Option<&T> {
      self.as_slice().last()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
      self.as_slice().len()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_empty(&self) -> bool {
      Vec::is_empty(self)
    }
  }

  impl<T, const N: usize> Container<T> for Deque<T, N> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn max_capacity(&self) -> usize {
      N
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) -> Result<(), T> {
      Deque::push_back(self, item)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn first(&self) -> Option<&T> {
      self.front()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn last(&self) -> Option<&T> {
      self.back()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
      Deque::len(self)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_empty(&self) -> bool {
      Deque::is_empty(self)
    }
  }
};

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;
  use std::vec::Vec;

  // --- () blackhole tests ---

  #[test]
  fn unit_push_discards() {
    let mut c = ();
    assert!(Container::<i32>::push(&mut c, 42).is_ok());
  }

  #[test]
  fn unit_len_is_zero() {
    let c = ();
    assert_eq!(Container::<i32>::len(&c), 0);
  }

  #[test]
  fn unit_is_empty() {
    let c = ();
    assert!(Container::<i32>::is_empty(&c));
  }

  #[test]
  fn unit_max_capacity() {
    let c = ();
    assert_eq!(Container::<i32>::max_capacity(&c), usize::MAX);
  }

  #[test]
  fn unit_first_is_none() {
    let c = ();
    assert!(Container::<i32>::first(&c).is_none());
  }

  #[test]
  fn unit_last_is_none() {
    let c = ();
    assert!(Container::<i32>::last(&c).is_none());
  }

  // --- PhantomData blackhole tests ---

  #[test]
  fn phantom_push_discards() {
    let mut c = core::marker::PhantomData::<i32>;
    assert!(Container::<i32>::push(&mut c, 99).is_ok());
    assert_eq!(Container::<i32>::len(&c), 0);
  }

  // --- Option<T> tests ---

  #[test]
  fn option_push_first_ok() {
    let mut c: Option<i32> = None;
    assert!(c.push(42).is_ok());
    assert_eq!(c, Some(42));
  }

  #[test]
  fn option_push_second_err() {
    let mut c: Option<i32> = Some(1);
    let result = c.push(2);
    assert_eq!(result, Err(2));
    assert_eq!(c, Some(1));
  }

  #[test]
  fn option_first_and_last() {
    let mut c: Option<i32> = None;
    assert!(c.first().is_none());
    assert!(c.last().is_none());
    c.push(10).unwrap();
    assert_eq!(c.first(), Some(&10));
    assert_eq!(c.last(), Some(&10));
  }

  #[test]
  fn option_len() {
    let mut c: Option<i32> = None;
    assert_eq!(c.len(), 0);
    assert!(c.is_empty());
    c.push(5).unwrap();
    assert_eq!(c.len(), 1);
    assert!(!c.is_empty());
  }

  #[test]
  fn option_max_capacity() {
    let c: Option<i32> = None;
    assert_eq!(c.max_capacity(), 1);
  }

  // --- Vec<T> tests ---

  #[test]
  fn vec_push_always_ok() {
    let mut c: Vec<i32> = Vec::new();
    Container::push(&mut c, 1).unwrap();
    Container::push(&mut c, 2).unwrap();
    Container::push(&mut c, 3).unwrap();
    assert_eq!(Container::len(&c), 3);
  }

  #[test]
  fn vec_first_and_last() {
    let mut c: Vec<i32> = Vec::new();
    assert!(Container::first(&c).is_none());
    assert!(Container::last(&c).is_none());
    Container::push(&mut c, 10).unwrap();
    Container::push(&mut c, 20).unwrap();
    assert_eq!(Container::first(&c), Some(&10));
    assert_eq!(Container::last(&c), Some(&20));
  }

  #[test]
  fn vec_max_capacity() {
    let c: Vec<i32> = Vec::new();
    assert_eq!(Container::max_capacity(&c), usize::MAX);
  }

  #[test]
  fn vec_is_empty() {
    let c: Vec<i32> = Vec::new();
    assert!(Container::is_empty(&c));
  }

  // --- &mut U delegation tests ---

  #[test]
  fn ref_mut_delegates() {
    let mut inner: Option<i32> = None;
    let c: &mut Option<i32> = &mut inner;
    assert!(c.push(42).is_ok());
    assert_eq!(c.first(), Some(&42));
    assert_eq!(c.last(), Some(&42));
    assert_eq!(c.len(), 1);
    assert_eq!(c.max_capacity(), 1);
  }
}
