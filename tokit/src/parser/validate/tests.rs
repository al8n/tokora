use crate::lexer::{DummyLexer, DummyToken};

use super::*;

fn assert_validate_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Spanned<DummyToken>, ()> {
  Parser::new().apply(
    Any::new()
      .spanned()
      .validate(|_tok: &Spanned<DummyToken>| Ok(())),
  )
}

fn assert_validate_parse_with_ctx_impl<'inp>()
-> impl Parse<'inp, DummyLexer, Spanned<DummyToken>, ()> {
  Parser::with_context(()).apply(
    Any::new()
      .spanned()
      .validate(|_tok: &Spanned<DummyToken>| Ok(())),
  )
}

fn assert_validate_with_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Spanned<DummyToken>, ()>
{
  Parser::new().apply(
    Any::new()
      .spanned()
      .validate_with(|_tok: &Spanned<DummyToken>, _| Ok(())),
  )
}

fn assert_validate_with_parse_with_ctx_impl<'inp>()
-> impl Parse<'inp, DummyLexer, Spanned<DummyToken>, ()> {
  Parser::with_context(()).apply(
    Any::new()
      .spanned()
      .validate_with(|_tok: &Spanned<DummyToken>, _| Ok(())),
  )
}

#[test]
fn assert_parse_impl() {
  let _ = assert_validate_parse_impl();
  let _ = assert_validate_parse_with_ctx_impl();
  let _ = assert_validate_with_parse_impl();
  let _ = assert_validate_with_parse_with_ctx_impl();
}
