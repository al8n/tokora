#![cfg(all(feature = "std", feature = "logos"))]

//! Tests for the `RepeatedWhile` (condition-based repetition WITHOUT separator) combinator.
//!
//! Exercises `parser/many/repeated_while/` (unbounded, at_least, at_most, bounded),
//! `parser/many/delim/repeated_while.rs` via `.delimited()`, and
//! `parser/many/handler/bounded.rs` via `.at_least()`, `.at_most()`, `.bounded()`.
//!
//! # Sentinel token
//!
//! `RepeatedWhile::parse` calls `peek_with_emitter` to decide whether to continue.
//! At EOF there is nothing to peek, which triggers a debug_assert.  We therefore
//! append `+` (a non-Num token) to every test string so the condition always sees
//! a stop token instead of hitting EOF.  The trailing `+` is left unconsumed;
//! `parse_str` does not require all tokens to be consumed.

mod common;

use generic_arraydeque::typenum::U1;
use tokit::{
  Accumulator, Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
  Token as TokenTrait,
  cache::Peeked,
  emitter::{
    FromSeparatedError, FullContainerEmitter, SeparatedEmitter, TooFewEmitter, TooManyEmitter,
  },
  error::{
    UnexpectedEot,
    syntax::{FullContainer, MissingSyntaxOf, TooFew, TooMany},
    token::{MissingTokenOf, UnexpectedToken, UnexpectedTokenOf},
  },
  input::Cursor,
  parser::Action,
  punct::Bracket,
  span::Spanned,
  utils::CowStr,
};

use common::{TestLexer, Token};

// -- Error type ---------------------------------------------------------------

#[derive(Debug)]
struct RWError;

impl From<()> for RWError {
  fn from(_: ()) -> Self {
    RWError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for RWError {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    RWError
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for RWError {
  fn from(_: FullContainer<S, Lang>) -> Self {
    RWError
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for RWError {
  fn from(_: TooFew<S, Lang>) -> Self {
    RWError
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for RWError {
  fn from(_: TooMany<S, Lang>) -> Self {
    RWError
  }
}

impl From<UnexpectedEot> for RWError {
  fn from(_: UnexpectedEot) -> Self {
    RWError
  }
}

impl<'inp> FromSeparatedError<'inp, TestLexer<'inp>> for RWError {
  fn from_missing_separator(_: CowStr, _: MissingTokenOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    RWError
  }

  fn from_missing_element(_: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    RWError
  }
}

// -- Custom emitter -----------------------------------------------------------

struct RWEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for RWEmitter {
  type Error = RWError;

  fn emit_lexer_error(
    &mut self,
    _: Spanned<
      <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
      <TestLexer<'inp> as Lexer<'inp>>::Span,
    >,
  ) -> Result<(), RWError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RWError)
  }

  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), RWError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RWError)
  }

  fn emit_error(
    &mut self,
    err: Spanned<RWError, <TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), RWError>
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

impl<'inp> SeparatedEmitter<'inp, TestLexer<'inp>> for RWEmitter {
  fn emit_missing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), RWError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RWError)
  }

  fn emit_missing_element(
    &mut self,
    _: MissingSyntaxOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), RWError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RWError)
  }
}

impl<'inp> FullContainerEmitter<'inp, TestLexer<'inp>> for RWEmitter {
  fn emit_full_container(
    &mut self,
    _: FullContainer<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), RWError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RWError)
  }
}

impl<'inp> TooFewEmitter<'inp, TestLexer<'inp>> for RWEmitter {
  fn emit_too_few(
    &mut self,
    _: TooFew<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), RWError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RWError)
  }
}

impl<'inp> TooManyEmitter<'inp, TestLexer<'inp>> for RWEmitter {
  fn emit_too_many(
    &mut self,
    _: TooMany<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), RWError>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(RWError)
  }
}

fn rw_ctx() -> ParserContext<'static, TestLexer<'static>, RWEmitter> {
  ParserContext::new(RWEmitter)
}

// -- Supertrait alias ---------------------------------------------------------

trait RWEmitterBound<'inp>:
  Emitter<'inp, TestLexer<'inp>, Error = RWError>
  + FullContainerEmitter<'inp, TestLexer<'inp>>
  + SeparatedEmitter<'inp, TestLexer<'inp>>
{
}

impl<'inp, E> RWEmitterBound<'inp> for E where
  E: Emitter<'inp, TestLexer<'inp>, Error = RWError>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
{
}

