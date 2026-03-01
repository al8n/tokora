//! Tests for delimited separator parsing.
//!
//! Covers `sep/delim/` and `sep_while/delim/` files:
//! - `try_num_sep.separated_by_comma().delimited::<Bracket>().collect()` (sep/delim)
//! - `parse_num_while.separated_by_comma_while(decide).delimited::<Bracket>().collect()` (sep_while/delim)
//! - Variants: plain, allow_trailing, allow_leading, allow_surrounded,
//!   require_trailing, require_leading, allow_leading_require_trailing,
//!   require_leading_allow_trailing, require_surrounded
//! - Count modifiers: at_least, at_most, bounded on each variant

mod common;

use generic_arraydeque::typenum::U1;
use tokit::{
  Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, SimpleSpan, TryParseInput,
  cache::Peeked,
  emitter::{
    FullContainerEmitter, FromSeparatedError, FromUnexpectedLeadingSeparatorError,
    FromUnexpectedTrailingSeparatorError, MissingLeadingSeparatorEmitter,
    MissingTrailingSeparatorEmitter, SeparatedEmitter, TooFewEmitter, TooManyEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  error::{
    UnexpectedEot,
    syntax::{FullContainer, MissingSyntaxOf, TooFew, TooMany},
    token::{MissingTokenOf, UnexpectedToken, UnexpectedTokenOf},
  },
  input::Cursor,
  parse_context::ParserContext,
  parser::Action,
  punct::Bracket,
  try_parse_input::ParseAttempt,
  utils::CowStr,
};

use common::{TestLexer, Token};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct DelimError;

impl From<()> for DelimError {
  fn from(_: ()) -> Self {
    DelimError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>>
  for DelimError
{
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    DelimError
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for DelimError {
  fn from(_: FullContainer<S, Lang>) -> Self {
    DelimError
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for DelimError {
  fn from(_: TooFew<S, Lang>) -> Self {
    DelimError
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for DelimError {
  fn from(_: TooMany<S, Lang>) -> Self {
    DelimError
  }
}

impl From<UnexpectedEot> for DelimError {
  fn from(_: UnexpectedEot) -> Self {
    DelimError
  }
}

impl<'inp> FromSeparatedError<'inp, TestLexer<'inp>> for DelimError {
  fn from_missing_separator(
    _name: CowStr,
    _err: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    DelimError
  }

  fn from_missing_element(_err: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    DelimError
  }
}

impl<'inp> FromUnexpectedLeadingSeparatorError<'inp, TestLexer<'inp>> for DelimError {
  fn from_unexpected_leading_separator(
    _name: CowStr,
    _err: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    DelimError
  }
}

impl<'inp> FromUnexpectedTrailingSeparatorError<'inp, TestLexer<'inp>> for DelimError {
  fn from_unexpected_trailing_separator(
    _name: CowStr,
    _err: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    DelimError
  }
}

// ── Full emitter (adds MissingTrailing/LeadingSeparatorEmitter) ────────────────
//
// `Fatal<DelimError>` does not implement MissingTrailingSeparatorEmitter or
// MissingLeadingSeparatorEmitter. For `require_*` variants we need a custom
// emitter that implements all of the required traits.

struct FullEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for FullEmitter {
  type Error = DelimError;

  fn emit_lexer_error(
    &mut self,
    _: tokit::span::Spanned<(), SimpleSpan>,
  ) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }

  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }

  fn emit_error(
    &mut self,
    err: tokit::span::Spanned<DelimError, SimpleSpan>,
  ) -> Result<(), DelimError>
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
  ) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }

  fn emit_missing_element(
    &mut self,
    _: MissingSyntaxOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }
}

impl<'inp> FullContainerEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_full_container(
    &mut self,
    _: FullContainer<SimpleSpan>,
  ) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }
}

impl<'inp> TooFewEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_too_few(&mut self, _: TooFew<SimpleSpan>) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }
}

impl<'inp> TooManyEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_too_many(&mut self, _: TooMany<SimpleSpan>) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }
}

impl<'inp> UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_unexpected_leading_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }
}

impl<'inp> UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }
}

impl<'inp> MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_trailing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }
}

impl<'inp> MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for FullEmitter {
  fn emit_missing_leading_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), DelimError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(DelimError)
  }
}

