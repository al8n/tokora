use crate::{
  lexer::{DummyLexer, DummyToken},
  parser::{Any, Parse, Parser},
};

use super::*;

fn assert_dispatch_on_kind_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::new().apply((Any::new(),).dispatch_on_kind(&[DummyToken]))
}

#[test]
fn assert_parse_impl() {
  let _ = assert_dispatch_on_kind_parse_impl();
}
