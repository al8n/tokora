#![cfg(all(feature = "std", feature = "combinators", feature = "logos_0_16"))]

//! The flat driver's `parse_into`, held to its loop's own contract: each token lexed once, the
//! lexer's state left as the caller set it, the operand parser asked first, each error at the
//! offset it is defined at, the handler told each item once, and its two modes.

mod flat_fixture;
mod flat_items;

use tokora::{
  Emitter, InputRef, Parse, ParseContext, Parser, ParserContext, SimpleSpan,
  emitter::{Fatal, Verbose},
  error::MaybeTerminal,
  input::TokenBudget,
  pratt::{Bare, Flat, NoCst, PrattHandler, PrattInfix, Precedenced, flat},
  span::Spanned,
  state::{RecursionLimiter, TokenLimiter},
  utils::typenum::U2,
};

use flat_fixture::{
  BareCacheIr, ByteLexer, Expr, Fail, FatalCtx, FatalIr, Landing, LimitedIr, Mode, Operand,
  Rekeying, Rolling, Signed, Skip, TABLE, Tok, TwoDigits, VerboseIr, forget_lexes, landing, lexes,
};
use flat_items::{Item, Items};

/// What a parse returned, what its handler was told, and where it left the input.
#[derive(Debug)]
struct Outcome {
  result: Result<(), Fail>,
  items: Vec<Item>,
  landing: Landing,
}

/// A checked driver at `floor`, capped at `cap`.
fn checked<'inp, const SKIP: bool, Ctx>(
  input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  operand: Operand,
  floor: u8,
  cap: usize,
) -> Result<Outcome, Fail>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  let mut items = Items::default();
  let result = flat(TABLE, operand)
    .min_precedence(floor)
    .max_depth(cap)
    .parse_into(input, &mut items);
  Ok(Outcome {
    result,
    items: items.0,
    landing: landing(input)?,
  })
}

/// A checked driver with neither a floor nor a cap.
fn open<'inp, const SKIP: bool, Ctx>(
  input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  operand: Operand,
) -> Result<Outcome, Fail>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  let mut items = Items::default();
  let result = flat(TABLE, operand).parse_into(input, &mut items);
  Ok(Outcome {
    result,
    items: items.0,
    landing: landing(input)?,
  })
}

/// A bare driver.
fn bare<'inp, const SKIP: bool, Ctx>(
  input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  operand: Operand,
) -> Result<Outcome, Fail>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  let mut items = Items::default();
  let result = flat(TABLE, operand).bare().parse_into(input, &mut items);
  Ok(Outcome {
    result,
    items: items.0,
    landing: landing(input)?,
  })
}

/// Runs `f` over `src` with a rejecting emitter and the default cache.
fn fatal<const SKIP: bool, O>(
  src: &str,
  f: impl FnMut(&mut FatalIr<'_, '_, SKIP>) -> Result<O, Fail>,
) -> O {
  Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(f)
    .parse_str(src)
    .expect("the cell catches every error it expects")
}

