use super::*;
use crate::cache::DefaultCache;
use crate::emitter::Fatal;
use crate::lexer::{DummyLexer, DummyToken};
use crate::parse_context::FatalContext;

#[test]
fn input_context_new_and_into_components() {
  let ctx = InputContext::new("emitter", 42u32);
  let (e, c) = ctx.into_components();
  assert_eq!(e, "emitter");
  assert_eq!(c, 42u32);
}

#[test]
fn input_context_different_types() {
  let ctx = InputContext::new(std::vec![1, 2, 3], Some("cache"));
  let (e, c) = ctx.into_components();
  assert_eq!(e, std::vec![1, 2, 3]);
  assert_eq!(c, Some("cache"));
}

#[test]
fn input_new_creates_input() {
  let input = Input::<'_, DummyLexer, FatalContext<'_, DummyLexer, ()>>::new("");
  // Just verify it compiles and can be created
  let _ = input;
}

#[test]
fn input_with_state_creates_input() {
  let input = Input::<'_, DummyLexer, FatalContext<'_, DummyLexer, ()>>::with_state("hello", ());
  let _ = input;
}

#[test]
fn input_with_state_and_cache() {
  let cache = DefaultCache::<'_, DummyLexer>::default();
  let input = Input::<'_, DummyLexer, FatalContext<'_, DummyLexer, ()>>::with_state_and_cache(
    "hello",
    (),
    cache,
  );
  let _ = input;
}

#[test]
fn input_clone() {
  let input = Input::<'_, DummyLexer, FatalContext<'_, DummyLexer, ()>>::new("hello");
  let cloned = input.clone();
  let _ = cloned;
}

#[test]
fn input_as_ref() {
  let mut input = Input::<'_, DummyLexer, FatalContext<'_, DummyLexer, ()>>::new("hello");
  let mut emitter = Fatal::<()>::new();
  let input_ref = input.as_ref(&mut emitter);
  let _ = input_ref;
}
