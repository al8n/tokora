use super::*;
use crate::lexer::{DummyLexer, DummyToken};
use crate::span::{SimpleSpan, Span, Spanned};
use ::generic_arraydeque::typenum::U3;

type DequeCache = GenericArrayDeque<CachedToken<DummyToken, (), SimpleSpan>, U3>;
type OptionCache = Option<CachedToken<DummyToken, (), SimpleSpan>>;

fn make_token(start: usize, end: usize) -> CachedToken<DummyToken, (), SimpleSpan> {
  CachedToken::new(Spanned::new(SimpleSpan::new(start, end), DummyToken), ())
}

// ── Cache::span ─────────────────────────────────────────────────────────

#[test]
fn deque_cache_span_empty() {
  let cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  assert!(<DequeCache as Cache<'_, DummyLexer>>::span(&cache).is_none());
}

#[test]
fn deque_cache_span_single() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  let span = <DequeCache as Cache<'_, DummyLexer>>::span(&cache).unwrap();
  assert_eq!(span.start(), 5);
  assert_eq!(span.end(), 10);
}

#[test]
fn deque_cache_span_multiple() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 20));
  let span = <DequeCache as Cache<'_, DummyLexer>>::span(&cache).unwrap();
  assert_eq!(span.start(), 5);
  assert_eq!(span.end(), 20);
}

#[test]
fn option_cache_span_empty() {
  let cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  assert!(<OptionCache as Cache<'_, DummyLexer>>::span(&cache).is_none());
}

#[test]
fn option_cache_span_with_token() {
  let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(3, 7));
  let span = <OptionCache as Cache<'_, DummyLexer>>::span(&cache).unwrap();
  assert_eq!(span.start(), 3);
  assert_eq!(span.end(), 7);
}

// ── Cache::front_span / back_span ────────────────────────────────────────

#[test]
fn deque_cache_front_span_and_back_span() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  assert!(<DequeCache as Cache<'_, DummyLexer>>::front_span(&cache).is_none());
  assert!(<DequeCache as Cache<'_, DummyLexer>>::back_span(&cache).is_none());
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 15));
  let front = <DequeCache as Cache<'_, DummyLexer>>::front_span(&cache).unwrap();
  let back = <DequeCache as Cache<'_, DummyLexer>>::back_span(&cache).unwrap();
  assert_eq!(front.start(), 0);
  assert_eq!(front.end(), 5);
  assert_eq!(back.start(), 5);
  assert_eq!(back.end(), 15);
}

#[test]
fn option_cache_front_span_and_back_span() {
  let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  assert!(<OptionCache as Cache<'_, DummyLexer>>::front_span(&cache).is_none());
  assert!(<OptionCache as Cache<'_, DummyLexer>>::back_span(&cache).is_none());
  let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(2, 8));
  let front = <OptionCache as Cache<'_, DummyLexer>>::front_span(&cache).unwrap();
  let back = <OptionCache as Cache<'_, DummyLexer>>::back_span(&cache).unwrap();
  assert_eq!(front.start(), 2);
  assert_eq!(back.start(), 2);
}

// ── Cache::peek_one ──────────────────────────────────────────────────────

#[test]
fn deque_cache_peek_one_empty() {
  let cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  assert!(<DequeCache as Cache<'_, DummyLexer>>::peek_one(&cache).is_none());
}

#[test]
fn deque_cache_peek_one_with_token() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  assert!(<DequeCache as Cache<'_, DummyLexer>>::peek_one(&cache).is_some());
  // peek_one does not consume
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
}

#[test]
fn option_cache_peek_one_empty() {
  let cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  assert!(<OptionCache as Cache<'_, DummyLexer>>::peek_one(&cache).is_none());
}

#[test]
fn option_cache_peek_one_with_token() {
  let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  assert!(<OptionCache as Cache<'_, DummyLexer>>::peek_one(&cache).is_some());
}

// ── Cache::try_pop_front_if ──────────────────────────────────────────────

#[test]
fn deque_cache_try_pop_front_if_ok() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let result =
    <DequeCache as Cache<'_, DummyLexer>>::try_pop_front_if::<(), _>(&mut cache, |_| Ok(()));
  assert!(matches!(result, Some(Ok(_))));
  assert!(<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
}

#[test]
fn deque_cache_try_pop_front_if_err() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let result = <DequeCache as Cache<'_, DummyLexer>>::try_pop_front_if(&mut cache, |_| Err("fail"));
  assert!(matches!(result, Some(Err("fail"))));
  // Token is NOT consumed on error
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
}

#[test]
fn deque_cache_try_pop_front_if_empty() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let result =
    <DequeCache as Cache<'_, DummyLexer>>::try_pop_front_if::<(), _>(&mut cache, |_| Ok(()));
  assert!(result.is_none());
}

#[test]
fn option_cache_try_pop_front_if_ok() {
  let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let result =
    <OptionCache as Cache<'_, DummyLexer>>::try_pop_front_if::<(), _>(&mut cache, |_| Ok(()));
  assert!(matches!(result, Some(Ok(_))));
}

#[test]
fn option_cache_try_pop_front_if_err() {
  let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let result = <OptionCache as Cache<'_, DummyLexer>>::try_pop_front_if(&mut cache, |_| Err(42));
  assert!(matches!(result, Some(Err(42))));
}

// ── Cache::push_many ──────────────────────────────────────────────────────

#[test]
fn deque_cache_push_many_all_fit() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let tokens = vec![make_token(0, 5), make_token(5, 10)];
  let overflow: Vec<_> =
    <DequeCache as Cache<'_, DummyLexer>>::push_many(&mut cache, tokens.into_iter()).collect();
  assert!(overflow.is_empty());
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 2);
}

#[test]
fn deque_cache_push_many_overflow() {
  let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
  let tokens = vec![
    make_token(0, 5),
    make_token(5, 10),
    make_token(10, 15),
    make_token(15, 20),
  ];
  let overflow: Vec<_> =
    <DequeCache as Cache<'_, DummyLexer>>::push_many(&mut cache, tokens.into_iter()).collect();
  assert_eq!(overflow.len(), 1);
  assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 3);
}

#[test]
fn option_cache_push_many_overflow() {
  let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  let tokens = vec![make_token(0, 5), make_token(5, 10)];
  let overflow: Vec<_> =
    <OptionCache as Cache<'_, DummyLexer>>::push_many(&mut cache, tokens.into_iter()).collect();
  assert_eq!(overflow.len(), 1);
  assert_eq!(<OptionCache as Cache<'_, DummyLexer>>::len(&cache), 1);
}

// ── Cache::pop_front_if (default impl) for Option cache ──────────────────

#[test]
fn option_cache_pop_front_if_match() {
  let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let popped = <OptionCache as Cache<'_, DummyLexer>>::pop_front_if(&mut cache, |_| true);
  assert!(popped.is_some());
  assert!(<OptionCache as Cache<'_, DummyLexer>>::is_empty(&cache));
}

#[test]
fn option_cache_pop_front_if_no_match() {
  let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
  let popped = <OptionCache as Cache<'_, DummyLexer>>::pop_front_if(&mut cache, |_| false);
  assert!(popped.is_none());
  assert_eq!(<OptionCache as Cache<'_, DummyLexer>>::len(&cache), 1);
}

#[test]
fn option_cache_pop_front_if_empty() {
  let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
  let popped = <OptionCache as Cache<'_, DummyLexer>>::pop_front_if(&mut cache, |_| true);
  assert!(popped.is_none());
}