/// Runs `f` over `src` with a rejecting emitter and a cache of no capacity.
fn uncached<const SKIP: bool, O>(
  src: &str,
  f: impl FnMut(&mut BareCacheIr<'_, '_, SKIP>) -> Result<O, Fail>,
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

fn operand(text: &str) -> Item {
  Item::Operand(String::from(text))
}

const LEFT: PrattInfix<(), (), ()> = PrattInfix::Left(());
const RIGHT: PrattInfix<(), (), ()> = PrattInfix::Right(());
const NEITHER: PrattInfix<(), (), ()> = PrattInfix::Neither(());

// ── Each token lexed once ─────────────────────────────────────────────────────────────────────

/// `-1*2+3^4`: a prefix operator, operands, an operator taken at once (`*`), one that waits for a
/// reduction (`+` reduces `*` first), and one pushed over a pending one (`^`).
const ONCE: &str = "-1*2+3^4;";

/// Every token up to the one that ends the expression is lexed once, whether the operand parser
/// or the driver reads it first, under the default cache and under a cache of no capacity, and in
/// both modes. The token that ends the expression is the one left staged: reading it lexes nothing.
#[test]
fn a_parse_lexes_each_token_once() {
  let expected: Vec<(usize, u8)> = (0..=8).map(|at| (at, 0)).collect();
  for mode in ["checked", "bare"] {
    let cached = fatal::<true, _>(ONCE, |input| {
      forget_lexes();
      let outcome = match mode {
        "checked" => open(input, Operand::STRICT)?,
        _ => bare(input, Operand::STRICT)?,
      };
      Ok((outcome, lexes()))
    });
    assert_eq!(cached.0.result, Ok(()), "{mode}");
    assert_eq!(cached.1, expected, "{mode}, default cache");
    assert_eq!(cached.0.landing.handback, 8, "{mode}");
    assert_eq!(cached.0.landing.next, Some((b';', 8)), "{mode}");
    assert_eq!(
      cached.0.landing.spent, 9,
      "{mode}: the staged `;` is the ninth lex, and the last"
    );

    let unbuffered = uncached::<true, _>(ONCE, |input| {
      forget_lexes();
      let outcome = match mode {
        "checked" => open(input, Operand::STRICT)?,
        _ => bare(input, Operand::STRICT)?,
      };
      Ok((outcome, lexes()))
    });
    assert_eq!(unbuffered.0.result, Ok(()), "{mode}");
    assert_eq!(unbuffered.1, expected, "{mode}, no cache");
    assert_eq!(unbuffered.0.landing.spent, 9, "{mode}");
  }
}

/// At the end of input nothing is staged, and nothing is lexed past the last token.
#[test]
fn a_parse_that_meets_the_end_of_input_stages_nothing() {
  let got = fatal::<true, _>("-1*2+3^4", |input| {
    forget_lexes();
    let outcome = open(input, Operand::STRICT)?;
    Ok((outcome, lexes()))
  });
  assert_eq!(got.0.result, Ok(()));
  assert_eq!(got.1, (0..8).map(|at| (at, 0)).collect::<Vec<_>>());
  assert_eq!(got.0.landing.handback, 8);
  assert_eq!(got.0.landing.next, None);
}

// ── The lexer's state ─────────────────────────────────────────────────────────────────────────

/// A caller switches the lexer's state, parses, consumes the token that ended the expression, and
/// switches back. Every token of the expression and its terminator is lexed under the caller's
/// state, the token after is lexed under the state restored, and the state the lexer ends with is
/// the caller's.
#[test]
fn the_lexer_s_state_is_the_caller_s() {
  let got = fatal::<true, _>("1+2;3", |input| {
    forget_lexes();
    input.set_state(Mode(1));
    let outcome = open(input, Operand::STRICT)?;
    input.set_state(Mode(0));
    let after = input.next()?.map(|tok| tok.data().0);
    Ok((outcome, after, lexes(), input.state()))
  });
  let (outcome, after, lexed, state) = got;
  assert_eq!(outcome.result, Ok(()));
  assert_eq!(
    outcome.landing.next,
    Some((b';', 3)),
    "the terminator is consumed, staged under the expression's state"
  );
  assert_eq!(after, Some(b'3'));
  assert_eq!(lexed, vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 0)]);
  assert_eq!(state, Mode(0));
}

// ── The operand parser first ──────────────────────────────────────────────────────────────────

/// A head that the operand parser and the table's prefix row would both take is the operand
/// parser's. Where it declines, the head is the table's prefix operator.
#[test]
fn the_operand_parser_is_asked_first() {
  let minus = Operand {
    crossing: false,
    extra: Some(b'-'),
  };
  let got = fatal::<true, _>("-1;", |input| open(input, minus));
  assert_eq!(got.result, Ok(()));
  assert_eq!(got.items, vec![operand("-")]);
  assert_eq!(got.landing.next, Some((b'1', 1)));

  let got = fatal::<true, _>("-1;", |input| open(input, Operand::STRICT));
  assert_eq!(got.result, Ok(()));
  assert_eq!(got.items, vec![Item::Prefix(b'-', 0, 40), operand("1")]);
  assert_eq!(got.landing.next, Some((b';', 2)));
}

