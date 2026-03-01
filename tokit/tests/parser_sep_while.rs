#![cfg(all(feature = "std", feature = "logos"))]

//! Tests for the `SeparatedWhile` (condition-closure-based) combinator.
//!
//! Exercises `separated_by_comma_while` with plain, `at_least`, `at_most`,
//! `bounded`, `allow_trailing`, `allow_leading`, and combined variants.
//!
//! # Sentinel token
//!
//! `SeparatedWhile::parse` calls `peek_with_emitter` after every failed
//! separator attempt.  At EOF there is nothing to peek, which triggers a
//! debug_assert.  We therefore append `+` (a non-comma, non-Num token) to
//! every test string so the condition always sees a stop token instead of
//! hitting EOF.  The trailing `+` is left unconsumed; `parse_str` does not
//! require all tokens to be consumed.

mod common;

use generic_arraydeque::typenum::U1;
use tokit::{
  Accumulator, Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser,
  cache::Peeked,
  emitter::{
    FromSeparatedError, FromUnexpectedLeadingSeparatorError, FromUnexpectedTrailingSeparatorError,
    FullContainerEmitter, SeparatedEmitter, TooFewEmitter, TooManyEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  error::{
    syntax::{FullContainer, MissingSyntaxOf, TooFew, TooMany},
    token::{MissingTokenOf, UnexpectedToken, UnexpectedTokenOf},
  },
  parser::Action,
  utils::CowStr,
};

use common::{TestLexer, Token};

// ── Local error type (satisfies orphan rule for separator traits) ─────────────

#[derive(Debug)]
struct WhileError;

impl From<()> for WhileError {
  fn from(_: ()) -> Self {
    WhileError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>>
  for WhileError
{
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    WhileError
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for WhileError {
  fn from(_: FullContainer<S, Lang>) -> Self {
    WhileError
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for WhileError {
  fn from(_: TooFew<S, Lang>) -> Self {
    WhileError
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for WhileError {
  fn from(_: TooMany<S, Lang>) -> Self {
    WhileError
  }
}

impl<'inp> FromSeparatedError<'inp, TestLexer<'inp>> for WhileError {
  fn from_missing_separator(_name: CowStr, _err: MissingTokenOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    WhileError
  }

  fn from_missing_element(_err: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    WhileError
  }
}

impl<'inp> FromUnexpectedLeadingSeparatorError<'inp, TestLexer<'inp>> for WhileError {
  fn from_unexpected_leading_separator(
    _name: CowStr,
    _err: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    WhileError
  }
}

impl<'inp> FromUnexpectedTrailingSeparatorError<'inp, TestLexer<'inp>> for WhileError {
  fn from_unexpected_trailing_separator(
    _name: CowStr,
    _err: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    WhileError
  }
}

// ── Supertrait bundling common emitter bounds ─────────────────────────────────

trait WhileEmitter<'inp>:
  Emitter<'inp, TestLexer<'inp>, Error = WhileError>
  + SeparatedEmitter<'inp, TestLexer<'inp>>
  + FullContainerEmitter<'inp, TestLexer<'inp>>
  + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
  + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
{
}

impl<'inp, E> WhileEmitter<'inp> for E where
  E: Emitter<'inp, TestLexer<'inp>, Error = WhileError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
{
}

// ── Condition: continue iff the next token is a number (element start) ────────
//
// The `SeparatedWhile` loop first tries to consume a separator (comma).
// The condition is called only when the separator was NOT found; at that
// point the peek buffer contains whatever comes next.  Returning `Continue`
// means "this token starts another element"; `Stop` means "we are done."
//
// Using a sentinel `+` token in test inputs ensures the condition always
// receives a non-EOF token, avoiding the debug_assert in `peek_with_emitter`.

fn decide_num<'inp, Ctx>(
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

// ── Element parser (ParseInput, not TryParseInput) ────────────────────────────

fn parse_num_while<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<i64, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = WhileError>,
{
  match inp.next()? {
    None => Err(WhileError),
    Some(tok) => match tok.into_data() {
      Token::Num(n) => Ok(n),
      _ => Err(WhileError),
    },
  }
}

// ── 1. Plain separated_by_comma_while (unbounded) ────────────────────────────

fn parse_while_list<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_basic() {
  // "1,2,3+" — the trailing `+` acts as the stop sentinel.
  let r: Result<Vec<i64>, WhileError> = Parser::new().apply(parse_while_list).parse_str("1,2,3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_single() {
  let r: Result<Vec<i64>, WhileError> = Parser::new().apply(parse_while_list).parse_str("42+");
  assert_eq!(r.unwrap(), vec![42]);
}

// ── 2. at_least ───────────────────────────────────────────────────────────────

fn parse_while_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_at_least_ok() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_at_least_2)
    .parse_str("1,2,3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_at_least_fail() {
  // Only 1 element; at_least(2) should fail.
  let r: Result<Vec<i64>, WhileError> = Parser::new().apply(parse_while_at_least_2).parse_str("1+");
  assert!(r.is_err());
}

// ── 3. at_most ────────────────────────────────────────────────────────────────

fn parse_while_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .at_most(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_at_most_ok() {
  let r: Result<Vec<i64>, WhileError> =
    Parser::new().apply(parse_while_at_most_2).parse_str("1,2+");
  assert_eq!(r.unwrap(), vec![1, 2]);
}

#[test]
fn test_sep_while_at_most_single() {
  let r: Result<Vec<i64>, WhileError> = Parser::new().apply(parse_while_at_most_2).parse_str("7+");
  assert_eq!(r.unwrap(), vec![7]);
}

// ── 4. bounded ────────────────────────────────────────────────────────────────

fn parse_while_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_bounded_ok() {
  let r: Result<Vec<i64>, WhileError> =
    Parser::new().apply(parse_while_bounded).parse_str("1,2,3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_bounded_too_few() {
  let r: Result<Vec<i64>, WhileError> = Parser::new().apply(parse_while_bounded).parse_str("1+");
  assert!(r.is_err());
}

// ── 5. allow_trailing ─────────────────────────────────────────────────────────

fn parse_while_allow_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_trailing_with_trailing() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_trailing)
    .parse_str("1,2,3,+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_trailing_without_trailing() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_trailing)
    .parse_str("1,2,3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

// ── 6. allow_leading ──────────────────────────────────────────────────────────

fn parse_while_allow_leading<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_leading_with_leading() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_leading)
    .parse_str(",1,2,3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_leading_without_leading() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_leading)
    .parse_str("1,2,3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

// ── 7. allow_surrounding (leading + trailing) ─────────────────────────────────

fn parse_while_allow_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_surrounded() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_surrounded)
    .parse_str(",1,2,3,+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

// ── 8. allow_trailing + at_least ──────────────────────────────────────────────

fn parse_while_allow_trailing_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_trailing_at_least_ok() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_trailing_at_least_2)
    .parse_str("1,2,3,+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_trailing_at_least_fail() {
  // Trailing comma, only 1 element; at_least(2) should fail.
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_trailing_at_least_2)
    .parse_str("1,+");
  assert!(r.is_err());
}

// ── 9. allow_surrounded + at_least ────────────────────────────────────────────

fn parse_while_allow_surrounded_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .at_least(2)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_surrounded_at_least_ok() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_surrounded_at_least_2)
    .parse_str(",1,2,3,+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_surrounded_at_least_fail() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_surrounded_at_least_2)
    .parse_str(",1,+");
  assert!(r.is_err());
}

// ── 10. allow_surrounded + at_most ────────────────────────────────────────────

fn parse_while_allow_surrounded_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .at_most(2)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_surrounded_at_most_ok() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_surrounded_at_most_2)
    .parse_str(",1,2,+");
  assert_eq!(r.unwrap(), vec![1, 2]);
}

// ── 11. allow_surrounded + bounded ────────────────────────────────────────────

fn parse_while_allow_surrounded_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .bounded(2, 4)
    .allow_leading()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_surrounded_bounded_ok() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_surrounded_bounded)
    .parse_str(",1,2,3,+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_surrounded_bounded_fail() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_surrounded_bounded)
    .parse_str(",1,+");
  assert!(r.is_err());
}

// ── 12. allow_leading + at_least ──────────────────────────────────────────────

fn parse_while_allow_leading_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_leading()
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_leading_at_least_ok() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_leading_at_least_2)
    .parse_str(",1,2,3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_leading_at_least_fail() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_leading_at_least_2)
    .parse_str(",1+");
  assert!(r.is_err());
}

// ── 13. allow_leading + at_most ───────────────────────────────────────────────

fn parse_while_allow_leading_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_leading()
    .at_most(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_leading_at_most_ok() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_leading_at_most_2)
    .parse_str(",1,2+");
  assert_eq!(r.unwrap(), vec![1, 2]);
}

// ── 14. allow_leading + bounded ───────────────────────────────────────────────

fn parse_while_allow_leading_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, WhileError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: WhileEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_leading()
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_allow_leading_bounded_ok() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_leading_bounded)
    .parse_str(",1,2,3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_sep_while_allow_leading_bounded_fail() {
  let r: Result<Vec<i64>, WhileError> = Parser::new()
    .apply(parse_while_allow_leading_bounded)
    .parse_str(",1+");
  assert!(r.is_err());
}
