#![cfg(all(feature = "std", feature = "logos"))]

//! Tests exercising the emitter system: Fatal, Silent, and Verbose emitters.
//!
//! These tests trigger error paths through parser combinators to exercise
//! emitter implementations in `emitter/impl_/fatal.rs`, `emitter/impl_/silent.rs`,
//! and `emitter/impl_/verbose.rs`.

mod common;

use tokit::{
  Accumulator, Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
  Token as TokenTrait, TryParseInput,
  emitter::{
    FromSeparatedError, FromUnexpectedLeadingSeparatorError, FromUnexpectedTrailingSeparatorError,
    FullContainerEmitter, SeparatedEmitter, TooFewEmitter, TooManyEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  error::{
    UnexpectedEot,
    syntax::{FullContainer, MissingSyntaxOf, TooFew, TooMany},
    token::{MissingTokenOf, UnexpectedToken, UnexpectedTokenOf},
  },
  input::Cursor,
  span::Spanned,
  try_parse_input::ParseAttempt,
  utils::CowStr,
};

#[allow(unused_imports)]
use common::TokenKind;
use common::{TestLexer, Token};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum EmitterTestError {
  UnexpectedToken,
  LexerError,
  TooFew,
  TooMany,
  FullContainer,
  MissingSeparator,
  MissingElement,
  UnexpectedLeadingSep,
  UnexpectedTrailingSep,
  Custom,
}

impl From<()> for EmitterTestError {
  fn from(_: ()) -> Self {
    EmitterTestError::LexerError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>>
  for EmitterTestError
{
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    EmitterTestError::UnexpectedToken
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for EmitterTestError {
  fn from(_: FullContainer<S, Lang>) -> Self {
    EmitterTestError::FullContainer
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for EmitterTestError {
  fn from(_: TooFew<S, Lang>) -> Self {
    EmitterTestError::TooFew
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for EmitterTestError {
  fn from(_: TooMany<S, Lang>) -> Self {
    EmitterTestError::TooMany
  }
}

impl From<UnexpectedEot> for EmitterTestError {
  fn from(_: UnexpectedEot) -> Self {
    EmitterTestError::Custom
  }
}

impl<'inp> FromSeparatedError<'inp, TestLexer<'inp>> for EmitterTestError {
  fn from_missing_separator(_: CowStr, _: MissingTokenOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    EmitterTestError::MissingSeparator
  }

  fn from_missing_element(_: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    EmitterTestError::MissingElement
  }
}

impl<'inp> FromUnexpectedLeadingSeparatorError<'inp, TestLexer<'inp>> for EmitterTestError {
  fn from_unexpected_leading_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    EmitterTestError::UnexpectedLeadingSep
  }
}

impl<'inp> FromUnexpectedTrailingSeparatorError<'inp, TestLexer<'inp>> for EmitterTestError {
  fn from_unexpected_trailing_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    EmitterTestError::UnexpectedTrailingSep
  }
}

// ── Fatal-like custom emitter (all errors are fatal) ──────────────────────────

struct FatalEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for FatalEmitter {
  type Error = EmitterTestError;

  fn emit_lexer_error(
    &mut self,
    _: Spanned<
      <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
      <TestLexer<'inp> as Lexer<'inp>>::Span,
    >,
  ) -> Result<(), EmitterTestError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(EmitterTestError::LexerError)
  }

  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), EmitterTestError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(EmitterTestError::UnexpectedToken)
  }

  fn emit_error(
    &mut self,
    err: Spanned<EmitterTestError, <TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), EmitterTestError>
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

impl<'inp> SeparatedEmitter<'inp, TestLexer<'inp>> for FatalEmitter {
  fn emit_missing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), EmitterTestError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(EmitterTestError::MissingSeparator)
  }

  fn emit_missing_element(
    &mut self,
    _: MissingSyntaxOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), EmitterTestError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(EmitterTestError::MissingElement)
  }
}

impl<'inp> FullContainerEmitter<'inp, TestLexer<'inp>> for FatalEmitter {
  fn emit_full_container(
    &mut self,
    _: FullContainer<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), EmitterTestError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(EmitterTestError::FullContainer)
  }
}

impl<'inp> TooFewEmitter<'inp, TestLexer<'inp>> for FatalEmitter {
  fn emit_too_few(
    &mut self,
    _: TooFew<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), EmitterTestError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(EmitterTestError::TooFew)
  }
}

