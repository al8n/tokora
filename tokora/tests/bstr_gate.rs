#![cfg(feature = "bstr_1")]

//! Regression test for the `parse_bstr` feature gate.
//!
//! `Parse::parse_bstr` / `Parse::parse_bstr_with_state` must be compiled in
//! whenever the canonical `bstr_1` feature is enabled, not only via the `bstr`
//! alias — mirroring the `parse_bytes` gate proof in `bytes_gate.rs`.
//!
//! This is a compile-time gate proof: `uses_parse_bstr` only type-checks when
//! both driver methods exist under `bstr_1` and route to a `Source = [u8]` lexer.

use tokora::{Lexer, Parse};

#[allow(dead_code)]
fn uses_parse_bstr<'inp, P, L, O, E, Lang>(
  p: P,
  q: P,
  src: &'inp bstr_1::BStr,
  state: L::State,
) -> (Result<O, E>, Result<O, E>)
where
  P: Parse<'inp, L, O, E, Lang>,
  L: Lexer<'inp, Source = [u8]>,
  L::State: Default,
  Lang: ?Sized,
{
  (p.parse_bstr(src), q.parse_bstr_with_state(src, state))
}

#[test]
fn parse_bstr_available_under_bstr_1() {
  // The gate is proven by `uses_parse_bstr` above type-checking. Exercise the
  // `bstr_1` dependency at runtime so this is a live target as well.
  let b = bstr_1::BStr::new(b"abc");
  assert_eq!(b.len(), 3);
}
