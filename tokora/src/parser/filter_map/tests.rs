use crate::lexer::{DummyLexer, DummyToken};

use super::*;

fn assert_filter_map_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
  Parser::new().apply(Any::new().filter_map(|_tok: DummyToken| Ok(())))
}

fn assert_filter_map_parse_with_ctx_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
  Parser::with_context(()).apply(Any::new().filter_map(|_tok: DummyToken| Ok(())))
}

#[test]
fn assert_parse_impl() {
  let _ = assert_filter_map_parse_impl();
  let _ = assert_filter_map_parse_with_ctx_impl();
}