// ── Emitter supertraits ───────────────────────────────────────────────────────

/// Standard emitter bounds for `allow_*` and `plain` delimited parsers.
trait DelimEmitter<'inp>:
  Emitter<'inp, TestLexer<'inp>, Error = DelimError>
  + SeparatedEmitter<'inp, TestLexer<'inp>>
  + FullContainerEmitter<'inp, TestLexer<'inp>>
  + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
  + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
{
}

impl<'inp, E> DelimEmitter<'inp> for E where
  E: Emitter<'inp, TestLexer<'inp>, Error = DelimError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
{
}

/// Emitter bounds for `require_*` variants (adds MissingTrailing/Leading).
trait FullDelimEmitter<'inp>:
  DelimEmitter<'inp>
  + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
  + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
  + TooFewEmitter<'inp, TestLexer<'inp>>
  + TooManyEmitter<'inp, TestLexer<'inp>>
{
}

impl<'inp, E> FullDelimEmitter<'inp> for E where
  E: DelimEmitter<'inp>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>
{
}

// ── Helper: create a Parser with FullEmitter context ─────────────────────────

fn full_ctx() -> ParserContext<'static, TestLexer<'static>, FullEmitter> {
  ParserContext::new(FullEmitter)
}

// ── Element parsers ────────────────────────────────────────────────────────────

/// TryParseInput: returns Accept(n) or Decline for a number token.
fn try_num_delim<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = DelimError>,
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

/// ParseInput: parses a number (used for sep_while variants).
fn parse_num_while<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<i64, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = DelimError>,
{
  match inp.next()? {
    None => Err(DelimError),
    Some(tok) => match tok.into_data() {
      Token::Num(n) => Ok(n),
      _ => Err(DelimError),
    },
  }
}

/// Decision condition for sep_while: Continue if next peeked token is Num, Stop otherwise.
fn decide_num<'inp, Ctx>(
  mut peeked: Peeked<'_, 'inp, TestLexer<'inp>, U1>,
  _: &mut Ctx::Emitter,
) -> Result<Action, DelimError>
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

// ════════════════════════════════════════════════════════════════════════════
// sep/delim/ tests
// ════════════════════════════════════════════════════════════════════════════

// ── 1. Unbounded (plain) ──────────────────────────────────────────────────────

fn sep_delim_plain<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_basic() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_plain).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_empty() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_plain).parse_str("[]").unwrap();
  assert_eq!(r, vec![]);
}

#[test]
fn test_sep_delim_single() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_plain).parse_str("[42]").unwrap();
  assert_eq!(r, vec![42]);
}

// ── 2. at_least ───────────────────────────────────────────────────────────────

fn sep_delim_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_at_least_ok() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_at_least_2).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_at_least_fail() {
  let r: Result<Vec<i64>, DelimError> =
    Parser::new().apply(sep_delim_at_least_2).parse_str("[1]");
  assert!(r.is_err());
}

// ── 3. at_most ────────────────────────────────────────────────────────────────

fn sep_delim_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_at_most_ok() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_at_most_2).parse_str("[1,2]").unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_delim_at_most_single() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_at_most_2).parse_str("[7]").unwrap();
  assert_eq!(r, vec![7]);
}

// ── 4. bounded ────────────────────────────────────────────────────────────────

fn sep_delim_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_bounded_ok() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_bounded_1_3).parse_str("[1,2]").unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_delim_bounded_fail_too_few() {
  let r: Result<Vec<i64>, DelimError> =
    Parser::new().apply(sep_delim_bounded_1_3).parse_str("[]");
  assert!(r.is_err());
}

// ── 5. allow_trailing ─────────────────────────────────────────────────────────

fn sep_delim_allow_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_trailing_with_trailing() {
  let r: Vec<i64> =
    Parser::new().apply(sep_delim_allow_trailing).parse_str("[1,2,3,]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_allow_trailing_without_trailing() {
  let r: Vec<i64> =
    Parser::new().apply(sep_delim_allow_trailing).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_allow_trailing_empty() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_allow_trailing).parse_str("[]").unwrap();
  assert_eq!(r, vec![]);
}

// ── 6. allow_trailing + at_least ──────────────────────────────────────────────

