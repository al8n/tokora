use crate::lexer::{DummyLexer, DummyToken};

use super::*;

fn assert_map_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
  Parser::new().apply(Any::new().map(|_tok: DummyToken| ()))
}

fn assert_map_parse_with_ctx_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
  Parser::with_context(()).apply(Any::new().map(|_tok: DummyToken| ()))
}

#[test]
fn assert_parse_impl() {
  let _ = assert_map_parse_impl();
  let _ = assert_map_parse_with_ctx_impl();
}
