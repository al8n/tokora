use generic_arraydeque::{ArrayLength, GenericArrayDeque};

/// Trait for container types used in parsers.
pub trait Container<T> {
  /// Push an item into the container.
  fn push(&mut self, item: T);

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
}

impl<T, U> Container<T> for &mut U
where
  U: Container<T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push(&mut self, item: T) {
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
      fn push(&mut self, _: T) {}

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
  fn push(&mut self, item: T) {
    *self = Some(item);
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
  fn push(&mut self, item: T) {
    GenericArrayDeque::push_back(self, item);
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
const _: () = {
  use std::{collections::VecDeque, vec::Vec};

  impl<T> Container<T> for Vec<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) {
      Vec::push(self, item);
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
    fn push(&mut self, item: T) {
      VecDeque::push_back(self, item);
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

  #[cfg(feature = "smallvec")]
  const _: () = {
    use smallvec::SmallVec;

    impl<A, T> Container<T> for SmallVec<A>
    where
      A: smallvec::Array<Item = T>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push(&mut self, item: T) {
        SmallVec::push(self, item);
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
};
