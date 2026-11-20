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

// --- VecDeque tests ---

#[test]
fn vecdeque_push_and_accessors() {
  use std::collections::VecDeque;
  let mut c: VecDeque<i32> = VecDeque::new();
  assert!(Container::is_empty(&c));
  assert!(Container::first(&c).is_none());
  assert!(Container::last(&c).is_none());
  Container::push(&mut c, 10).unwrap();
  Container::push(&mut c, 20).unwrap();
  assert_eq!(Container::len(&c), 2);
  assert_eq!(Container::first(&c), Some(&10));
  assert_eq!(Container::last(&c), Some(&20));
  assert_eq!(Container::max_capacity(&c), usize::MAX);
}

// --- ArrayDeque tests ---

#[test]
fn hybrid_arraydeque_push_and_accessors() {
  use hybrid_arraydeque::typenum::U4;
  let mut c: ArrayDeque<i32, U4> = ArrayDeque::new();
  assert!(Container::is_empty(&c));
  assert!(Container::first(&c).is_none());
  assert!(Container::last(&c).is_none());
  assert_eq!(Container::max_capacity(&c), 4);
  Container::push(&mut c, 1).unwrap();
  Container::push(&mut c, 2).unwrap();
  assert_eq!(Container::len(&c), 2);
  assert_eq!(Container::first(&c), Some(&1));
  assert_eq!(Container::last(&c), Some(&2));
}

#[test]
fn hybrid_arraydeque_push_overflow() {
  use hybrid_arraydeque::typenum::U2;
  let mut c: ArrayDeque<i32, U2> = ArrayDeque::new();
  assert!(Container::push(&mut c, 1).is_ok());
  assert!(Container::push(&mut c, 2).is_ok());
  assert_eq!(Container::push(&mut c, 3), Err(3));
}

// --- Ignored blackhole tests ---

#[test]
fn ignored_push_discards() {
  let mut c = crate::utils::Ignored::<i32>::default();
  assert!(Container::<i32>::push(&mut c, 42).is_ok());
  assert_eq!(Container::<i32>::len(&c), 0);
  assert!(Container::<i32>::is_empty(&c));
  assert!(Container::<i32>::first(&c).is_none());
  assert!(Container::<i32>::last(&c).is_none());
  assert_eq!(Container::<i32>::max_capacity(&c), usize::MAX);
}

// --- Stack tests ---

/// Holds one stack to the obligations `Stack` writes down: `pop` is the item pushed last and not
/// yet popped, `last` and `last_mut` are that same item, and `len` counts what is still held.
fn drive_stack<S: Stack<i32>>(mut stack: S) {
  assert_eq!(stack.pop(), None);
  assert!(stack.last_mut().is_none());
  stack.push(1).unwrap();
  stack.push(2).unwrap();
  stack.push(3).unwrap();
  assert_eq!(stack.len(), 3);
  assert_eq!(stack.last(), Some(&3));
  *stack.last_mut().unwrap() = 30;
  assert_eq!(stack.last(), Some(&30));
  assert_eq!(stack.pop(), Some(30));
  assert_eq!(stack.len(), 2);
  stack.push(4).unwrap();
  assert_eq!(stack.pop(), Some(4));
  assert_eq!(stack.pop(), Some(2));
  assert_eq!(stack.pop(), Some(1));
  assert_eq!(stack.pop(), None);
  assert!(stack.is_empty());
}

#[test]
fn vec_is_a_stack() {
  drive_stack(Vec::new());
}

#[test]
fn vecdeque_is_a_stack() {
  drive_stack(std::collections::VecDeque::new());
}

#[test]
fn hybrid_arraydeque_is_a_stack() {
  use hybrid_arraydeque::typenum::U4;
  drive_stack(ArrayDeque::<i32, U4>::new());
}

#[test]
fn ref_mut_stack_delegates() {
  let mut inner: Vec<i32> = Vec::new();
  drive_stack(&mut inner);
  assert!(inner.is_empty());
}

// --- The length bound of zero-sized items ---

/// Holds a stack of zero-sized items already at `usize::MAX` to `Container`'s refusal: the push
/// is refused and the item handed back, the length does not move, and a pop makes room for the
/// next push.
fn refuses_at_the_length_bound<S: Stack<()>>(mut stack: S) {
  assert_eq!(stack.len(), usize::MAX);
  assert_eq!(stack.max_capacity(), usize::MAX);
  assert_eq!(stack.push(()), Err(()));
  assert_eq!(stack.len(), usize::MAX);
  assert_eq!(stack.pop(), Some(()));
  assert_eq!(stack.len(), usize::MAX - 1);
  assert_eq!(stack.push(()), Ok(()));
  assert_eq!(stack.len(), usize::MAX);
}

/// `usize::MAX` units, put in place by one slice copy, which moves no bytes and allocates nothing.
/// `vec![(); usize::MAX]` would clone the unit once for each item, in a loop that a debug build
/// and Miri both run.
fn full_of_units() -> Vec<()> {
  let mut units = Vec::new();
  units.extend_from_slice(&[(); usize::MAX]);
  units
}

