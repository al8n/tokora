#![cfg(all(feature = "std", feature = "logos"))]
#![allow(warnings)]

//! Integration tests for:
//!   - parser/padded.rs   (Padded, PaddedLeft, PaddedRight)
//!   - parser/peek/peek_then.rs (PeekThen via try_parse_input / Decision)
//!   - parser/any.rs      (Any -- spanned, sliced, located, EOI)
//!   - parser/expect.rs   (Expect -- spanned, sliced, located, try_expect variants)

use generic_arraydeque::typenum::U1;
use tokit::slice::Sliced;
use tokit::{
  Emitter, InputRef, Parse, ParseContext, ParseInput, Parser, ParserContext, Token as TokenT,
  TryParseInput,
  cache::DefaultCache,
  emitter::Ignored,
  error::UnexpectedEot,
  error::token::UnexpectedToken,
  logos::{self, Logos},
  parser::{Action, Any, expect},
  span::Spanned,
  try_parse_input::ParseAttempt,
  utils::Expected,
};

// ═══════════════════════════════════════════════════════════════════════════
//  Trivia-aware lexer (whitespace is a token with is_trivia => true)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Logos, PartialEq)]
#[logos(crate = logos)]
pub enum TToken {
  #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap_or(0))]
  Num(i64),
  #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
  Ident,
  #[token("+")]
  Plus,
  #[token(",")]
  Comma,
  #[token(";")]
  Semi,
  #[regex(r"[ \t\r\n]+")]
  Ws,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TTokenKind {
  Num,
  Ident,
  Plus,
  Comma,
  Semi,
  Ws,
}

impl core::fmt::Display for TTokenKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      TTokenKind::Num => write!(f, "number"),
      TTokenKind::Ident => write!(f, "identifier"),
      TTokenKind::Plus => write!(f, "+"),
      TTokenKind::Comma => write!(f, ","),
      TTokenKind::Semi => write!(f, ";"),
      TTokenKind::Ws => write!(f, "whitespace"),
    }
  }
}

impl core::fmt::Display for TToken {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      TToken::Num(n) => write!(f, "{n}"),
      TToken::Ident => write!(f, "identifier"),
      TToken::Plus => write!(f, "+"),
      TToken::Comma => write!(f, ","),
      TToken::Semi => write!(f, ";"),
      TToken::Ws => write!(f, "whitespace"),
    }
  }
}

impl From<&TToken> for TTokenKind {
  fn from(t: &TToken) -> Self {
    match t {
      TToken::Num(_) => TTokenKind::Num,
      TToken::Ident => TTokenKind::Ident,
      TToken::Plus => TTokenKind::Plus,
      TToken::Comma => TTokenKind::Comma,
      TToken::Semi => TTokenKind::Semi,
      TToken::Ws => TTokenKind::Ws,
    }
  }
}

impl TokenT<'_> for TToken {
  type Kind = TTokenKind;
  type Error = ();

  fn kind(&self) -> TTokenKind {
    TTokenKind::from(self)
  }

  fn is_trivia(&self) -> bool {
    matches!(self, TToken::Ws)
  }
}

type TLexer<'a> = tokit::lexer::LogosLexer<'a, TToken>;

type TriviaIgnoredContext<'inp> =
  ParserContext<'inp, TLexer<'inp>, Ignored, DefaultCache<'inp, TLexer<'inp>>>;

macro_rules! trivia_parser {
  () => {
    Parser::with_context(TriviaIgnoredContext::new(Ignored::default()))
  };
}

// ═══════════════════════════════════════════════════════════════════════════
//  Non-trivia lexer (shared common module)
// ═══════════════════════════════════════════════════════════════════════════

mod common;
use common::{TestLexer, Token, TokenKind};

// ═══════════════════════════════════════════════════════════════════════════
//  Error helper
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
struct TestError;

impl From<()> for TestError {
  fn from(_: ()) -> Self {
    TestError
  }
}

impl<S, Lang: ?Sized> From<UnexpectedEot<S, Lang>> for TestError {
  fn from(_: UnexpectedEot<S, Lang>) -> Self {
    TestError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>>
  for TestError
{
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    TestError
  }
}

// ═══════════════════════════════════════════════════════════════════════════
//  PADDED TESTS (using Ignored emitter so sync_to works with trivia)
// ═══════════════════════════════════════════════════════════════════════════

// ── Padded (both sides) ─────────────────────────────────────────────────

#[test]
fn padded_no_whitespace() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<TToken, ()> {
    expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded()
    .parse_input(inp)
  }
  let result = trivia_parser!().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result, TToken::Num(42)));
}

#[test]
fn padded_leading_whitespace() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<TToken, ()> {
    expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded()
    .parse_input(inp)
  }
  let result = trivia_parser!().apply(parse).parse_str("  42").unwrap();
  assert!(matches!(result, TToken::Num(42)));
}

