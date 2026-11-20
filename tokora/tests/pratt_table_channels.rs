#![cfg(all(feature = "std", feature = "combinators", feature = "logos_0_16"))]

//! The typed driver's two channels over a table, `PrattTable::lhs` and `PrattTable::rhs`, alone.
//!
//! They are the typed side of the comparison the flat driver is tested by, and they are new code,
//! so they are tested first on their own. Every expectation in this file is written by hand, and
//! no flat driver is named in it.

mod flat_fixture;

use tokora::{
  Emitter, InputRef, Parse, ParseContext, ParseInput, Parser, ParserContext, SimpleSpan,
  emitter::{Fatal, Verbose},
  input::TokenBudget,
  pratt::{PrattInfix, PrattLHS, PrattRHS, pratt},
  span::Spanned,
};

use flat_fixture::{
  ByteLexer, Fail, FatalIr, Landing, Operand, TABLE, Tok, TwoDigits, VerboseIr, fold_infix,
  fold_postfix, fold_prefix, landing,
};

/// What the left-hand-side channel reported.
#[derive(Debug, PartialEq, Eq)]
enum Lhs {
  /// The operand parser's operand.
  Operand(String),
  /// A prefix operator: its byte, its offset and its power.
  Prefix(u8, usize, u8),
}

/// What the right-hand-side channel reported.
#[derive(Debug, PartialEq, Eq)]
enum Rhs {
  /// An infix operator: its byte, its offset, its associativity and its power.
  Infix(u8, usize, PrattInfix<(), (), ()>, u8),
  /// A postfix operator: its byte, its offset and its power.
  Postfix(u8, usize, u8),
  /// The end of the expression.
  End,
}

type Reported<T> = Result<T, Fail>;

/// Asks the left-hand-side channel once.
fn lhs<'inp, const SKIP: bool, Ctx>(
  input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  operand: Operand,
) -> Reported<Lhs>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  let reported: Result<PrattLHS<String, Spanned<Tok, SimpleSpan>, u8>, Fail> =
    TABLE.lhs(operand).parse_input(input);
  reported.map(|reported| match reported {
    PrattLHS::Operand(text) => Lhs::Operand(text),
    PrattLHS::Prefix(prefix) => {
      let (tok, power) = prefix.into_components();
      Lhs::Prefix(tok.data().0, tok.span().start(), power)
    }
  })
}

/// Asks the right-hand-side channel once.
fn rhs<'inp, const SKIP: bool, Ctx>(
  input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
) -> Reported<Rhs>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  type Tk = Spanned<Tok, SimpleSpan>;
  let reported: Result<PrattRHS<Tk, Tk, Tk, Tk, u8>, Fail> = TABLE.rhs().parse_input(input);
  reported.map(|reported| match reported {
    PrattRHS::Infix(infix) => {
      let (operator, power) = infix.into_components();
      let (associativity, tok) = match operator {
        PrattInfix::Left(tok) => (PrattInfix::Left(()), tok),
        PrattInfix::Right(tok) => (PrattInfix::Right(()), tok),
        PrattInfix::Neither(tok) => (PrattInfix::Neither(()), tok),
      };
      Rhs::Infix(tok.data().0, tok.span().start(), associativity, power)
    }
    PrattRHS::Postfix(postfix) => {
      let (tok, power) = postfix.into_components();
      Rhs::Postfix(tok.data().0, tok.span().start(), power)
    }
    PrattRHS::End => Rhs::End,
    PrattRHS::Adjacent(_) => panic!("the channel never reports a zero-token operator"),
  })
}

/// The typed driver over the two channels, at `floor`.
fn typed<'inp, const SKIP: bool, Ctx>(
  input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  operand: Operand,
  floor: u8,
) -> Reported<String>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  pratt(
    TABLE.lhs(operand),
    TABLE.rhs(),
    fold_prefix,
    fold_infix,
    fold_postfix,
  )
  .min_precedence(floor)
  .parse_input(input)
}

