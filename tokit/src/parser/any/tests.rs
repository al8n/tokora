use crate::lexer::{DummyLexer, DummyToken};

use super::*;

fn assert_any_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::new().apply(Any::spanned().map(Spanned::into_data))
}

fn assert_any_parse_with_context_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::with_context(()).apply(Any::new().spanned().map(Spanned::into_data))
}

#[test]
fn assert_parse_impl() {
  let _ = assert_any_parse_impl();
  let _ = assert_any_parse_with_context_impl();
}