fn sep_delim_allow_trailing_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_delim_allow_trailing_at_least_2)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_allow_trailing_at_least_fail() {
  let r: Result<Vec<i64>, DelimError> =
    Parser::new().apply(sep_delim_allow_trailing_at_least_2).parse_str("[1,]");
  assert!(r.is_err());
}

// ── 7. allow_trailing + at_most ───────────────────────────────────────────────

fn sep_delim_allow_trailing_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_delim_allow_trailing_at_most_2)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 8. allow_trailing + bounded ───────────────────────────────────────────────

fn sep_delim_allow_trailing_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_delim_allow_trailing_bounded_1_3)
    .parse_str("[2,3,]")
    .unwrap();
  assert_eq!(r, vec![2, 3]);
}

// ── 9. allow_leading ──────────────────────────────────────────────────────────

fn sep_delim_allow_leading<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_leading_with_leading() {
  let r: Vec<i64> =
    Parser::new().apply(sep_delim_allow_leading).parse_str("[,1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_allow_leading_without_leading() {
  let r: Vec<i64> =
    Parser::new().apply(sep_delim_allow_leading).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_allow_leading_empty() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_allow_leading).parse_str("[]").unwrap();
  assert_eq!(r, vec![]);
}

// ── 10. allow_leading + at_least ──────────────────────────────────────────────

fn sep_delim_allow_leading_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .allow_leading()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_leading_at_least_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_delim_allow_leading_at_least_2)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

#[test]
fn test_sep_delim_allow_leading_at_least_fail() {
  let r: Result<Vec<i64>, DelimError> =
    Parser::new().apply(sep_delim_allow_leading_at_least_2).parse_str("[,1]");
  assert!(r.is_err());
}

// ── 11. allow_leading + at_most ───────────────────────────────────────────────

fn sep_delim_allow_leading_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .allow_leading()
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_leading_at_most_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_delim_allow_leading_at_most_2)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 12. allow_leading + bounded ───────────────────────────────────────────────

fn sep_delim_allow_leading_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .allow_leading()
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_leading_bounded_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_delim_allow_leading_bounded_1_3)
    .parse_str("[,2,3]")
    .unwrap();
  assert_eq!(r, vec![2, 3]);
}

// ── 13. allow_surrounded (allow_trailing + allow_leading) ─────────────────────

fn sep_delim_allow_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_surrounded_both() {
  let r: Vec<i64> =
    Parser::new().apply(sep_delim_allow_surrounded).parse_str("[,1,2,3,]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_allow_surrounded_none() {
  let r: Vec<i64> =
    Parser::new().apply(sep_delim_allow_surrounded).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_allow_surrounded_empty() {
  let r: Vec<i64> = Parser::new().apply(sep_delim_allow_surrounded).parse_str("[]").unwrap();
  assert_eq!(r, vec![]);
}

// ── 14. allow_surrounded + at_least ───────────────────────────────────────────

fn sep_delim_allow_surrounded_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .at_least(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_delim_allow_surrounded_at_least_2)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_allow_surrounded_at_least_fail() {
  let r: Result<Vec<i64>, DelimError> =
    Parser::new().apply(sep_delim_allow_surrounded_at_least_2).parse_str("[,1,]");
  assert!(r.is_err());
}

// ── 15. allow_surrounded + at_most ────────────────────────────────────────────

fn sep_delim_allow_surrounded_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .at_most(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_delim_allow_surrounded_at_most_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 16. allow_surrounded + bounded ────────────────────────────────────────────

fn sep_delim_allow_surrounded_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .bounded(1, 3)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_delim_allow_surrounded_bounded_1_3)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 17. require_trailing ──────────────────────────────────────────────────────

fn sep_delim_require_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_trailing)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_require_trailing_missing() {
  let r: Result<Vec<i64>, DelimError> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_trailing)
    .parse_str("[1,2,3]");
  assert!(r.is_err());
}

// ── 18. require_trailing + at_least ───────────────────────────────────────────

fn sep_delim_require_trailing_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_trailing_at_least_2)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 19. require_trailing + at_most ────────────────────────────────────────────

fn sep_delim_require_trailing_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_trailing_at_most_2)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 20. require_trailing + bounded ────────────────────────────────────────────

fn sep_delim_require_trailing_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_trailing_bounded_1_3)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 21. require_leading ───────────────────────────────────────────────────────