/// Runs `f` over `src` with a rejecting emitter.
fn fatal<const SKIP: bool, O>(
  src: &str,
  f: impl FnMut(&mut FatalIr<'_, '_, SKIP>) -> Result<O, Fail>,
) -> O {
  Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(f)
    .parse_str(src)
    .expect("the cell catches every error it expects")
}

/// Runs `f` over `src` with a rejecting emitter, the default cache, and a budget of `budget`
/// items.
fn within<const SKIP: bool, O>(
  src: &str,
  budget: usize,
  f: impl FnMut(&mut FatalIr<'_, '_, SKIP>) -> Result<O, Fail>,
) -> O {
  Parser::with_context(
    ParserContext::new(Fatal::new()).with_token_budget(TokenBudget::with_limitation(budget)),
  )
  .apply(f)
  .parse_str(src)
  .expect("the cell catches every error it expects")
}

// ── The operand parser first, and a prefix operator by the table ──────────────────────────────

/// A scanner stop the operand parser met and declined over is the channel's terminal error, never
/// a missing operand: over `42` within a budget of one item, an operand parser that wants two
/// digits peeks at both, finds the window cut short, and declines, with `4` left in the cache.
#[test]
fn a_decline_over_a_scanner_stop_is_the_terminal_error() {
  let got = within::<true, _>("42", 1, |input| {
    let reported: Reported<PrattLHS<String, Spanned<Tok, SimpleSpan>, u8>> =
      TABLE.lhs(TwoDigits).parse_input(input);
    Ok(reported.map(|_| ()))
  });
  assert_eq!(
    got,
    Err(Fail::Eot {
      at: 0,
      terminal: true
    })
  );
}

/// A head that the operand parser and the table's prefix row would both take is the operand
/// parser's. Where the operand parser declines it, it is the table's prefix operator.
#[test]
fn the_operand_parser_is_asked_before_the_table() {
  let minus = Operand {
    crossing: false,
    extra: Some(b'-'),
  };
  let got = fatal::<true, _>("-5;", |input| Ok((lhs(input, minus), landing(input)?)));
  assert_eq!(got.0, Ok(Lhs::Operand(String::from("-"))));
  assert_eq!(got.1.handback, 1);

  let got = fatal::<true, _>("-5;", |input| {
    Ok((lhs(input, Operand::STRICT), landing(input)?))
  });
  assert_eq!(got.0, Ok(Lhs::Prefix(b'-', 0, 40)));
  assert_eq!(
    got.1,
    Landing {
      handback: 1,
      next: Some((b'5', 1)),
      spent: 1,
    },
    "the operator is committed, and the operand after it is the next head"
  );
}

/// A prefix operator is read, and committed, only where the operand parser declined: the operand
/// after it is the operand parser's again.
#[test]
fn a_prefix_operator_is_read_where_the_operand_parser_declined() {
  let got = fatal::<true, _>("~5;", |input| {
    let first = lhs(input, Operand::STRICT);
    let second = lhs(input, Operand::STRICT);
    Ok((first, second, landing(input)?))
  });
  assert_eq!(got.0, Ok(Lhs::Prefix(b'~', 0, 12)));
  assert_eq!(got.1, Ok(Lhs::Operand(String::from("5"))));
  assert_eq!(got.2.handback, 2);
  assert_eq!(got.2.next, Some((b';', 2)));
}

// ── Trivia ────────────────────────────────────────────────────────────────────────────────────

/// At a term, a trivia token the operand parser declined is crossed and committed, and the operand
/// parser asked again. An operand parser that crosses trivia itself gets the same answer.
#[test]
fn trivia_is_crossed_and_committed_at_a_term() {
  for operand in [Operand::STRICT, Operand::CROSSING] {
    let got = fatal::<false, _>("  7;", |input| Ok((lhs(input, operand), landing(input)?)));
    assert_eq!(got.0, Ok(Lhs::Operand(String::from("7"))), "{operand:?}");
    assert_eq!(got.1.handback, 3, "{operand:?}");
    assert_eq!(got.1.next, Some((b';', 3)), "{operand:?}");
  }
}

