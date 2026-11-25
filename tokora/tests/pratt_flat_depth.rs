#![cfg(all(feature = "std", feature = "combinators", feature = "logos_0_16"))]

//! Deep chains on a small stack: the flat driver's `parse_into`, in both modes, its `folding`, and
//! a machine fed from a slice, each parse a chain of 2^17 operators on a thread of 256 KiB, an
//! eighth of the stack tokora's figures are stated on. Then the cap: under either adapter, the
//! operator past `max_depth` is a full stack, terminal, and recovery re-raises it.

mod flat_fixture;

use core::cell::Cell;
use std::thread;

use tokora::{
  Emitter, InputRef, Parse, ParseAttempt, ParseContext, ParseInput, Parser, ParserContext,
  SimpleSpan, TryParseInput,
  emitter::Fatal,
  error::MaybeTerminal,
  pratt::{
    PrattHandler, PrattInfix, PrattPoll, PrattRHS, PrattReduction, PrattResolver, Precedenced,
    flat, pratt,
  },
  span::Spanned,
};

use flat_fixture::{Fail, FatalIr, LexErr, Operand, Skip, TABLE, Tok, landing};

/// The stack each deep case runs on.
const STACK: usize = 256 * 1024;

/// How many operators a deep chain holds. Miri interprets each step, so it reads a short chain.
const DEPTH: usize = if cfg!(miri) { 1 << 6 } else { 1 << 17 };

/// Runs `body` on a thread of [`STACK`] bytes, and hands back what it returns.
fn on_a_small_stack<T: Send + 'static>(body: impl FnOnce() -> T + Send + 'static) -> T {
  thread::Builder::new()
    .stack_size(STACK)
    .spawn(body)
    .expect("a thread of 256 KiB is spawned")
    .join()
    .expect("the parse finished on the small stack")
}

/// Counts the items it is told.
#[derive(Debug, Default)]
struct Count(usize);

type ErrorIn<'inp, Ctx> =
  <<Ctx as ParseContext<'inp, Skip<'inp>>>::Emitter as Emitter<'inp, Skip<'inp>>>::Error;

impl<'inp, Ctx> PrattHandler<'inp, u8, Skip<'inp>, String, Ctx> for Count
where
  Ctx: ParseContext<'inp, Skip<'inp>>,
{
  fn on_prefix(
    &mut self,
    _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
    _operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 += 1;
    Ok(())
  }

  fn on_operand(
    &mut self,
    _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
    _operand: String,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 += 1;
    Ok(())
  }

  fn on_infix(
    &mut self,
    _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
    _operator: Precedenced<
      PrattInfix<Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>>,
      u8,
    >,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 += 1;
    Ok(())
  }

  fn on_postfix(
    &mut self,
    _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
    _operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 += 1;
    Ok(())
  }
}

/// `n` operators of a chain: `1+1+…+1;`, `1^1^…^1;`, or `--…-1;`.
#[derive(Debug, Clone, Copy)]
enum Chain {
  Left,
  Right,
  Prefix,
}

impl Chain {
  fn source(self, n: usize) -> String {
    let mut src = String::new();
    match self {
      Chain::Left | Chain::Right => {
        let operator = if matches!(self, Chain::Left) {
          '+'
        } else {
          '^'
        };
        src.push('1');
        for _ in 0..n {
          src.push(operator);
          src.push('1');
        }
      }
      Chain::Prefix => {
        for _ in 0..n {
          src.push('-');
        }
        src.push('1');
      }
    }
    src.push(';');
    src
  }

  /// The items a parse of `n` operators tells.
  fn items(self, n: usize) -> usize {
    match self {
      Chain::Left | Chain::Right => 2 * n + 1,
      Chain::Prefix => n + 1,
    }
  }
}

/// Parses `n` operators of `chain` into a counting handler, in `Checked` mode or in `Bare` mode,
/// and hands back the result, the count and the token after the expression.
fn parse(chain: Chain, n: usize, bare: bool) -> (Result<(), Fail>, usize, Option<(u8, usize)>) {
  let src = chain.source(n);
  Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(|input: &mut FatalIr<'_, '_, true>| {
      let mut count = Count::default();
      let result = if bare {
        flat(TABLE, Operand::STRICT)
          .bare()
          .parse_into(input, &mut count)
      } else {
        flat(TABLE, Operand::STRICT).parse_into(input, &mut count)
      };
      Ok((result, count.0, landing(input)?.next))
    })
    .parse_str(&src)
    .expect("the parse's own error is caught")
}

