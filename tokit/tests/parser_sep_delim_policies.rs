#![cfg(all(feature = "std", feature = "logos"))]

//! Comprehensive tests for all separator policies combined with delimited
//! parsing and count modifiers (unbounded, at_least, at_most, bounded).
//!
//! Covers 8 policies x 4 count variants x 2 test types (success + error) = 64 tests.

mod common;

use tokit::{
  Accumulator, Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
  Token as TokenTrait, TryParseInput,
  emitter::{
    FromSeparatedError, FromUnexpectedLeadingSeparatorError, FromUnexpectedTrailingSeparatorError,
    FullContainerEmitter, MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter,
    SeparatedEmitter, TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
    UnexpectedTrailingSeparatorEmitter,
  },
  error::{
    UnexpectedEot,
    syntax::{FullContainer, MissingSyntaxOf, TooFew, TooMany},
    token::{MissingTokenOf, UnexpectedToken, UnexpectedTokenOf},
  },
  input::Cursor,
  punct::Bracket,
  span::Spanned,
  try_parse_input::ParseAttempt,
  utils::CowStr,
};

use common::{TestLexer, Token};

// ── Error type ────────────────────────────────────────────────────────────────

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

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for E {
  fn from(_: FullContainer<S, Lang>) -> Self {
    E
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for E {
  fn from(_: TooFew<S, Lang>) -> Self {
    E
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for E {
  fn from(_: TooMany<S, Lang>) -> Self {
    E
  }
}

impl From<UnexpectedEot> for E {
  fn from(_: UnexpectedEot) -> Self {
    E
  }
}

impl<'inp> FromSeparatedError<'inp, TestLexer<'inp>> for E {
  fn from_missing_separator(_: CowStr, _: MissingTokenOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    E
  }

  fn from_missing_element(_: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    E
  }
}

impl<'inp> FromUnexpectedLeadingSeparatorError<'inp, TestLexer<'inp>> for E {
  fn from_unexpected_leading_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    E
  }
}

impl<'inp> FromUnexpectedTrailingSeparatorError<'inp, TestLexer<'inp>> for E {
  fn from_unexpected_trailing_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    E
  }
}

// ── Full emitter for require_* variants ───────────────────────────────────────

struct FullEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for FullEmitter {
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

impl<'inp> SeparatedEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }

