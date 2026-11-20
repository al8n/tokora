use hybrid_arraydeque::{ArrayDeque, ArraySize};

/// Trait for container types used in parsers.
///
/// # What the repetition drivers read
///
/// A `Container` is a **destination**, and the drivers treat it as one. They hand it parsed
/// elements and read back exactly two things: whether [`push`](Self::push) took an element, and
/// what [`max_capacity`](Self::max_capacity) says. Everything else a repetition construct
/// reports — how many elements it parsed, whether that count satisfies `at_least` / `at_most`,
/// and the count a [`FullContainer`](crate::error::FullContainer) names — is the
/// drivers' own bookkeeping, computed without consulting this trait. That is deliberate:
/// `Container` is implementable outside this crate, so an accounting law that rested on an
/// implementation behaving itself would be a law with no keeper.
///
/// The two obligations below are the ones that cannot be replaced by the drivers' own
/// bookkeeping, because they are what the two values *mean*. Nothing else an implementation
/// does is relied upon.
pub trait Container<T> {
  /// Takes `item`, or refuses it and hands it back unchanged.
  ///
  /// **Refuse an item only when the container cannot hold it.** `Err` is this crate's sole
  /// capacity channel: a refusal becomes a
  /// [`FullContainer`](crate::error::FullContainer) diagnostic naming
  /// [`max_capacity`](Self::max_capacity), so a refusal for any other reason reports a capacity
  /// the container never reached.
  ///
  /// A refusal is not a parse failure. The drivers drop the refused element, count it like any
  /// other parsed element, and carry on.
  ///
  /// # When this crate's containers refuse, and when growing panics instead
  ///
  /// - **A container with a fixed capacity refuses once it is full:** `ArrayDeque`, `Option`,
  ///   heapless's `Vec` and `Deque`, tinyvec's `ArrayVec` and `SliceVec`, and the Pratt family's
  ///   `PrattStack` where the crate cannot allocate.
  /// - **A container that grows refuses zero-sized items only, at `usize::MAX`:** `Vec`,
  ///   `VecDeque`, `SmallVec`, tinyvec's `TinyVec`, and `PrattStack` where the crate can allocate.
  ///   Each grows as its collection does. Zero-sized items take no memory, so nothing but the
  ///   length stops them, and the push refuses the item the length could not count.
  /// - **The three that discard, `()`, `PhantomData` and `Ignored`, take every item and refuse
  ///   none,** and `&mut C` answers as `C` does.
  /// - **Growing is not refusing.** A growth whose capacity overflows, by needing a layout of more
  ///   than `isize::MAX` bytes, panics as `Vec::push` does. An allocation failure goes to the global
  ///   allocation error handler, which aborts the process in a default `std` build. Neither reaches
  ///   the caller as an `Err`, so neither becomes a diagnostic.
  fn push(&mut self, item: T) -> Result<(), T>;

  /// Returns the first item in the container, if any.
  fn first(&self) -> Option<&T>;

  /// Returns the last item in the container, if any.
  fn last(&self) -> Option<&T>;

  /// Returns the number of items in the container.
  fn len(&self) -> usize;

  /// Returns `true` if the container is empty.
  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Returns the number of items this container can hold.
  ///
  /// `usize::MAX` when there is no fixed bound, e.g. for `Vec<T>`; the real bound otherwise.
  /// This is the capacity a [`FullContainer`](crate::error::FullContainer) diagnostic
  /// names, so it must be the bound [`push`](Self::push) refuses at.
  ///
  /// For a container that grows, `usize::MAX` is a logical bound. Zero-sized items meet it and are
  /// refused there. For a sized item, growth can panic before it: no layout holds more than
  /// `isize::MAX` bytes, and a collection that grows by doubling asks for more room than one item
  /// needs, so a 1 GiB `Vec<u8>` on a 32-bit target panics on the push that would double it. See
  /// [`push`](Self::push).
  fn max_capacity(&self) -> usize;
}

/// A [`Container`] that is also a stack: it gives back what was pushed last.
///
/// # Its obligations
///
/// `Stack` is implementable outside this crate, so what a reader of one may rely on is written
/// here, as [`Container`] writes its own:
///
/// - [`pop`](Self::pop) removes and returns the item pushed last and not yet popped, and answers
///   `None` only when every item pushed has been popped.
/// - [`last`](Container::last) and [`last_mut`](Self::last_mut) are that same item.
/// - [`len`](Container::len) counts the items pushed and not yet popped.
/// - [`push`](Container::push) keeps [`Container`]'s contract: it refuses only what the stack
///   cannot hold, and [`max_capacity`](Container::max_capacity) is the bound it refuses at. A
///   stack that grows may panic while growing, before that bound, as `push` documents; that is not
///   a refusal.
/// - **A `push` or a `pop` that unwinds leaves the items as they were.** A push that unwinds has
///   not taken its item, and a pop that unwinds has not removed one. A reader counts the items by
///   what these two return, and an unwind returns nothing, so a change that an unwind hides is a
///   count the reader can no longer keep.
///
/// Every stack this crate implements keeps that last obligation. None of their pushes or pops runs
/// code of the item's. A push into a fixed stack and every pop cannot unwind at all, and a stack
/// that grows, `Vec`, `VecDeque`, `SmallVec`, and the Pratt family's `PrattStack` where the crate
/// can allocate, unwinds only while it grows, before the item is placed.
///
/// Breaking them is a logic error. What a reader of the stack does then is unspecified, a panic
/// or a wrong result, but it is never undefined behaviour: the crate reads a `Stack` through
/// safe code alone.
pub trait Stack<T>: Container<T> {
  /// Removes the item pushed last and not yet popped, and returns it, or `None` when the stack is
  /// empty.
  fn pop(&mut self) -> Option<T>;