#[test]
fn padded_trailing_whitespace() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<TToken, ()> {
    expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded()
    .parse_input(inp)
  }
  let result = trivia_parser!().apply(parse).parse_str("42  ").unwrap();
  assert!(matches!(result, TToken::Num(42)));
}

#[test]
fn padded_both_sides_whitespace() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<TToken, ()> {
    expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded()
    .parse_input(inp)
  }
  let result = trivia_parser!().apply(parse).parse_str("  42  ").unwrap();
  assert!(matches!(result, TToken::Num(42)));
}

#[test]
fn padded_tabs_and_newlines() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<TToken, ()> {
    expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded()
    .parse_input(inp)
  }
  let result = trivia_parser!()
    .apply(parse)
    .parse_str("\t\n 99 \n\t")
    .unwrap();
  assert!(matches!(result, TToken::Num(99)));
}

#[test]
fn padded_sequence_two_tokens() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<(TToken, TToken), ()> {
    let a = expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded()
    .parse_input(inp)?;
    let b = expect(|t: &TToken| {
      if matches!(t, TToken::Ident) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Ident))
      }
    })
    .padded()
    .parse_input(inp)?;
    Ok((a, b))
  }
  let (a, b) = trivia_parser!()
    .apply(parse)
    .parse_str("  42   foo  ")
    .unwrap();
  assert!(matches!(a, TToken::Num(42)));
  assert!(matches!(b, TToken::Ident));
}

#[test]
fn padded_consumes_trailing_then_eoi() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<bool, ()> {
    let _num = expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded()
    .parse_input(inp)?;
    Ok(inp.is_eoi())
  }
  let is_eoi = trivia_parser!().apply(parse).parse_str("42  ").unwrap();
  assert!(is_eoi, "padded should consume trailing whitespace");
}

// ── PaddedLeft ──────────────────────────────────────────────────────────

#[test]
fn padded_left_skips_leading() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<TToken, ()> {
    expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded_left()
    .parse_input(inp)
  }
  let result = trivia_parser!().apply(parse).parse_str("  42").unwrap();
  assert!(matches!(result, TToken::Num(42)));
}

#[test]
fn padded_left_no_whitespace() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<TToken, ()> {
    expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded_left()
    .parse_input(inp)
  }
  let result = trivia_parser!().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result, TToken::Num(42)));
}

#[test]
fn padded_left_preserves_trailing() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<bool, ()> {
    let _num = expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded_left()
    .parse_input(inp)?;
    Ok(inp.is_eoi())
  }
  let is_eoi = trivia_parser!().apply(parse).parse_str("  42  ").unwrap();
  assert!(
    !is_eoi,
    "trailing whitespace should remain after padded_left"
  );
}

// ── PaddedRight ─────────────────────────────────────────────────────────

#[test]
fn padded_right_skips_trailing() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<bool, ()> {
    let _num = expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded_right()
    .parse_input(inp)?;
    Ok(inp.is_eoi())
  }
  let is_eoi = trivia_parser!().apply(parse).parse_str("42  ").unwrap();
  assert!(
    is_eoi,
    "trailing whitespace should be consumed by padded_right"
  );
}

#[test]
fn padded_right_no_whitespace() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<TToken, ()> {
    expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded_right()
    .parse_input(inp)
  }
  let result = trivia_parser!().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result, TToken::Num(42)));
}

#[test]
fn padded_right_does_not_skip_leading() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TLexer<'inp>, TriviaIgnoredContext<'inp>>,
  ) -> Result<TToken, ()> {
    expect(|t: &TToken| {
      if matches!(t, TToken::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TTokenKind::Num))
      }
    })
    .padded_right()
    .parse_input(inp)
  }
  let result: Result<TToken, ()> = trivia_parser!().apply(parse).parse_str("  42");
  assert!(
    result.is_err(),
    "padded_right should not skip leading whitespace"
  );
}

// ═══════════════════════════════════════════════════════════════════════════
//  PEEK_THEN TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn peek_then_passes_when_condition_ok() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .peek_then::<_, U1>(|_peeked, _emitter| Ok(()))
    .parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result, Token::Num(42)));
}

#[test]
fn peek_then_fails_when_condition_err() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .peek_then::<_, U1>(|_peeked, _emitter| Err(TestError))
    .parse_input(inp)
  }
  let result: Result<Token, TestError> = Parser::new().apply(parse).parse_str("42");
  assert!(result.is_err());
}

#[test]
fn peek_then_inspects_peeked_len() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .peek_then::<_, U1>(|peeked, _emitter| {
      if peeked.len() > 0 {
        Ok(())
      } else {
        Err(TestError)
      }
    })
    .parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result, Token::Num(42)));
}