/// A digit, as one operand. It crosses no trivia, and declines anything else.
#[derive(Debug, Clone, Copy)]
struct One;

impl<'inp, Ctx> TryParseInput<'inp, Skip<'inp>, u32, Ctx> for One
where
  Ctx: ParseContext<'inp, Skip<'inp>>,
{
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
  ) -> Result<ParseAttempt<u32>, ErrorIn<'inp, Ctx>> {
    Ok(
      match input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
        Some(_) => ParseAttempt::Accept(1),
        None => ParseAttempt::Decline,
      },
    )
  }
}

/// A prefix or a postfix operator's fold: the count of operands it holds is its operand's.
fn keep<'inp, Ctx>(
  _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
  operand: u32,
  _operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
) -> Result<u32, ErrorIn<'inp, Ctx>>
where
  Ctx: ParseContext<'inp, Skip<'inp>>,
{
  Ok(operand)
}

/// An infix operator's fold: it holds its two operands' counts.
fn add<'inp, Ctx>(
  _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
  left: u32,
  right: u32,
  _operator: Precedenced<
    PrattInfix<Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>>,
    u8,
  >,
) -> Result<u32, ErrorIn<'inp, Ctx>>
where
  Ctx: ParseContext<'inp, Skip<'inp>>,
{
  Ok(left + right)
}

/// Folds `n` operators of `chain` into the count of its operands, and hands back the result and
/// the token after the expression. The folds build nothing that nests, so a deep chain measures the
/// driver, and no destructor.
fn fold(chain: Chain, n: usize) -> (Result<u32, Fail>, Option<(u8, usize)>) {
  let src = chain.source(n);
  Parser::with_context(ParserContext::new(Fatal::new()))
    .apply(|input: &mut FatalIr<'_, '_, true>| {
      let result = flat(TABLE, One).folding(keep, add, keep).parse_input(input);
      Ok((result, landing(input)?.next))
    })
    .parse_str(&src)
    .expect("the parse's own error is caught")
}

/// How many operands `n` operators of `chain` hold.
fn operands(chain: Chain, n: usize) -> u32 {
  let operands = match chain {
    Chain::Left | Chain::Right => n + 1,
    Chain::Prefix => 1,
  };
  u32::try_from(operands).expect("a chain this suite builds")
}

/// One operator, on the same thread: the measure of a stack too small for any parse at all.
#[test]
fn a_control_of_one_operator_parses_on_the_small_stack() {
  for bare in [false, true] {
    let got = on_a_small_stack(move || parse(Chain::Left, 1, bare));
    assert_eq!(got, (Ok(()), 3, Some((b';', 3))), "bare: {bare}");
  }
  let got = on_a_small_stack(|| fold(Chain::Left, 1));
  assert_eq!(got, (Ok(2), Some((b';', 3))), "folding");
}

/// 2^17 operators of each shape fold on the small stack, to the count a fold of integers gives:
/// `folding`'s native stack does not grow with the operators pending.
#[test]
fn a_deep_chain_folds_on_the_small_stack() {
  for chain in [Chain::Left, Chain::Right, Chain::Prefix] {
    let got = on_a_small_stack(move || fold(chain, DEPTH));
    let end = chain.source(DEPTH).len() - 1;
    assert_eq!(
      got,
      (Ok(operands(chain, DEPTH)), Some((b';', end))),
      "{chain:?}"
    );
  }
}

/// For contrast, the typed driver refuses the right chain and the prefix chain at the input's
/// recursion budget. It runs on the test's own thread: its frames are the native stack's.
#[test]
fn the_typed_driver_refuses_a_deep_right_chain_and_a_deep_prefix_chain() {
  for chain in [Chain::Right, Chain::Prefix] {
    let src = chain.source(DEPTH);
    let got = Parser::with_context(ParserContext::new(Fatal::new()))
      .apply(|input: &mut FatalIr<'_, '_, true>| {
        Ok(pratt(TABLE.lhs(One), TABLE.rhs(), keep, add, keep).parse_input(input))
      })
      .parse_str(&src)
      .expect("the parse's own error is caught");
    assert!(matches!(got, Err(Fail::Limit { .. })), "{chain:?}: {got:?}");
  }
}

