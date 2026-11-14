#![cfg(all(feature = "std", feature = "combinators"))]

//! A head a non-final partial input withholds is lexed **once**.
//!
//! The frontier rules withhold an item whose decision reached the end of a non-final buffer, and
//! the lex that decided so is charged to the token budget. A peek that meets such a head keeps
//! the verdict, and the consume that follows at the same place returns the same `Incomplete`
//! without lexing the head again or charging for it again. Lexing it twice charged it twice, and
//! where the peek had spent the budget the consume met a refusal and reported a **terminal** end
//! of input over a buffer that only needed more bytes.
//!
//! The verdict is kept under everything that decides it — the offset its run began at, the lexer
//! regime, the buffer's length and finality, and the dedup watermark its run's diagnostics
//! raised — and the scenarios here change one of those at a time.
//!
//! The lexer is hand-written and conforming: it reads nothing past a token but the terminator at
//! the token's end, and says so with [`ReadFrontier::SpanEnd`], so a token is withheld exactly when
//! its span reaches the end of a non-final buffer. Its state picks the vocabulary, words or single
//! letters, which is what lets a state switch change the verdict at the same offset.
//! [`frontier_lexes`] counts the lexes that decided an item at the buffer end: the work under test.
//! Each test names the wrong implementation it refuses.

use core::cell::Cell;

use tokora::{
  InputRef, Lexer, ParseInput, ParserContext, ReadFrontier, ScanLookahead, SimpleSpan, Token,
  cache::DefaultCache,
  combinator::Any,
  emitter::Verbose,
  error::{Incomplete, MaybeIncomplete, MaybeTerminal, UnexpectedEot, UnexpectedToken},
  input::{Budget, Partial, PartialSession, RedriveFromBase, SessionRefusal, TokenBudget},
  parse_partial,
};

// ── the instrument ──────────────────────────────────────────────────────────

thread_local! {
  /// One increment per lex whose item reaches the end of the buffer.
  static FRONTIER_LEXES: Cell<usize> = const { Cell::new(0) };
}

fn reset() {
  FRONTIER_LEXES.with(|c| c.set(0));
}

/// How many lexes decided an item at the end of the buffer since the last [`reset`].
fn frontier_lexes() -> usize {
  FRONTIER_LEXES.with(Cell::get)
}

// ── the vocabulary and the lexer ────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TokKind;

impl core::fmt::Display for TokKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("token")
  }
}

#[derive(Clone, Debug, PartialEq)]
struct Tok;

/// A byte that is neither a letter nor a space.
#[derive(Clone, Debug, PartialEq)]
struct LexErr;

impl Token<'_> for Tok {
  type Kind = TokKind;
  type Error = LexErr;

  const SCAN_LOOKAHEAD: ScanLookahead = ScanLookahead::WithinSpan;

  fn kind(&self) -> TokKind {
    TokKind
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

/// The lexer state: which vocabulary the next token is lexed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
  /// A token is a run of lowercase letters. Deciding where it ends reads the byte at its end.
  Words,
  /// A token is one lowercase letter, and nothing past it is read.
  Letters,
}

impl tokora::state::State for Mode {
  type Error = ();

  fn check(&self) -> Result<(), ()> {
    Ok(())
  }
}

/// Skips spaces; lexes a token in the current [`Mode`]; lexes any other byte as a one-byte error.
struct ModeLexer<'a> {
  src: &'a str,
  start: usize,
  end: usize,
  state: Mode,
}

impl<'a> Lexer<'a> for ModeLexer<'a> {
  type State = Mode;
  type Source = str;
  type Token = Tok;
  type Span = SimpleSpan;
  type Offset = usize;

  fn new(src: &'a str) -> Self {
    Self::with_state(src, Mode::Words)
  }

  fn with_state(src: &'a str, state: Mode) -> Self {
    Self {
      src,
      start: 0,
      end: 0,
      state,
    }
  }

  fn check(&self) -> Result<(), LexErr> {
    Ok(())
  }

  fn state(&self) -> &Mode {
    &self.state
  }

  fn state_mut(&mut self) -> &mut Mode {
    &mut self.state
  }

  fn into_state(self) -> Mode {
    self.state
  }

  fn source(&self) -> &'a str {
    self.src
  }

  fn span(&self) -> SimpleSpan {
    SimpleSpan::new(self.start, self.end)
  }

  fn slice(&self) -> &'a str {
    &self.src[self.start..self.end]
  }

  fn lex(&mut self) -> Option<Result<Tok, LexErr>> {
    let b = self.src.as_bytes();
    let mut at = self.end;
    while at < b.len() && b[at] == b' ' {
      at += 1;
    }
    self.start = at;
    if at >= b.len() {
      // The post-exhaustion span: well-formed, ending at the lexer's final position.
      self.end = at;
      return None;
    }
    if !b[at].is_ascii_lowercase() {
      self.end = at + 1;
      return Some(Err(LexErr));
    }
    let mut end = at + 1;
    if self.state == Mode::Words {
      while end < b.len() && b[end].is_ascii_lowercase() {
        end += 1;
      }
    }
    self.end = end;
    if end == b.len() {
      FRONTIER_LEXES.with(|c| c.set(c.get() + 1));
    }
    Some(Ok(Tok))
  }

  fn read_frontier(&self) -> ReadFrontier<usize> {
    ReadFrontier::SpanEnd
  }

  fn bump(&mut self, n: &usize) {
    self.end += *n;
    self.start = self.end;
  }
}

