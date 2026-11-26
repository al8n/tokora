//! The Pratt rows: the typed driver, the flat driver in its three readings, and the token-level
//! driver, each over one table and one source.
//!
//! These are the first rows of this binary that are no repetition engine. `workloads.rs` holds the
//! thirteen that are, and nothing here changes one of them: this module has its own registry, and
//! `main.rs` reads both.
//!
//! ## The rows
//!
//! | row | drives | for |
//! |---|---|---|
//! | `pratt_typed_left`, `pratt_typed_right` | `pratt()` over `PrattTable::lhs` and `rhs` | the flat driver's comparison, and 0.15.0's merge-base |
//! | `pratt_flat_fold_left`, `pratt_flat_fold_right` | `flat(..).folding(..)` | what evaluating as you parse costs against the typed driver |
//! | `pratt_flat_collect_left`, `pratt_flat_collect_right` | `flat(..).parse_into(..)` in `Checked` mode, with a handler that sums | what the unit machine costs |
//! | `pratt_flat_bare_left`, `pratt_flat_bare_right`, `pratt_flat_bare_shallow` | the same with `bare()` | what `Bare` costs |
//! | `pratt_flat_control_left`, `pratt_flat_control_right`, `pratt_flat_control_shallow` | the first design's loop, written by hand below, with the same handler | what `Bare` must equal |
//! | `pratt_token_left`, `pratt_token_right` | `InputRef::pratt` | the token-level driver's instrument (#366) |
//! | `pratt_token_shallow` | `InputRef::pratt`, over many expressions of one operator each | its per-expression costs |
//!
//! ## One table, one source
//!
//! Every row reads `PTok`'s `PrattToken` implementation, and a source is one of three: a
//! left-associative chain, a right-associative chain, and many expressions of one operator each.
//! `pratt_fixture.rs` has them, with the accumulators a driver must produce over each. A row
//! compares its driver's result with the accumulator before it counts anything, so a driver that
//! skips a fold, or an operator, fails the run rather than reading faster.
//!
//! ## Why the shallow source is there
//!
//! A chain of 63 operators divides a cost paid once for each expression by 63 before any threshold
//! sees it, and the machine's initialisation and its storage are costs of that kind. The shallow
//! source parses the same number of operands as a chain source, as 128 expressions of one operator
//! each where a chain source holds 4, so such a cost is paid 32 times as often there. It is the
//! same argument as the repetition rows' two depths (`workloads.rs`, *Deep and shallow*), and it
//! is why `bare`, its control and the token-level driver have a shallow row: the three whose
//! per-expression cost is in question.
//!
//! ## What the control is
//!
//! `control_drain` is the loop of the flat driver's first design, written out by hand: the operand
//! parser first, the head read with `try_expect_map_or_stop` and classified by the table, the
//! operator committed by the read that classified it, and one bit noted for which error a missing
//! operand is. It tells the same `Sum` handler the same items. `pratt_flat_bare_*` reads
//! `Flat::bare`, which is meant to compile to that loop, so each bare row is meant to read its
//! control's instruction count, and the difference of the two is the number to look at.
//!
//! What the control leaves out is the scanner-trip verdicts the library's loop takes where an
//! expression ends or refuses: their witness is crate-private, so the harness cannot call it. The
//! shallow row reads that, once for each expression. It does take the descent-trip baseline at
//! every term, which is public.
//!
//! A difference that a step of the mode explains is repaired in that step. A residue that no step
//! explains is not chased: two loops written apart can differ by a few instructions through
//! register allocation alone, and the crate has measured one extracted function moving three rows
//! by 0.5% to 2.4% that way (`tokora/src/combinator/many/macros.rs`). It goes to the owner, with a
//! diff of the two hot loops' disassembly.
//!
//! ## Recorded figures
//!
//! Instructions an iteration, from `ci/icount/measure.py`'s head-side reading in the `instruction
//! counts` job (`icount-readings`, `head.json`, `ir_per_iter`), rounded to a whole instruction. They
//! are from this branch's `perf` run 45744931146, on a GitHub-hosted `ubuntu-latest` runner (image
//! `ubuntu-26.04`, version 20261116.202.1); the readings do not record the CPU. A source holds 256
//! operands: 4 chains of 63 operators, or 128 expressions of one.
//!
//! | row | Ir/iteration | against |
//! |---|---:|---|
//! | `pratt_typed_left` | 347,119 | |
//! | `pratt_typed_right` | 352,951 | |
//! | `pratt_flat_fold_left` | 281,508 | 0.811 of `pratt_typed_left` |
//! | `pratt_flat_fold_right` | 264,452 | 0.749 of `pratt_typed_right` |
//! | `pratt_flat_collect_left` | 213,702 | |
//! | `pratt_flat_collect_right` | 201,376 | |
//! | `pratt_flat_bare_left` | 166,396 | +50 over `pratt_flat_control_left` |
//! | `pratt_flat_bare_right` | 166,396 | +50 over `pratt_flat_control_right` |
//! | `pratt_flat_bare_shallow` | 196,404 | +1,538 over `pratt_flat_control_shallow` |
//! | `pratt_flat_control_left` | 166,346 | |
//! | `pratt_flat_control_right` | 166,346 | |
//! | `pratt_flat_control_shallow` | 194,866 | |
//! | `pratt_token_left` | 203,059 | PR 3's before |
//! | `pratt_token_right` | 187,187 | PR 3's before |
//! | `pratt_token_shallow` | 210,871 | PR 3's before |
//!
//! `Checked` less `Bare`, for each operator, is the unit machine's price: the collect rows less the
//! bare rows, over 252 operators in a chain source.

