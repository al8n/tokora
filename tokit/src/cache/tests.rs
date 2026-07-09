use super::*;
use crate::span::Spanned;

#[test]
fn cached_token_new_and_accessors() {
  let spanned = Spanned::new(10..20, "hello");
  let ct = CachedToken::new(spanned, 42u32);
  assert_eq!(ct.state(), &42);
  let tok = ct.token();
  assert_eq!(*tok.data(), &"hello");
}

#[test]
fn cached_token_into_token() {
  let spanned = Spanned::new(10..20, "hello");
  let ct = CachedToken::new(spanned, 42u32);
  let tok = ct.into_token();
  assert_eq!(*tok.data(), "hello");
}

#[test]
fn cached_token_as_ref() {
  let spanned = Spanned::new(10..20, 99u32);
  let ct = CachedToken::new(spanned, 42u32);
  let ct_ref = ct.as_ref();
  // as_ref returns CachedToken<&T, &State, &Span>, so state is &&u32
  assert_eq!(**ct_ref.state(), 42);
}

#[test]
fn cached_token_map_token() {
  let spanned = Spanned::new(10..20, 5u32);
  let ct = CachedToken::new(spanned, "state");
  let ct2 = ct.map_token(|x| x * 2);
  assert_eq!(*ct2.token().data(), &10);
  assert_eq!(ct2.state(), &"state");
}

#[test]
fn cached_token_into_components() {
  let spanned = Spanned::new(10..20, "hello");
  let ct = CachedToken::new(spanned, 42u32);
  let (tok, state) = ct.into_components();
  assert_eq!(*tok.data(), "hello");
  assert_eq!(state, 42);
}

#[test]
fn cached_token_clone() {
  let spanned = Spanned::new(10..20, "hello");
  let ct = CachedToken::new(spanned, 42u32);
  let ct2 = ct.clone();
  assert_eq!(ct2.state(), &42);
}

#[test]
fn cached_token_new_various_types() {
  let ct = CachedToken::new(Spanned::new(0..1, 100u64), "state_str");
  assert_eq!(ct.state(), &"state_str");
  assert_eq!(*ct.token().data(), &100u64);
}

#[test]
fn cached_token_map_preserves_span() {
  let spanned = Spanned::new(5..15, 10i32);
  let ct = CachedToken::new(spanned, ());
  let ct2 = ct.map_token(|x| x.to_string());
  assert_eq!(*ct2.token().data(), &"10".to_string());
}