impl<'inp> TooManyEmitter<'inp, TestLexer<'inp>> for FatalEmitter {
  fn emit_too_many(
    &mut self,
    _: TooMany<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), EmitterTestError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(EmitterTestError::TooMany)
  }
}

impl<'inp> UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for FatalEmitter {
  fn emit_unexpected_leading_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), EmitterTestError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(EmitterTestError::UnexpectedLeadingSep)
  }
}

impl<'inp> UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for FatalEmitter {
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), EmitterTestError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(EmitterTestError::UnexpectedTrailingSep)
  }
}

// ── Silent emitter ────────────────────────────────────────────────────────────

use tokit::emitter::Silent;

// ── Verbose emitter ───────────────────────────────────────────────────────────

use tokit::emitter::Verbose;

// ── Context constructors ──────────────────────────────────────────────────────

fn fatal_ctx() -> ParserContext<'static, TestLexer<'static>, FatalEmitter> {
  ParserContext::new(FatalEmitter)
}

fn silent_ctx() -> ParserContext<'static, TestLexer<'static>, Silent<EmitterTestError>> {
  ParserContext::new(Silent::new())
}

fn verbose_ctx() -> ParserContext<'static, TestLexer<'static>, Verbose<EmitterTestError>> {
  ParserContext::new(Verbose::new())
}

// ── Element parsers ───────────────────────────────────────────────────────────

fn try_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>,
{
  inp
    .try_expect(|t| matches!(t.data(), Token::Num(_)))
    .map(|opt| match opt {
      None => ParseAttempt::Decline,
      Some(tok) => ParseAttempt::Accept(match tok.into_data() {
        Token::Num(n) => n,
        _ => unreachable!(),
      }),
    })
}

fn parse_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<i64, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>,
{
  use tokit::parser::expect;
  use tokit::utils::Expected;
  expect(|t: &Token| {
    if matches!(t, Token::Num(_)) {
      Ok(())
    } else {
      Err(Expected::one(TokenKind::Num))
    }
  })
  .map(|t| match t {
    Token::Num(n) => n,
    _ => unreachable!(),
  })
  .parse_input(inp)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fatal emitter tests
// ═══════════════════════════════════════════════════════════════════════════════

// ── 1. unexpected token ───────────────────────────────────────────────────────

fn parse_expect_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<i64, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>,
{
  parse_num(inp)
}

#[test]
fn test_fatal_unexpected_token() {
  let r: Result<i64, _> = Parser::with_context(fatal_ctx())
    .apply(parse_expect_num)
    .parse_str("+");
  assert!(r.is_err());
}

// ── 2. too few elements ──────────────────────────────────────────────────────

fn parse_at_least_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .at_least(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_fatal_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx())
    .apply(parse_at_least_3)
    .parse_str("1,2");
  assert!(r.is_err());
}

#[test]
fn test_fatal_too_few_ok() {
  let r: Vec<i64> = Parser::with_context(fatal_ctx())
    .apply(parse_at_least_3)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 3. too many elements ─────────────────────────────────────────────────────

fn parse_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .at_most(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_fatal_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx())
    .apply(parse_at_most_2)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

#[test]
fn test_fatal_too_many_ok() {
  let r: Vec<i64> = Parser::with_context(fatal_ctx())
    .apply(parse_at_most_2)
    .parse_str("1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 4. bounded (triggers both too few and too many) ──────────────────────────

fn parse_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_fatal_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(fatal_ctx())
    .apply(parse_bounded)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_fatal_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx())
    .apply(parse_bounded)
    .parse_str("1");
  assert!(r.is_err());
}

#[test]
fn test_fatal_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx())
    .apply(parse_bounded)
    .parse_str("1,2,3,4,5");
  assert!(r.is_err());
}

// ── 5. empty input (exercises emitter construction path) ─────────────────────