#[test]
fn peek_then_on_empty_input() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .peek_then::<_, U1>(|peeked, _emitter| {
      if peeked.is_empty() {
        Err(TestError)
      } else {
        Ok(())
      }
    })
    .parse_input(inp)
  }
  let result: Result<Token, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(result.is_err());
}

// ── PeekThen additional tests ──────────────────────────────────────────

#[test]
fn peek_then_with_different_token_types() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Plus) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Plus))
      }
    })
    .peek_then::<_, U1>(|_peeked, _emitter| Ok(()))
    .parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("+").unwrap();
  assert!(matches!(result, Token::Plus));
}

#[test]
fn peek_then_rejects_wrong_token() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .peek_then::<_, U1>(|peeked, _emitter| {
      // Reject if peeked token is not what we expect
      if peeked.is_empty() {
        Err(TestError)
      } else {
        Ok(())
      }
    })
    .parse_input(inp)
  }
  // "+" is not a Num, so expect will fail even though peek_then succeeds
  let result: Result<Token, TestError> = Parser::new().apply(parse).parse_str("+");
  assert!(result.is_err());
}

#[test]
fn peek_then_sequence_after_peek() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<(Token, Token), TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    let a = expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .peek_then::<_, U1>(|_peeked, _emitter| Ok(()))
    .parse_input(inp)?;
    let b: Token = Any::new().parse_input(inp)?;
    Ok((a, b))
  }
  let (a, b) = Parser::new().apply(parse).parse_str("42 +").unwrap();
  assert!(matches!(a, Token::Num(42)));
  assert!(matches!(b, Token::Plus));
}

// ═══════════════════════════════════════════════════════════════════════════
//  ANY PARSER TESTS (spanned, sliced, located, EOI)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn any_basic_token() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::new().parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result, Token::Num(42)));
}

#[test]
fn any_eoi_error() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::new().parse_input(inp)
  }
  let result: Result<Token, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(result.is_err());
}

#[test]
fn any_spanned() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Spanned<Token, tokit::SimpleSpan>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::spanned().parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result.data(), Token::Num(42)));
}

#[test]
fn any_spanned_eoi() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Spanned<Token, tokit::SimpleSpan>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::spanned().parse_input(inp)
  }
  let result: Result<_, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(result.is_err());
}

#[test]
fn any_sliced() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Sliced<Token, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::sliced().parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result.data(), Token::Num(42)));
  assert_eq!(*result.slice_ref(), "42");
}

#[test]
fn any_sliced_eoi() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Sliced<Token, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::sliced().parse_input(inp)
  }
  let result: Result<_, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(result.is_err());
}

#[test]
fn any_located() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<tokit::Located<Token, tokit::SimpleSpan, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::located().parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(*result, Token::Num(42)));
}

#[test]
fn any_located_eoi() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<tokit::Located<Token, tokit::SimpleSpan, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::located().parse_input(inp)
  }
  let result: Result<_, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(result.is_err());
}

#[test]
fn any_multiple_tokens() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<(Token, Token), TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    let a: Token = Any::new().parse_input(inp)?;
    let b: Token = Any::new().parse_input(inp)?;
    Ok((a, b))
  }
  let (a, b) = Parser::new().apply(parse).parse_str("42 +").unwrap();
  assert!(matches!(a, Token::Num(42)));
  assert!(matches!(b, Token::Plus));
}

// ── Any with _of() variants ────────────────────────────────────────────

#[test]
fn any_of_basic() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::of().parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("+").unwrap();
  assert!(matches!(result, Token::Plus));
}

#[test]
fn any_spanned_of() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Spanned<Token, tokit::SimpleSpan>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::spanned_of().parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("+").unwrap();
  assert!(matches!(result.data(), Token::Plus));
}

#[test]
fn any_sliced_of() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Sliced<Token, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::sliced_of().parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("+").unwrap();
  assert_eq!(*result.slice_ref(), "+");
}

#[test]
fn any_located_of() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<tokit::Located<Token, tokit::SimpleSpan, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    Any::located_of().parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("+").unwrap();
  assert!(matches!(*result, Token::Plus));
}

// ═══════════════════════════════════════════════════════════════════════════
//  EXPECT PARSER TESTS (spanned, sliced, located, try_expect variants)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn expect_match() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result, Token::Num(42)));
}

#[test]
fn expect_mismatch() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .parse_input(inp)
  }
  let result: Result<Token, TestError> = Parser::new().apply(parse).parse_str("+");
  assert!(result.is_err());
}

#[test]
fn expect_eoi() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .parse_input(inp)
  }
  let result: Result<Token, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(result.is_err());
}

#[test]
fn expect_spanned_match() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Spanned<Token, tokit::SimpleSpan>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .spanned()
    .parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result.data(), Token::Num(42)));
}