use std::hint::black_box;

use tokora::{
  Emitter, EmitterView, InputRef, Parse, ParseAttempt, ParseContext, ParseInput, SimpleSpan, Token,
  emitter::PrattEmitter,
  error::{UnexpectedEoLhs, UnexpectedEoRhs},
  pratt::{PrattHandler, PrattInfix, PrattLHS, PrattRHS, Precedenced, flat, pratt},
  span::Spanned,
  token::PrattToken,
};

use crate::pratt_fixture::{
  PErr, PExpr, PLex, PTok, Sum, TABLE, fold_infix, fold_postfix, fold_prefix, left_chain,
  pratt_parser, right_chain, shallow, try_operand,
};

/// Runs `$step` once for each `;`-terminated expression to the end of input, and requires the
/// terminator after each. A missing or displaced one is an error rather than an exit, as in
/// `pratt_typed.rs`'s drain: a loop that ended on it would report success over a source it had
/// read one expression of.
macro_rules! drain_loop {
  ($inp:ident, $step:block) => {
    loop {
      // Scope the peek borrow: an empty input ends the drain before an expression is attempted
      // that is not there.
      let done = {
        let peeked = $inp.peek_one()?;
        peeked.is_none()
      };
      if done {
        break;
      }
      $step
      if $inp
        .try_expect(|t| matches!(t.data(), PTok::Semi))?
        .is_none()
      {
        return Err(PErr);
      }
    }
  };
}

// ── The typed driver ────────────────────────────────────────────────────────────────────────

fn typed_drain<'inp, Ctx>(inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>) -> Result<i64, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  let mut driver = pratt(
    TABLE.lhs(try_operand),
    TABLE.rhs(),
    fold_prefix,
    fold_infix,
    fold_postfix,
  );
  let mut acc = 0i64;
  drain_loop!(inp, {
    acc = acc.wrapping_add(driver.parse_input(inp)?);
  });
  Ok(acc)
}

// ── The flat driver ─────────────────────────────────────────────────────────────────────────

fn flat_fold_drain<'inp, Ctx>(inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>) -> Result<i64, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  let mut driver = flat(TABLE, try_operand).folding(fold_prefix, fold_infix, fold_postfix);
  let mut acc = 0i64;
  drain_loop!(inp, {
    acc = acc.wrapping_add(driver.parse_input(inp)?);
  });
  Ok(acc)
}

fn flat_collect_drain<'inp, Ctx>(inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>) -> Result<i64, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  let mut driver = flat(TABLE, try_operand);
  let mut sum = Sum(0);
  drain_loop!(inp, {
    driver.parse_into(inp, &mut sum)?;
  });
  Ok(sum.0)
}

fn flat_bare_drain<'inp, Ctx>(inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>) -> Result<i64, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  let mut driver = flat(TABLE, try_operand).bare();
  let mut sum = Sum(0);
  drain_loop!(inp, {
    driver.parse_into(inp, &mut sum)?;
  });
  Ok(sum.0)
}

// ── The control ─────────────────────────────────────────────────────────────────────────────

/// What the read at a term found, where the operand parser declined.
enum TermHead {
  /// A trivia token, committed.
  Trivia,
  /// A prefix operator of the table, committed, at its power.
  Prefix(i64),
}

/// What the read after an operand found.
enum OperatorHead {
  /// A trivia token, committed.
  Trivia,
  /// An infix operator, committed, with its associativity and power.
  Infix(PrattInfix<(), (), ()>, i64),
  /// A postfix operator, committed, at its power.
  Postfix(i64),
}

