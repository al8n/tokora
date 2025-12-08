use generic_arraydeque::{ArrayLength, GenericArrayDeque};

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

/// Trait for container types that support delimiters.
pub trait DelimiterContainer<Open, Close, T>: Container<T> {
  /// Returns the opening delimiter of the container, if any.
  fn open(&self) -> Option<&Open>;

  /// Returns the closing delimiter of the container, if any.
  fn close(&self) -> Option<&Close>;

  /// Pushes an opening delimiter into the container.
  /// 
  /// Returns `Some(open)` if the container is full and cannot accept more opening delimiters.
  fn push_open(&mut self, open: Open) -> Option<Open>;

  /// Pushes a closing delimiter into the container.
  /// 
  /// Returns `Some(close)` if the container is full and cannot accept more closing delimiters.
  fn push_close(&mut self, close: Close) -> Option<Close>;
}

impl<T, U, Open, Close> DelimiterContainer<Open, Close, T> for &mut U
where
  U: DelimiterContainer<Open, Close, T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn open(&self) -> Option<&Open> {
    U::open(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn close(&self) -> Option<&Close> {
    U::close(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_open(&mut self, open: Open) -> Option<Open> {
    U::push_open(self, open)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_close(&mut self, close: Close) -> Option<Close> {
    U::push_close(self, close)
  }
}

/// Trait for container types that support separators.
pub trait SeparatorsContainer<Separator, T>: Container<T> {
  /// Pushes a separator into the container.
  /// 
  /// Returns `Some(separator)` if the container is full and cannot accept more separators.
  fn push_separator(&mut self, sep: Separator) -> Option<Separator>;
}

impl<T, U, Separator> SeparatorsContainer<Separator, T> for &mut U
where
  U: SeparatorsContainer<Separator, T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_separator(&mut self, sep: Separator) -> Option<Separator> {
    U::push_separator(self, sep)
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

    impl<T, Open, Close> DelimiterContainer<Open, Close, T> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn open(&self) -> Option<&Open> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn close(&self) -> Option<&Close> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push_open(&mut self, _: Open) -> Option<Open> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push_close(&mut self, _: Close) -> Option<Close> {
        None
      }
    }

    impl<T, Separator> SeparatorsContainer<Separator, T> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push_separator(&mut self, _: Separator) -> Option<Separator> {
        None
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

impl<T, Open, Close> DelimiterContainer<Open, Close, T> for Option<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn open(&self) -> Option<&Open> {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn close(&self) -> Option<&Close> {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_open(&mut self, _: Open) -> Option<Open> {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_close(&mut self, _: Close) -> Option<Close> {
    None
  }
}

impl<T, Separator> SeparatorsContainer<Separator, T> for Option<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_separator(&mut self, _: Separator) -> Option<Separator> {
    None
  }
}

impl<T, N> Container<T> for GenericArrayDeque<T, N>
where
  N: ArrayLength,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push(&mut self, item: T) -> Option<T> {
    GenericArrayDeque::push_back(self, item)
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

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn capacity() -> usize {
    N::to_usize()
  }
}

impl<T, Open, Close, N> DelimiterContainer<Open, Close, T> for GenericArrayDeque<T, N>
where
  N: ArrayLength,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn open(&self) -> Option<&Open> {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn close(&self) -> Option<&Close> {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_open(&mut self, _: Open) -> Option<Open> {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_close(&mut self, _: Close) -> Option<Close> {
    None
  }
}

impl<T, Separator, N> SeparatorsContainer<Separator, T> for GenericArrayDeque<T, N>
where
  N: ArrayLength,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_separator(&mut self, _: Separator) -> Option<Separator> {
    None
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

  impl<Open, Close, T> DelimiterContainer<Open, Close, T> for Vec<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn open(&self) -> Option<&Open> {
      None
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn close(&self) -> Option<&Close> {
      None
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_open(&mut self, _: Open) -> Option<Open> {
      None
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_close(&mut self, _: Close) -> Option<Close> {
      None
    }
  }

  impl<Separator, T> SeparatorsContainer<Separator, T> for Vec<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_separator(&mut self, _: Separator) -> Option<Separator> {
      None
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

  impl<Open, Close, T> DelimiterContainer<Open, Close, T> for VecDeque<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn open(&self) -> Option<&Open> {
      None
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn close(&self) -> Option<&Close> {
      None
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_open(&mut self, _: Open) -> Option<Open> {
      None
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_close(&mut self, _: Close) -> Option<Close> {
      None
    }
  }

  impl<Separator, T> SeparatorsContainer<Separator, T> for VecDeque<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_separator(&mut self, _: Separator) -> Option<Separator> {
      None
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

    impl<Open, Close, A, T> DelimiterContainer<Open, Close, T> for SmallVec<A>
    where
      A: smallvec::Array<Item = T>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn open(&self) -> Option<&Open> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn close(&self) -> Option<&Close> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push_open(&mut self, _: Open) -> Option<Open> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push_close(&mut self, _: Close) -> Option<Close> {
        None
      }
    }

    impl<Separator, A, T> SeparatorsContainer<Separator, T> for SmallVec<A>
    where
      A: smallvec::Array<Item = T>,
    {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push_separator(&mut self, _: Separator) -> Option<Separator> {
        None
      }
    }
  };
};
