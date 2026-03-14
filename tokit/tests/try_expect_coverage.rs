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

// ── try_expect_<punct> methods ──────────────────────────────────────────────

#[test]
fn try_expect_comma_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_comma()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str(",");
  assert!(r.unwrap());
}

#[test]
fn try_expect_comma_decline() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_comma()?.is_none())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("42");
  assert!(r.unwrap());
}

#[test]
fn try_expect_semicolon() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_semicolon()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str(";");
  assert!(r.unwrap());
}

#[test]
fn try_expect_open_paren() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_open_paren()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("(");
  assert!(r.unwrap());
}

#[test]
fn try_expect_close_paren() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_close_paren()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str(")");
  assert!(r.unwrap());
}

#[test]
fn try_expect_open_bracket() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_open_bracket()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("[");
  assert!(r.unwrap());
}

#[test]
fn try_expect_close_bracket() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_close_bracket()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("]");
  assert!(r.unwrap());
}

#[test]
fn try_expect_open_brace() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_open_brace()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("{");
  assert!(r.unwrap());
}

#[test]
fn try_expect_close_brace() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    Ok(inp.try_expect_close_brace()?.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("}");
  assert!(r.unwrap());
}

// ── try_expect with empty cache ─────────────────────────────────────────────

#[test]
fn try_expect_empty_cache_match() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    let tok = inp.try_expect(|t| matches!(t.data(), Token::Num(_)))?;
    Ok(tok.is_some())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("42");
  assert!(r.unwrap());
}

#[test]
fn try_expect_empty_cache_no_match() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    let tok = inp.try_expect(|t| matches!(t.data(), Token::Comma))?;
    Ok(tok.is_none())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str("42");
  assert!(r.unwrap());
}

// ── try_expect_map with empty cache ─────────────────────────────────────────

#[test]
fn try_expect_map_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<Option<i64>, E> {
    let result = inp.try_expect_map(|t| match t.data() {
      Token::Num(n) => Some(*n),
      _ => None,
    })?;
    Ok(result.map(|(n, _)| n))
  }
  let r: Result<Option<i64>, _> = Parser::with_context(ctx()).apply(parse).parse_str("42");
  assert_eq!(r.unwrap(), Some(42));
}

#[test]
fn try_expect_map_decline() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    let result = inp.try_expect_map::<i64, _>(|t| match t.data() {
      Token::Num(n) => Some(*n),
      _ => None,
    })?;
    Ok(result.is_none())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str(",");
  assert!(r.unwrap());
}

// ── try_expect_and_then ─────────────────────────────────────────────────────

#[test]
fn try_expect_and_then_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<Option<i64>, E> {
    let result = inp.try_expect_and_then(|t| match t.data() {
      Token::Num(n) => Some(Ok(*n)),
      _ => None,
    })?;
    Ok(result.map(|(n, _)| n))
  }
  let r: Result<Option<i64>, _> = Parser::with_context(ctx()).apply(parse).parse_str("42");
  assert_eq!(r.unwrap(), Some(42));
}

#[test]
fn try_expect_and_then_decline() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    let result = inp.try_expect_and_then::<i64, _>(|t| match t.data() {
      Token::Num(n) => Some(Ok(*n)),
      _ => None,
    })?;
    Ok(result.is_none())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str(",");
  assert!(r.unwrap());
}

#[test]
fn try_expect_and_then_cached_decline() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<bool, E> {
    let _ = inp.peek_one()?;
    let result = inp.try_expect_and_then::<i64, _>(|t| match t.data() {
      Token::Num(n) => Some(Ok(*n)),
      _ => None,
    })?;
    Ok(result.is_none())
  }
  let r: Result<bool, _> = Parser::with_context(ctx()).apply(parse).parse_str(",");
  assert!(r.unwrap());
}

#[test]
fn try_expect_and_then_cached_success() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
  ) -> Result<Option<i64>, E> {
    let _ = inp.peek_one()?;
    let result = inp.try_expect_and_then(|t| match t.data() {
      Token::Num(n) => Some(Ok(*n)),
      _ => None,
    })?;
    Ok(result.map(|(n, _)| n))
  }
  let r: Result<Option<i64>, _> = Parser::with_context(ctx()).apply(parse).parse_str("42");
  assert_eq!(r.unwrap(), Some(42));
}