/// One expression, read by the loop `pratt_flat_bare_*` is meant to compile to, telling `handler`
/// each item as written.
#[inline(always)]
fn control_expression<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>,
  handler: &mut Sum,
) -> Result<(), PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  // The one bit: whether the operator read last is an infix operator, which then waits for its
  // right operand. It decides which error a missing operand is.
  let mut infix_waits = false;
  loop {
    // A term. The operand parser is asked first, and a token it declines is staged.
    let trips = inp.trip_snapshot();
    if let ParseAttempt::Accept(value) = try_operand(inp)? {
      handler.on_operand(inp, value)?;
    } else {
      if inp.tripped_during_attempt(trips) {
        return Err(PErr);
      }
      let head = inp.try_expect_map_or_stop(|tok| {
        let token: &PTok = tok.data();
        if <PTok as Token<'_>>::is_trivia(token) {
          return Some(TermHead::Trivia);
        }
        match <PTok as PrattToken<'_, PExpr, i64>>::try_pratt_lhs(token) {
          Some(PrattLHS::Prefix(prefix)) => Some(TermHead::Prefix(prefix.into_precedence())),
          Some(PrattLHS::Operand(())) | None => None,
        }
      })?;
      match head {
        Some((TermHead::Trivia, _)) => {}
        Some((TermHead::Prefix(power), operator)) => {
          infix_waits = false;
          handler.on_prefix(inp, Precedenced::new(operator, power))?;
        }
        None => {
          let at = inp.span().end();
          return Err(if infix_waits {
            UnexpectedEoRhs::<usize, ()>::eorhs_of(at).into()
          } else {
            UnexpectedEoLhs::<usize, ()>::eolhs_of(at).into()
          });
        }
      }
      continue;
    }

    // An operator, or the ending.
    loop {
      let mut adjacent = false;
      let head = inp.try_expect_map_or_stop(|tok| {
        let token: &PTok = tok.data();
        if <PTok as Token<'_>>::is_trivia(token) {
          return Some(OperatorHead::Trivia);
        }
        match <PTok as PrattToken<'_, PExpr, i64>>::try_pratt_rhs(token)? {
          PrattRHS::Infix(infix) => {
            let (associativity, power) = infix.into_components();
            Some(OperatorHead::Infix(associativity, power))
          }
          PrattRHS::Postfix(postfix) => Some(OperatorHead::Postfix(postfix.into_precedence())),
          PrattRHS::Adjacent(_) => {
            adjacent = true;
            None
          }
          PrattRHS::End => None,
        }
      })?;
      match head {
        Some((OperatorHead::Trivia, _)) => {}
        Some((OperatorHead::Infix(associativity, power), operator)) => {
          infix_waits = true;
          let operator = match associativity {
            PrattInfix::Left(()) => PrattInfix::Left(operator),
            PrattInfix::Right(()) => PrattInfix::Right(operator),
            PrattInfix::Neither(()) => PrattInfix::Neither(operator),
          };
          handler.on_infix(inp, Precedenced::new(operator, power))?;
          break;
        }
        Some((OperatorHead::Postfix(power), operator)) => {
          handler.on_postfix(inp, Precedenced::new(operator, power))?;
        }
        None => {
          return if adjacent {
            Err(UnexpectedEoRhs::<usize, ()>::eorhs_of(inp.span().end()).into())
          } else {
            Ok(())
          };
        }
      }
    }
  }
}

fn control_drain<'inp, Ctx>(inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>) -> Result<i64, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  let mut sum = Sum(0);
  drain_loop!(inp, {
    control_expression(inp, &mut sum)?;
  });
  Ok(sum.0)
}

// ── The token-level driver ──────────────────────────────────────────────────────────────────

fn int_of(tok: &Spanned<PTok, SimpleSpan>) -> i64 {
  match tok.data() {
    PTok::Int(n) => *n,
    _ => 0,
  }
}

/// Named `fn`s, not closures: the fold bounds are higher-ranked over the emitter borrow, which a
/// closure cannot satisfy. The arithmetic is `fold_infix`'s, so the token-level rows owe the
/// accumulator the AST rows do.
fn token_prefix<'inp, E>(
  op: Spanned<PTok, SimpleSpan>,
  operand: Spanned<PTok, SimpleSpan>,
  _: EmitterView<'_, 'inp, PLex<'inp>, E>,
) -> Result<Spanned<PTok, SimpleSpan>, PErr> {
  Ok(Spanned::new(
    op.span(),
    PTok::Int(int_of(&operand).wrapping_neg()),
  ))
}

fn token_infix<'inp, E>(
  left: Spanned<PTok, SimpleSpan>,
  right: Spanned<PTok, SimpleSpan>,
  _infix: Spanned<PrattInfix<PTok, PTok, PTok>, SimpleSpan>,
  _: EmitterView<'_, 'inp, PLex<'inp>, E>,
) -> Result<Spanned<PTok, SimpleSpan>, PErr> {
  let span = left.span();
  let value = int_of(&left).wrapping_mul(31).wrapping_add(int_of(&right));
  Ok(Spanned::new(span, PTok::Int(value)))
}

