use super::*;

#[test]
fn unit_cache_len_is_zero() {
  let cache: () = ();
  assert_eq!(<() as Cache<'_, crate::lexer::DummyLexer>>::len(&cache), 0);
}

#[test]
fn unit_cache_remaining_is_zero() {
  let cache: () = ();
  assert_eq!(
    <() as Cache<'_, crate::lexer::DummyLexer>>::remaining(&cache),
    0
  );
}

#[test]
fn unit_cache_front_is_none() {
  let cache: () = ();
  assert!(<() as Cache<'_, crate::lexer::DummyLexer>>::front(&cache).is_none());
}

#[test]
fn unit_cache_back_is_none() {
  let cache: () = ();
  assert!(<() as Cache<'_, crate::lexer::DummyLexer>>::back(&cache).is_none());
}

#[test]
fn unit_cache_pop_front_is_none() {
  let mut cache: () = ();
  assert!(<() as Cache<'_, crate::lexer::DummyLexer>>::pop_front(&mut cache).is_none());
}

#[test]
fn unit_cache_pop_back_is_none() {
  let mut cache: () = ();
  assert!(<() as Cache<'_, crate::lexer::DummyLexer>>::pop_back(&mut cache).is_none());
}

#[test]
fn unit_cache_clear_is_noop() {
  let mut cache: () = ();
  <() as Cache<'_, crate::lexer::DummyLexer>>::clear(&mut cache);
}

#[test]
fn unit_cache_new() {
  <() as Cache<'_, crate::lexer::DummyLexer>>::new();
  assert_eq!((), ());
}

#[test]
fn unit_cache_with_options() {
  <() as Cache<'_, crate::lexer::DummyLexer>>::with_options(());
  assert_eq!((), ());
}

#[test]
fn unit_cache_push_front_returns_err() {
  use crate::cache::CachedToken;
  use crate::lexer::DummyToken;
  use crate::span::{SimpleSpan, Spanned};
  let mut cache: () = ();
  let tok = CachedToken::new(Spanned::new(SimpleSpan::new(0, 5), DummyToken), ());
  let result = <() as Cache<'_, crate::lexer::DummyLexer>>::push_front(&mut cache, tok);
  assert!(result.is_err());
}

#[test]
fn unit_cache_push_back_returns_err() {
  use crate::cache::CachedToken;
  use crate::lexer::DummyToken;
  use crate::span::{SimpleSpan, Spanned};
  let mut cache: () = ();
  let tok = CachedToken::new(Spanned::new(SimpleSpan::new(0, 5), DummyToken), ());
  let result = <() as Cache<'_, crate::lexer::DummyLexer>>::push_back(&mut cache, tok);
  assert!(result.is_err());
}

#[test]
fn unit_cache_peek_is_noop() {
  use generic_arraydeque::typenum::U1;
  let cache: () = ();
  let mut buf = GenericArrayDeque::new();
  <() as Cache<'_, crate::lexer::DummyLexer>>::peek::<U1>(&cache, &mut buf);
  assert!(buf.is_empty());
}
