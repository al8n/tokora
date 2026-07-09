#![cfg(all(feature = "std", feature = "logos"))]
mod common;

// Exhaustive tests for sep/parse code paths with every separator policy
// combined with every count modifier (at_least, at_most, bounded).

use tokit::{
  Accumulator, Emitter, InputRef, Parse, ParseContext, ParseInput, Parser, ParserContext,
  TryParseInput,
  emitter::{
    Fatal, FullContainerEmitter, MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter,
    SeparatedEmitter, TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
    UnexpectedTrailingSeparatorEmitter,
  },
  error::{
    UnexpectedEot,
    syntax::{FullContainer, MissingSyntax, TooFew, TooMany},
    token::{MissingToken, SeparatedError, UnexpectedToken},
  },
  try_parse_input::ParseAttempt,
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

impl<'a, Kind: Clone, O, Lang: ?Sized> From<MissingToken<'a, Kind, O, Lang>> for E {
  fn from(_: MissingToken<'a, Kind, O, Lang>) -> Self {
    E
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, Kind, S, Lang>> for E {
  fn from(_: SeparatedError<'a, T, Kind, S, Lang>) -> Self {
    E
  }
}

impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for E {
  fn from(_: MissingSyntax<O, Lang>) -> Self {
    E
  }
}

fn full_ctx() -> ParserContext<'static, TestLexer<'static>, Fatal<E>> {
  ParserContext::new(Fatal::new())
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

// ═════════════════════════════════════════════════════════════════════════════
// 1. No policy (plain separated)
// ═════════════════════════════════════════════════════════════════════════════

// ── 1a. at_least(2) ──────────────────────────────────────────────────────────

fn parse_plain_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn plain_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_plain_at_least)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn plain_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_plain_at_least)
    .parse_str("1");
  assert!(r.is_err());
}

// ── 1b. at_most(3) ──────────────────────────────────────────────────────────

fn parse_plain_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .at_most(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn plain_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_plain_at_most)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn plain_at_most_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_plain_at_most)
    .parse_str("1,2,3,4");
  assert!(r.is_err());
}

// ── 1c. bounded(2, 4) ──────────────────────────────────────────────────────

fn parse_plain_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
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
fn plain_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_plain_bounded)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn plain_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_plain_bounded)
    .parse_str("1");
  assert!(r.is_err());
}

#[test]
fn plain_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_plain_bounded)
    .parse_str("1,2,3,4,5");
  assert!(r.is_err());
}

// ═════════════════════════════════════════════════════════════════════════════
// 2. allow_leading
// ═════════════════════════════════════════════════════════════════════════════

// ── 2a. at_least(2) ──────────────────────────────────────────────────────────

fn parse_allow_leading_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_leading()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_at_least)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_leading_at_least_ok_no_leading() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_at_least)
    .parse_str("1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_leading_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_at_least)
    .parse_str(",1");
  assert!(r.is_err());
}

// ── 2b. at_most(3) ──────────────────────────────────────────────────────────

fn parse_allow_leading_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_leading()
    .at_most(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_at_most)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_leading_at_most_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_at_most)
    .parse_str(",1,2,3,4");
  assert!(r.is_err());
}

// ── 2c. bounded(2, 4) ──────────────────────────────────────────────────────

fn parse_allow_leading_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_leading()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_bounded)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_leading_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_bounded)
    .parse_str(",1");
  assert!(r.is_err());
}

#[test]
fn allow_leading_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_bounded)
    .parse_str(",1,2,3,4,5");
  assert!(r.is_err());
}

// ═════════════════════════════════════════════════════════════════════════════
// 3. allow_trailing
// ═════════════════════════════════════════════════════════════════════════════

// ── 3a. at_least(2) ──────────────────────────────────────────────────────────

fn parse_allow_trailing_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_trailing_at_least)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_trailing_at_least_ok_no_trailing() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_trailing_at_least)
    .parse_str("1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_trailing_at_least)
    .parse_str("1,");
  assert!(r.is_err());
}

// ── 3b. at_most(3) ──────────────────────────────────────────────────────────

fn parse_allow_trailing_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_most(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_trailing_at_most)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_trailing_at_most_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_trailing_at_most)
    .parse_str("1,2,3,4,");
  assert!(r.is_err());
}

// ── 3c. bounded(2, 4) ──────────────────────────────────────────────────────

fn parse_allow_trailing_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_trailing_bounded)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_trailing_bounded)
    .parse_str("1,");
  assert!(r.is_err());
}

#[test]
fn allow_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_trailing_bounded)
    .parse_str("1,2,3,4,5,");
  assert!(r.is_err());
}

// ═════════════════════════════════════════════════════════════════════════════
// 4. allow_surrounded (allow_trailing + allow_leading)
// ═════════════════════════════════════════════════════════════════════════════

