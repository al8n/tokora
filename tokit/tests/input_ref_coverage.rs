#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use common::{TestLexer, Token, TokenKind};
use tokit::{
  Emitter, InputRef, Parse, ParseContext, Parser, ParserContext, cache::DefaultCache,
  emitter::Ignored, utils::Expected,
};

/// Type alias for an Ignored-emitter context (swallows unexpected token errors).
type IgnoredContext<'inp> =
  ParserContext<'inp, TestLexer<'inp>, Ignored, DefaultCache<'inp, TestLexer<'inp>>>;

/// Helper macro to create a parser with the Ignored emitter.
macro_rules! ignored_parser {
  () => {
    Parser::with_context(IgnoredContext::new(Ignored::default()))
  };
}

// ── consume_cached tests ────────────────────────────────────────────────────

#[test]
fn consume_cached_one_empty() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Option<i64>, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    let tok = inp.consume_cached_one();
    Ok(tok.map(|t| match t.into_data() {
      Token::Num(n) => n,
      _ => -1,
    }))
  }

  let r = ignored_parser!().apply(parse).parse_str("42");
  assert_eq!(r.unwrap(), None);
}

#[test]
fn consume_cached_one_after_peek() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Option<i64>, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    let _ = inp.peek_one()?;
    let tok = inp.consume_cached_one();
    Ok(tok.map(|t| match t.into_data() {
      Token::Num(n) => n,
      _ => -1,
    }))
  }

  let r = ignored_parser!().apply(parse).parse_str("42");
  assert_eq!(r.unwrap(), Some(42));
}

#[test]
fn consume_all_cached_empty() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    Ok(inp.consume_all_cached().is_none())
  }

  let r = ignored_parser!().apply(parse).parse_str("1 2 3");
  assert!(r.unwrap());
}

#[test]
fn consume_all_cached_after_peek() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    let _ = inp.peek_one()?;
    Ok(inp.consume_all_cached().is_some())
  }

  let r = ignored_parser!().apply(parse).parse_str("1 2 3");
  assert!(r.unwrap());
}

// ── sync_through tests ──────────────────────────────────────────────────────

#[test]
fn sync_through_finds_token() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    let found = inp.sync_through(
      |t| matches!(t.data(), Token::Comma),
      || Some(Expected::one(TokenKind::Comma)),
    )?;
    Ok(found.is_some())
  }

  let r = ignored_parser!().apply(parse).parse_str("1 2 , 3");
  assert!(r.unwrap());
}

#[test]
fn sync_through_no_match() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    let found = inp.sync_through(
      |t| matches!(t.data(), Token::Comma),
      || Some(Expected::one(TokenKind::Comma)),
    )?;
    Ok(found.is_none())
  }

  let r = ignored_parser!().apply(parse).parse_str("1 2 3");
  assert!(r.unwrap());
}