fn sep_delim_require_leading<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_leading_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_leading)
    .parse_str("[,1,2,3]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_require_leading_missing() {
  let r: Result<Vec<i64>, DelimError> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_leading)
    .parse_str("[1,2,3]");
  assert!(r.is_err());
}

// ── 22. require_leading + at_least ────────────────────────────────────────────

fn sep_delim_require_leading_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_leading()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_leading_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_leading_at_least_2)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 23. require_leading + at_most ─────────────────────────────────────────────

fn sep_delim_require_leading_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_leading()
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_leading_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_leading_at_most_2)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 24. require_leading + bounded ─────────────────────────────────────────────

fn sep_delim_require_leading_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_leading()
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_leading_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_leading_bounded_1_3)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 25. allow_leading_require_trailing ────────────────────────────────────────

fn sep_delim_allow_leading_require_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_leading_require_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_allow_leading_require_trailing)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 26. allow_leading_require_trailing + at_least ─────────────────────────────

fn sep_delim_allow_leading_require_trailing_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_leading_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_allow_leading_require_trailing_at_least_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 27. allow_leading_require_trailing + at_most ──────────────────────────────

fn sep_delim_allow_leading_require_trailing_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .at_most(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_leading_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_allow_leading_require_trailing_at_most_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 28. allow_leading_require_trailing + bounded ──────────────────────────────

fn sep_delim_allow_leading_require_trailing_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .bounded(1, 3)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_allow_leading_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_allow_leading_require_trailing_bounded_1_3)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 29. require_leading_allow_trailing ────────────────────────────────────────

fn sep_delim_require_leading_allow_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_leading_allow_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_leading_allow_trailing)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 30. require_leading_allow_trailing + at_least ─────────────────────────────

fn sep_delim_require_leading_allow_trailing_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .at_least(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_leading_allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_leading_allow_trailing_at_least_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 31. require_leading_allow_trailing + at_most ──────────────────────────────

fn sep_delim_require_leading_allow_trailing_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .at_most(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_leading_allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_leading_allow_trailing_at_most_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 32. require_leading_allow_trailing + bounded ──────────────────────────────

fn sep_delim_require_leading_allow_trailing_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .allow_trailing()
    .bounded(1, 3)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_leading_allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_leading_allow_trailing_bounded_1_3)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 33. require_surrounded ────────────────────────────────────────────────────

fn sep_delim_require_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_surrounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_surrounded)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_delim_require_surrounded_missing_leading() {
  let r: Result<Vec<i64>, DelimError> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_surrounded)
    .parse_str("[1,2,3,]");
  assert!(r.is_err());
}

// ── 34. require_surrounded + at_least ────────────────────────────────────────

