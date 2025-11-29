/// Trait for container types used in parsers.
pub trait Container<T> {
  /// Push an item into the container.
  ///
  /// Returns `Some(item)` if the container is full and cannot accept more items.
  fn push(&mut self, item: T) -> Option<T>;

  /// Returns the first item in the container, if any.
  fn first(&self) -> Option<&T>;

  /// Returns the last item in the container, if any.
  fn last(&self) -> Option<&T>;

  /// Returns the number of items in the container.
  fn len(&self) -> usize;

  /// Returns the maximum number of items the container can hold, if applicable.
  ///
  /// This is not reflects the actual capacity of the container, but rather a logical limit.
  /// For example, a dynamically sized container may return `usize::MAX`.
  fn capacity() -> usize;

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
  fn push(&mut self, item: T) -> Option<T> {
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

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn capacity() -> usize {
    U::capacity()
  }
}

macro_rules! blackhole {
  ($ty:ty) => {
    impl<T> Container<T> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push(&mut self, _: T) -> Option<T> {
        None
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

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn capacity() -> usize {
        usize::MAX
      }
    }
  };
}

blackhole!(());
blackhole!(core::marker::PhantomData<T>);
blackhole!(crate::utils::marker::Ignored<T>);
blackhole!(crate::lexer::BlackHole);

impl<T> Container<T> for Option<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push(&mut self, item: T) -> Option<T> {
    if self.is_none() {
      *self = Some(item);
      None
    } else {
      Some(item)
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

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn capacity() -> usize {
    1
  }
}

impl<T, N> Container<T> for generic_arraydeque::GenericArrayDeque<T, N>
where
  N: generic_arraydeque::ArrayLength,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push(&mut self, item: T) -> Option<T> {
    generic_arraydeque::GenericArrayDeque::push_back(self, item)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn first(&self) -> Option<&T> {
    generic_arraydeque::GenericArrayDeque::front(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn last(&self) -> Option<&T> {
    generic_arraydeque::GenericArrayDeque::back(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    generic_arraydeque::GenericArrayDeque::len(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn capacity() -> usize {
    N::to_usize()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::{collections::VecDeque, vec::Vec};

  impl<T> Container<T> for Vec<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) -> Option<T> {
      Vec::push(self, item);
      None
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

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn capacity() -> usize {
      usize::MAX
    }
  }

  impl<T> Container<T> for VecDeque<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, item: T) -> Option<T> {
      VecDeque::push_back(self, item);
      None
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

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn capacity() -> usize {
      usize::MAX
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
      fn push(&mut self, item: T) -> Option<T> {
        SmallVec::push(self, item);
        None
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

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn capacity() -> usize {
        usize::MAX
      }
    }
  };
};