/// Before an operator, a trivia token is crossed and committed, and the head read again.
#[test]
fn trivia_is_crossed_and_committed_before_an_operator() {
  let got = fatal::<false, _>(" +1", |input| Ok((rhs(input), landing(input)?)));
  assert_eq!(got.0, Ok(Rhs::Infix(b'+', 1, PrattInfix::Left(()), 10)));
  assert_eq!(got.1.handback, 2);
  assert_eq!(got.1.next, Some((b'1', 2)));

  let got = fatal::<false, _>("  !", |input| Ok((rhs(input), landing(input)?)));
  assert_eq!(got.0, Ok(Rhs::Postfix(b'!', 2, 50)));
}

/// A token that is no operator of the table is the end, and stays on the input. The channel alone
/// keeps the trivia it crossed to find it.
#[test]
fn a_token_that_is_no_operator_is_the_end() {
  let got = fatal::<true, _>(";", |input| Ok((rhs(input), landing(input)?)));
  assert_eq!(got.0, Ok(Rhs::End));
  assert_eq!(got.1.handback, 0);
  assert_eq!(got.1.next, Some((b';', 0)));

  let got = fatal::<false, _>("  ;", |input| Ok((rhs(input), landing(input)?)));
  assert_eq!(got.0, Ok(Rhs::End));
  assert_eq!(got.1.handback, 2);
  assert_eq!(got.1.next, Some((b';', 2)));
}

/// Through the typed driver, an end hands the trivia the channel crossed back: the driver rolls
/// its probe back, and the input stands before the trivia.
#[test]
fn the_typed_driver_hands_back_the_trivia_its_rhs_channel_crossed() {
  let got = fatal::<false, _>("1  ;", |input| {
    Ok((typed(input, Operand::STRICT, 0), landing(input)?))
  });
  assert_eq!(got.0, Ok(String::from("1")));
  assert_eq!(got.1.handback, 1);
  assert_eq!(got.1.next, Some((b' ', 1)));
}

// ── A missing operand ─────────────────────────────────────────────────────────────────────────

/// Where the operand parser declines and no prefix operator stands, the channel reports
/// `UnexpectedEoLhs` at `span().end()`, with the token left on the input. Over a lexer that skips
/// trivia that is the end of what was committed before the term: the operator's end, after an
/// infix operator.
#[test]
fn a_missing_operand_is_eolhs_at_span_end_over_a_skipping_lexer() {
  let got = fatal::<true, _>(";", |input| {
    Ok((lhs(input, Operand::STRICT), landing(input)?))
  });
  assert_eq!(
    got.0,
    Err(Fail::EoLhs {
      at: 0,
      terminal: false
    })
  );
  assert_eq!(got.1.next, Some((b';', 0)));

  // `1`@0, `+`@2, `;`@4: the operator ends at 3.
  let got = fatal::<true, _>("1 + ;", |input| {
    Ok((typed(input, Operand::STRICT, 0), landing(input)?))
  });
  assert_eq!(
    got.0,
    Err(Fail::EoLhs {
      at: 3,
      terminal: false
    })
  );
  assert_eq!(got.1.handback, 3);
  assert_eq!(got.1.next, Some((b';', 4)));
}

/// Over a lexer that surfaces trivia, the trivia crossed at the term is committed, so the offset is
/// the start of the token that is no operand.
#[test]
fn a_missing_operand_is_eolhs_at_span_end_over_a_surfacing_lexer() {
  let got = fatal::<false, _>("  ;", |input| {
    Ok((lhs(input, Operand::STRICT), landing(input)?))
  });
  assert_eq!(
    got.0,
    Err(Fail::EoLhs {
      at: 2,
      terminal: false
    })
  );
  assert_eq!(got.1.next, Some((b';', 2)));

  for operand in [Operand::STRICT, Operand::CROSSING] {
    let got = fatal::<false, _>("1 +  ;", |input| {
      Ok((typed(input, operand, 0), landing(input)?))
    });
    assert_eq!(
      got.0,
      Err(Fail::EoLhs {
        at: 5,
        terminal: false
      }),
      "{operand:?}"
    );
    assert_eq!(got.1.handback, 5, "{operand:?}");
    assert_eq!(got.1.next, Some((b';', 5)), "{operand:?}");
  }
}