// ── the error type ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
enum Fail {
  Lex,
  Unexpected,
  /// End of input, carrying whether the crate marked it terminal.
  Eot(bool),
  Incomplete,
  Refused,
}

impl From<LexErr> for Fail {
  fn from(_: LexErr) -> Self {
    Fail::Lex
  }
}

impl<'a, T, K: Clone, S, Lg: ?Sized> From<UnexpectedToken<'a, T, K, S, Lg>> for Fail {
  fn from(_: UnexpectedToken<'a, T, K, S, Lg>) -> Self {
    Fail::Unexpected
  }
}

impl From<UnexpectedEot<usize>> for Fail {
  fn from(e: UnexpectedEot<usize>) -> Self {
    Fail::Eot(e.is_terminal())
  }
}

impl From<Incomplete<usize>> for Fail {
  fn from(_: Incomplete<usize>) -> Self {
    Fail::Incomplete
  }
}

impl From<SessionRefusal> for Fail {
  fn from(_: SessionRefusal) -> Self {
    Fail::Refused
  }
}

impl MaybeIncomplete for Fail {
  fn is_incomplete(&self) -> bool {
    matches!(self, Fail::Incomplete)
  }
}

impl MaybeTerminal for Fail {
  fn is_terminal(&self) -> bool {
    match self {
      Fail::Eot(terminal) => *terminal,
      Fail::Refused => true,
      _ => false,
    }
  }
}

// ── contexts and drivers ────────────────────────────────────────────────────

type Lex<'a> = ModeLexer<'a>;
/// The default cache, which retains lookahead tokens.
type Retaining<'a> = DefaultCache<'a, Lex<'a>>;
type Ctx<'a> = ParserContext<'a, Lex<'a>, Verbose<Fail>, Retaining<'a>>;
type In<'inp, 'c> = InputRef<'inp, 'c, Lex<'inp>, Ctx<'inp>, (), Partial>;

/// A collecting emitter under a ceiling of `budget` lexed items.
fn ctx<'a>(budget: usize) -> Ctx<'a> {
  Ctx::new(Verbose::new()).with_token_budget(TokenBudget::with_limitation(budget))
}

/// Parses `src` as a partial input, starting in `mode` under a ceiling of `budget` items, and
/// sealed when `is_final`.
fn run<'inp, O>(
  src: &'inp str,
  mode: Mode,
  budget: usize,
  is_final: bool,
  f: impl FnMut(&mut In<'inp, '_>) -> Result<O, Fail>,
) -> Result<O, Fail> {
  parse_partial(ctx(budget), src, mode, is_final, f)
}

// ── the shapes ──────────────────────────────────────────────────────────────

/// `Any`, unwrapped: the control.
fn bare(inp: &mut In<'_, '_>) -> Result<Tok, Fail> {
  Any::of().parse_input(inp)
}

/// The consumed token's own span, read off the token: no peek in front of it.
fn own_span(inp: &mut In<'_, '_>) -> Result<SimpleSpan, Fail> {
  Ok(Any::spanned().parse_input(inp)?.span)
}

/// `Any` under one `spanned_trimmed`, whose start peeks the head first.
fn spanned(inp: &mut In<'_, '_>) -> Result<SimpleSpan, Fail> {
  Ok(Any::of().spanned_trimmed().parse_input(inp)?.span)
}

/// `Any` under two `spanned_trimmed`s: the outer start's peek meets the head, then the inner one's.
fn nested(inp: &mut In<'_, '_>) -> Result<(SimpleSpan, SimpleSpan), Fail> {
  let outer = Any::of()
    .spanned_trimmed()
    .spanned_trimmed()
    .parse_input(inp)?;
  Ok((outer.span, outer.data.span))
}

/// `Any` under `InputRef::spanning_trimmed`, the imperative spelling of the same bracket.
fn spanning(inp: &mut In<'_, '_>) -> Result<SimpleSpan, Fail> {
  let (span, _) = inp.spanning_trimmed(bare)?;
  Ok(span)
}