/// An operand parser that crosses no trivia is asked again once the driver has crossed it.
#[test]
fn a_strict_operand_parser_behind_trivia_is_asked_again() {
  let got = fatal::<false, _>("  7 ;", |input| open(input, Operand::STRICT));
  assert_eq!(got.result, Ok(()));
  assert_eq!(got.items, vec![operand("7")]);
  assert_eq!(got.landing.handback, 4, "the trailing trivia is consumed");
  assert_eq!(got.landing.next, Some((b';', 4)));
}

// ── The errors, with the input they leave and the offset they carry ───────────────────────────

/// A missing operand is `UnexpectedEoLhs` at the start and after a prefix operator, and
/// `UnexpectedEoRhs` directly after an infix operator, at `span().end()`, with the token found in
/// its place left on the input. Over a lexer that skips trivia the offset is the end of the
/// operator before it.
#[test]
fn a_missing_operand_over_a_skipping_lexer() {
  let no_term = |at| Fail::EoLhs {
    at,
    terminal: false,
  };
  let no_right = |at| Fail::EoRhs {
    at,
    terminal: false,
  };

  let got = fatal::<true, _>(";", |input| open(input, Operand::STRICT));
  assert_eq!(got.result, Err(no_term(0)));
  assert_eq!(got.landing.next, Some((b';', 0)));

  // `1`@0, `+`@2, `-`@4, `;`@6.
  let got = fatal::<true, _>("1 + - ;", |input| open(input, Operand::STRICT));
  assert_eq!(got.result, Err(no_term(5)));
  assert_eq!(got.landing.handback, 5);
  assert_eq!(got.landing.next, Some((b';', 6)));

  // `1`@0, `+`@2, `;`@4.
  let got = fatal::<true, _>("1 + ;", |input| open(input, Operand::STRICT));
  assert_eq!(got.result, Err(no_right(3)));
  assert_eq!(got.landing.handback, 3);
  assert_eq!(got.landing.next, Some((b';', 4)));
}

/// Over a lexer that surfaces trivia, the trivia at the term is committed first, so the offset is
/// the start of the token found in the operand's place.
#[test]
fn a_missing_operand_over_a_surfacing_lexer() {
  for operand in [Operand::STRICT, Operand::CROSSING] {
    let got = fatal::<false, _>("1 +  ;", |input| open(input, operand));
    assert_eq!(
      got.result,
      Err(Fail::EoRhs {
        at: 5,
        terminal: false
      }),
      "{operand:?}"
    );
    assert_eq!(got.landing.next, Some((b';', 5)), "{operand:?}");

    let got = fatal::<false, _>("1 + - ;", |input| open(input, operand));
    assert_eq!(
      got.result,
      Err(Fail::EoLhs {
        at: 6,
        terminal: false
      }),
      "{operand:?}"
    );
    assert_eq!(got.landing.next, Some((b';', 6)), "{operand:?}");
  }
}

/// A repeated non-associative operator is `NonAssociativeChain` at `span().end()`, with the
/// operator left on the input: the handback. The trivia before it is committed, so over a lexer
/// that surfaces trivia the offset is the operator's head.
#[test]
fn a_repeated_non_associative_operator_is_refused_where_it_is_read() {
  let got = fatal::<true, _>("1<2<3;", |input| open(input, Operand::STRICT));
  assert_eq!(got.result, Err(Fail::Chain { at: 3 }));
  assert_eq!(got.landing.handback, 3);
  assert_eq!(got.landing.next, Some((b'<', 3)));
  assert_eq!(
    got.items,
    vec![
      operand("1"),
      Item::Infix(b'<', 1, NEITHER, 15),
      operand("2")
    ]
  );

  let got = fatal::<false, _>("1<2   <3;", |input| open(input, Operand::STRICT));
  assert_eq!(got.result, Err(Fail::Chain { at: 6 }));
  assert_eq!(got.landing.handback, 6);
  assert_eq!(got.landing.next, Some((b'<', 6)));
}

/// An operator below the floor ends the expression, and is the next token.
#[test]
fn an_operator_below_the_floor_ends_the_expression() {
  let got = fatal::<true, _>("1*2+3;", |input| {
    checked(input, Operand::STRICT, 11, usize::MAX)
  });
  assert_eq!(got.result, Ok(()));
  assert_eq!(
    got.items,
    vec![operand("1"), Item::Infix(b'*', 1, LEFT, 20), operand("2")]
  );
  assert_eq!(got.landing.handback, 3);
  assert_eq!(got.landing.next, Some((b'+', 3)));
}