fn token_postfix<'inp, E>(
  operand: Spanned<PTok, SimpleSpan>,
  _close: Spanned<PTok, SimpleSpan>,
  _: EmitterView<'_, 'inp, PLex<'inp>, E>,
) -> Result<Spanned<PTok, SimpleSpan>, PErr> {
  Ok(operand)
}

fn token_drain<'inp, Ctx>(inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>) -> Result<i64, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr> + PrattEmitter<'inp, PLex<'inp>>,
{
  let mut acc = 0i64;
  drain_loop!(inp, {
    match inp.pratt::<_, _, _, PExpr, i64>(
      token_prefix::<Ctx::Emitter>,
      token_infix::<Ctx::Emitter>,
      token_postfix::<Ctx::Emitter>,
    )? {
      Some(tok) => acc = acc.wrapping_add(int_of(&tok)),
      // The source holds an expression where one was asked for, so this is a fixture that has
      // drifted, and it is loud.
      None => return Err(PErr),
    }
  });
  Ok(acc)
}

// ── The registry ────────────────────────────────────────────────────────────────────────────

/// The checksum one iteration adds to the run's: the accumulator reduced to something small and
/// never zero. `ci/icount/measure.py` requires the final checksum to scale exactly with the
/// iteration count, as integers, so it must not wrap; and a run that parsed nothing reads zero.
fn reduce(got: i64) -> usize {
  got.rem_euclid(1_000_003) as usize + 1
}

/// The failure arm of a row's check, out of line so that the check itself is a compare and a
/// branch.
#[cold]
#[inline(never)]
fn mismatch(row: &str, expected: i64, got: i64) -> ! {
  panic!(
    "{row}: the fixture parsed, but to the wrong accumulator: expected {expected}, got {got}. \
     The expectation is computed by the source's generator from the operands it wrote, never by \
     running a parser, so a driver consumed the whole source while folding, or telling, something \
     other than what it describes. A row that does less work than it should reads fewer \
     instructions, and that is the shape this check refuses."
  )
}

/// The one place a Pratt row's name is written, as `workloads!` is the one place a repetition
/// row's is: it generates `NAMES` and the `match` that runs one.
///
/// A row names its driver, its source, and which of the source's two accumulators the driver
/// owes: `fold` for the AST drivers, `collect` for what `parse_into` tells a `Sum`.
macro_rules! pratt_workloads {
  ($($name:literal => $driver:ident($src:expr, $expect:ident)),* $(,)?) => {
    /// Every Pratt row, in the order the table above has them.
    pub const NAMES: &[&str] = &[$($name),*];

    /// Runs `iters` iterations of `name`, returning a checksum folded over every iteration, or
    /// `None` if the name is not in this table.
    pub fn run(name: &str, iters: u64) -> Option<usize> {
      match name {
        $(
          $name => {
            let src = $src;
            let text: &str = src.text.as_str();
            let mut acc = 0usize;
            for _ in 0..iters {
              let got = pratt_parser()
                .apply($driver)
                .parse_str(black_box(text))
                .expect(concat!($name, ": the fixture no longer parses"));
              if got != src.$expect {
                mismatch($name, src.$expect, got);
              }
              acc = acc.wrapping_add(black_box(reduce(got)));
            }
            Some(acc)
          }
        )*
        _ => None,
      }
    }
  };
}

pratt_workloads! {
  "pratt_typed_left" => typed_drain(left_chain(), fold),
  "pratt_typed_right" => typed_drain(right_chain(), fold),
  "pratt_flat_fold_left" => flat_fold_drain(left_chain(), fold),
  "pratt_flat_fold_right" => flat_fold_drain(right_chain(), fold),
  "pratt_flat_collect_left" => flat_collect_drain(left_chain(), collect),
  "pratt_flat_collect_right" => flat_collect_drain(right_chain(), collect),
  "pratt_flat_bare_left" => flat_bare_drain(left_chain(), collect),
  "pratt_flat_bare_right" => flat_bare_drain(right_chain(), collect),
  "pratt_flat_bare_shallow" => flat_bare_drain(shallow(), collect),
  "pratt_flat_control_left" => control_drain(left_chain(), collect),
  "pratt_flat_control_right" => control_drain(right_chain(), collect),
  "pratt_flat_control_shallow" => control_drain(shallow(), collect),
  "pratt_token_left" => token_drain(left_chain(), fold),
  "pratt_token_right" => token_drain(right_chain(), fold),
  "pratt_token_shallow" => token_drain(shallow(), fold),
}