// ── 4a. at_least(2) ──────────────────────────────────────────────────────────

fn parse_allow_surrounded_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_least(2)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_surrounded_at_least)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_surrounded_at_least_ok_no_surround() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_surrounded_at_least)
    .parse_str("1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_surrounded_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_surrounded_at_least)
    .parse_str(",1,");
  assert!(r.is_err());
}

// ── 4b. at_most(3) ──────────────────────────────────────────────────────────

fn parse_allow_surrounded_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .at_most(3)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_surrounded_at_most)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_surrounded_at_most_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_surrounded_at_most)
    .parse_str(",1,2,3,4,");
  assert!(r.is_err());
}

// ── 4c. bounded(2, 4) ──────────────────────────────────────────────────────

fn parse_allow_surrounded_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .allow_trailing()
    .bounded(2, 4)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_surrounded_bounded)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_surrounded_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_surrounded_bounded)
    .parse_str(",1,");
  assert!(r.is_err());
}

#[test]
fn allow_surrounded_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_surrounded_bounded)
    .parse_str(",1,2,3,4,5,");
  assert!(r.is_err());
}

// ═════════════════════════════════════════════════════════════════════════════
// 5. require_leading
// ═════════════════════════════════════════════════════════════════════════════

// ── 5a. at_least(2) ──────────────────────────────────────────────────────────

fn parse_require_leading_at_least<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_least)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_least)
    .parse_str(",1");
  assert!(r.is_err());
}

#[test]
fn require_leading_at_least_missing_leading() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_least)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 5b. at_most(3) ──────────────────────────────────────────────────────────

fn parse_require_leading_at_most<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_most)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_at_most_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_most)
    .parse_str(",1,2,3,4");
  assert!(r.is_err());
}

#[test]
fn require_leading_at_most_missing_leading() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_most)
    .parse_str("1,2");
  assert!(r.is_err());
}

// ── 5c. bounded(2, 4) ──────────────────────────────────────────────────────

fn parse_require_leading_bounded<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_bounded)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_bounded)
    .parse_str(",1");
  assert!(r.is_err());
}

#[test]
fn require_leading_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_bounded)
    .parse_str(",1,2,3,4,5");
  assert!(r.is_err());
}

// ═════════════════════════════════════════════════════════════════════════════
// 6. require_trailing
// ═════════════════════════════════════════════════════════════════════════════

// ── 6a. at_least(2) ──────────────────────────────────────────────────────────

fn parse_require_trailing_at_least<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_least)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_least)
    .parse_str("1,");
  assert!(r.is_err());
}

#[test]
fn require_trailing_at_least_missing_trailing() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_least)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 6b. at_most(3) ──────────────────────────────────────────────────────────

fn parse_require_trailing_at_most<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .at_most(3)
    .collect()
    .parse_input(inp)
}

#[test]
fn require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_most)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_trailing_at_most_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_most)
    .parse_str("1,2,3,4,");
  assert!(r.is_err());
}

#[test]
fn require_trailing_at_most_missing_trailing() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_most)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 6c. bounded(2, 4) ──────────────────────────────────────────────────────

fn parse_require_trailing_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_bounded)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_bounded)
    .parse_str("1,");
  assert!(r.is_err());
}

#[test]
fn require_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_bounded)
    .parse_str("1,2,3,4,5,");
  assert!(r.is_err());
}

// ═════════════════════════════════════════════════════════════════════════════
// 7. require_surrounded (require_trailing + require_leading)
// ═════════════════════════════════════════════════════════════════════════════

// ── 7a. at_least(2) ──────────────────────────────────────────────────────────

fn parse_require_surrounded_at_least<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn require_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_least)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_surrounded_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_least)
    .parse_str(",1,");
  assert!(r.is_err());
}

#[test]
fn require_surrounded_at_least_missing_leading() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_least)
    .parse_str("1,2,3,");
  assert!(r.is_err());
}

// ── 7b. at_most(3) ──────────────────────────────────────────────────────────

fn parse_require_surrounded_at_most<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn require_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_most)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_surrounded_at_most_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_most)
    .parse_str(",1,2,3,4,");
  assert!(r.is_err());
}

#[test]
fn require_surrounded_at_most_missing_trailing() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_most)
    .parse_str(",1,2,3");
  assert!(r.is_err());
}

// ── 7c. bounded(2, 4) ──────────────────────────────────────────────────────

fn parse_require_surrounded_bounded<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn require_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_bounded)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_surrounded_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_bounded)
    .parse_str(",1,");
  assert!(r.is_err());
}

#[test]
fn require_surrounded_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_bounded)
    .parse_str(",1,2,3,4,5,");
  assert!(r.is_err());
}

// ═════════════════════════════════════════════════════════════════════════════
// 8. allow_leading_require_trailing (require_trailing + allow_leading)
// ═════════════════════════════════════════════════════════════════════════════