/// The operator past the cap is `PrattStackFull` at the end of that operator, which is consumed,
/// reporting the cap. It is terminal, and the trip is counted on the input session.
#[test]
fn the_operator_past_the_cap_is_a_full_stack() {
  let got = fatal::<true, _>("1^2^3^4;", |input| {
    let baseline = input.trip_snapshot();
    let outcome = checked(input, Operand::STRICT, 0, 2)?;
    Ok((outcome, input.tripped_during_attempt(baseline)))
  });
  let (outcome, tripped) = got;
  assert_eq!(outcome.result, Err(Fail::Full { at: 6, capacity: 2 }));
  assert!(outcome.result.unwrap_err().is_terminal());
  assert!(tripped, "the trip is counted on the input session");
  assert_eq!(outcome.landing.next, Some((b'4', 6)));
}

// ── A zero-token operator ─────────────────────────────────────────────────────────────────────

/// An `Adjacent` row below the floor is declined: the expression ends there. At a latched power it
/// repeats the chain. Otherwise it is refused as `UnexpectedEoRhs`, its token left on the input, in
/// both modes.
#[test]
fn an_adjacent_row_is_declined_chained_or_refused() {
  let got = fatal::<true, _>("1@;", |input| {
    checked(input, Operand::STRICT, 20, usize::MAX)
  });
  assert_eq!(got.result, Ok(()));
  assert_eq!(got.landing.next, Some((b'@', 1)));

  let got = fatal::<true, _>("1<2@;", |input| open(input, Operand::STRICT));
  assert_eq!(got.result, Err(Fail::Chain { at: 3 }));
  assert_eq!(got.landing.next, Some((b'@', 3)));

  let refused = Err(Fail::EoRhs {
    at: 1,
    terminal: false,
  });
  let got = fatal::<true, _>("1@;", |input| open(input, Operand::STRICT));
  assert_eq!(got.result, refused);
  assert_eq!(got.landing.next, Some((b'@', 1)));
  let got = fatal::<true, _>("1@;", |input| bare(input, Operand::STRICT));
  assert_eq!(got.result, refused);
  assert_eq!(got.landing.next, Some((b'@', 1)));
}

// ── A scanner stop ────────────────────────────────────────────────────────────────────────────

/// A scanner stop at a head is the terminal end-of-input error, never an ending and never a
/// missing operand, at a term and at an operator, with an accepting emitter and with a rejecting
/// one.
#[test]
fn a_scanner_stop_is_the_terminal_error() {
  for (budget, at) in [(0, 0), (1, 1)] {
    let stop = Err(Fail::Eot { at, terminal: true });

    let mut items = Items::default();
    let rejecting = Parser::with_context(
      ParserContext::new(Fatal::new()).with_token_budget(TokenBudget::with_limitation(budget)),
    )
    .apply(|input: &mut FatalIr<'_, '_, true>| {
      Ok(flat(TABLE, Operand::STRICT).parse_into(input, &mut items))
    })
    .parse_str("1+2;")
    .expect("the parse's own error is caught");
    assert_eq!(rejecting, stop, "budget {budget}, a rejecting emitter");

    let accepting = Parser::with_context(
      ParserContext::new(Verbose::new()).with_token_budget(TokenBudget::with_limitation(budget)),
    )
    .apply(|input: &mut VerboseIr<'_, '_, true>| {
      Ok(flat(TABLE, Operand::STRICT).parse_into(input, &mut items))
    })
    .parse_str("1+2;")
    .expect("the parse's own error is caught");
    assert_eq!(accepting, stop, "budget {budget}, an accepting emitter");
  }
}

