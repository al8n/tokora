#![cfg(all(feature = "std", feature = "logos"))]

//! Tests for `TryParseInput::repeated().delimited()` (repeated parser without
//! separator, wrapped in delimiters).
//!
//! Covers `parser/many/delim/repeated/` (unbounded, at_least, at_most, bounded).

mod common;

use tokit::{
  Accumulator, Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
  Token as TokenTrait, TryParseInput,
  emitter::{FullContainerEmitter, TooFewEmitter, TooManyEmitter},
  error::{
    UnexpectedEot,
    syntax::{FullContainer, TooFew, TooMany},
    token::{UnexpectedToken, UnexpectedTokenOf},
  },
  input::Cursor,
  punct::Bracket,
  span::Spanned,
  try_parse_input::ParseAttempt,
};

use common::{TestLexer, Token};

// -- Error type ---------------------------------------------------------------

#[derive(Debug)]
struct RDError;

impl From<()> for RDError {
  fn from(_: ()) -> Self {
    RDError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for RDError {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    RDError
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for RDError {
  fn from(_: FullContainer<S, Lang>) -> Self {
    RDError
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for RDError {
  fn from(_: TooFew<S, Lang>) -> Self {
    RDError
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for RDError {
  fn from(_: TooMany<S, Lang>) -> Self {
    RDError
  }
}

impl From<UnexpectedEot> for RDError {
  fn from(_: UnexpectedEot) -> Self {
    RDError
  }
}

// -- Custom emitter -----------------------------------------------------------

struct RDEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for RDEmitter {
  type Error = RDError;

  fn emit_lexer_error(
    &mut self,
    _: Spanned<
      <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
      <TestLexer<'inp> as Lexer<'inp>>::Span,
    >,
  ) -> Result<(), RDError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RDError)
  }

  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), RDError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RDError)
  }

  fn emit_error(
    &mut self,
    err: Spanned<RDError, <TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), RDError>
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

impl<'inp> FullContainerEmitter<'inp, TestLexer<'inp>> for RDEmitter {
  fn emit_full_container(
    &mut self,
    _: FullContainer<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), RDError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RDError)
  }
}

impl<'inp> TooFewEmitter<'inp, TestLexer<'inp>> for RDEmitter {
  fn emit_too_few(
    &mut self,
    _: TooFew<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), RDError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RDError)
  }
}

impl<'inp> TooManyEmitter<'inp, TestLexer<'inp>> for RDEmitter {
  fn emit_too_many(
    &mut self,
    _: TooMany<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), RDError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RDError)
  }
}

fn rd_ctx() -> ParserContext<'static, TestLexer<'static>, RDEmitter> {
  ParserContext::new(RDEmitter)
}

// -- Element parser (TryParseInput) -------------------------------------------

fn try_num_rd<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, RDError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RDError>,
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

// == 1. Plain repeated delimited (unbounded) ==================================

fn parse_rd_list<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RDError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RDError>
    + FullContainerEmitter<'inp, TestLexer<'inp>>,
{
  try_num_rd
    .repeated()
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_delimited_basic() {
  let r: Result<Vec<i64>, RDError> =
    Parser::with_context(rd_ctx()).apply(parse_rd_list).parse_str("[1 2 3]");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_repeated_delimited_empty() {
  let r: Result<Vec<i64>, RDError> =
    Parser::with_context(rd_ctx()).apply(parse_rd_list).parse_str("[]");
  assert_eq!(r.unwrap(), vec![]);
}

#[test]
fn test_repeated_delimited_single() {
  let r: Result<Vec<i64>, RDError> =
    Parser::with_context(rd_ctx()).apply(parse_rd_list).parse_str("[42]");
  assert_eq!(r.unwrap(), vec![42]);
}

// == 2. at_least ==============================================================

fn parse_rd_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RDError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RDError>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  try_num_rd
    .repeated()
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_delimited_at_least_ok() {
  let r: Result<Vec<i64>, RDError> = Parser::with_context(rd_ctx())
    .apply(parse_rd_at_least_2)
    .parse_str("[1 2 3]");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_repeated_delimited_at_least_fail() {
  let r: Result<Vec<i64>, RDError> = Parser::with_context(rd_ctx())
    .apply(parse_rd_at_least_2)
    .parse_str("[1]");
  assert!(r.is_err());
}

// == 3. at_most ===============================================================

fn parse_rd_at_most_3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RDError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RDError>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_rd
    .repeated()
    .at_most(3)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_delimited_at_most_ok() {
  let r: Result<Vec<i64>, RDError> = Parser::with_context(rd_ctx())
    .apply(parse_rd_at_most_3)
    .parse_str("[1 2]");
  assert_eq!(r.unwrap(), vec![1, 2]);
}

#[test]
fn test_repeated_delimited_at_most_single() {
  let r: Result<Vec<i64>, RDError> = Parser::with_context(rd_ctx())
    .apply(parse_rd_at_most_3)
    .parse_str("[5]");
  assert_eq!(r.unwrap(), vec![5]);
}

// == 4. bounded ===============================================================

fn parse_rd_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RDError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RDError>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num_rd
    .repeated()
    .bounded(2, 4)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_delimited_bounded_ok() {
  let r: Result<Vec<i64>, RDError> = Parser::with_context(rd_ctx())
    .apply(parse_rd_bounded)
    .parse_str("[1 2 3]");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_repeated_delimited_bounded_too_few() {
  let r: Result<Vec<i64>, RDError> = Parser::with_context(rd_ctx())
    .apply(parse_rd_bounded)
    .parse_str("[1]");
  assert!(r.is_err());
}

// == 5. Error: missing open bracket ==========================================

#[test]
fn test_repeated_delimited_missing_open() {
  let r: Result<Vec<i64>, RDError> =
    Parser::with_context(rd_ctx()).apply(parse_rd_list).parse_str("1 2 3]");
  assert!(r.is_err());
}

// == 6. Error: wrong close token (not a bracket) =============================

#[test]
fn test_repeated_delimited_wrong_close() {
  // After parsing elements, the parser expects `]` but sees `+` instead.
  let r: Result<Vec<i64>, RDError> =
    Parser::with_context(rd_ctx()).apply(parse_rd_list).parse_str("[1 2 3 +");
  assert!(r.is_err());
}
