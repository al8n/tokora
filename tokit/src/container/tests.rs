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

// --- GenericArrayDeque tests ---

#[test]
fn generic_arraydeque_push_and_accessors() {
  use generic_arraydeque::typenum::U4;
  let mut c: GenericArrayDeque<i32, U4> = GenericArrayDeque::new();
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
fn generic_arraydeque_push_overflow() {
  use generic_arraydeque::typenum::U2;
  let mut c: GenericArrayDeque<i32, U2> = GenericArrayDeque::new();
  assert!(Container::push(&mut c, 1).is_ok());
  assert!(Container::push(&mut c, 2).is_ok());
  assert_eq!(Container::push(&mut c, 3), Err(3));
}

// --- Ignored blackhole tests ---

#[test]
fn ignored_push_discards() {
  let mut c = crate::utils::marker::Ignored::<i32>::default();
  assert!(Container::<i32>::push(&mut c, 42).is_ok());
  assert_eq!(Container::<i32>::len(&c), 0);
  assert!(Container::<i32>::is_empty(&c));
  assert!(Container::<i32>::first(&c).is_none());
  assert!(Container::<i32>::last(&c).is_none());
  assert_eq!(Container::<i32>::max_capacity(&c), usize::MAX);
}
