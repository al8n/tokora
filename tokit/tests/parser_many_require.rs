#![cfg(all(feature = "std", feature = "logos"))]

//! Tests for sep/parse and sep_while/parse with require_* separator policies.
//!
//! These variants require `MissingTrailingSeparatorEmitter` or
//! `MissingLeadingSeparatorEmitter` (not implemented by `Fatal`), so they need
//! a custom emitter.

mod common;

use generic_arraydeque::typenum::U1;
use tokit::{
  Accumulator, Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
  Token as TokenTrait, TryParseInput,
  cache::Peeked,
  emitter::{
    FromSeparatedError, FromUnexpectedLeadingSeparatorError, FromUnexpectedTrailingSeparatorError,
    FullContainerEmitter, MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter,
    SeparatedEmitter, TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
    UnexpectedTrailingSeparatorEmitter,
  },
  error::syntax::MissingSyntaxOf,
  error::{
    UnexpectedEot,
    syntax::{FullContainer, TooFew, TooMany},
    token::{MissingTokenOf, UnexpectedToken, UnexpectedTokenOf},
  },
  input::Cursor,
  parser::Action,
  span::Spanned,
  try_parse_input::ParseAttempt,
  utils::CowStr,
};

use common::{TestLexer, Token};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct ManyReqError;

impl From<()> for ManyReqError {
  fn from(_: ()) -> Self {
    ManyReqError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>>
  for ManyReqError
{
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    ManyReqError
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for ManyReqError {
  fn from(_: FullContainer<S, Lang>) -> Self {
    ManyReqError
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for ManyReqError {
  fn from(_: TooFew<S, Lang>) -> Self {
    ManyReqError
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for ManyReqError {
  fn from(_: TooMany<S, Lang>) -> Self {
    ManyReqError
  }
}

impl From<UnexpectedEot> for ManyReqError {
  fn from(_: UnexpectedEot) -> Self {
    ManyReqError
  }
}

impl<'inp> FromSeparatedError<'inp, TestLexer<'inp>> for ManyReqError {
  fn from_missing_separator(_: CowStr, _: MissingTokenOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    ManyReqError
  }

  fn from_missing_element(_: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    ManyReqError
  }
}

impl<'inp> FromUnexpectedLeadingSeparatorError<'inp, TestLexer<'inp>> for ManyReqError {
  fn from_unexpected_leading_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    ManyReqError
  }
}

impl<'inp> FromUnexpectedTrailingSeparatorError<'inp, TestLexer<'inp>> for ManyReqError {
  fn from_unexpected_trailing_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    ManyReqError
  }
}

// ── Full emitter ──────────────────────────────────────────────────────────────

struct FullEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for FullEmitter {
  type Error = ManyReqError;

  fn emit_lexer_error(
    &mut self,
    _: Spanned<
      <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
      <TestLexer<'inp> as Lexer<'inp>>::Span,
    >,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }

  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }

  fn emit_error(
    &mut self,
    err: Spanned<ManyReqError, <TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), ManyReqError>
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

impl<'inp> SeparatedEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }

  fn emit_missing_element(
    &mut self,
    _: MissingSyntaxOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }
}

impl<'inp> FullContainerEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_full_container(
    &mut self,
    _: FullContainer<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }
}

impl<'inp> TooFewEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_too_few(
    &mut self,
    _: TooFew<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }
}

impl<'inp> TooManyEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_too_many(
    &mut self,
    _: TooMany<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }
}

impl<'inp> UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_unexpected_leading_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }
}

impl<'inp> UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }
}

impl<'inp> MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_trailing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }
}

impl<'inp> MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_leading_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), ManyReqError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(ManyReqError)
  }
}

fn full_ctx() -> ParserContext<'static, TestLexer<'static>, FullEmitter> {
  ParserContext::new(FullEmitter)
}

// ── Element parsers ───────────────────────────────────────────────────────────

/// TryParseInput element parser for sep/parse tests.
fn try_num_req<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>,
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

/// ParseInput element parser for sep_while/parse tests.
fn parse_num_while_req<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<i64, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>,
{
  match inp.next()? {
    None => Err(ManyReqError),
    Some(tok) => match tok.into_data() {
      Token::Num(n) => Ok(n),
      _ => Err(ManyReqError),
    },
  }
}