/// A production that looks before it leaps, as every alternation does, and measures nothing.
fn peek_then_any(inp: &mut In<'_, '_>) -> Result<Tok, Fail> {
  let _ = inp.peek_one()?;
  bare(inp)
}

// ── the head withheld under a spent budget ──────────────────────────────────
//
// `x` as a non-final input, under a ceiling of one item. Lexing `x` reaches the end of the buffer,
// so it is withheld, and that one lex spends the budget.

/// Every shape stays `Incomplete`, as the bare consume is. Refuses a consume that lexes the
/// withheld head again: the ceiling refuses that second lex, and the consume reports a terminal
/// end of input over a buffer that only needed more bytes.
#[test]
fn each_shape_stays_incomplete_when_the_peek_spent_the_budget_on_the_head() {
  assert_eq!(
    run("x", Mode::Words, 1, false, bare),
    Err(Fail::Incomplete),
    "witness: the bare consume meets the frontier"
  );
  assert_eq!(
    run("x", Mode::Words, 1, false, spanned),
    Err(Fail::Incomplete),
    "spanned_trimmed"
  );
  assert_eq!(
    run("x", Mode::Words, 1, false, nested),
    Err(Fail::Incomplete),
    "nested spanned_trimmed"
  );
  assert_eq!(
    run("x", Mode::Words, 1, false, spanning),
    Err(Fail::Incomplete),
    "spanning_trimmed"
  );
}

/// The refill: the same buffer sealed, under the same ceiling of one item. Each shape takes `x`
/// with `x`'s span: the start's peek lexes `x` once, and the retaining cache serves it to the
/// consume. Refuses a fix that holds `x` back on a sealed input, or lexes it a second time there,
/// which the ceiling refuses.
#[test]
fn each_shape_takes_the_head_once_the_input_is_sealed() {
  assert_eq!(run("x", Mode::Words, 1, true, bare), Ok(Tok));
  assert_eq!(
    run("x", Mode::Words, 1, true, spanned),
    Ok(SimpleSpan::new(0, 1))
  );
  assert_eq!(
    run("x", Mode::Words, 1, true, nested),
    Ok((SimpleSpan::new(0, 1), SimpleSpan::new(0, 1)))
  );
  assert_eq!(
    run("x", Mode::Words, 1, true, spanning),
    Ok(SimpleSpan::new(0, 1))
  );
}

/// With room in the budget the outcome was already `Incomplete`, and the defect was the work: each
/// shape lexes `x` once and is charged once. Refuses a fix that only stops the terminal report — a
/// refund, or a ceiling that skips the lex it would refuse — and still lexes the head twice.
#[test]
fn the_head_is_lexed_and_charged_once_for_the_peek_and_the_consume() {
  /// The shape's outcome, the items charged, and the lexes that reached the buffer end.
  fn cost<O>(shape: fn(&mut In<'_, '_>) -> Result<O, Fail>) -> (Result<O, Fail>, usize, usize) {
    reset();
    let (outcome, spent) = run("x", Mode::Words, 64, false, |inp| {
      let outcome = shape(inp);
      Ok((outcome, inp.token_budget().spent()))
    })
    .expect("the closure hands back the shape's outcome");
    (outcome, spent, frontier_lexes())
  }

  assert_eq!(
    cost(bare),
    (Err(Fail::Incomplete), 1, 1),
    "witness: the bare consume"
  );
  assert_eq!(
    cost(spanned),
    (Err(Fail::Incomplete), 1, 1),
    "spanned_trimmed"
  );
  assert_eq!(
    cost(nested),
    (Err(Fail::Incomplete), 1, 1),
    "nested spanned_trimmed"
  );
  assert_eq!(
    cost(spanning),
    (Err(Fail::Incomplete), 1, 1),
    "spanning_trimmed"
  );
}

/// The latent case: no trimmed span anywhere, only a peek and then a consume, under the same input
/// and ceiling. Refuses a fix confined to the trimmed wrappers, which leaves every other
/// peek-then-consume production reporting a terminal end of input.
#[test]
fn a_plain_peek_then_consume_stays_incomplete() {
  reset();
  assert_eq!(
    run("x", Mode::Words, 1, false, peek_then_any),
    Err(Fail::Incomplete)
  );
  assert_eq!(frontier_lexes(), 1, "the consume did not lex `x` again");
}