/// 2^17 operators of each shape parse, in both modes, on the small stack: the driver's native
/// stack does not grow with the operators pending.
#[test]
fn a_deep_chain_parses_into_a_handler_on_the_small_stack() {
  for chain in [Chain::Left, Chain::Right, Chain::Prefix] {
    for bare in [false, true] {
      let got = on_a_small_stack(move || parse(chain, DEPTH, bare));
      let end = chain.source(DEPTH).len() - 1;
      assert_eq!(
        got,
        (Ok(()), chain.items(DEPTH), Some((b';', end))),
        "{chain:?}, bare: {bare}"
      );
    }
  }
}

/// What a machine fed from a slice is fed.
#[derive(Debug, Clone, Copy)]
enum Feed {
  Operand,
  Prefix,
  Infix(PrattInfix<(), (), ()>, u8),
}

type Counting = PrattResolver<u32, (), (), (), (), (), u8>;

/// The value of one reduction: how many operands it holds.
fn value(reduction: PrattReduction<u32, (), (), (), (), (), u8>) -> u32 {
  match reduction {
    PrattReduction::Prefix(prefix) => prefix.into_components().0,
    PrattReduction::Infix(infix) => {
      let (left, right, _) = infix.into_components();
      left + right
    }
    PrattReduction::Postfix(postfix) => postfix.into_components().0,
  }
}

fn reduce(machine: &mut Counting) {
  match machine.poll() {
    PrattPoll::Reduce(reduction) => machine
      .handle_reduced(value(reduction))
      .expect("the machine takes its own reduction's value"),
    PrattPoll::Term | PrattPoll::Operator | PrattPoll::Reducing | PrattPoll::Done(_) => {
      panic!("the machine owed a reduction")
    }
  }
}

/// Feeds `items` to a machine, with no input, and resolves them to how many operands they hold.
fn feed(items: &[Feed]) -> u32 {
  let mut machine = Counting::new();
  for item in items {
    match item {
      Feed::Operand => machine.handle_operand(1).expect("an operand in a term"),
      Feed::Prefix => machine
        .handle_prefix(Precedenced::new((), 40))
        .expect("a prefix operator in a term"),
      Feed::Infix(associativity, power) => {
        let offered = PrattRHS::Infix(Precedenced::new(*associativity, *power));
        while machine.offer(&offered).is_reduce() {
          machine
            .close()
            .expect("an operand ends where a reduction is owed");
          reduce(&mut machine);
        }
        machine
          .handle_infix(Precedenced::new(*associativity, *power))
          .expect("an infix operator the offer takes");
      }
    }
  }
  machine
    .finish()
    .expect("the expression ends after an operand");
  loop {
    match machine.poll() {
      PrattPoll::Reduce(reduction) => machine
        .handle_reduced(value(reduction))
        .expect("the machine takes its own reduction's value"),
      PrattPoll::Done(operands) => return operands,
      PrattPoll::Term | PrattPoll::Operator | PrattPoll::Reducing => {
        panic!("after `finish` the machine reduces, and then it is done")
      }
    }
  }
}

/// The same three chains, fed to a machine from a slice, resolve on the small stack.
#[test]
fn a_machine_fed_from_a_slice_resolves_a_deep_chain_on_the_small_stack() {
  for chain in [Chain::Left, Chain::Right, Chain::Prefix] {
    let items: Vec<Feed> = match chain {
      Chain::Left | Chain::Right => {
        let infix = if matches!(chain, Chain::Left) {
          Feed::Infix(PrattInfix::Left(()), 10)
        } else {
          Feed::Infix(PrattInfix::Right(()), 30)
        };
        let mut items = vec![Feed::Operand];
        for _ in 0..DEPTH {
          items.push(infix);
          items.push(Feed::Operand);
        }
        items
      }
      Chain::Prefix => {
        let mut items = vec![Feed::Prefix; DEPTH];
        items.push(Feed::Operand);
        items
      }
    };
    let operands = on_a_small_stack(move || feed(&items));
    let expected = match chain {
      Chain::Left | Chain::Right => DEPTH + 1,
      Chain::Prefix => 1,
    };
    assert_eq!(operands as usize, expected, "{chain:?}");
  }
}

// ── The cap ───────────────────────────────────────────────────────────────────────────────────

/// `1^1^…` with 70 operators: under a cap of 64 the 65th is refused. It sits at 129 and ends at 130.
fn capped_chain() -> String {
  Chain::Right.source(70)
}