/// Decision function for sep_while/parse tests.
fn decide_num_req<'inp, Ctx>(
  mut peeked: Peeked<'_, 'inp, TestLexer<'inp>, U1>,
  _: &mut Ctx::Emitter,
) -> Result<Action, <Ctx::Emitter as Emitter<'inp, TestLexer<'inp>>>::Error>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
{
  Ok(match peeked.pop_front() {
    None => Action::Stop,
    Some(tok) => {
      let tok = tok
        .as_maybe_ref()
        .map(|t| t.token().copied(), |t| t.token())
        .into_inner();
      if matches!(**tok.data(), Token::Num(_)) {
        Action::Continue
      } else {
        Action::Stop
      }
    }
  })
}

// ═══════════════════════════════════════════════════════════════════════════════
// sep/parse tests (TryParseInput + no delimiter)
// ═══════════════════════════════════════════════════════════════════════════════

// ── 1. bounded ────────────────────────────────────────────────────────────────

fn parse_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_bounded)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_bounded)
    .parse_str("1");
  assert!(r.is_err());
}

#[test]
fn test_sep_parse_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_bounded)
    .parse_str("1,2,3,4,5");
  assert!(r.is_err());
}

// ── 2. require_trailing unbounded ─────────────────────────────────────────────

fn parse_require_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_require_trailing_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 3. require_trailing at_least ──────────────────────────────────────────────

fn parse_require_trailing_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_least)
    .parse_str("1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_require_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_least)
    .parse_str("1,");
  assert!(r.is_err());
}

// ── 4. require_trailing at_most ───────────────────────────────────────────────

fn parse_require_trailing_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .at_most(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_most)
    .parse_str("1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_require_trailing_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_most)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 5. require_trailing bounded ───────────────────────────────────────────────

fn parse_require_trailing_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_bounded)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_require_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_bounded)
    .parse_str("1,");
  assert!(r.is_err());
}

// ── 6. require_leading unbounded ──────────────────────────────────────────────

fn parse_require_leading<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_leading_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_require_leading_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 7. require_leading at_least ───────────────────────────────────────────────

fn parse_require_leading_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_leading()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_leading_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_least)
    .parse_str(",1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_require_leading_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_least)
    .parse_str(",1");
  assert!(r.is_err());
}

// ── 8. require_leading at_most ────────────────────────────────────────────────

fn parse_require_leading_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_leading()
    .at_most(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_leading_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_most)
    .parse_str(",1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_require_leading_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_most)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 9. require_leading bounded ────────────────────────────────────────────────

fn parse_require_leading_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_leading()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_leading_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_bounded)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_require_leading_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_bounded)
    .parse_str(",1");
  assert!(r.is_err());
}

// ── 10. require_surrounded unbounded ─────────────────────────────────────────
// Chain: .require_trailing().require_leading()

fn parse_require_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_surrounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_require_surrounded_fail_no_leading() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded)
    .parse_str("1,2,3,");
  assert!(r.is_err());
}

// ── 11. require_surrounded at_least ──────────────────────────────────────────

fn parse_require_surrounded_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_least)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_require_surrounded_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_least)
    .parse_str(",1,");
  assert!(r.is_err());
}

// ── 12. require_surrounded at_most ───────────────────────────────────────────

fn parse_require_surrounded_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .at_most(3)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_most)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_require_surrounded_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_most)
    .parse_str("1,2,3,");
  assert!(r.is_err());
}

// ── 13. require_surrounded bounded ───────────────────────────────────────────

fn parse_require_surrounded_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .bounded(2, 4)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_bounded)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_require_surrounded_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_bounded)
    .parse_str(",1,");
  assert!(r.is_err());
}

// ── 14. allow_leading_require_trailing unbounded ──────────────────────────────
// Chain: .require_trailing().allow_leading()

fn parse_allow_leading_require_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_no_leading_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 15. allow_leading_require_trailing at_least ───────────────────────────────

fn parse_allow_leading_require_trailing_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_least)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_least)
    .parse_str(",1,");
  assert!(r.is_err());
}

// ── 16. allow_leading_require_trailing at_most ────────────────────────────────

fn parse_allow_leading_require_trailing_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .at_most(3)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_most)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_most)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 17. allow_leading_require_trailing bounded ────────────────────────────────