  fn emit_missing_element(&mut self, _: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> FullContainerEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_full_container(
    &mut self,
    _: FullContainer<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> TooFewEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_too_few(&mut self, _: TooFew<<TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> TooManyEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_too_many(&mut self, _: TooMany<<TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_unexpected_leading_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_trailing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

impl<'inp> MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_leading_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }
}

fn full_ctx() -> ParserContext<'static, TestLexer<'static>, FullEmitter> {
  ParserContext::new(FullEmitter)
}

// ── Element parser ────────────────────────────────────────────────────────────

fn try_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
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

// ═══════════════════════════════════════════════════════════════════════════════
// 1. allow_leading
// ═══════════════════════════════════════════════════════════════════════════════

// ── 1a. allow_leading unbounded ───────────────────────────────────────────────

fn parse_al<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_unbounded_ok() {
  let r: Vec<i64> = Parser::new().apply(parse_al).parse_str("[,1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_leading_unbounded_no_leading_ok() {
  let r: Vec<i64> = Parser::new().apply(parse_al).parse_str("[1,2]").unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 1b. allow_leading at_least ────────────────────────────────────────────────

fn parse_al_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_leading()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_at_least_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_al_at_least)
    .parse_str("[,1,2,3]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_leading_at_least_exact_min() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_al_at_least)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 1c. allow_leading at_most ─────────────────────────────────────────────────

fn parse_al_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_leading()
    .at_most(3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_at_most_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_al_at_most)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_leading_at_most_single() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_al_at_most)
    .parse_str("[,7]")
    .unwrap();
  assert_eq!(r, vec![7]);
}

// ── 1d. allow_leading bounded ─────────────────────────────────────────────────

fn parse_al_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_leading()
    .bounded(2, 4)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_bounded_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_al_bounded)
    .parse_str("[,1,2,3]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_leading_bounded_at_max() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_al_bounded)
    .parse_str("[,1,2,3,4]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3, 4]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. allow_trailing
// ═══════════════════════════════════════════════════════════════════════════════

// ── 2a. allow_trailing unbounded ──────────────────────────────────────────────

fn parse_at<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_trailing_unbounded_ok() {
  let r: Vec<i64> = Parser::new().apply(parse_at).parse_str("[1,2,3,]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_trailing_unbounded_no_trailing_ok() {
  let r: Vec<i64> = Parser::new().apply(parse_at).parse_str("[1,2]").unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 2b. allow_trailing at_least ───────────────────────────────────────────────

fn parse_at_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_at_at_least)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_trailing_at_least_exact_min() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_at_at_least)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 2c. allow_trailing at_most ────────────────────────────────────────────────

fn parse_at_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_most(3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_at_at_most)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_trailing_at_most_at_max() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_at_at_most)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 2d. allow_trailing bounded ────────────────────────────────────────────────

fn parse_at_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .bounded(2, 4)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_at_bounded)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_trailing_bounded_at_max() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_at_bounded)
    .parse_str("[1,2,3,4,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3, 4]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. require_leading
// ═══════════════════════════════════════════════════════════════════════════════

// ── 3a. require_leading unbounded ─────────────────────────────────────────────

fn parse_rl<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_unbounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rl)
    .parse_str("[,1,2,3]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_unbounded_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rl)
    .parse_str("[1,2,3]");
  assert!(r.is_err());
}

// ── 3b. require_leading at_least ──────────────────────────────────────────────

fn parse_rl_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_leading()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rl_at_least)
    .parse_str("[,1,2,3]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_at_least_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rl_at_least)
    .parse_str("[1,2,3]");
  assert!(r.is_err());
}

// ── 3c. require_leading at_most ───────────────────────────────────────────────

fn parse_rl_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_leading()
    .at_most(3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rl_at_most)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn require_leading_at_most_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rl_at_most)
    .parse_str("[1,2]");
  assert!(r.is_err());
}

// ── 3d. require_leading bounded ───────────────────────────────────────────────

fn parse_rl_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_leading()
    .bounded(2, 4)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rl_bounded)
    .parse_str("[,1,2,3]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_bounded_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rl_bounded)
    .parse_str("[1,2,3]");
  assert!(r.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. require_trailing
// ═══════════════════════════════════════════════════════════════════════════════

// ── 4a. require_trailing unbounded ────────────────────────────────────────────

fn parse_rt<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_trailing_unbounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rt)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_trailing_unbounded_empty_ok() {
  // Empty brackets should succeed (no elements, no trailing needed)
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rt)
    .parse_str("[]")
    .unwrap();
  assert_eq!(r, vec![]);
}

// ── 4b. require_trailing at_least ─────────────────────────────────────────────

fn parse_rt_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rt_at_least)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_trailing_at_least_exact_min() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rt_at_least)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 4c. require_trailing at_most ──────────────────────────────────────────────

fn parse_rt_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .at_most(3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rt_at_most)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn require_trailing_at_most_at_max() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rt_at_most)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 4d. require_trailing bounded ──────────────────────────────────────────────

fn parse_rt_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .bounded(2, 4)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rt_bounded)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_trailing_bounded_at_max() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rt_bounded)
    .parse_str("[1,2,3,4,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3, 4]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. allow_surrounded (allow_trailing + allow_leading)
// ═══════════════════════════════════════════════════════════════════════════════

// ── 5a. allow_surrounded unbounded ────────────────────────────────────────────

fn parse_as<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_surrounded_unbounded_both_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_as)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_surrounded_unbounded_neither_ok() {
  let r: Vec<i64> = Parser::new().apply(parse_as).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 5b. allow_surrounded at_least ─────────────────────────────────────────────

fn parse_as_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_least(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_as_at_least)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_surrounded_at_least_exact_min() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_as_at_least)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 5c. allow_surrounded at_most ──────────────────────────────────────────────