// ── A zero-token operator ─────────────────────────────────────────────────────────────────────

/// A row that says `Adjacent` is `UnexpectedEoRhs` at `span().end()`, with the token left on the
/// input: past the trivia crossed after the left operand.
#[test]
fn an_adjacent_row_is_eorhs_with_its_token_left_on_the_input() {
  let got = fatal::<true, _>("@;", |input| Ok((rhs(input), landing(input)?)));
  assert_eq!(
    got.0,
    Err(Fail::EoRhs {
      at: 0,
      terminal: false
    })
  );
  assert_eq!(got.1.next, Some((b'@', 0)));

  let got = fatal::<false, _>("1 @;", |input| {
    Ok((typed(input, Operand::STRICT, 0), landing(input)?))
  });
  assert_eq!(
    got.0,
    Err(Fail::EoRhs {
      at: 2,
      terminal: false
    })
  );
  assert_eq!(got.1.handback, 2);
  assert_eq!(got.1.next, Some((b'@', 2)));
}

/// The channel classifies before the typed driver compares a power, so it refuses such a row below
/// the driver's floor too, where the row's power would have ended the expression.
#[test]
fn an_adjacent_row_is_refused_below_the_floor_too() {
  let got = fatal::<true, _>("1@;", |input| {
    Ok((typed(input, Operand::STRICT, 20), landing(input)?))
  });
  assert_eq!(
    got.0,
    Err(Fail::EoRhs {
      at: 1,
      terminal: false
    })
  );
  assert_eq!(got.1.next, Some((b'@', 1)));
}

// ── A scanner stop ────────────────────────────────────────────────────────────────────────────

/// A scanner stop at a head is the terminal end-of-input error from either channel, and never a
/// missing operand or an end, with an accepting emitter and with a rejecting one.
#[test]
fn a_scanner_stop_is_terminal_from_either_channel() {
  let stop = |at| Fail::Eot { at, terminal: true };

  let lhs_fatal = Parser::with_context(
    ParserContext::new(Fatal::new()).with_token_budget(TokenBudget::with_limitation(0)),
  )
  .apply(|input: &mut FatalIr<'_, '_, true>| lhs(input, Operand::STRICT))
  .parse_str("1;");
  assert_eq!(lhs_fatal, Err(stop(0)));

  let lhs_verbose = Parser::with_context(
    ParserContext::new(Verbose::new()).with_token_budget(TokenBudget::with_limitation(0)),
  )
  .apply(|input: &mut VerboseIr<'_, '_, true>| lhs(input, Operand::STRICT))
  .parse_str("1;");
  assert_eq!(lhs_verbose, Err(stop(0)));

  let rhs_fatal = Parser::with_context(
    ParserContext::new(Fatal::new()).with_token_budget(TokenBudget::with_limitation(1)),
  )
  .apply(|input: &mut FatalIr<'_, '_, true>| {
    assert_eq!(
      lhs(input, Operand::STRICT),
      Ok(Lhs::Operand(String::from("1")))
    );
    rhs(input)
  })
  .parse_str("1+2");
  assert_eq!(rhs_fatal, Err(stop(1)));

  let rhs_verbose = Parser::with_context(
    ParserContext::new(Verbose::new()).with_token_budget(TokenBudget::with_limitation(1)),
  )
  .apply(|input: &mut VerboseIr<'_, '_, true>| {
    assert_eq!(
      lhs(input, Operand::STRICT),
      Ok(Lhs::Operand(String::from("1")))
    );
    rhs(input)
  })
  .parse_str("1+2");
  assert_eq!(rhs_verbose, Err(stop(1)));
}
