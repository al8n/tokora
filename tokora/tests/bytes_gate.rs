#![cfg(feature = "bytes_1")]

//! Regression test for the `parse_bytes` feature gate.
//!
//! `Parse::parse_bytes` / `Parse::parse_bytes_with_state` must be compiled in
//! whenever the canonical `bytes_1` feature is enabled, not only via the `bytes`
//! alias. Previously they were `#[cfg(feature = "bytes")]`, so a consumer that
//! enabled `bytes_1` directly (as a consumer enabling only the versioned feature does)
//! got `Bytes: Source` but the drivers were compiled out.
//!
//! This is a compile-time gate proof: `uses_parse_bytes` only type-checks when
//! both driver methods exist under `bytes_1`.

use tokora::{Lexer, Parse};

#[allow(dead_code)]
fn uses_parse_bytes<'inp, P, L, O, E, Lang>(
  p: P,
  q: P,
  src: &'inp bytes_1::Bytes,
  state: L::State,
) -> (Result<O, E>, Result<O, E>)
where
  P: Parse<'inp, L, O, E, Lang>,
  L: Lexer<'inp, Source = [u8]>,
  L::State: Default,
  Lang: ?Sized,
{
  (p.parse_bytes(src), q.parse_bytes_with_state(src, state))
}

#[test]
fn parse_bytes_available_under_bytes_1() {
  // The gate is proven by `uses_parse_bytes` above type-checking. Exercise the
  // `bytes_1` dependency at runtime so this is a live target as well.
  let b = bytes_1::Bytes::from_static(b"abc");
  assert_eq!(b.len(), 3);
}
