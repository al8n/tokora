#![cfg(all(feature = "std", feature = "logos"))]

//! Tests for fold, try_fold, fold_while, try_fold_while, rfold, rfold_while combinators.

mod common;

use common::{TestLexer, Token, TokenKind};
use generic_arraydeque::typenum::U1;
use tokit::{
  Emitter, InputRef, Parse, ParseContext, ParseInput, Parser, TryParseInput,
  cache::Peeked,
  parser::{Action, expect},
  try_parse_input::ParseAttempt,
  utils::Expected,
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_num<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
{
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

fn try_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, ()>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
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

// Condition: continue while next token is Num, stop otherwise.
// Uses same pattern as decide_num in parser_sep_while.rs.
fn while_num<'inp, Ctx>(
  mut peeked: Peeked<'_, 'inp, TestLexer<'inp>, U1>,
  _emitter: &mut Ctx::Emitter,
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

// ── TryParseInput::fold ───────────────────────────────────────────────────────

#[test]
fn fold_sum_multiple_numbers() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num.fold(|| 0i64, |acc, x| acc + x).parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("1 2 3 4").unwrap(), 10);
}

#[test]
fn fold_empty_input_returns_init() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num.fold(|| 99i64, |acc, x| acc + x).parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("").unwrap(), 99);
}

#[test]
fn fold_single_number() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num.fold(|| 0i64, |acc, x| acc + x).parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("42").unwrap(), 42);
}

#[test]
fn fold_product() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num.fold(|| 1i64, |acc, x| acc * x).parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("2 3 4").unwrap(), 24);
}

#[test]
fn fold_stops_at_non_num() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num.fold(|| 0i64, |acc, x| acc + x).parse_input(inp)
  }
  // Stops after "1 2" when it sees "+"
  assert_eq!(Parser::new().apply(p).parse_str("1 2 +").unwrap(), 3);
}

// ── TryParseInput::try_fold ───────────────────────────────────────────────────

#[test]
fn try_fold_sum_multiple_numbers() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num
      .try_fold(|| 0i64, |acc, x| Ok(acc + x))
      .parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("5 10 15").unwrap(), 30);
}

#[test]
fn try_fold_empty_input() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num
      .try_fold(|| 100i64, |acc, x| Ok(acc + x))
      .parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("").unwrap(), 100);
}

#[test]
fn try_fold_accumulator_fails_propagates_error() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num
      .try_fold(|| 0i64, |_acc, x| if x > 10 { Err(()) } else { Ok(x) })
      .parse_input(inp)
  }
  // 5 is fine, then 20 causes failure
  assert!(Parser::new().apply(p).parse_str("5 20").is_err());
  assert_eq!(Parser::new().apply(p).parse_str("3 5").unwrap(), 5);
}

// ── ParseInput::fold_while ────────────────────────────────────────────────────

#[test]
fn fold_while_sum_while_num() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    parse_num
      .fold_while::<_, _, _, U1>(while_num::<Ctx>, || 0i64, |acc, x| acc + x)
      .parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("1 2 3 +").unwrap(), 6);
}

#[test]
fn fold_while_stops_immediately_on_non_num() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    parse_num
      .fold_while::<_, _, _, U1>(while_num::<Ctx>, || 0i64, |acc, x| acc + x)
      .parse_input(inp)
  }
  // "+" is not a Num, stops immediately with init value
  assert_eq!(Parser::new().apply(p).parse_str("+").unwrap(), 0);
}

#[test]
fn fold_while_empty_input_returns_init() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    parse_num
      .fold_while::<_, _, _, U1>(while_num::<Ctx>, || 42i64, |acc, x| acc + x)
      .parse_input(inp)
  }
  // EOF peeks as None → stops immediately
  assert_eq!(Parser::new().apply(p).parse_str("+").unwrap(), 42);
}