// -- Condition: continue iff next token is a Num ------------------------------

fn decide_num_rw<'inp, Ctx>(
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

// -- Element parser (ParseInput) ----------------------------------------------

fn parse_num_rw<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<i64, RWError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RWError>,
{
  match inp.next()? {
    None => Err(RWError),
    Some(tok) => match tok.into_data() {
      Token::Num(n) => Ok(n),
      _ => Err(RWError),
    },
  }
}

// == 1. Plain repeated_while (unbounded) ======================================

fn parse_rw_list<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RWError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: RWEmitterBound<'inp>,
{
  parse_num_rw
    .repeated_while::<_, U1>(decide_num_rw::<Ctx>)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_while_basic() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_list)
    .parse_str("1 2 3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_repeated_while_single() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_list)
    .parse_str("42+");
  assert_eq!(r.unwrap(), vec![42]);
}

// == 2. at_least ==============================================================

fn parse_rw_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RWError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: RWEmitterBound<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_rw
    .repeated_while::<_, U1>(decide_num_rw::<Ctx>)
    .at_least(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_while_at_least_ok() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_at_least_2)
    .parse_str("1 2 3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_repeated_while_at_least_fail() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_at_least_2)
    .parse_str("1+");
  assert!(r.is_err());
}

// == 3. at_most ===============================================================

fn parse_rw_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RWError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: RWEmitterBound<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_rw
    .repeated_while::<_, U1>(decide_num_rw::<Ctx>)
    .at_most(2)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_while_at_most_ok() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_at_most_2)
    .parse_str("1 2+");
  assert_eq!(r.unwrap(), vec![1, 2]);
}

#[test]
fn test_repeated_while_at_most_single() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_at_most_2)
    .parse_str("7+");
  assert_eq!(r.unwrap(), vec![7]);
}

// == 4. bounded ===============================================================

fn parse_rw_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RWError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: RWEmitterBound<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_rw
    .repeated_while::<_, U1>(decide_num_rw::<Ctx>)
    .bounded(2, 4)
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_while_bounded_ok() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_bounded)
    .parse_str("1 2 3+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_repeated_while_bounded_too_few() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_bounded)
    .parse_str("1+");
  assert!(r.is_err());
}

// == 5. Delimited repeated_while (unbounded) ==================================

fn parse_rw_delimited<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RWError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: RWEmitterBound<'inp>,
{
  parse_num_rw
    .repeated_while::<_, U1>(decide_num_rw::<Ctx>)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_while_delimited() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_delimited)
    .parse_str("[1 2 3]+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_repeated_while_delimited_empty() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_delimited)
    .parse_str("[]+");
  assert_eq!(r.unwrap(), vec![]);
}

// == 6. Delimited at_least ====================================================

fn parse_rw_delimited_at_least_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RWError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: RWEmitterBound<'inp> + TooFewEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_rw
    .repeated_while::<_, U1>(decide_num_rw::<Ctx>)
    .at_least(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_while_delimited_at_least_ok() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_delimited_at_least_2)
    .parse_str("[1 2 3]+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_repeated_while_delimited_at_least_fail() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_delimited_at_least_2)
    .parse_str("[1]+");
  assert!(r.is_err());
}

// == 7. Delimited at_most =====================================================

fn parse_rw_delimited_at_most_2<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RWError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: RWEmitterBound<'inp> + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_rw
    .repeated_while::<_, U1>(decide_num_rw::<Ctx>)
    .at_most(2)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_while_delimited_at_most_ok() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_delimited_at_most_2)
    .parse_str("[1 2]+");
  assert_eq!(r.unwrap(), vec![1, 2]);
}

// == 8. Delimited bounded =====================================================

fn parse_rw_delimited_bounded<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, RWError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: RWEmitterBound<'inp>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  parse_num_rw
    .repeated_while::<_, U1>(decide_num_rw::<Ctx>)
    .bounded(2, 4)
    .delimited::<Bracket<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

#[test]
fn test_repeated_while_delimited_bounded_ok() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_delimited_bounded)
    .parse_str("[1 2 3]+");
  assert_eq!(r.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_repeated_while_delimited_bounded_too_few() {
  let r: Result<Vec<i64>, RWError> = Parser::with_context(rw_ctx())
    .apply(parse_rw_delimited_bounded)
    .parse_str("[1]+");
  assert!(r.is_err());
}
