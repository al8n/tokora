//! `PrattStack` in every tier: stack order within and across the inline bound, the refusal past
//! `N` where nothing can spill, and the four `DefaultPrattStack` holds inline. Each cell names
//! `core` alone, so the file runs in the tier with no allocator too.

use super::*;
use hybrid_arraydeque::typenum::U2;

#[test]
fn within_the_inline_bound_it_is_a_stack() {
  let mut stack = PrattStack::<u32, U4>::new();
  assert!(stack.is_empty());
  assert_eq!(stack.pop(), None);
  assert!(stack.last_mut().is_none());
  for item in 0..4 {
    assert_eq!(stack.push(item), Ok(()));
  }
  assert_eq!(stack.len(), 4);
  assert_eq!(stack.first(), Some(&0));
  assert_eq!(stack.last(), Some(&3));
  *stack.last_mut().unwrap() = 30;
  assert_eq!(stack.last(), Some(&30));
  assert_eq!(stack.pop(), Some(30));
  assert_eq!(stack.pop(), Some(2));
  assert_eq!(stack.pop(), Some(1));
  assert_eq!(stack.pop(), Some(0));
  assert_eq!(stack.pop(), None);
  assert!(stack.is_empty());
}

#[test]
fn default_is_new() {
  assert_eq!(PrattStack::<u32, U4>::default(), PrattStack::new());
}

#[test]
fn a_clone_equals_its_original_and_not_a_popped_one() {
  let mut stack = PrattStack::<u32, U2>::new();
  stack.push(1).unwrap();
  stack.push(2).unwrap();
  let clone = stack.clone();
  assert_eq!(clone, stack);
  assert_eq!(stack.pop(), Some(2));
  assert_ne!(clone, stack);
}

/// The default's four, at run time beside the `const` assertion: the fifth item is the first one
/// that does not fit inline.
#[test]
fn the_default_holds_four_inline() {
  let mut stack = DefaultPrattStack::<u32>::new();
  for item in 0..4 {
    assert_eq!(stack.push(item), Ok(()));
  }
  assert_eq!(stack.inline.len(), 4);
  assert!(stack.spilled().is_empty());
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn across_the_inline_bound_it_spills_and_keeps_stack_order() {
  let mut stack = PrattStack::<u32, U2>::new();
  assert_eq!(stack.max_capacity(), usize::MAX);
  for item in 0..9 {
    assert_eq!(stack.push(item), Ok(()));
  }
  assert_eq!(stack.len(), 9);
  assert_eq!(stack.inline.len(), 2);
  assert_eq!(stack.spilled().len(), 7);
  assert_eq!(stack.first(), Some(&0));
  assert_eq!(stack.last(), Some(&8));

  // Down across the bound, then back up across it.
  for expected in (1..9).rev() {
    assert_eq!(stack.pop(), Some(expected));
  }
  assert_eq!(stack.len(), 1);
  assert_eq!(stack.last(), Some(&0));
  for item in 10..13 {
    assert_eq!(stack.push(item), Ok(()));
  }
  assert_eq!(stack.inline.len(), 2);
  assert_eq!(stack.spilled().len(), 2);
  *stack.last_mut().unwrap() = 120;
  assert_eq!(stack.pop(), Some(120));
  assert_eq!(stack.pop(), Some(11));
  assert_eq!(stack.pop(), Some(10));
  assert_eq!(stack.pop(), Some(0));
  assert_eq!(stack.pop(), None);
  assert!(stack.is_empty());
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn equality_is_of_the_items_and_not_of_the_history() {
  let mut deep = PrattStack::<u32, U2>::new();
  for item in 0..5 {
    deep.push(item).unwrap();
  }
  for _ in 0..3 {
    assert!(deep.pop().is_some());
  }
  let mut shallow = PrattStack::<u32, U2>::new();
  shallow.push(0).unwrap();
  shallow.push(1).unwrap();
  assert_eq!(deep, shallow);
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn the_default_spills_the_fifth() {
  let mut stack = DefaultPrattStack::<u32>::new();
  for item in 0..5 {
    assert_eq!(stack.push(item), Ok(()));
  }
  assert_eq!(stack.inline.len(), 4);
  assert_eq!(stack.spilled(), &[4]);
  assert_eq!(stack.pop(), Some(4));
  assert!(stack.spilled().is_empty());
  assert_eq!(stack.last(), Some(&3));
}

/// Zero-sized items take no memory, so nothing but the length stops them: a stack of `()` that
/// already holds `usize::MAX` items refuses the next one, before anything moves, where its length
/// would otherwise overflow.
///
/// The state is built directly, with no `usize::MAX` pushes: four units inline, and the rest put
/// on the heap by one slice copy, which moves no bytes and allocates nothing. `vec![(); n]` would
/// not do: it clones its element once for each item, in a loop that a debug build and Miri run.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn a_stack_of_zero_sized_items_refuses_the_push_its_length_cannot_count() {
  let mut stack = DefaultPrattStack::<()>::new();
  for _ in 0..4 {
    assert_eq!(stack.push(()), Ok(()));
  }
  stack.spill.extend_from_slice(&[(); usize::MAX - 4]);
  assert_eq!(stack.len(), usize::MAX);
  assert_eq!(stack.max_capacity(), usize::MAX);

  assert_eq!(stack.push(()), Err(()));
  assert_eq!(stack.len(), usize::MAX);
  assert!(!stack.is_empty());

  assert_eq!(stack.pop(), Some(()));
  assert_eq!(stack.len(), usize::MAX - 1);
  assert_eq!(stack.push(()), Ok(()));
  assert_eq!(stack.len(), usize::MAX);
  assert_eq!(stack.push(()), Err(()));
}

/// Without its heap half the stack refuses the push past `N`, hands the item back, and stays a
/// stack: the refusal changes nothing, and a pop makes room again.
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[test]
fn without_an_allocator_the_default_refuses_the_fifth() {
  let mut stack = DefaultPrattStack::<u32>::new();
  assert_eq!(stack.max_capacity(), 4);
  for item in 0..4 {
    assert_eq!(stack.push(item), Ok(()));
  }
  assert_eq!(stack.push(4), Err(4));
  assert_eq!(stack.len(), 4);
  assert_eq!(stack.last(), Some(&3));
  assert_eq!(stack.pop(), Some(3));
  assert_eq!(stack.push(4), Ok(()));
  assert_eq!(stack.last(), Some(&4));
  assert_eq!(stack.push(5), Err(5));
}