/// A scanner stop the operand parser met and declined over is the terminal error, never a
/// missing operand. Over `42` within a budget of one item, an operand parser that wants two digits
/// peeks at both, finds the window cut short, and declines, with `4` left in the cache, in both
/// modes.
#[test]
fn a_decline_over_a_scanner_stop_is_the_terminal_error() {
  let stop = Err(Fail::Eot {
    at: 0,
    terminal: true,
  });
  for bare in [false, true] {
    let got = within::<true, _>("42", 1, |input| {
      let mut items = Items::default();
      Ok(if bare {
        flat(TABLE, TwoDigits).bare().parse_into(input, &mut items)
      } else {
        flat(TABLE, TwoDigits).parse_into(input, &mut items)
      })
    });
    assert_eq!(got, stop, "bare: {bare}");
  }
}

/// Tells no one, and after each operand peeks two tokens ahead: a handler whose lookahead can meet
/// a scanner stop and return `Ok` over it.
struct Peeks;

impl<'inp, const SKIP: bool, Ctx> PrattHandler<'inp, u8, ByteLexer<'inp, SKIP>, String, Ctx>
  for Peeks
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  fn on_prefix(
    &mut self,
    _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    _operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), Fail> {
    Ok(())
  }

  fn on_operand(
    &mut self,
    input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    _operand: String,
  ) -> Result<(), Fail> {
    let _ = input.peek::<U2>()?;
    Ok(())
  }

  fn on_infix(
    &mut self,
    _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    _operator: Precedenced<
      PrattInfix<Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>>,
      u8,
    >,
  ) -> Result<(), Fail> {
    Ok(())
  }

  fn on_postfix(
    &mut self,
    _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    _operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), Fail> {
    Ok(())
  }
}

/// A stop a handler's lookahead met is the terminal error at the expression's end too. Over `1.2`
/// within a budget of two items, the handler peeks past `1`, caches `.` and trips on `2`, and
/// returns `Ok`. The `.` it left is no operator, and the expression would end there with `1`, in
/// both modes.
#[test]
fn a_stop_a_handler_met_ends_the_expression_in_the_stop() {
  let stop = Err(Fail::Eot {
    at: 1,
    terminal: true,
  });
  for bare in [false, true] {
    let got = within::<true, _>("1.2", 2, |input| {
      Ok(if bare {
        flat(TABLE, Operand::STRICT)
          .bare()
          .parse_into(input, &mut Peeks)
      } else {
        flat(TABLE, Operand::STRICT).parse_into(input, &mut Peeks)
      })
    });
    assert_eq!(got, stop, "bare: {bare}");
  }
}

/// A stop a rollback took off the input is still the terminal error, read off the trip counter.
/// Over `42` under a lexer-side limit of one token, an operand parser reads both digits inside an
/// attempt, trips on `2`, and rolls back, which restores the limit's count and clears the latch.
/// The driver's own read then finds `4`, no prefix operator, in both modes.
#[test]
fn a_stop_a_rollback_erased_is_still_the_terminal_error() {
  let stop = Err(Fail::Eot {
    at: 0,
    terminal: true,
  });
  for bare in [false, true] {
    let got = Parser::with_context(ParserContext::new(Fatal::new()))
      .apply(|input: &mut LimitedIr<'_, '_>| {
        Ok(if bare {
          flat(TABLE, Rolling).bare().parse_into(input, &mut ())
        } else {
          flat(TABLE, Rolling).parse_into(input, &mut ())
        })
      })
      .parse_with_state("42", TokenLimiter::with_limitation(1))
      .expect("the parse's own error is caught");
    assert_eq!(got, stop, "bare: {bare}");
  }
}

/// A trip after a re-key is still a trip. Over `42`, an operand parser installs a lexer-side limit
/// of one token, reads both digits inside an attempt, trips on `2`, and rolls back to a point after
/// the new limit. The regime changed only before the trip, and recovering by re-keying is the
/// caller's: the refusal is terminal, in both modes.
#[test]
fn a_trip_after_a_re_key_is_still_the_terminal_error() {
  let stop = Err(Fail::Eot {
    at: 0,
    terminal: true,
  });
  for bare in [false, true] {
    let got = Parser::with_context(ParserContext::new(Fatal::new()))
      .apply(|input: &mut LimitedIr<'_, '_>| {
        Ok(if bare {
          flat(TABLE, Rekeying).bare().parse_into(input, &mut ())
        } else {
          flat(TABLE, Rekeying).parse_into(input, &mut ())
        })
      })
      .parse_str("42")
      .expect("the parse's own error is caught");
    assert_eq!(got, stop, "bare: {bare}");
  }
}