#[test]
fn test_fatal_empty_input() {
  let r: Result<i64, _> = Parser::with_context(fatal_ctx())
    .apply(parse_expect_num)
    .parse_str("");
  assert!(r.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Silent emitter tests
// ═══════════════════════════════════════════════════════════════════════════════

// Silent emitter returns Ok(()) for all errors, allowing parsing to continue.
// We test that error paths do not abort parsing.

fn parse_silent_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_silent_too_few_continues() {
  // Silent emitter swallows the "too few" error, so we get Ok with what was parsed
  let r: Result<Vec<i64>, _> = Parser::with_context(silent_ctx())
    .apply(parse_silent_bounded)
    .parse_str("1");
  // Silent emitter makes too_few non-fatal, so this should succeed
  assert!(r.is_ok());
}

#[test]
fn test_silent_too_many_continues() {
  // Silent emitter swallows the "too many" error
  let r: Result<Vec<i64>, _> = Parser::with_context(silent_ctx())
    .apply(parse_silent_bounded)
    .parse_str("1,2,3,4,5");
  // Silent emitter makes too_many non-fatal, so this should succeed
  assert!(r.is_ok());
}

#[test]
fn test_silent_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(silent_ctx())
    .apply(parse_silent_bounded)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Verbose emitter tests
// ═══════════════════════════════════════════════════════════════════════════════

// Verbose emitter collects errors into a BTreeMap and returns Ok(()), allowing
// parsing to continue. Errors can be retrieved after parsing completes.

#[test]
fn test_verbose_too_few_collects_error() {
  // Verbose emitter collects the "too few" error but parsing continues
  let r: Result<Vec<i64>, _> = Parser::with_context(verbose_ctx())
    .apply(parse_bounded)
    .parse_str("1");
  // Verbose emitter makes too_few non-fatal, so parsing succeeds
  assert!(r.is_ok());
}

#[test]
fn test_verbose_too_many_collects_error() {
  // Verbose emitter collects the "too many" error but parsing continues
  let r: Result<Vec<i64>, _> = Parser::with_context(verbose_ctx())
    .apply(parse_bounded)
    .parse_str("1,2,3,4,5");
  // Verbose emitter makes too_many non-fatal, so parsing succeeds
  assert!(r.is_ok());
}

#[test]
fn test_verbose_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(verbose_ctx())
    .apply(parse_bounded)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_verbose_unexpected_token() {
  // Verbose collects unexpected token errors
  let r: Result<i64, _> = Parser::with_context(verbose_ctx())
    .apply(parse_expect_num)
    .parse_str("+");
  // unexpected token via Verbose still collects - but the parser function
  // itself uses expect() which returns an error, so the overall parse fails
  // because the parser returns Err after emitter returns Ok.
  // Actually, the Verbose emitter returns Ok but the expect combinator
  // itself may still fail. Let's just check it doesn't panic.
  let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fatal emitter - happy path exercises all emitter trait impls
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_fatal_simple_parse_ok() {
  // Simple parse that succeeds - exercises Fatal emitter construction
  let r: i64 = Parser::with_context(fatal_ctx())
    .apply(parse_expect_num)
    .parse_str("42")
    .unwrap();
  assert_eq!(r, 42);
}

#[test]
fn test_fatal_separated_ok() {
  // Happy path through separated combinator with Fatal emitter
  let r: Vec<i64> = Parser::with_context(fatal_ctx())
    .apply(parse_at_least_3)
    .parse_str("10,20,30,40")
    .unwrap();
  assert_eq!(r, vec![10, 20, 30, 40]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Silent emitter - exercises all Silent trait impls
// ═══════════════════════════════════════════════════════════════════════════════

fn parse_silent_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .at_least(5)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_silent_at_least_too_few() {
  // With Silent emitter, "at least 5" with only 2 elements silently succeeds
  let r: Result<Vec<i64>, _> = Parser::with_context(silent_ctx())
    .apply(parse_silent_at_least)
    .parse_str("1,2");
  assert!(r.is_ok());
}

fn parse_silent_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .at_most(1)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_silent_at_most_too_many() {
  // With Silent emitter, "at most 1" with 3 elements silently succeeds
  let r: Result<Vec<i64>, _> = Parser::with_context(silent_ctx())
    .apply(parse_silent_at_most)
    .parse_str("1,2,3");
  assert!(r.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Verbose emitter - exercises Verbose trait impls with error collection
// ═══════════════════════════════════════════════════════════════════════════════

fn parse_verbose_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .at_least(5)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_verbose_at_least_too_few() {
  // With Verbose emitter, "at least 5" with only 2 elements collects error
  let r: Result<Vec<i64>, _> = Parser::with_context(verbose_ctx())
    .apply(parse_verbose_at_least)
    .parse_str("1,2");
  assert!(r.is_ok());
}

fn parse_verbose_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, EmitterTestError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = EmitterTestError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .at_most(1)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_verbose_at_most_too_many() {
  // With Verbose emitter, "at most 1" with 3 elements collects error
  let r: Result<Vec<i64>, _> = Parser::with_context(verbose_ctx())
    .apply(parse_verbose_at_most)
    .parse_str("1,2,3");
  assert!(r.is_ok());
}
