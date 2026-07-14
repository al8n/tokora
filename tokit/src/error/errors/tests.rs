use super::*;
use generic_arraydeque::ConstGenericArrayDeque;

#[test]
fn test_new() {
  let _: Errors<&str> = Errors::new();
}

#[test]
fn test_push_and_len() {
  let mut errors = Errors::new();
  errors.push("Error 1");
  assert_eq!(errors.len(), 1);
  errors.push("Error 2");
  assert_eq!(errors.len(), 2);
}

#[test]
fn test_clear() {
  let mut errors = Errors::new();
  errors.push("Error");
  errors.clear();
  assert!(errors.is_empty());
}

#[test]
fn test_iteration() {
  let mut errors = Errors::new();
  errors.push(1);
  errors.push(2);

  let sum: i32 = errors.iter().sum();
  assert_eq!(sum, 3);
}

#[test]
fn test_overflow_tracking() {
  type SmallErrors<'a> = Errors<&'a str, ConstGenericArrayDeque<&'a str, 1>>;
  let mut errors: SmallErrors<'_> = Errors::from_container(ConstGenericArrayDeque::<_, 1>::new());

  assert!(!errors.overflowed());
  errors.push("first");
  assert_eq!(errors.len(), 1);
  assert_eq!(errors.remaining_capacity(), Some(0));
  assert!(errors.is_full());

  errors.push("second");
  assert!(errors.overflowed());
  assert_eq!(errors.len(), 1);
}

#[test]
fn test_try_push_reports_error() {
  type SmallErrors<'a> = Errors<&'a str, ConstGenericArrayDeque<&'a str, 1>>;
  let mut errors: SmallErrors<'_> = Errors::from_container(ConstGenericArrayDeque::<_, 1>::new());

  assert!(errors.try_push("first").is_ok());
  assert!(errors.try_push("second").is_err());
  assert!(errors.overflowed());
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[test]
fn test_with_capacity() {
  let errors: Errors<&str> = Errors::with_capacity(10);
  assert_eq!(errors.capacity(), 10);
  assert!(errors.is_empty());
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[test]
fn test_pop() {
  use crate::error::ErrorContainer;

  let mut errors = Errors::new();
  errors.push(1);
  errors.push(2);

  assert_eq!(errors.pop(), Some(1));
  assert_eq!(errors.pop(), Some(2));
  assert_eq!(errors.pop(), None);
}