/// Recovering from a stop is the caller's, once the expression has ended. Under a lexer-side limit
/// of one token, an expression over `12` trips inside its operand parser's attempt and ends in the
/// terminal stop. The caller then installs a wider limit, and the same expression parses.
#[test]
fn the_caller_recovers_from_a_stop_by_re_keying() {
  let got = Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(|input: &mut LimitedIr<'_, '_>| {
      let first = flat(TABLE, Rolling).parse_into(input, &mut ());
      input.set_state(TokenLimiter::with_limitation(8));
      let second = flat(TABLE, Rolling).parse_into(input, &mut ());
      Ok((first, second))
    })
    .parse_with_state("12", TokenLimiter::with_limitation(1))
    .expect("the parse's own error is caught");
  assert_eq!(
    got,
    (
      Err(Fail::Eot {
        at: 0,
        terminal: true
      }),
      Ok(())
    )
  );
}

/// A refused descent the operand parser caught is no absence either. Over `-1;` with a recursion
/// limit of 0, the operand parser's descent for a signed operand is refused, and it declines at
/// `-`: the driver does not take `-` for a prefix operator after it, and the parse ends in the
/// terminal stop at 0, in both modes.
#[test]
fn a_decline_over_a_refused_descent_is_the_terminal_error() {
  let stop = Err(Fail::Eot {
    at: 0,
    terminal: true,
  });
  for bare in [false, true] {
    let got = Parser::with_context(
      ParserContext::new(Fatal::new()).with_recursion_limiter(RecursionLimiter::with_limitation(0)),
    )
    .apply(|input: &mut FatalIr<'_, '_, true>| {
      Ok(if bare {
        flat(TABLE, Signed).bare().parse_into(input, &mut ())
      } else {
        flat(TABLE, Signed).parse_into(input, &mut ())
      })
    })
    .parse_str("-1;")
    .expect("the parse's own error is caught");
    assert_eq!(got, stop, "bare: {bare}");
  }
}

// ── The handler's contract ────────────────────────────────────────────────────────────────────

/// Each item is told once, in written order. No trivia token is told, and no operator the mode
/// left on the input.
#[test]
fn the_handler_is_told_each_item_once_in_written_order() {
  let got = fatal::<false, _>(" -1 * 2 + 3 ! = 4 ;", |input| {
    checked(input, Operand::STRICT, 6, usize::MAX)
  });
  assert_eq!(got.result, Ok(()));
  assert_eq!(
    got.items,
    vec![
      Item::Prefix(b'-', 1, 40),
      operand("1"),
      Item::Infix(b'*', 4, LEFT, 20),
      operand("2"),
      Item::Infix(b'+', 8, LEFT, 10),
      operand("3"),
      Item::Postfix(b'!', 12, 50),
    ]
  );
  assert_eq!(
    got.landing.next,
    Some((b'=', 14)),
    "`=` is below the floor and is not told"
  );
}

/// A handler that reads on: the rest of `<>` in `on_infix`, and a digit after `!` in `on_postfix`.
#[derive(Debug, Default)]
struct ReadsOn {
  items: Items,
  read: Vec<(u8, usize)>,
}

impl<'inp, const SKIP: bool, Ctx> PrattHandler<'inp, u8, ByteLexer<'inp, SKIP>, String, Ctx>
  for ReadsOn
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  fn on_prefix(
    &mut self,
    input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), Fail> {
    self.items.on_prefix(input, operator)
  }

  fn on_operand(
    &mut self,
    input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    operand: String,
  ) -> Result<(), Fail> {
    self.items.on_operand(input, operand)
  }

  fn on_infix(
    &mut self,
    input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    operator: Precedenced<
      PrattInfix<Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>>,
      u8,
    >,
  ) -> Result<(), Fail> {
    let (PrattInfix::Left(tok) | PrattInfix::Right(tok) | PrattInfix::Neither(tok)) =
      operator.token_ref();
    let spelled = tok.data().0 == b'<';
    self.items.on_infix(input, operator)?;
    if spelled && let Some(rest) = input.try_expect(|tok| tok.data().0 == b'>')? {
      self.read.push((rest.data().0, rest.span().start()));
    }
    Ok(())
  }

  fn on_postfix(
    &mut self,
    input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
    operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), Fail> {
    let indexed = operator.token_ref().data().0 == b'!';
    self.items.on_postfix(input, operator)?;
    if indexed && let Some(rest) = input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
      self.read.push((rest.data().0, rest.span().start()));
    }
    Ok(())
  }
}