fn parse_allow_leading_require_trailing_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .require_trailing()
    .bounded(2, 4)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_bounded)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_bounded)
    .parse_str(",1,");
  assert!(r.is_err());
}

// ── 18. require_leading_allow_trailing unbounded ──────────────────────────────
// Chain: .allow_trailing().require_leading()

fn parse_require_leading_allow_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .allow_trailing()
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_no_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 19. require_leading_allow_trailing at_least ───────────────────────────────

fn parse_require_leading_allow_trailing_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .allow_trailing()
    .at_least(2)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_least)
    .parse_str(",1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_least)
    .parse_str(",1");
  assert!(r.is_err());
}

// ── 20. require_leading_allow_trailing at_most ────────────────────────────────

fn parse_require_leading_allow_trailing_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .allow_trailing()
    .at_most(3)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_most)
    .parse_str(",1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_most)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 21. require_leading_allow_trailing bounded ────────────────────────────────

fn parse_require_leading_allow_trailing_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_req
    .separated_by_comma()
    .allow_trailing()
    .bounded(2, 4)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_bounded)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_bounded)
    .parse_str(",1");
  assert!(r.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// sep_while/parse tests (ParseInput + Decision, no delimiter)
// ═══════════════════════════════════════════════════════════════════════════════

// ── 22. sep_while require_trailing unbounded ──────────────────────────────────

fn parse_while_require_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing)
    .parse_str("1,2,3,+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_require_trailing_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing)
    .parse_str("1,2,3+");
  assert!(r.is_err());
}

// ── 23. sep_while require_trailing at_least ───────────────────────────────────

fn parse_while_require_trailing_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing_at_least)
    .parse_str("1,2,+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_require_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing_at_least)
    .parse_str("1,+");
  assert!(r.is_err());
}

// ── 24. sep_while require_trailing at_most ────────────────────────────────────

fn parse_while_require_trailing_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .at_most(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing_at_most)
    .parse_str("1,2,+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_require_trailing_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing_at_most)
    .parse_str("1,2,3+");
  assert!(r.is_err());
}

// ── 25. sep_while require_trailing bounded ────────────────────────────────────

fn parse_while_require_trailing_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing_bounded)
    .parse_str("1,2,3,+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_require_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing_bounded)
    .parse_str("1,+");
  assert!(r.is_err());
}

// ── 26. sep_while require_leading unbounded ───────────────────────────────────

fn parse_while_require_leading<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_leading_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading)
    .parse_str(",1,2,3+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_require_leading_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading)
    .parse_str("1,2,3+");
  assert!(r.is_err());
}

// ── 27. sep_while require_leading at_least ────────────────────────────────────

fn parse_while_require_leading_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_leading()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_leading_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_at_least)
    .parse_str(",1,2+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_require_leading_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_at_least)
    .parse_str(",1+");
  assert!(r.is_err());
}

// ── 28. sep_while require_leading at_most ─────────────────────────────────────

fn parse_while_require_leading_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_leading()
    .at_most(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_leading_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_at_most)
    .parse_str(",1,2+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_require_leading_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_at_most)
    .parse_str("1,2,3+");
  assert!(r.is_err());
}

// ── 29. sep_while require_leading bounded ─────────────────────────────────────

fn parse_while_require_leading_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_leading()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_leading_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_bounded)
    .parse_str(",1,2,3+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_require_leading_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_bounded)
    .parse_str(",1+");
  assert!(r.is_err());
}

// ── 30. sep_while require_surrounded unbounded ───────────────────────────────

fn parse_while_require_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_surrounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded)
    .parse_str(",1,2,3,+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_require_surrounded_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded)
    .parse_str("1,2,3,+");
  assert!(r.is_err());
}

// ── 31. sep_while require_surrounded at_least ────────────────────────────────

fn parse_while_require_surrounded_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .at_least(2)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded_at_least)
    .parse_str(",1,2,+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_require_surrounded_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded_at_least)
    .parse_str(",1,+");
  assert!(r.is_err());
}

// ── 32. sep_while require_surrounded at_most ─────────────────────────────────

fn parse_while_require_surrounded_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .at_most(3)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded_at_most)
    .parse_str(",1,2,+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_require_surrounded_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded_at_most)
    .parse_str("1,2,3,+");
  assert!(r.is_err());
}

