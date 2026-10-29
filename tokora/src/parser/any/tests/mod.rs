#[cfg(feature = "map")]
use crate::lexer::{DummyLexer, DummyToken};

#[cfg(feature = "map")]
use super::*;

#[cfg(all(feature = "std", feature = "logos_0_16"))]
mod slice;

// Both cells run `Any::spanned().map(..)`, so they need the `map` family too.
#[cfg(feature = "map")]
fn assert_any_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::new().apply(Any::spanned().map(Spanned::into_data))
}

#[cfg(feature = "map")]
fn assert_any_parse_with_context_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::with_context(()).apply(Any::new().spanned().map(Spanned::into_data))
}

#[cfg(feature = "map")]
#[test]
fn assert_parse_impl() {
  let _ = assert_any_parse_impl();
  let _ = assert_any_parse_with_context_impl();
}
