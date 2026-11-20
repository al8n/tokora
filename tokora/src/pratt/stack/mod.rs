//! The stack storage for an expression's pending operators: `N` items inline, and past them the
//! heap, where the crate can allocate.

use hybrid_arraydeque::{ArrayDeque, ArraySize, typenum::U4};

use crate::container::{Container, Stack};

#[cfg(any(feature = "std", feature = "alloc"))]
use std::vec::Vec;

/// A stack for the operators an expression keeps pending: its first `N` items inline and, where
/// the crate can allocate, the rest on the heap.
///
/// [`DefaultPrattStack`] is this at four.
///
/// # Past `N`
///
/// - **With `alloc` or `std`**, a push goes to a `Vec` beside the inline array, allocated the first
///   time one is needed. A push is never refused, and
///   [`max_capacity`](Container::max_capacity) is `usize::MAX`.
/// - **With neither**, the push is refused and the item handed back, by [`Container`]'s contract,
///   and [`max_capacity`](Container::max_capacity) is `N`. Where an expression keeps more pending,
///   name a larger `N`: `PrattStack<_, U16>`.
///
/// It is one type in every tier. A build's features choose what happens past `N`, and nothing a
/// caller writes does. So a library cannot rely on the refusal: one crate anywhere in a build that
/// turns `alloc` on makes every `PrattStack` in it spill instead. A `no_std` library tested on a
/// host where a dependency turns `alloc` on never meets the refusal there; test it with
/// `--no-default-features` as well.
///
/// # How many operators an expression keeps pending
///
/// An operator is pending while its operand is being read. A left-associative chain keeps one
/// however long it is: `a + b + c + d` never holds two. A right-associative chain, a run of prefix
/// operators and a ladder of rising powers keep one for each operator: in a C-family table, with
/// `=` right-associative, `a = b = c = d = e = 0` holds five.
#[derive(Debug, Clone)]
pub struct PrattStack<T, N: ArraySize = U4> {
  /// The bottom `N` items, bottom first.
  inline: ArrayDeque<T, N>,
  /// The items above the inline ones, bottom first. It holds any only while `inline` is full: a
  /// push goes inline while this is empty and there is room, and a pop takes from here first. So
  /// how the items are split between the two is a function of the items alone.
  #[cfg(any(feature = "std", feature = "alloc"))]
  spill: Vec<T>,
}

/// [`PrattStack`] at four items inline, in every tier.
///
/// Past four, a push goes to the heap where the crate can allocate, and is refused where it
/// cannot. [`PrattStack`] says which expressions keep more than four operators pending.
pub type DefaultPrattStack<T> = PrattStack<T, U4>;

// `DefaultPrattStack` holds four inline in every tier: its alias cannot move without this line.
const _: () = assert!(<DefaultPrattStack<()>>::INLINE == 4);

impl<T, N: ArraySize> PrattStack<T, N> {
  /// How many items it holds inline: `N`.
  const INLINE: usize = N::USIZE;

  /// An empty stack. It allocates nothing: a push reaches the heap only past `N` items.
  #[inline]
  pub const fn new() -> Self {
    Self {
      inline: ArrayDeque::new(),
      #[cfg(any(feature = "std", feature = "alloc"))]
      spill: Vec::new(),
    }
  }

  /// The items above the inline ones, bottom first.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[inline(always)]
  fn spilled(&self) -> &[T] {
    &self.spill
  }

  /// The items above the inline ones: none, where the crate cannot allocate.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  #[inline(always)]
  fn spilled(&self) -> &[T] {
    &[]
  }
}

impl<T, N: ArraySize> Default for PrattStack<T, N> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

/// Equal items in the same order. How the items are split between the inline array and the heap
/// is a function of the items, so comparing the two halves compares the stacks.
impl<T: PartialEq, N: ArraySize> PartialEq for PrattStack<T, N> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.inline == other.inline && self.spilled() == other.spilled()
  }
}

impl<T: Eq, N: ArraySize> Eq for PrattStack<T, N> {}

impl<T, N: ArraySize> Container<T> for PrattStack<T, N> {
  /// Inline while nothing has spilled and the array has room, and to the heap after that. Never
  /// refused.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[inline]
  fn push(&mut self, item: T) -> Result<(), T> {
    let item = if self.spill.is_empty() {
      match self.inline.push_back(item) {
        None => return Ok(()),
        Some(refused) => refused,
      }
    } else {
      item
    };
    self.spill.push(item);
    Ok(())
  }

  /// Inline, and refused once `N` are held: there is nowhere else to put it.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  #[inline]
  fn push(&mut self, item: T) -> Result<(), T> {
    match self.inline.push_back(item) {
      None => Ok(()),
      Some(refused) => Err(refused),
    }
  }

  #[inline]
  fn first(&self) -> Option<&T> {
    match self.inline.front() {
      Some(item) => Some(item),
      None => self.spilled().first(),
    }
  }

  #[inline]
  fn last(&self) -> Option<&T> {
    match self.spilled().last() {
      Some(item) => Some(item),
      None => self.inline.back(),
    }
  }

  #[inline]
  fn len(&self) -> usize {
    self.inline.len() + self.spilled().len()
  }

  /// `usize::MAX`: a push is never refused where the crate can allocate.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[inline]
  fn max_capacity(&self) -> usize {
    usize::MAX
  }

  /// `N`: a push past it is refused where the crate cannot allocate.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  #[inline]
  fn max_capacity(&self) -> usize {
    Self::INLINE
  }
}

impl<T, N: ArraySize> Stack<T> for PrattStack<T, N> {
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[inline]
  fn pop(&mut self) -> Option<T> {
    match Vec::pop(&mut self.spill) {
      Some(item) => Some(item),
      None => self.inline.pop_back(),
    }
  }

  #[cfg(not(any(feature = "std", feature = "alloc")))]
  #[inline]
  fn pop(&mut self) -> Option<T> {
    self.inline.pop_back()
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  #[inline]
  fn last_mut(&mut self) -> Option<&mut T> {
    match <[T]>::last_mut(&mut self.spill) {
      Some(item) => Some(item),
      None => self.inline.back_mut(),
    }
  }

  #[cfg(not(any(feature = "std", feature = "alloc")))]
  #[inline]
  fn last_mut(&mut self) -> Option<&mut T> {
    self.inline.back_mut()
  }
}

#[cfg(test)]
mod tests;
