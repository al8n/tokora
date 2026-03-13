#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use tokit::{
  Emitter, InputRef, Lexer, Parse, Parser, ParserContext, Token as TokenTrait,
  error::{
    UnexpectedEot,
    token::{UnexpectedToken, UnexpectedTokenOf},
  },
  input::Cursor,
  span::Spanned,
};

use common::{TestLexer, Token};

#[derive(Debug)]
struct E;
impl From<()> for E {
  fn from(_: ()) -> Self {
    E
  }
}
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    E
  }
}
impl From<UnexpectedEot> for E {
  fn from(_: UnexpectedEot) -> Self {
    E
  }
}

struct TestEm;
impl<'inp> Emitter<'inp, TestLexer<'inp>> for TestEm {
  type Error = E;
  fn emit_lexer_error(
    &mut self,
    _: Spanned<
      <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
      <TestLexer<'inp> as Lexer<'inp>>::Span,
    >,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
  fn emit_unexpected_token(&mut self, _: UnexpectedTokenOf<'inp, TestLexer<'inp>>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
  fn emit_error(&mut self, err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(err.into_data())
  }
  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>)
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
  }
}

fn ctx() -> ParserContext<'static, TestLexer<'static>, TestEm> {
  ParserContext::new(TestEm)
}

// ── Alias try_expect methods ─────────────────────────────────────────────────

#[test]
fn try_expect_minus_alias_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    // try_expect_minus is an alias for try_expect_hyphen
    Ok(inp.try_expect_minus()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("-");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_minus_alias_decline() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_minus()?.is_none())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("42");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_hyphen_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_hyphen()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("-");
  assert_eq!(r.unwrap(), true);
}

// ── Alias expect methods ─────────────────────────────────────────────────────

#[test]
fn expect_minus_alias_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    inp.expect_minus()?;
    Ok(true)
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("-");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn expect_minus_alias_eot() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    inp.expect_minus()?;
    Ok(true)
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("");
  assert!(r.is_err());
}

#[test]
fn expect_hyphen_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    inp.expect_hyphen()?;
    Ok(true)
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("-");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn expect_hyphen_wrong_token() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    inp.expect_hyphen()?;
    Ok(true)
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("42");
  assert!(r.is_err());
}

// ── Additional non-alias expect methods ──────────────────────────────────────

#[test]
fn expect_asterisk_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    inp.expect_asterisk()?;
    Ok(true)
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("*");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_asterisk_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_asterisk()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("*");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_plus_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_plus()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("+");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_slash_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_slash()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("/");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn try_expect_equal_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_equal()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("=");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn expect_plus_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    inp.expect_plus()?;
    Ok(true)
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("+");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn expect_slash_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    inp.expect_slash()?;
    Ok(true)
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("/");
  assert_eq!(r.unwrap(), true);
}

#[test]
fn expect_equal_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    inp.expect_equal()?;
    Ok(true)
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("=");
  assert_eq!(r.unwrap(), true);
}