// ── 8a. at_least(2) ──────────────────────────────────────────────────────────

fn parse_allow_leading_require_trailing_at_least<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_least)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_leading_require_trailing_at_least_ok_no_leading() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_least)
    .parse_str("1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_leading_require_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_least)
    .parse_str(",1,");
  assert!(r.is_err());
}

#[test]
fn allow_leading_require_trailing_at_least_missing_trailing() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_least)
    .parse_str(",1,2");
  assert!(r.is_err());
}

// ── 8b. at_most(3) ──────────────────────────────────────────────────────────

fn parse_allow_leading_require_trailing_at_most<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_most)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn allow_leading_require_trailing_at_most_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_most)
    .parse_str(",1,2,3,4,");
  assert!(r.is_err());
}

#[test]
fn allow_leading_require_trailing_at_most_missing_trailing() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_at_most)
    .parse_str(",1,2,3");
  assert!(r.is_err());
}

// ── 8c. bounded(2, 4) ──────────────────────────────────────────────────────

fn parse_allow_leading_require_trailing_bounded<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn allow_leading_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_bounded)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn allow_leading_require_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_bounded)
    .parse_str(",1,");
  assert!(r.is_err());
}

#[test]
fn allow_leading_require_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_require_trailing_bounded)
    .parse_str(",1,2,3,4,5,");
  assert!(r.is_err());
}

// ═════════════════════════════════════════════════════════════════════════════
// 9. require_leading_allow_trailing (allow_trailing + require_leading)
// ═════════════════════════════════════════════════════════════════════════════

// ── 9a. at_least(2) ──────────────────────────────────────────────────────────

fn parse_require_leading_allow_trailing_at_least<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_least)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn require_leading_allow_trailing_at_least_ok_no_trailing() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_least)
    .parse_str(",1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn require_leading_allow_trailing_at_least_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_least)
    .parse_str(",1");
  assert!(r.is_err());
}

#[test]
fn require_leading_allow_trailing_at_least_missing_leading() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_least)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 9b. at_most(3) ──────────────────────────────────────────────────────────

fn parse_require_leading_allow_trailing_at_most<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_most)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn require_leading_allow_trailing_at_most_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_most)
    .parse_str(",1,2,3,4,");
  assert!(r.is_err());
}

#[test]
fn require_leading_allow_trailing_at_most_missing_leading() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_at_most)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

// ── 9c. bounded(2, 4) ──────────────────────────────────────────────────────

fn parse_require_leading_allow_trailing_bounded<'inp, Ctx>(
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
    .collect()
    .parse_input(inp)
}

#[test]
fn require_leading_allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_bounded)
    .parse_str(",1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_allow_trailing_bounded_ok_no_trailing() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_bounded)
    .parse_str(",1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn require_leading_allow_trailing_bounded_too_few() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_bounded)
    .parse_str(",1");
  assert!(r.is_err());
}

#[test]
fn require_leading_allow_trailing_bounded_too_many() {
  let r: Result<Vec<i64>, _> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_allow_trailing_bounded)
    .parse_str(",1,2,3,4,5,");
  assert!(r.is_err());
}

// ═════════════════════════════════════════════════════════════════════════════
// Boundary / edge case tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn plain_at_least_exactly_min() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_plain_at_least)
    .parse_str("1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn plain_at_most_exactly_max() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_plain_at_most)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn plain_at_most_single() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_plain_at_most)
    .parse_str("1")
    .unwrap();
  assert_eq!(r, vec![1]);
}

#[test]
fn plain_bounded_exactly_min() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_plain_bounded)
    .parse_str("1,2")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn plain_bounded_exactly_max() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_plain_bounded)
    .parse_str("1,2,3,4")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3, 4]);
}

#[test]
fn allow_trailing_at_most_single_with_trailing() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_trailing_at_most)
    .parse_str("1,")
    .unwrap();
  assert_eq!(r, vec![1]);
}

#[test]
fn require_surrounded_bounded_exactly_min() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_bounded)
    .parse_str(",1,2,")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn require_surrounded_bounded_exactly_max() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_bounded)
    .parse_str(",1,2,3,4,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3, 4]);
}

#[test]
fn allow_leading_at_most_single() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_allow_leading_at_most)
    .parse_str(",1")
    .unwrap();
  assert_eq!(r, vec![1]);
}

#[test]
fn require_leading_at_most_single() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_leading_at_most)
    .parse_str(",1")
    .unwrap();
  assert_eq!(r, vec![1]);
}

#[test]
fn require_trailing_at_most_single() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_trailing_at_most)
    .parse_str("1,")
    .unwrap();
  assert_eq!(r, vec![1]);
}

#[test]
fn require_surrounded_at_most_single() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(parse_require_surrounded_at_most)
    .parse_str(",1,")
    .unwrap();
  assert_eq!(r, vec![1]);
}
