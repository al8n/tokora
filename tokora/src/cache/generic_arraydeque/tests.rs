use super::*;
use crate::lexer::{DummyLexer, DummyToken};
use crate::span::{SimpleSpan, Spanned};
use generic_arraydeque::typenum::U3;

type DequeCache = GenericArrayDeque<CachedToken<DummyToken, (), SimpleSpan>, U3>;

fn make_token(start: usize, end: usize) -> CachedToken<DummyToken, (), SimpleSpan> {
  CachedToken::new(Spanned::new(SimpleSpan::new(start, end), DummyToken), ())
}

#[test]
fn deque_cache_new_is_empty() {
  let cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 0);
  assert!(<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
}

#[test]
fn deque_cache_with_options_is_empty() {
  let cache = <DequeCache as Cache<'_, DummyLexer>>::with_options(());
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 0);
}

#[test]
fn deque_cache_push_back_and_len() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let tok = make_token(0, 5);
  let result = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, tok);
  assert!(result.is_ok());
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
  assert!(!<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
}

#[test]
fn deque_cache_push_back_multiple() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 3);
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 0);
}

#[test]
fn deque_cache_push_back_when_full_returns_err() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
  let result = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(15, 20));
  assert!(result.is_err());
}

#[test]
fn deque_cache_push_front_and_len() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let tok = make_token(0, 5);
  let result = <DequeCache as Cache<'_, DummyLexer>>::push_front(&mut cache, tok);
  assert!(result.is_ok());
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
}

#[test]
fn deque_cache_pop_front() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  assert!(<DequeCache as Cache<'_, DummyLexer>>::pop_front(&mut cache).is_none());
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  let popped = <DequeCache as Cache<'_, DummyLexer>>::pop_front(&mut cache);
  assert!(popped.is_some());
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
}

#[test]
fn deque_cache_pop_back() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  assert!(<DequeCache as Cache<'_, DummyLexer>>::pop_back(&mut cache).is_none());
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let popped = <DequeCache as Cache<'_, DummyLexer>>::pop_back(&mut cache);
  assert!(popped.is_some());
  assert!(<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
}

#[test]
fn deque_cache_pop_front_if_match() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let popped = <DequeCache as Cache<'_, DummyLexer>>::pop_front_if(&mut cache, |_| true);
  assert!(popped.is_some());
  assert!(<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
}

#[test]
fn deque_cache_pop_front_if_no_match() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let popped = <DequeCache as Cache<'_, DummyLexer>>::pop_front_if(&mut cache, |_| false);
  assert!(popped.is_none());
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
}

#[test]
fn deque_cache_clear() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  <DequeCache as Cache<'_, DummyLexer>>::clear(&mut cache);
  assert!(<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
}

#[test]
fn deque_cache_front_and_back() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  assert!(<DequeCache as Cache<'_, DummyLexer>>::front(&cache).is_none());
  assert!(<DequeCache as Cache<'_, DummyLexer>>::back(&cache).is_none());
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  let front = <DequeCache as Cache<'_, DummyLexer>>::front(&cache);
  let back = <DequeCache as Cache<'_, DummyLexer>>::back(&cache);
  assert!(front.is_some());
  assert!(back.is_some());
}

#[test]
fn deque_cache_remaining() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 3);
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 2);
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 1);
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 0);
}

#[test]
fn deque_cache_peek_empty() {
  use generic_arraydeque::typenum::U2;
  let cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let mut buf = GenericArrayDeque::new();
  <DequeCache as Cache<'_, DummyLexer>>::peek::<U2>(&cache, &mut buf);
  assert!(buf.is_empty());
}

#[test]
fn deque_cache_peek_with_tokens() {
  use generic_arraydeque::typenum::U2;
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  let mut buf = GenericArrayDeque::new();
  <DequeCache as Cache<'_, DummyLexer>>::peek::<U2>(&cache, &mut buf);
  assert_eq!(buf.len(), 2);
}

#[test]
fn deque_cache_peek_capped_by_buffer() {
  use generic_arraydeque::typenum::U1;
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
  let mut buf = GenericArrayDeque::new();
  // Buffer capacity is 1, should only get 1 token even though cache has 3
  <DequeCache as Cache<'_, DummyLexer>>::peek::<U1>(&cache, &mut buf);
  assert_eq!(buf.len(), 1);
}

#[test]
fn deque_cache_push_front_when_full_returns_err() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
  let result = <DequeCache as Cache<'_, DummyLexer>>::push_front(&mut cache, make_token(15, 20));
  assert!(result.is_err());
}