fn sep_delim_require_surrounded_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .at_least(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_surrounded_at_least_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 35. require_surrounded + at_most ─────────────────────────────────────────

fn sep_delim_require_surrounded_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .at_most(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_surrounded_at_most_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 36. require_surrounded + bounded ─────────────────────────────────────────

fn sep_delim_require_surrounded_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  try_num_delim
    .separated_by_comma()
    .require_trailing()
    .bounded(1, 3)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_delim_require_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_delim_require_surrounded_bounded_1_3)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ════════════════════════════════════════════════════════════════════════════
// sep_while/delim/ tests
// ════════════════════════════════════════════════════════════════════════════

// ── 37. Unbounded (plain) while ───────────────────────────────────────────────

fn sep_while_delim_plain<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_basic() {
  let r: Vec<i64> = Parser::new().apply(sep_while_delim_plain).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_delim_empty() {
  let r: Vec<i64> = Parser::new().apply(sep_while_delim_plain).parse_str("[]").unwrap();
  assert_eq!(r, vec![]);
}

#[test]
fn test_sep_while_delim_single() {
  let r: Vec<i64> = Parser::new().apply(sep_while_delim_plain).parse_str("[99]").unwrap();
  assert_eq!(r, vec![99]);
}

// ── 38. while + at_least ──────────────────────────────────────────────────────

fn sep_while_delim_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_at_least_ok() {
  let r: Vec<i64> =
    Parser::new().apply(sep_while_delim_at_least_2).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_delim_at_least_fail() {
  let r: Result<Vec<i64>, DelimError> =
    Parser::new().apply(sep_while_delim_at_least_2).parse_str("[1]");
  assert!(r.is_err());
}

// ── 39. while + at_most ───────────────────────────────────────────────────────

fn sep_while_delim_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_at_most_ok() {
  let r: Vec<i64> =
    Parser::new().apply(sep_while_delim_at_most_2).parse_str("[1,2]").unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 40. while + bounded ───────────────────────────────────────────────────────

fn sep_while_delim_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_bounded_ok() {
  let r: Vec<i64> =
    Parser::new().apply(sep_while_delim_bounded_1_3).parse_str("[1,2]").unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 41. while + allow_trailing ────────────────────────────────────────────────

fn sep_while_delim_allow_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_trailing_with_trailing() {
  let r: Vec<i64> =
    Parser::new().apply(sep_while_delim_allow_trailing).parse_str("[1,2,3,]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_delim_allow_trailing_without_trailing() {
  let r: Vec<i64> =
    Parser::new().apply(sep_while_delim_allow_trailing).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 42. while + allow_trailing + at_least ─────────────────────────────────────

fn sep_while_delim_allow_trailing_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_while_delim_allow_trailing_at_least_2)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 43. while + allow_trailing + at_most ──────────────────────────────────────

fn sep_while_delim_allow_trailing_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_while_delim_allow_trailing_at_most_2)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 44. while + allow_trailing + bounded ──────────────────────────────────────

fn sep_while_delim_allow_trailing_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_while_delim_allow_trailing_bounded_1_3)
    .parse_str("[2,3,]")
    .unwrap();
  assert_eq!(r, vec![2, 3]);
}

// ── 45. while + allow_leading ─────────────────────────────────────────────────

fn sep_while_delim_allow_leading<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_leading_with_leading() {
  let r: Vec<i64> =
    Parser::new().apply(sep_while_delim_allow_leading).parse_str("[,1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_delim_allow_leading_without_leading() {
  let r: Vec<i64> =
    Parser::new().apply(sep_while_delim_allow_leading).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 46. while + allow_leading + at_least ──────────────────────────────────────

fn sep_while_delim_allow_leading_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_leading()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_leading_at_least_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_while_delim_allow_leading_at_least_2)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 47. while + allow_leading + at_most ───────────────────────────────────────

fn sep_while_delim_allow_leading_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_leading()
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_leading_at_most_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_while_delim_allow_leading_at_most_2)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 48. while + allow_leading + bounded ───────────────────────────────────────

fn sep_while_delim_allow_leading_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_leading()
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_leading_bounded_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_while_delim_allow_leading_bounded_1_3)
    .parse_str("[,2,3]")
    .unwrap();
  assert_eq!(r, vec![2, 3]);
}

// ── 49. while + allow_surrounded ──────────────────────────────────────────────

fn sep_while_delim_allow_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_surrounded_both() {
  let r: Vec<i64> =
    Parser::new().apply(sep_while_delim_allow_surrounded).parse_str("[,1,2,3,]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_delim_allow_surrounded_none() {
  let r: Vec<i64> =
    Parser::new().apply(sep_while_delim_allow_surrounded).parse_str("[1,2,3]").unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 50. while + allow_surrounded + at_least ───────────────────────────────────

fn sep_while_delim_allow_surrounded_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .at_least(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_while_delim_allow_surrounded_at_least_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 51. while + allow_surrounded + at_most ────────────────────────────────────

fn sep_while_delim_allow_surrounded_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .at_most(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_while_delim_allow_surrounded_at_most_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 52. while + allow_surrounded + bounded ────────────────────────────────────

fn sep_while_delim_allow_surrounded_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: DelimEmitter<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .bounded(1, 3)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::new()
    .apply(sep_while_delim_allow_surrounded_bounded_1_3)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 53. while + require_trailing ─────────────────────────────────────────────

fn sep_while_delim_require_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_trailing)
    .parse_str("[1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_delim_require_trailing_missing() {
  let r: Result<Vec<i64>, DelimError> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_trailing)
    .parse_str("[1,2,3]");
  assert!(r.is_err());
}

// ── 54. while + require_trailing + at_least ───────────────────────────────────

fn sep_while_delim_require_trailing_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_trailing_at_least_2)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 55. while + require_trailing + at_most ────────────────────────────────────

fn sep_while_delim_require_trailing_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_trailing_at_most_2)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 56. while + require_trailing + bounded ────────────────────────────────────

fn sep_while_delim_require_trailing_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_trailing_bounded_1_3)
    .parse_str("[1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 57. while + require_leading ───────────────────────────────────────────────

fn sep_while_delim_require_leading<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_leading_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_leading)
    .parse_str("[,1,2,3]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_delim_require_leading_missing() {
  let r: Result<Vec<i64>, DelimError> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_leading)
    .parse_str("[1,2,3]");
  assert!(r.is_err());
}

