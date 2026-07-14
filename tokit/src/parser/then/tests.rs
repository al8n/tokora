use crate::lexer::{DummyLexer, DummyToken};

use super::*;

fn assert_ignore_then_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::new().apply(Any::new().ignore_then(Any::new()))
}

fn assert_then_ignore_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::new().apply(Any::new().then_ignore(Any::new()))
}

#[test]
fn assert_parse_impl() {
  let _ = assert_ignore_then_parse_impl();
  let _ = assert_then_ignore_parse_impl();
}