fn parse_as_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_most(3)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_as_at_most)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_surrounded_at_most_at_max() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_as_at_most)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 5d. allow_surrounded bounded ──────────────────────────────────────────────

fn parse_as_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .bounded(2, 4)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_as_bounded)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_surrounded_bounded_at_max() {
  let r: Vec<i64> = Parser::new()
    .apply(parse_as_bounded)
    .parse_str("[,1,2,3,4,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3, 4]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. require_surrounded (require_trailing + require_leading)
// ═══════════════════════════════════════════════════════════════════════════════

// ── 6a. require_surrounded unbounded ──────────────────────────────────────────

fn parse_rs<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_surrounded_unbounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rs)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_surrounded_unbounded_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rs)
    .parse_str("[1,2,3,]");
  assert!(r.is_err());
}

// ── 6b. require_surrounded at_least ───────────────────────────────────────────

fn parse_rs_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rs_at_least)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_surrounded_at_least_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rs_at_least)
    .parse_str("[1,2,3,]");
  assert!(r.is_err());
}

// ── 6c. require_surrounded at_most ────────────────────────────────────────────

fn parse_rs_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .at_most(3)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rs_at_most)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn require_surrounded_at_most_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rs_at_most)
    .parse_str("[1,2,]");
  assert!(r.is_err());
}

// ── 6d. require_surrounded bounded ────────────────────────────────────────────

fn parse_rs_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .bounded(2, 4)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rs_bounded)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_surrounded_bounded_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rs_bounded)
    .parse_str("[1,2,3,]");
  assert!(r.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. allow_leading_require_trailing (require_trailing + allow_leading)
// ═══════════════════════════════════════════════════════════════════════════════

// ── 7a. allow_leading_require_trailing unbounded ──────────────────────────────

fn parse_alrt<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_require_trailing_unbounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_alrt)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_leading_require_trailing_unbounded_no_leading_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_alrt)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 7b. allow_leading_require_trailing at_least ───────────────────────────────

fn parse_alrt_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_alrt_at_least)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_leading_require_trailing_at_least_three() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_alrt_at_least)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 7c. allow_leading_require_trailing at_most ────────────────────────────────

fn parse_alrt_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .at_most(3)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_alrt_at_most)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_leading_require_trailing_at_most_at_max() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_alrt_at_most)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 7d. allow_leading_require_trailing bounded ────────────────────────────────

fn parse_alrt_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .bounded(2, 4)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_alrt_bounded)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_leading_require_trailing_bounded_at_max() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_alrt_bounded)
    .parse_str("[,1,2,3,4,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3, 4]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. require_leading_allow_trailing (allow_trailing + require_leading)
// ═══════════════════════════════════════════════════════════════════════════════

// ── 8a. require_leading_allow_trailing unbounded ──────────────────────────────

fn parse_rlat<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_allow_trailing_unbounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rlat)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_allow_trailing_unbounded_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rlat)
    .parse_str("[1,2,3,]");
  assert!(r.is_err());
}

// ── 8b. require_leading_allow_trailing at_least ───────────────────────────────

fn parse_rlat_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_least(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rlat_at_least)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn require_leading_allow_trailing_at_least_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rlat_at_least)
    .parse_str("[1,2,]");
  assert!(r.is_err());
}

// ── 8c. require_leading_allow_trailing at_most ────────────────────────────────

fn parse_rlat_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_most(3)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rlat_at_most)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn require_leading_allow_trailing_at_most_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rlat_at_most)
    .parse_str("[1,2,]");
  assert!(r.is_err());
}

// ── 8d. require_leading_allow_trailing bounded ────────────────────────────────

fn parse_rlat_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .bounded(2, 4)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_rlat_bounded)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_allow_trailing_bounded_missing_leading_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_rlat_bounded)
    .parse_str("[1,2,3,]");
  assert!(r.is_err());
}