/// What a handler commits lands in the expression, and the driver then reads the next head
/// afresh: after `<>` the operand, and after `!5` the operator.
#[test]
fn a_handler_that_reads_on_is_followed_by_a_fresh_read() {
  let got = fatal::<true, _>("1<>2!5+3;", |input| {
    let mut handler = ReadsOn::default();
    let result = flat(TABLE, Operand::STRICT).parse_into(input, &mut handler);
    Ok((result, handler, landing(input)?))
  });
  let (result, handler, landing) = got;
  assert_eq!(result, Ok(()));
  assert_eq!(handler.read, vec![(b'>', 2), (b'5', 5)]);
  assert_eq!(
    handler.items.0,
    vec![
      operand("1"),
      Item::Infix(b'<', 1, NEITHER, 15),
      operand("2"),
      Item::Postfix(b'!', 4, 50),
      Item::Infix(b'+', 6, LEFT, 10),
      operand("3"),
    ]
  );
  assert_eq!(landing.next, Some((b';', 8)));
}

// ── The two modes ─────────────────────────────────────────────────────────────────────────────

/// A bare driver takes a repeated non-associative operator, tells it, and reads on.
#[test]
fn a_bare_driver_takes_a_repeated_non_associative_operator() {
  let got = fatal::<true, _>("1<2<3;", |input| bare(input, Operand::STRICT));
  assert_eq!(got.result, Ok(()));
  assert_eq!(
    got.items,
    vec![
      operand("1"),
      Item::Infix(b'<', 1, NEITHER, 15),
      operand("2"),
      Item::Infix(b'<', 3, NEITHER, 15),
      operand("3"),
    ]
  );
  assert_eq!(got.landing.next, Some((b';', 5)));
}

/// A bare driver has no floor: the operator of the lowest power is taken, as is every operator its
/// table names.
#[test]
fn a_bare_driver_takes_the_lowest_power() {
  let got = fatal::<true, _>("1=2&3;", |input| bare(input, Operand::STRICT));
  assert_eq!(got.result, Ok(()));
  assert_eq!(
    got.items,
    vec![
      operand("1"),
      Item::Infix(b'=', 1, RIGHT, 5),
      operand("2"),
      Item::Infix(b'&', 3, RIGHT, 10),
      operand("3"),
    ]
  );
}

/// A bare driver's two missing-operand errors are a checked driver's, on the same inputs.
#[test]
fn a_bare_driver_reports_a_missing_operand_as_a_checked_one_does() {
  for src in ["1 + - ;", "1 + ;", ";", "- ;", "1 + - - ;"] {
    let checked = fatal::<true, _>(src, |input| open(input, Operand::STRICT));
    let bare = fatal::<true, _>(src, |input| bare(input, Operand::STRICT));
    assert!(checked.result.is_err(), "{src}");
    assert_eq!(bare.result, checked.result, "{src}");
    assert_eq!(bare.landing, checked.landing, "{src}");
    assert_eq!(bare.items, checked.items, "{src}");
  }
}

/// A bare driver is smaller than a checked one by the stack: it holds none.
#[test]
fn a_bare_driver_holds_no_stack() {
  type CheckedDriver = Flat<Expr, u8, Operand, Skip<'static>, String, FatalCtx<'static, true>>;
  type BareDriver =
    Flat<Expr, u8, Operand, Skip<'static>, String, FatalCtx<'static, true>, (), NoCst, Bare>;
  const _: () = assert!(core::mem::size_of::<Bare>() == 0);
  const _: () = assert!(core::mem::size_of::<BareDriver>() < core::mem::size_of::<CheckedDriver>());
  assert_eq!(
    core::mem::size_of::<BareDriver>(),
    core::mem::size_of::<Operand>()
  );
}