// ── 33. sep_while require_surrounded bounded ─────────────────────────────────

fn parse_while_require_surrounded_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .bounded(2, 4)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded_bounded)
    .parse_str(",1,2,3,+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_require_surrounded_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded_bounded)
    .parse_str(",1,+");
  assert!(r.is_err());
}

// ── 34. sep_while allow_leading_require_trailing unbounded ────────────────────

fn parse_while_allow_leading_require_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_leading_require_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing)
    .parse_str(",1,2,3,+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_leading_require_trailing_no_leading_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing)
    .parse_str("1,2,3,+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_leading_require_trailing_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing)
    .parse_str("1,2,3+");
  assert!(r.is_err());
}

// ── 35. sep_while allow_leading_require_trailing at_least ────────────────────

fn parse_while_allow_leading_require_trailing_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .at_least(2)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_leading_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing_at_least)
    .parse_str(",1,2,+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_allow_leading_require_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing_at_least)
    .parse_str(",1,+");
  assert!(r.is_err());
}

// ── 36. sep_while allow_leading_require_trailing at_most ─────────────────────

fn parse_while_allow_leading_require_trailing_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .at_most(3)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_leading_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing_at_most)
    .parse_str(",1,2,+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_allow_leading_require_trailing_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing_at_most)
    .parse_str("1,2,3+");
  assert!(r.is_err());
}

// ── 37. sep_while allow_leading_require_trailing bounded ─────────────────────

fn parse_while_allow_leading_require_trailing_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .require_trailing()
    .bounded(2, 4)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_leading_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing_bounded)
    .parse_str(",1,2,+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_allow_leading_require_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing_bounded)
    .parse_str(",1,+");
  assert!(r.is_err());
}

// ── 38. sep_while require_leading_allow_trailing unbounded ────────────────────

fn parse_while_require_leading_allow_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .allow_trailing()
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_leading_allow_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing)
    .parse_str(",1,2,3,+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_require_leading_allow_trailing_no_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing)
    .parse_str(",1,2,3+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_require_leading_allow_trailing_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing)
    .parse_str("1,2,3+");
  assert!(r.is_err());
}

// ── 39. sep_while require_leading_allow_trailing at_least ────────────────────

fn parse_while_require_leading_allow_trailing_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .allow_trailing()
    .at_least(2)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_leading_allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing_at_least)
    .parse_str(",1,2+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_require_leading_allow_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing_at_least)
    .parse_str(",1+");
  assert!(r.is_err());
}

// ── 40. sep_while require_leading_allow_trailing at_most ─────────────────────

fn parse_while_require_leading_allow_trailing_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .allow_trailing()
    .at_most(3)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_leading_allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing_at_most)
    .parse_str(",1,2+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_require_leading_allow_trailing_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing_at_most)
    .parse_str("1,2,3+");
  assert!(r.is_err());
}

// ── 41. sep_while require_leading_allow_trailing bounded ─────────────────────

fn parse_while_require_leading_allow_trailing_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .allow_trailing()
    .bounded(2, 4)
    .require_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_require_leading_allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing_bounded)
    .parse_str(",1,2,3+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_require_leading_allow_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing_bounded)
    .parse_str(",1+");
  assert!(r.is_err());
}

// ── 42. sep_while allow_trailing at_most ─────────────────────────────────────

fn parse_while_allow_trailing_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .allow_trailing()
    .at_most(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_trailing_at_most)
    .parse_str("1,2,+")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_while_allow_trailing_at_most_fail() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_trailing_at_most)
    .parse_str("1,2,3,4,5+");
  assert!(r.is_err());
}

// ── 43. sep_while allow_trailing bounded ─────────────────────────────────────

fn parse_while_allow_trailing_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ManyReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ManyReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while_req
    .separated_by_comma_while::<_, U1>(decide_num_req::<Ctx>)
    .allow_trailing()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_trailing_bounded)
    .parse_str("1,2,3,+")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_trailing_bounded)
    .parse_str("1+");
  assert!(r.is_err());
}

// ── Additional too_many tests for bounded variants (sep/parse) ──────────────

