use crate::lexer::{DummyLexer, DummyToken};

use super::*;

fn assert_filter_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::new().apply(Any::new().filter(|_tok: &DummyToken| Ok(())))
}

fn assert_filter_parse_with_ctx_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::with_context(()).apply(Any::new().filter(|_tok: &DummyToken| Ok(())))
}

#[test]
fn assert_parse_impl() {
  let _ = assert_filter_parse_impl();
  let _ = assert_filter_parse_with_ctx_impl();
}