// ── 58. while + require_leading + at_least ────────────────────────────────────

fn sep_while_delim_require_leading_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_leading()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_leading_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_leading_at_least_2)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 59. while + require_leading + at_most ─────────────────────────────────────

fn sep_while_delim_require_leading_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_leading()
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_leading_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_leading_at_most_2)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 60. while + require_leading + bounded ─────────────────────────────────────

fn sep_while_delim_require_leading_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_leading()
    .bounded(1, 3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_leading_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_leading_bounded_1_3)
    .parse_str("[,1,2]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 61. while + allow_leading_require_trailing ────────────────────────────────

fn sep_while_delim_allow_leading_require_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_leading_require_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_allow_leading_require_trailing)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 62. while + allow_leading_require_trailing + at_least ─────────────────────

fn sep_while_delim_allow_leading_require_trailing_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .at_least(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_leading_require_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_allow_leading_require_trailing_at_least_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 63. while + allow_leading_require_trailing + at_most ──────────────────────

fn sep_while_delim_allow_leading_require_trailing_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .at_most(2)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_leading_require_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_allow_leading_require_trailing_at_most_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 64. while + allow_leading_require_trailing + bounded ──────────────────────

fn sep_while_delim_allow_leading_require_trailing_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .bounded(1, 3)
    .allow_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_allow_leading_require_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_allow_leading_require_trailing_bounded_1_3)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 65. while + require_leading_allow_trailing ────────────────────────────────

fn sep_while_delim_require_leading_allow_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_leading_allow_trailing_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_leading_allow_trailing)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── 66. while + require_leading_allow_trailing + at_least ─────────────────────

fn sep_while_delim_require_leading_allow_trailing_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .at_least(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_leading_allow_trailing_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_leading_allow_trailing_at_least_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 67. while + require_leading_allow_trailing + at_most ──────────────────────

fn sep_while_delim_require_leading_allow_trailing_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .at_most(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_leading_allow_trailing_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_leading_allow_trailing_at_most_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 68. while + require_leading_allow_trailing + bounded ──────────────────────

fn sep_while_delim_require_leading_allow_trailing_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .allow_trailing()
    .bounded(1, 3)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_leading_allow_trailing_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_leading_allow_trailing_bounded_1_3)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 69. while + require_surrounded ───────────────────────────────────────────

fn sep_while_delim_require_surrounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_surrounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_surrounded)
    .parse_str("[,1,2,3,]")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

#[test]
fn test_sep_while_delim_require_surrounded_missing_leading() {
  let r: Result<Vec<i64>, DelimError> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_surrounded)
    .parse_str("[1,2,3,]");
  assert!(r.is_err());
}

// ── 70. while + require_surrounded + at_least ────────────────────────────────

fn sep_while_delim_require_surrounded_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .at_least(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_surrounded_at_least_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_surrounded_at_least_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 71. while + require_surrounded + at_most ─────────────────────────────────

fn sep_while_delim_require_surrounded_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .at_most(2)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_surrounded_at_most_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_surrounded_at_most_2)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}

// ── 72. while + require_surrounded + bounded ─────────────────────────────────

fn sep_while_delim_require_surrounded_bounded_1_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, DelimError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: FullDelimEmitter<'inp>,
{
  parse_num_while
    .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
    .require_trailing()
    .bounded(1, 3)
    .require_leading()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_sep_while_delim_require_surrounded_bounded_ok() {
  let r: Vec<i64> = Parser::with_context(full_ctx())
    .apply(sep_while_delim_require_surrounded_bounded_1_3)
    .parse_str("[,1,2,]")
    .unwrap();
  assert_eq!(r, vec![1, 2]);
}