#[test]
fn fold_while_single_element() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    parse_num
      .fold_while::<_, _, _, U1>(while_num::<Ctx>, || 0i64, |acc, x| acc + x)
      .parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("7 +").unwrap(), 7);
}

// ── ParseInput::try_fold_while ────────────────────────────────────────────────

#[test]
fn try_fold_while_sum_with_fallible_acc() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    parse_num
      .try_fold_while::<_, _, _, U1>(while_num::<Ctx>, || 0i64, |acc, x| Ok(acc + x))
      .parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("10 20 30 +").unwrap(), 60);
}

#[test]
fn try_fold_while_acc_fails_propagates_error() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    parse_num
      .try_fold_while::<_, _, _, U1>(
        while_num::<Ctx>,
        || 0i64,
        |_acc, x| if x > 5 { Err(()) } else { Ok(x) },
      )
      .parse_input(inp)
  }
  assert!(Parser::new().apply(p).parse_str("3 10 +").is_err());
}

#[test]
fn try_fold_while_empty_returns_init() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    parse_num
      .try_fold_while::<_, _, _, U1>(while_num::<Ctx>, || 55i64, |acc, x| Ok(acc + x))
      .parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("+").unwrap(), 55);
}

// ── rfold_while ───────────────────────────────────────────────────────────────
// rfold processes elements right-to-left (last element first).
// With acc = |acc, x| acc * 2 + x, the result differs from left fold.
// fold([1,2,3], 0, |acc, x| acc*2+x) = ((0*2+1)*2+2)*2+3 = 11
// rfold([1,2,3], 0, |acc, x| acc*2+x) = 3 first, then 2, then 1:
//   0*2+3=3, 3*2+2=8, 8*2+1=17

#[test]
fn rfold_while_reverses_fold() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    parse_num
      .rfold_while::<_, _, _, U1>(while_num::<Ctx>, || 0i64, |acc, x| acc * 2 + x)
      .parse_input(inp)
  }
  // rfold processes right-to-left: 3 first, then 2, then 1
  // 0*2+3=3, 3*2+2=8, 8*2+1=17
  let result = Parser::new().apply(p).parse_str("1 2 3 +").unwrap();
  assert_eq!(result, 17);
}

#[test]
fn rfold_while_empty_returns_init() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    parse_num
      .rfold_while::<_, _, _, U1>(while_num::<Ctx>, || 0i64, |acc, x| acc + x)
      .parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("+").unwrap(), 0);
}

// ── rfold (alloc-based) ───────────────────────────────────────────────────────
// rfold on TryParseInput: also processes right-to-left.

#[test]
fn rfold_reverses_accumulated() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    // rfold([1,2,3], 0, |acc,x| acc*2+x): 0*2+3=3, 3*2+2=8, 8*2+1=17
    try_num
      .rfold(|| 0i64, |acc, x| acc * 2 + x)
      .parse_input(inp)
  }
  let result = Parser::new().apply(p).parse_str("1 2 3").unwrap();
  assert_eq!(result, 17);
}

#[test]
fn rfold_empty_returns_init() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num.rfold(|| 99i64, |acc, x| acc + x).parse_input(inp)
  }
  assert_eq!(Parser::new().apply(p).parse_str("").unwrap(), 99);
}

#[test]
fn rfold_single_element() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num
      .rfold(|| 0i64, |acc, x| acc * 2 + x)
      .parse_input(inp)
  }
  // single element [5]: 0*2+5=5
  assert_eq!(Parser::new().apply(p).parse_str("5").unwrap(), 5);
}

#[test]
fn rfold_stops_at_non_num() {
  fn p<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, ()>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ()>,
  {
    try_num.rfold(|| 0i64, |acc, x| acc + x).parse_input(inp)
  }
  // Stops at "+" and sums [1, 2] from right = 1+2 = 3
  assert_eq!(Parser::new().apply(p).parse_str("1 2 +").unwrap(), 3);
}
