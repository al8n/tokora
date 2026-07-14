#![cfg(feature = "hipstr_0_8")]

//! Regression test for the `parse_hipstr` feature gate.
//!
//! `Parse::parse_hipstr` / `Parse::parse_hipstr_with_state` must be compiled in
//! whenever the canonical `hipstr_0_8` feature is enabled, not only via the
//! `hipstr` alias — mirroring the `parse_bytes` gate proof in `bytes_gate.rs`.
//!
//! This is a compile-time gate proof: `uses_parse_hipstr` only type-checks when
//! both driver methods exist under `hipstr_0_8` and route — via `HipStr::as_str`
//! — to a `Source = str` lexer.

use tokora::{Lexer, Parse};

#[allow(dead_code)]
fn uses_parse_hipstr<'inp, P, L, O, E, Lang>(
  p: P,
  q: P,
  src: &'inp hipstr_0_8::HipStr<'inp>,
  state: L::State,
) -> (Result<O, E>, Result<O, E>)
where
  P: Parse<'inp, L, O, E, Lang>,
  L: Lexer<'inp, Source = str>,
  L::State: Default,
  Lang: ?Sized,
{
  (p.parse_hipstr(src), q.parse_hipstr_with_state(src, state))
}

#[test]
fn parse_hipstr_available_under_hipstr_0_8() {
  // The gate is proven by `uses_parse_hipstr` above type-checking. Exercise the
  // `hipstr_0_8` dependency at runtime so this is a live target as well.
  let h = hipstr_0_8::HipStr::borrowed("abc");
  assert_eq!(h.len(), 3);
}
