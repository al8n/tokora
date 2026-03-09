#![cfg(all(feature = "std", feature = "logos"))]
mod common;

// Exhaustive tests for sep/parse code paths with every separator policy
// combined with every count modifier (at_least, at_most, bounded).

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

// ── Full emitter ──────────────────────────────────────────────────────────────

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