#[test]
fn test_sep_parse_require_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_bounded)
    .parse_str("1,2,3,4,5,");
  assert!(r.is_err());
}

#[test]
fn test_sep_parse_require_leading_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_bounded)
    .parse_str(",1,2,3,4,5");
  assert!(r.is_err());
}

#[test]
fn test_sep_parse_require_surrounded_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_bounded)
    .parse_str(",1,2,3,4,5,");
  assert!(r.is_err());
}

#[test]
fn test_sep_parse_allow_leading_require_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_bounded)
    .parse_str(",1,2,3,4,5,");
  assert!(r.is_err());
}

#[test]
fn test_sep_parse_require_leading_allow_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_bounded)
    .parse_str(",1,2,3,4,5,");
  assert!(r.is_err());
}

// ── Additional too_many tests for bounded variants (sep_while/parse) ────────

#[test]
fn test_sep_while_require_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing_bounded)
    .parse_str("1,2,3,4,5,+");
  assert!(r.is_err());
}

#[test]
fn test_sep_while_require_leading_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_bounded)
    .parse_str(",1,2,3,4,5+");
  assert!(r.is_err());
}

#[test]
fn test_sep_while_require_surrounded_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded_bounded)
    .parse_str(",1,2,3,4,5,+");
  assert!(r.is_err());
}

#[test]
fn test_sep_while_allow_leading_require_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_leading_require_trailing_bounded)
    .parse_str(",1,2,3,4,5,+");
  assert!(r.is_err());
}

#[test]
fn test_sep_while_require_leading_allow_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading_allow_trailing_bounded)
    .parse_str(",1,2,3,4,5,+");
  assert!(r.is_err());
}

#[test]
fn test_sep_while_allow_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_allow_trailing_bounded)
    .parse_str("1,2,3,4,5,+");
  assert!(r.is_err());
}

// ── Empty input tests ───────────────────────────────────────────────────────

#[test]
fn test_sep_parse_require_trailing_empty() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing)
    .parse_str("");
  // empty input returns empty vec (no items to require separator for)
  assert_eq!(r.unwrap(), vec![]);
}

#[test]
fn test_sep_parse_require_leading_empty() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading)
    .parse_str("");
  assert_eq!(r.unwrap(), vec![]);
}

#[test]
fn test_sep_parse_require_surrounded_empty() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded)
    .parse_str("");
  assert_eq!(r.unwrap(), vec![]);
}

// ── Boundary tests (exactly at min/max) ────────────────────────────────────

#[test]
fn test_sep_parse_bounded_exactly_min() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_bounded)
    .parse_str("1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_parse_bounded_exactly_max() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_bounded)
    .parse_str("1,2,3,4")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3, 4]);
}

// ── Additional require_trailing missing trailing sep tests ──────────────────

#[test]
fn test_sep_parse_require_trailing_no_trailing() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing)
    .parse_str("1,2,3");
  // Missing required trailing comma
  assert!(r.is_err());
}

#[test]
fn test_sep_parse_require_leading_no_leading() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading)
    .parse_str("1,2,3");
  // Missing required leading comma
  assert!(r.is_err());
}

#[test]
fn test_sep_parse_require_surrounded_no_trailing() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded)
    .parse_str(",1,2,3");
  // Has leading but missing trailing
  assert!(r.is_err());
}

// ── sep_while empty/single tests ────────────────────────────────────────────

#[test]
fn test_sep_while_require_trailing_empty() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing)
    .parse_str("+");
  // sentinel only, no items → empty vec
  assert_eq!(r.unwrap(), vec![]);
}

#[test]
fn test_sep_while_require_leading_empty() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading)
    .parse_str("+");
  assert_eq!(r.unwrap(), vec![]);
}

#[test]
fn test_sep_while_require_surrounded_empty() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_surrounded)
    .parse_str("+");
  assert_eq!(r.unwrap(), vec![]);
}

#[test]
fn test_sep_while_require_trailing_no_trailing() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_trailing)
    .parse_str("1,2,3+");
  // Missing required trailing comma
  assert!(r.is_err());
}

#[test]
fn test_sep_while_require_leading_no_leading() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_while_require_leading)
    .parse_str("1,2,3+");
  // Missing required leading comma
  assert!(r.is_err());
}