/// The 65th pending operator under `max_depth(64)` is `PrattStackFull`, terminal, at the end of
/// that operator, reporting 64, and the trip is counted on the input session, under either adapter.
#[test]
fn the_operator_past_a_cap_of_64_is_a_full_stack() {
  let src = capped_chain();
  for folding in [false, true] {
    let got = Parser::with_context(ParserContext::new(Fatal::new()))
      .apply(|input: &mut FatalIr<'_, '_, true>| {
        let baseline = input.trip_snapshot();
        let result = if folding {
          flat(TABLE, One)
            .max_depth(64)
            .folding(keep, add, keep)
            .parse_input(input)
            .map(|_| ())
        } else {
          flat(TABLE, Operand::STRICT)
            .max_depth(64)
            .parse_into(input, &mut Count::default())
        };
        Ok((result, input.tripped_during_attempt(baseline)))
      })
      .parse_str(&src)
      .expect("the parse's own error is caught");
    let (result, tripped) = got;
    let full = Fail::Full {
      at: 130,
      capacity: 64,
    };
    assert_eq!(result, Err(full.clone()), "folding: {folding}");
    assert!(full.is_terminal());
    assert!(
      tripped,
      "folding: {folding}: the trip is counted on the input session"
    );
  }
}

thread_local! {
  /// How many times a recoverer below ran.
  static RECOVERED: Cell<usize> = const { Cell::new(0) };
}

/// A recoverer that would synthesize a value, if it were ever run.
fn recoverer<'inp, Ctx, E>(
  _input: &mut InputRef<'inp, '_, Skip<'inp>, Ctx>,
  _error: E,
) -> Result<(), E>
where
  Ctx: ParseContext<'inp, Skip<'inp>>,
  Ctx::Emitter: Emitter<'inp, Skip<'inp>, Error = E>,
{
  RECOVERED.with(|ran| ran.set(ran.get() + 1));
  Ok(())
}

/// `recover` re-raises the full stack under a delegating error type, under either adapter: the
/// error is terminal.
#[test]
fn recovery_re_raises_a_full_stack_under_a_delegating_error() {
  RECOVERED.with(|ran| ran.set(0));
  let src = capped_chain();
  for folding in [false, true] {
    let got = Parser::with_context(ParserContext::new(Fatal::new()))
      .apply(|input: &mut FatalIr<'_, '_, true>| {
        let primary = |input: &mut FatalIr<'_, '_, true>| {
          if folding {
            flat(TABLE, One)
              .max_depth(64)
              .folding(keep, add, keep)
              .parse_input(input)
              .map(|_| ())
          } else {
            flat(TABLE, Operand::STRICT)
              .max_depth(64)
              .parse_into(input, &mut Count::default())
          }
        };
        Ok(primary.recover(recoverer).parse_input(input))
      })
      .parse_str(&src)
      .expect("the parse's own error is caught");
    assert_eq!(
      got,
      Err(Fail::Full {
        at: 130,
        capacity: 64
      }),
      "folding: {folding}"
    );
  }
  assert_eq!(RECOVERED.with(Cell::get), 0, "the recoverer never ran");
}

/// The unit sink discards the value, and with it the terminal mark. The trip counted on the input
/// session is what `recover` reads beside it, so it re-raises all the same.
impl From<LexErr> for () {
  fn from(_: LexErr) -> Self {}
}

type UnitIr<'inp, 'c> = InputRef<'inp, 'c, Skip<'inp>, ParserContext<'inp, Skip<'inp>, Fatal<()>>>;

/// `recover` re-raises the full stack under `()`, which keeps no terminal mark of its own, under
/// either adapter.
#[test]
fn recovery_re_raises_a_full_stack_under_the_unit_error() {
  RECOVERED.with(|ran| ran.set(0));
  let src = capped_chain();
  for folding in [false, true] {
    let got = Parser::with_context(ParserContext::new(Fatal::<()>::new()))
      .apply(|input: &mut UnitIr<'_, '_>| {
        let primary = |input: &mut UnitIr<'_, '_>| {
          if folding {
            flat(TABLE, One)
              .max_depth(64)
              .folding(keep, add, keep)
              .parse_input(input)
              .map(|_| ())
          } else {
            flat(TABLE, Operand::STRICT)
              .max_depth(64)
              .parse_into(input, &mut Count::default())
          }
        };
        Ok(primary.recover(recoverer).parse_input(input))
      })
      .parse_str(&src)
      .expect("the parse's own error is caught");
    assert_eq!(got, Err(()), "folding: {folding}");
  }
  assert_eq!(RECOVERED.with(Cell::get), 0, "the recoverer never ran");
}