#[test]
fn a_full_vec_of_units_refuses_the_next_push() {
  refuses_at_the_length_bound(full_of_units());
}

#[test]
fn a_full_vecdeque_of_units_refuses_the_next_push() {
  // `From<Vec>` takes the vector's buffer as it is, so this moves nothing either.
  refuses_at_the_length_bound(std::collections::VecDeque::from(full_of_units()));
}

#[cfg(feature = "smallvec_1")]
#[test]
fn a_full_smallvec_of_units_refuses_the_next_push() {
  // A `SmallVec` holds zero-sized items inline at any length, so this one never spills.
  let mut units = smallvec_1::SmallVec::<[(); 4]>::new();
  units.extend_from_slice(&[(); usize::MAX]);
  refuses_at_the_length_bound(units);
}

/// `TinyVec` is a `Container` and not a `Stack`, so the pop that makes room is its own.
#[cfg(feature = "tinyvec_1")]
#[test]
fn a_full_tinyvec_of_units_refuses_the_next_push() {
  let mut units = tinyvec_1::TinyVec::<[(); 4]>::Heap(full_of_units());
  assert_eq!(Container::max_capacity(&units), usize::MAX);
  assert_eq!(Container::push(&mut units, ()), Err(()));
  assert_eq!(Container::len(&units), usize::MAX);
  assert_eq!(units.pop(), Some(()));
  assert_eq!(Container::push(&mut units, ()), Ok(()));
  assert_eq!(Container::len(&units), usize::MAX);
}

/// A heapless `Deque` whose items wrap round the end of its ring. With `N` at `usize::MAX`, which
/// zero-sized items allow, the deque's own `len` overflows computing `back + N - front`; the
/// adapter's length is the sum of the two halves of the ring, which is at most `N`. The deque is a
/// `Container` and no `Stack`, so the pops are its own.
#[cfg(feature = "heapless_0_9")]
#[test]
fn a_heapless_deque_wrapped_round_the_ring_counts_its_items() {
  let mut units = heapless_0_9::Deque::<(), { usize::MAX }>::new();
  // One at the front moves `front` to the last slot of the ring, one at the back moves `back` to
  // the second: the two items now straddle the wrap.
  units.push_front(()).unwrap();
  units.push_back(()).unwrap();
  assert_eq!(Container::len(&units), 2);
  assert!(!Container::is_empty(&units));
  assert_eq!(units.pop_back(), Some(()));
  assert_eq!(Container::len(&units), 1);
  assert_eq!(units.pop_back(), Some(()));
  assert_eq!(Container::len(&units), 0);
  assert!(Container::is_empty(&units));
}

// --- tinyvec's `u16` length, over a backing array longer than it can count ---

/// A backing array one slot longer than a `u16` length can count.
#[cfg(feature = "tinyvec_1")]
const PAST_U16: usize = u16::MAX as usize + 1;

/// The fixed adapter keeps `u16::MAX` items over a `PAST_U16`-slot array, reports that as its
/// capacity, and refuses the next one before tinyvec's own push could write it and overflow the
/// length.
#[cfg(feature = "tinyvec_1")]
#[cfg_attr(
  miri,
  ignore = "65,536 pushes through safe code: a length bound, and miri interprets every push"
)]
#[test]
fn a_tinyvec_arrayvec_past_its_u16_length_refuses_the_next_item() {
  let held = u16::MAX as usize;
  let mut items = std::boxed::Box::new(tinyvec_1::ArrayVec::<[u8; PAST_U16]>::default());
  assert_eq!(Container::max_capacity(&*items), held);
  for i in 0..held {
    assert_eq!(Container::push(&mut *items, i as u8), Ok(()));
  }
  assert_eq!(Container::push(&mut *items, 0), Err(0));
  assert_eq!(Container::len(&*items), held);
  assert!(items.iter().enumerate().all(|(i, &item)| item == i as u8));
}

/// The growable adapter moves a full inline vector to the heap before the push that would pass its
/// `u16` length, so a `TinyVec` over the same array keeps all `PAST_U16` items, in order.
#[cfg(feature = "tinyvec_1")]
#[cfg_attr(
  miri,
  ignore = "65,536 pushes through safe code: a length bound, and miri interprets every push"
)]
#[test]
fn a_tinyvec_past_its_u16_length_moves_to_the_heap_and_keeps_every_item() {
  let mut items = std::boxed::Box::new(tinyvec_1::TinyVec::<[u8; PAST_U16]>::default());
  for i in 0..PAST_U16 {
    assert_eq!(Container::push(&mut *items, i as u8), Ok(()));
  }
  assert!(items.is_heap());
  assert_eq!(Container::len(&*items), PAST_U16);
  assert!(items.iter().enumerate().all(|(i, &item)| item == i as u8));
}