/// A session asks for more input instead of latching shut, and the refill succeeds. Refuses the
/// terminal stop the second lex produced: a session latches on a terminal error and refuses
/// every attempt after it, so the sealed buffer was never parsed.
#[test]
fn a_session_asks_for_more_input_instead_of_latching() {
  let mut session = PartialSession::new(Mode::Words, Budget::Unbounded, RedriveFromBase);

  assert_eq!(
    session.parse(ctx(1), "x", false, spanned),
    Err(Fail::Incomplete)
  );
  assert!(!session.is_latched(), "an `Incomplete` latches nothing");

  assert_eq!(
    session.parse(ctx(1), "x", true, spanned),
    Ok(SimpleSpan::new(0, 1))
  );
}

// ── the key ─────────────────────────────────────────────────────────────────

/// `xy` is withheld as a word and not as a letter: under `Letters`, `x` ends at 1, before the
/// buffer end. Switched between the peek and the consume, the state decides again: the consume
/// lexes `x` under `Letters` and takes it, charged a second time. Refuses a verdict that outlives
/// a state switch, which would answer the consume with the word's `Incomplete`.
#[test]
fn a_state_switch_between_the_peek_and_the_consume_lexes_again() {
  let out = run("xy", Mode::Words, 2, false, |inp| {
    let peeked = inp.peek_one()?.is_some();
    inp.set_state(Mode::Letters);
    let span = own_span(inp)?;
    Ok((peeked, span, inp.token_budget().spent()))
  });
  assert_eq!(out, Ok((false, SimpleSpan::new(0, 1), 2)));
}

/// `ab x`: a transaction consumes `ab` and meets `x` withheld, and rolls back to 0. The outer parse
/// consumes `ab` again and reaches `x` at the same offset, under the same state and the same
/// buffer, so the verdict answers: `x` is not lexed again, and the ceiling of three items — `ab`
/// twice and `x` once — is enough. Refuses a verdict a rollback clears, whose re-lex meets the
/// spent ceiling and reports a terminal end of input.
#[test]
fn a_rollback_across_the_withheld_point_keeps_a_verdict_whose_key_matches() {
  reset();
  let out = run("ab x", Mode::Words, 3, false, |inp| {
    let mut txn = inp.begin();
    bare(&mut txn)?;
    let reached = bare(&mut txn);
    txn.rollback();
    bare(inp)?;
    let again = bare(inp);
    Ok((reached, again, inp.token_budget().spent()))
  });
  assert_eq!(out, Ok((Err(Fail::Incomplete), Err(Fail::Incomplete), 3)));
  assert_eq!(frontier_lexes(), 1, "`x` was lexed once");
}

/// `xy` under `Letters`: a transaction switches to `Words`, peeks, and `xy` is withheld; the
/// rollback puts `Letters` back. The verdict's offset and buffer still match and its regime does
/// not, so the consume lexes again and takes `x`. Refuses a verdict keyed by offset and buffer
/// alone, which would answer with the word's `Incomplete`.
#[test]
fn a_rollback_to_another_state_leaves_the_verdict_unused() {
  let out = run("xy", Mode::Letters, 64, false, |inp| {
    let mut txn = inp.begin();
    txn.set_state(Mode::Words);
    let peeked = txn.peek_one()?.is_some();
    txn.rollback();
    Ok((peeked, own_span(inp)?))
  });
  assert_eq!(out, Ok((false, SimpleSpan::new(0, 1))));
}

/// `! x`: the run that reaches `x` crosses the lexer error `!` and reports it. Kept, the report
/// stands and the consume is answered from the verdict without reporting `!` twice. Rolled back,
/// the report is taken back with the transaction, and the consume lexes again and reports `!`
/// again. Refuses a verdict that ignores the reports its run raised, which would leave `!`
/// unreported after the rollback.
#[test]
fn a_rollback_that_takes_back_the_runs_report_lexes_again() {
  reset();
  let kept = run("! x", Mode::Words, 64, false, |inp| {
    let peeked = inp.peek_one()?.is_some();
    let consumed = bare(inp);
    Ok((peeked, consumed, inp.emitter_ref().diagnostics().len()))
  });
  assert_eq!(kept, Ok((false, Err(Fail::Incomplete), 1)));
  assert_eq!(frontier_lexes(), 1, "witness: kept, `x` was lexed once");

  reset();
  let rolled_back = run("! x", Mode::Words, 64, false, |inp| {
    let mut txn = inp.begin();
    let peeked = txn.peek_one()?.is_some();
    let reported = txn.emitter_ref().diagnostics().len();
    txn.rollback();
    let taken_back = inp.emitter_ref().diagnostics().len();
    let consumed = bare(inp);
    Ok((
      peeked,
      reported,
      taken_back,
      consumed,
      inp.emitter_ref().diagnostics().len(),
    ))
  });
  assert_eq!(rolled_back, Ok((false, 1, 0, Err(Fail::Incomplete), 1)));
  assert_eq!(frontier_lexes(), 2, "rolled back, `x` was lexed again");
}