#[test]
fn expect_spanned_eoi() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Spanned<Token, tokit::SimpleSpan>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .spanned()
    .parse_input(inp)
  }
  let result: Result<_, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(result.is_err());
}

#[test]
fn expect_sliced_match() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Sliced<Token, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .sliced()
    .parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result.data(), Token::Num(42)));
  assert_eq!(*result.slice_ref(), "42");
}

#[test]
fn expect_sliced_mismatch() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Sliced<Token, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .sliced()
    .parse_input(inp)
  }
  let result: Result<_, TestError> = Parser::new().apply(parse).parse_str("+");
  assert!(result.is_err());
}

#[test]
fn expect_located_match() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<tokit::Located<Token, tokit::SimpleSpan, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .located()
    .parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(*result, Token::Num(42)));
}

#[test]
fn expect_located_eoi() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<tokit::Located<Token, tokit::SimpleSpan, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .located()
    .parse_input(inp)
  }
  let result: Result<_, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(result.is_err());
}

// ── try_expect variants ─────────────────────────────────────────────────

#[test]
fn try_expect_accept() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    let mut p =
      tokit::parser::try_expect::<_, TestLexer, Ctx>(|t: &Token| matches!(t, Token::Num(_)));
    let result = p.try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Accept(_)))
  }
  let r: Result<bool, TestError> = Parser::new().apply(parse).parse_str("42");
  assert!(r.unwrap());
}

#[test]
fn try_expect_decline() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    let mut p =
      tokit::parser::try_expect::<_, TestLexer, Ctx>(|t: &Token| matches!(t, Token::Num(_)));
    let result = p.try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }
  let r: Result<bool, TestError> = Parser::new().apply(parse).parse_str("+");
  assert!(r.unwrap());
}

#[test]
fn try_expect_eoi_decline() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    let mut p =
      tokit::parser::try_expect::<_, TestLexer, Ctx>(|t: &Token| matches!(t, Token::Num(_)));
    let result = p.try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }
  let r: Result<bool, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(r.unwrap());
}

// ── try_expect via &Expect (spanned TryParseInput) ──────────────────────

#[test]
fn try_expect_spanned_accept() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    let p = tokit::parser::try_expect::<_, TestLexer, Ctx>(|t: &Token| matches!(t, Token::Num(_)));
    // Use &Expect which implements TryParseInput for Spanned
    let result = (&p).try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Accept(_)))
  }
  let r: Result<bool, TestError> = Parser::new().apply(parse).parse_str("42");
  assert!(r.unwrap());
}

#[test]
fn try_expect_spanned_decline() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    let p = tokit::parser::try_expect::<_, TestLexer, Ctx>(|t: &Token| matches!(t, Token::Num(_)));
    let result = (&p).try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }
  let r: Result<bool, TestError> = Parser::new().apply(parse).parse_str("+");
  assert!(r.unwrap());
}

// ── try_expect on empty via &Expect ─────────────────────────────────────

#[test]
fn try_expect_spanned_eoi() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    let p = tokit::parser::try_expect::<_, TestLexer, Ctx>(|t: &Token| matches!(t, Token::Num(_)));
    let result = (&p).try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }
  let r: Result<bool, TestError> = Parser::new().apply(parse).parse_str("");
  assert!(r.unwrap());
}

// ── expect_of / try_expect_of ───────────────────────────────────────────

#[test]
fn expect_of_match() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Token, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    tokit::parser::expect_of::<_, TestLexer, Ctx, ()>(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .parse_input(inp)
  }
  let result = Parser::new().apply(parse).parse_str("42").unwrap();
  assert!(matches!(result, Token::Num(42)));
}

#[test]
fn try_expect_of_accept() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<bool, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    let mut p =
      tokit::parser::try_expect_of::<_, TestLexer, Ctx, ()>(|t: &Token| matches!(t, Token::Num(_)));
    let result = p.try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Accept(_)))
  }
  let r: Result<bool, TestError> = Parser::new().apply(parse).parse_str("42");
  assert!(r.unwrap());
}

// ── Expect with spanned via With<Expect, PhantomSpan> ───────────────────

#[test]
fn expect_spanned_mismatch() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Spanned<Token, tokit::SimpleSpan>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .spanned()
    .parse_input(inp)
  }
  let result: Result<_, TestError> = Parser::new().apply(parse).parse_str("+");
  assert!(result.is_err());
}

#[test]
fn expect_located_mismatch() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<tokit::Located<Token, tokit::SimpleSpan, &'inp str>, TestError>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = TestError>,
  {
    expect(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .located()
    .parse_input(inp)
  }
  let result: Result<_, TestError> = Parser::new().apply(parse).parse_str("+");
  assert!(result.is_err());
}