  /// The item [`pop`](Self::pop) would return, mutably, or `None` when the stack is empty.
  fn last_mut(&mut self) -> Option<&mut T>;
}

impl<T, U> Container<T> for &mut U
where
  U: Container<T>,
{
  #[inline(always)]
  fn max_capacity(&self) -> usize {
    U::max_capacity(self)
  }

  #[inline(always)]
  fn push(&mut self, item: T) -> Result<(), T> {
    U::push(self, item)
  }

  #[inline(always)]
  fn first(&self) -> Option<&T> {
    U::first(self)
  }

  #[inline(always)]
  fn last(&self) -> Option<&T> {
    U::last(self)
  }

  #[inline(always)]
  fn len(&self) -> usize {
    U::len(self)
  }
}

impl<T, U> Stack<T> for &mut U
where
  U: Stack<T>,
{
  #[inline(always)]
  fn pop(&mut self) -> Option<T> {
    U::pop(self)
  }

  #[inline(always)]
  fn last_mut(&mut self) -> Option<&mut T> {
    U::last_mut(self)
  }
}

macro_rules! blackhole {
  ($ty:ty) => {
    impl<T> Container<T> for $ty {
      #[inline(always)]
      fn max_capacity(&self) -> usize {
        usize::MAX
      }

      #[inline(always)]
      fn push(&mut self, _: T) -> Result<(), T> {
        Ok(())
      }

      #[inline(always)]
      fn first(&self) -> Option<&T> {
        None
      }

      #[inline(always)]
      fn last(&self) -> Option<&T> {
        None
      }

      #[inline(always)]
      fn len(&self) -> usize {
        0
      }
    }
  };
}

blackhole!(());
blackhole!(core::marker::PhantomData<T>);
blackhole!(crate::utils::Ignored<T>);

impl<T> Container<T> for Option<T> {
  #[inline(always)]
  fn max_capacity(&self) -> usize {
    1
  }

  #[inline(always)]
  fn push(&mut self, item: T) -> Result<(), T> {
    if self.is_none() {
      *self = Some(item);
      Ok(())
    } else {
      Err(item)
    }
  }

  #[inline(always)]
  fn first(&self) -> Option<&T> {
    self.as_ref()
  }

  #[inline(always)]
  fn last(&self) -> Option<&T> {
    self.as_ref()
  }

  #[inline(always)]
  fn len(&self) -> usize {
    if self.is_some() { 1 } else { 0 }
  }
}

impl<T, N> Container<T> for ArrayDeque<T, N>
where
  N: ArraySize,
{
  #[inline(always)]
  fn max_capacity(&self) -> usize {
    N::to_usize()
  }

  #[inline(always)]
  fn push(&mut self, item: T) -> Result<(), T> {
    match ArrayDeque::push_back(self, item) {
      None => Ok(()),
      Some(e) => Err(e),
    }
  }

  #[inline(always)]
  fn first(&self) -> Option<&T> {
    ArrayDeque::front(self)
  }

  #[inline(always)]
  fn last(&self) -> Option<&T> {
    ArrayDeque::back(self)
  }

  #[inline(always)]
  fn len(&self) -> usize {
    ArrayDeque::len(self)
  }
}

impl<T, N> Stack<T> for ArrayDeque<T, N>
where
  N: ArraySize,
{
  #[inline(always)]
  fn pop(&mut self) -> Option<T> {
    ArrayDeque::pop_back(self)
  }

  #[inline(always)]
  fn last_mut(&mut self) -> Option<&mut T> {
    ArrayDeque::back_mut(self)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
const _: () = {
  use std::{collections::VecDeque, vec::Vec};

  impl<T> Container<T> for Vec<T> {
    #[inline(always)]
    fn max_capacity(&self) -> usize {
      usize::MAX
    }

    #[inline(always)]
    fn push(&mut self, item: T) -> Result<(), T> {
      // Refused at `max_capacity`, as the trait requires, rather than left to `Vec::push`'s
      // capacity-overflow panic. Only zero-sized items reach it: they take no memory, so a `Vec`
      // holds them up to `usize::MAX` without allocating. Sized items run out of memory first,
      // and for them the test is the constant `false` and compiles away.
      if core::mem::size_of::<T>() == 0 && Vec::len(self) == usize::MAX {
        return Err(item);
      }
      Vec::push(self, item);
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
      Vec::len(self)
    }
  }

  impl<T> Stack<T> for Vec<T> {
    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
      Vec::pop(self)
    }

    #[inline(always)]
    fn last_mut(&mut self) -> Option<&mut T> {
      self.as_mut_slice().last_mut()
    }
  }

  impl<T> Container<T> for VecDeque<T> {
    #[inline(always)]
    fn max_capacity(&self) -> usize {
      usize::MAX
    }

    #[inline(always)]
    fn push(&mut self, item: T) -> Result<(), T> {
      // Refused at `max_capacity` rather than left to the deque's capacity-overflow panic, for the
      // reason `Vec`'s impl above gives.
      if core::mem::size_of::<T>() == 0 && VecDeque::len(self) == usize::MAX {
        return Err(item);
      }
      VecDeque::push_back(self, item);
      Ok(())
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
      VecDeque::len(self)
    }
  }

  impl<T> Stack<T> for VecDeque<T> {
    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
      VecDeque::pop_back(self)
    }

    #[inline(always)]
    fn last_mut(&mut self) -> Option<&mut T> {
      VecDeque::back_mut(self)
    }
  }
};

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests;
