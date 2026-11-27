//! `InputRef::look_ahead`: speculation that is always taken back, whose lexing the token budget
//! reports apart, and which puts back the cache window it was entered with.
//!
//! Every behavioural cell runs a control that does not go through `look_ahead` — the same script
//! with the look-ahead's body removed, or with `attempt` in its place — so a defect in the
//! look-ahead cannot move the number it is compared against.

use core::cell::Cell;

use hybrid_arraydeque::typenum::U2;

use super::{
  BombCtx, BombEmitter, BombErr, BombLexer, BombTally, arm_span_clone, arm_span_drop,
  arm_tally_clone, bomb_fired, disarm_span_clone, disarm_span_drop, disarm_tally_clone, lex_calls,
  reset_bomb_fired, reset_lex_calls,
};
use crate::{
  InputRef, ParseAttempt, SimpleSpan, Token,
  cache::{CachedToken, DefaultCache},
  input::{Input, InputContext, Partial, TokenBudget},
  span::Spanned,
};

// ── The fixture: one item per byte, and a regime every token carries ──────────────────

thread_local! {
  /// Every `LaLexer::lex` call, whether or not it produced: the meter for "did this read lex?".
  static LA_LEXES: Cell<usize> = const { Cell::new(0) };
}

fn la_lexes() -> usize {
  LA_LEXES.with(Cell::get)
}

/// The lexer's regime: the mode a letter is lexed in. Every token carries the mode that lexed it,
/// which is what lets a cell tell a token served from a copy from the same token lexed again.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct LaState {
  mode: u8,
}

impl crate::state::State for LaState {
  type Error = ();

  fn check(&self) -> Result<(), ()> {
    Ok(())
  }
}

/// A letter, and the mode it was lexed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LaTok {
  byte: u8,
  mode: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LaKind;

impl core::fmt::Display for LaKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("letter")
  }
}

/// A plain lexer error: one byte in no token's language. `check` stays `Ok`, so a scan reports it
/// and goes on past it.
#[derive(Debug, Clone)]
struct LaErr;

impl From<LaErr> for BombErr {
  fn from(_: LaErr) -> Self {
    BombErr::Lex
  }
}

impl Token<'_> for LaTok {
  type Kind = LaKind;
  type Error = LaErr;

  const SCAN_LOOKAHEAD: crate::ScanLookahead = crate::ScanLookahead::WithinSpan;

  fn kind(&self) -> LaKind {
    LaKind
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

/// One item per byte: a lowercase letter is a token, a space is skipped, and anything else is a
/// one-byte plain lexer error.
struct LaLexer<'a> {
  src: &'a str,
  start: usize,
  end: usize,
  state: LaState,
}

impl<'a> crate::Lexer<'a> for LaLexer<'a> {
  type State = LaState;
  type Source = str;
  type Token = LaTok;
  type Span = SimpleSpan;
  type Offset = usize;

  fn new(src: &'a str) -> Self {
    Self::with_state(src, LaState::default())
  }

  fn with_state(src: &'a str, state: LaState) -> Self {
    Self {
      src,
      start: 0,
      end: 0,
      state,
    }
  }

  fn check(&self) -> Result<(), LaErr> {
    Ok(())
  }

  fn state(&self) -> &LaState {
    &self.state
  }

  fn state_mut(&mut self) -> &mut LaState {
    &mut self.state
  }

  fn into_state(self) -> LaState {
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

  fn lex(&mut self) -> Option<Result<LaTok, LaErr>> {
    LA_LEXES.with(|c| c.set(c.get() + 1));
    let b = self.src.as_bytes();
    let mut at = self.end;
    while at < b.len() && b[at] == b' ' {
      at += 1;
    }
    self.start = at;
    if at >= b.len() {
      self.end = at;
      return None;
    }
    self.end = at + 1;
    match b[at] {
      byte @ b'a'..=b'z' => Some(Ok(LaTok {
        byte,
        mode: self.state.mode,
      })),
      _ => Some(Err(LaErr)),
    }
  }

  fn read_frontier(&self) -> crate::ReadFrontier<usize> {
    crate::ReadFrontier::SpanEnd
  }

  fn bump(&mut self, n: &usize) {
    self.end += *n;
    self.start = self.end;
  }
}

type LaCtx<'a> = (BombEmitter, DefaultCache<'a, LaLexer<'a>>);
type LaRef<'inp, 'cl> = InputRef<'inp, 'cl, LaLexer<'inp>, LaCtx<'inp>, ()>;

fn context<'a>() -> InputContext<BombEmitter, DefaultCache<'a, LaLexer<'a>>> {
  // Named in full: the alias's lexer parameter is reachable only through the projections the
  // entry type is built from, so nothing could infer it from the expected type.
  InputContext::new(
    BombEmitter::default(),
    DefaultCache::<'a, LaLexer<'a>>::default(),
  )
}

fn complete(src: &str) -> Input<'_, LaLexer<'_>, LaCtx<'_>, ()> {
  Input::with_state_and_context(src, LaState::default(), context())
}

fn budgeted(src: &str, max: usize) -> Input<'_, LaLexer<'_>, LaCtx<'_>, ()> {
  Input::with_state_and_context(
    src,
    LaState::default(),
    context().with_token_budget(TokenBudget::with_limitation(max)),
  )
}

/// Up to `n` consumes, and the letters they returned. A consume that answers `None` adds nothing.
fn consume(inp: &mut LaRef<'_, '_>, n: usize) -> std::vec::Vec<u8> {
  (0..n)
    .filter_map(|_| inp.next().unwrap().map(|t| t.data.byte))
    .collect()
}

/// One consume, and the whole token it returned, mode included.
fn next_token(inp: &mut LaRef<'_, '_>) -> Option<LaTok> {
  inp.next().unwrap().map(|t| t.data)
}

/// `spent() - look_ahead_spent()`, the difference a recovery allowance reads.
fn kept(inp: &LaRef<'_, '_>) -> usize {
  let budget = inp.token_budget();
  budget.spent() - budget.look_ahead_spent()
}

/// The front of the stream as the restore path sees it: the cache's entries in order, and the
/// push count a restore drops the post-save tail by.
#[derive(Debug, PartialEq, Eq)]
struct Window {
  cache: std::vec::Vec<(SimpleSpan, LaTok)>,
  pushes: u64,
}

fn window(inp: &LaRef<'_, '_>) -> Window {
  Window {
    cache: inp
      .cache()
      .iter()
      .map(|entry| entry.clone().into_token().into_components())
      .collect(),
    pushes: inp.session.lineage.cache_pushes(),
  }
}

// ── T1: the exclusion ─────────────────────────────────────────────────────────────────

/// A look-ahead that lexes three items excludes exactly three, and what follows it is charged as
/// it would be without the look-ahead.
#[test]
fn a_look_ahead_excludes_what_it_lexed() {
  let mut control = complete("a b c d");
  let mut ctl = control.as_ref();
  let control_read = consume(&mut ctl, 4);
  let control_spent = ctl.token_budget().spent();

  let mut input = complete("a b c d");
  let mut inp = input.as_ref();
  let inside = inp.look_ahead(|inp| consume(inp, 3));
  assert_eq!(inside, b"abc".to_vec());
  assert_eq!(
    inp.token_budget().look_ahead_spent(),
    3,
    "the look-ahead lexed three items, and the settle excludes all three"
  );
  assert_eq!(consume(&mut inp, 4), control_read);
  assert_eq!(
    kept(&inp),
    control_spent,
    "the difference is the spend of the same reads without the look-ahead"
  );
}

// ── T2: the gap ───────────────────────────────────────────────────────────────────────

/// **The gap.** The head was peeked across a run of lexer errors, so the cache holds it and the
/// committed position is in front of the errors. The look-ahead consumes the head and then peeks
/// behind it, so its rollback has a pushed entry to prune and a head to drop. The window comes
/// back: the head is served without a lex, the errors are not lexed again, and the difference is
/// the control's spend.
#[test]
fn a_look_ahead_restores_the_head_it_found_and_the_gap_stays_lexed_once() {
  const SRC: &str = "x ??? a b c";

  let mut control = complete(SRC);
  let mut ctl = control.as_ref();
  assert_eq!(consume(&mut ctl, 1), b"x".to_vec());
  let _ = ctl.peek_one().unwrap();
  assert_eq!(consume(&mut ctl, 2), b"ab".to_vec());
  let control_spent = ctl.token_budget().spent();
  let control_reports = ctl.emitter().emissions;

  let mut input = complete(SRC);
  let mut inp = input.as_ref();
  assert_eq!(consume(&mut inp, 1), b"x".to_vec());
  let _ = inp.peek_one().unwrap();
  let before = window(&inp);
  let cursor_before = *inp.cursor().as_inner();
  let reports_before = inp.emitter().emissions;
  assert_eq!(before.cache.len(), 1, "the peek left the head resident");
  assert_eq!(
    reports_before, 3,
    "the peek reported the three errors in front of the head"
  );

  let inside = inp.look_ahead(|inp| {
    let head = consume(inp, 1);
    let _ = inp.peek::<U2>().unwrap();
    head
  });
  assert_eq!(inside, b"a".to_vec());
  assert_eq!(
    window(&inp),
    before,
    "the window and its push count are the ones the look-ahead was entered with"
  );
  assert_eq!(*inp.cursor().as_inner(), cursor_before);
  assert_eq!(inp.emitter().emissions, reports_before);
  assert_eq!(
    inp.token_budget().look_ahead_spent(),
    2,
    "the look-ahead lexed the two tokens behind the head, and nothing else"
  );

  let spent = inp.token_budget().spent();
  assert_eq!(consume(&mut inp, 1), b"a".to_vec());
  assert_eq!(
    inp.token_budget().spent(),
    spent,
    "the head is served from the restored window: no lex, so no gap re-lexed in front of it"
  );
  assert_eq!(consume(&mut inp, 1), b"b".to_vec());
  assert_eq!(kept(&inp), control_spent);
  assert_eq!(
    inp.emitter().emissions,
    control_reports,
    "each lexer error is reported once"
  );
}

// ── T3: nesting ───────────────────────────────────────────────────────────────────────

/// A look-ahead inside a look-ahead is counted once, by the outermost: the outer excludes its
/// whole spend, six items, and not six plus the inner's three.
#[test]
fn nested_look_aheads_are_counted_once_by_the_outermost() {
  let mut input = complete("a b c d e f g h");
  let mut inp = input.as_ref();
  let mut after_inner = (0, 0);
  let inside = inp.look_ahead(|inp| {
    let mut read = consume(inp, 2);
    read.extend(inp.look_ahead(|inp| consume(inp, 3)));
    let budget = inp.token_budget();
    after_inner = (budget.spent(), budget.look_ahead_spent());
    read.extend(consume(inp, 1));
    read
  });
  assert_eq!(
    inside,
    b"abcdec".to_vec(),
    "the inner look-ahead read c, d and e and gave them back, so the outer reads c again"
  );
  assert_eq!(
    after_inner,
    (5, 3),
    "inside the outer look-ahead, the inner's settle has excluded its own three"
  );
  let budget = inp.token_budget();
  assert_eq!(budget.spent(), 6);
  assert_eq!(
    budget.look_ahead_spent(),
    6,
    "the outer excludes its whole spend once: 6, not the 9 an addition would give"
  );
  assert!(budget.look_ahead_spent() <= budget.spent());
  assert_eq!(
    kept(&inp),
    0,
    "nothing was kept, so nothing stays in the difference"
  );
  assert_eq!(consume(&mut inp, 8), b"abcdefgh".to_vec());
  assert_eq!(kept(&inp), 8, "the control: eight consumes, charged eight");
}

// ── T4: across an enclosing decline ───────────────────────────────────────────────────

/// A look-ahead inside a declined attempt stays excluded after the decline, and the attempt's own
/// lexing stays in the difference. Re-running the attempt adds its own look-ahead's exclusion.
#[test]
fn a_look_ahead_inside_a_declined_attempt_stays_excluded_and_a_rerun_adds_its_own() {
  let mut control = complete("a b c d e");
  let mut ctl = control.as_ref();

  let mut input = complete("a b c d e");
  let mut inp = input.as_ref();
  for run in 1..=2usize {
    let declined = ctl.attempt(|inp| {
      let _ = consume(inp, 1);
      None::<()>
    });
    assert!(declined.is_none());

    let declined = inp.attempt(|inp| {
      let _ = consume(inp, 1);
      let seen = inp.look_ahead(|inp| consume(inp, 2));
      assert_eq!(seen, b"bc".to_vec());
      None::<()>
    });
    assert!(declined.is_none());
    assert_eq!(
      inp.token_budget().look_ahead_spent(),
      2 * run,
      "run {run}: both counters are durable, so the decline keeps the exclusion with the charge"
    );
    assert_eq!(
      kept(&inp),
      ctl.token_budget().spent(),
      "run {run}: the attempt's own lexing of `a` stays in the difference, as the control's does"
    );
  }
}

// ── T5: only a look-ahead excludes ────────────────────────────────────────────────────

/// A declined `attempt`, `try_attempt` and `attempt_parse` and a rolled-back `Transaction` all
/// stay in the difference.
#[test]
fn only_a_look_ahead_excludes() {
  let mut input = complete("a b c d e f");
  let mut inp = input.as_ref();

  let declined = inp.attempt(|inp| {
    let _ = consume(inp, 5);
    None::<()>
  });
  assert!(declined.is_none());
  assert_eq!(
    (
      inp.token_budget().spent(),
      inp.token_budget().look_ahead_spent()
    ),
    (5, 0),
    "a declined attempt that lexed five keeps all five in the difference"
  );

  let failed: Result<(), ()> = inp.try_attempt(|inp| {
    let _ = consume(inp, 2);
    Err(())
  });
  assert!(failed.is_err());

  let parsed = inp.attempt_parse(|inp| {
    let _ = consume(inp, 2);
    Ok::<ParseAttempt<()>, ()>(ParseAttempt::Decline)
  });
  assert!(matches!(parsed, Ok(ParseAttempt::Decline)));

  {
    let mut txn = inp.begin();
    let _ = txn.next().unwrap();
    let _ = txn.next().unwrap();
    txn.rollback();
  }

  assert_eq!(
    (
      inp.token_budget().spent(),
      inp.token_budget().look_ahead_spent()
    ),
    (11, 0),
    "every declining form re-lexed what it read, and none of it is excluded"
  );
}

// ── T6: an unwind out of the closure ──────────────────────────────────────────────────

/// An unwind out of the closure settles as a return does: the guard rolls back, then the window
/// and the exclusion are installed, and the input reads as if the look-ahead had returned.
#[test]
fn an_unwind_out_of_the_closure_settles_the_window_and_the_exclusion() {
  const SRC: &str = "x ??? a b c";

  let mut control = complete(SRC);
  let mut ctl = control.as_ref();
  assert_eq!(consume(&mut ctl, 1), b"x".to_vec());
  let _ = ctl.peek_one().unwrap();
  assert_eq!(consume(&mut ctl, 3), b"abc".to_vec());
  let control_spent = ctl.token_budget().spent();

  let mut input = complete(SRC);
  let mut inp = input.as_ref();
  assert_eq!(consume(&mut inp, 1), b"x".to_vec());
  let _ = inp.peek_one().unwrap();
  let before = window(&inp);
  let cursor_before = *inp.cursor().as_inner();
  let kept_before = kept(&inp);

  let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    inp.look_ahead(|inp| -> usize {
      let _ = consume(inp, 1);
      let _ = inp.peek::<U2>().unwrap();
      panic!("the look-ahead's closure unwinds")
    })
  }));
  assert!(caught.is_err());

  assert_eq!(window(&inp), before, "the window came back on the unwind");
  assert_eq!(*inp.cursor().as_inner(), cursor_before);
  assert_eq!(
    kept(&inp),
    kept_before,
    "and the exclusion was settled: the difference is where the look-ahead found it"
  );
  assert_eq!(inp.token_budget().look_ahead_spent(), 2);
  assert_eq!(inp.points(), 0);
  assert_eq!(inp.session.lineage.pinned_len(), 0, "no pin stranded");
  assert_eq!(
    inp.session.lineage.live_len(),
    0,
    "no lineage entry stranded"
  );

  let spent = inp.token_budget().spent();
  assert_eq!(consume(&mut inp, 1), b"a".to_vec());
  assert_eq!(
    inp.token_budget().spent(),
    spent,
    "the head is served without a lex"
  );
  assert_eq!(
    consume(&mut inp, 2),
    b"bc".to_vec(),
    "and the input is usable"
  );
  assert_eq!(kept(&inp), control_spent);
}

// ── T6b: an unwind out of the look-ahead's own rollback ───────────────────────────────

/// What a cell reads back after a look-ahead, or the attempt standing in for it, unwound.
#[derive(Debug, PartialEq, Eq)]
struct AfterUnwind {
  unwound: bool,
  committed: (usize, usize),
  cached: usize,
  pushes: u64,
  spent: usize,
  look_ahead_spent: usize,
  pins: usize,
  live: usize,
  marks: usize,
  next: Option<(usize, usize)>,
  next_lexes: usize,
}

/// Runs a look-ahead — or, with `look` false, an `attempt` in its place — whose closure consumes
/// the resident head, peeks two behind it, and then arms the `bomb_at`-th `L::Span` drop (0 =
/// disarmed). Everything from there on is the rollback's: its first phase evicts the two peeked
/// entries before it restores anything, and its last drops the span it replaced.
///
/// Returns what the input holds afterwards, how many bombs fired, and how many span drops the run
/// performed after the arming.
fn unwind_in_the_rollback(look: bool, bomb_at: usize) -> (AfterUnwind, usize, usize) {
  let mut input = Input::<BombLexer<'_>, BombCtx<'_>, ()>::with_state_and_context(
    "ab cd ef gh",
    BombTally::default(),
    InputContext::new(
      BombEmitter::default(),
      DefaultCache::<'_, BombLexer<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();
  let _ = inp.peek_one().unwrap();

  reset_bomb_fired();
  let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    if look {
      inp.look_ahead(|inp| {
        let _ = inp.next().unwrap();
        let _ = inp.peek::<U2>().unwrap();
        arm_span_drop(bomb_at);
        true
      })
    } else {
      inp
        .attempt(|inp| {
          let _ = inp.next().unwrap();
          let _ = inp.peek::<U2>().unwrap();
          arm_span_drop(bomb_at);
          None::<bool>
        })
        .is_some()
    }
  }));
  let drops = disarm_span_drop();
  let fired = bomb_fired();

  let committed = (inp.span().start, inp.span().end);
  let cached = inp.cache().len();
  let pushes = inp.session.lineage.cache_pushes();
  let spent = inp.token_budget().spent();
  let look_ahead_spent = inp.token_budget().look_ahead_spent();
  let pins = inp.session.lineage.pinned_len();
  let live = inp.session.lineage.live_len();
  let marks = inp.emitter().live_rows();
  reset_lex_calls();
  let next = inp.next().unwrap().map(|t| (t.span.start, t.span.end));
  let after = AfterUnwind {
    unwound: caught.is_err(),
    committed,
    cached,
    pushes,
    spent,
    look_ahead_spent,
    pins,
    live,
    marks,
    next,
    next_lexes: lex_calls(),
  };
  (after, fired, drops)
}

/// An unwind out of the look-ahead's own rollback — caller code it runs before it restores
/// anything, and the destructor of what it replaced — installs nothing and excludes nothing. At
/// every span drop that rollback performs, the input is the one `attempt` leaves on the same edge:
/// the head is served once, the push count agrees with the cache, and the count is untouched.
#[test]
fn an_unwind_out_of_the_look_ahead_s_own_rollback_leaves_what_attempt_leaves() {
  let (returned, _, total) = unwind_in_the_rollback(true, 0);
  assert!(!returned.unwound);
  assert!(
    total >= 2,
    "the rollback must drop the two peeked entries and the span it replaced, and dropped \
     {total} spans: the sweep below would measure nothing"
  );
  for bomb_at in 1..=total {
    let (look, look_fired, _) = unwind_in_the_rollback(true, bomb_at);
    let (attempt, attempt_fired, _) = unwind_in_the_rollback(false, bomb_at);
    assert_eq!(
      (look_fired, attempt_fired),
      (1, 1),
      "span drop #{bomb_at} of {total}: the armed destructor was not reached on both runs"
    );
    assert!(
      look.unwound,
      "span drop #{bomb_at} of {total}: the look-ahead did not unwind"
    );
    assert_eq!(
      look.look_ahead_spent, 0,
      "span drop #{bomb_at} of {total}: a rollback that unwound excludes nothing"
    );
    assert_eq!(
      look, attempt,
      "span drop #{bomb_at} of {total}: the input differs from the one `attempt` leaves"
    );
  }
}

// ── T6c: an unwind out of the settle's own drop site ──────────────────────────────────

/// What a cell reads off a Bomb input: its committed span, its cache window and push count, the
/// two counts, and what is registered on its behalf.
#[derive(Debug, PartialEq, Eq)]
struct Reading {
  committed: (usize, usize),
  cached: usize,
  front: Option<(usize, usize)>,
  pushes: u64,
  spent: usize,
  look_ahead_spent: usize,
  pins: usize,
  live: usize,
  marks: usize,
}

type BombInputRef<'inp, 'cl> = InputRef<'inp, 'cl, BombLexer<'inp>, BombCtx<'inp>, ()>;

fn reading(inp: &mut BombInputRef<'_, '_>) -> Reading {
  Reading {
    committed: (inp.span().start, inp.span().end),
    cached: inp.cache().len(),
    front: inp
      .cache()
      .front()
      .map(|entry| (entry.token().span.start, entry.token().span.end)),
    pushes: inp.session.lineage.cache_pushes(),
    spent: inp.token_budget().spent(),
    look_ahead_spent: inp.token_budget().look_ahead_spent(),
    pins: inp.session.lineage.pinned_len(),
    live: inp.session.lineage.live_len(),
    marks: inp.emitter().live_rows(),
  }
}

/// What one run of `peek_then_settle` leaves: the input on entry and afterwards, the panic's
/// message if the run unwound, how many bombs fired and how many `L::Span` drops the run made
/// after the arming, and then the next consume and the lexes it ran.
struct PeekRun {
  entry: Reading,
  after: Reading,
  unwound: Option<std::string::String>,
  fired: usize,
  drops: usize,
  next: Option<(usize, usize)>,
  next_lexes: usize,
}

/// Runs a look-ahead — or, with `look` false, an `attempt` in its place — whose closure only
/// peeks, the resident head and the entry behind it, and then arms the `bomb_at`-th `L::Span` drop
/// (0 = disarmed). `f` consumes nothing, so the rollback prunes the entry it pushed and keeps the
/// head. A look-ahead's settle then swaps its copy of the window in and drops the head it
/// displaced, the last span drop of the run.
fn peek_then_settle(look: bool, bomb_at: usize) -> PeekRun {
  let mut input = Input::<BombLexer<'_>, BombCtx<'_>, ()>::with_state_and_context(
    "ab cd ef gh",
    BombTally::default(),
    InputContext::new(
      BombEmitter::default(),
      DefaultCache::<'_, BombLexer<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();
  let _ = inp.peek_one().unwrap();
  let entry = reading(&mut inp);

  reset_bomb_fired();
  let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    if look {
      inp.look_ahead(|inp| {
        let _ = inp.peek::<U2>().unwrap();
        arm_span_drop(bomb_at);
        true
      })
    } else {
      inp
        .attempt(|inp| {
          let _ = inp.peek::<U2>().unwrap();
          arm_span_drop(bomb_at);
          None::<bool>
        })
        .is_some()
    }
  }));
  let drops = disarm_span_drop();
  let fired = bomb_fired();
  let unwound = caught.err().map(|payload| {
    payload
      .downcast_ref::<std::string::String>()
      .cloned()
      .unwrap_or_default()
  });
  let after = reading(&mut inp);
  reset_lex_calls();
  let next = inp.next().unwrap().map(|t| (t.span.start, t.span.end));
  PeekRun {
    entry,
    after,
    unwound,
    fired,
    drops,
    next,
    next_lexes: lex_calls(),
  }
}

/// A destructor that panics at the settle's own drop site finds the settle whole. Every write of
/// the settle comes before that drop, so the panic leaves the look-ahead with the entry window
/// back, the count raised and the difference at its entry value, and the head is then served
/// without a lex.
///
/// The armed drop is the head the settle displaced: the one span drop the look-ahead makes that an
/// `attempt` running the same closure does not, and the last of the run.
#[test]
fn an_unwind_out_of_the_settle_s_drop_site_leaves_the_settle_whole() {
  let returned = peek_then_settle(true, 0);
  let attempted = peek_then_settle(false, 0);
  assert!(returned.unwound.is_none() && attempted.unwound.is_none());
  assert_eq!(
    returned.drops,
    attempted.drops + 1,
    "the settle drops one span after the rollback's own, the head it displaced"
  );

  let armed = peek_then_settle(true, returned.drops);
  assert_eq!(armed.fired, 1, "the armed destructor was not reached");
  let message = armed
    .unwound
    .as_deref()
    .expect("a panic at the settle's drop site leaves the look-ahead");
  assert!(
    message.contains("0..2) panics"),
    "the armed drop is the displaced head's, at 0..2: {message}"
  );
  assert_eq!(
    (armed.after.cached, armed.after.front, armed.after.pushes),
    (armed.entry.cached, armed.entry.front, armed.entry.pushes),
    "the entry window is installed: its length, its head and its push count"
  );
  assert!(
    armed.after.look_ahead_spent > armed.entry.look_ahead_spent,
    "the count was raised before the drop"
  );
  assert_eq!(
    armed.after.spent - armed.after.look_ahead_spent,
    armed.entry.spent - armed.entry.look_ahead_spent,
    "the difference is at its entry value"
  );
  assert_eq!(
    armed.after, returned.after,
    "and the input reads as after the look-ahead that returned"
  );
  assert_eq!(armed.next, Some((0, 2)), "the head is the front");
  assert_eq!(armed.next_lexes, 0, "and it is served without a lex");
}

// ── A panicking `Clone` at entry ──────────────────────────────────────────────────────

/// The window is copied before anything is registered, so a `Clone` that unwinds there strands no
/// pin, no lineage entry and no emitter mark, and moves nothing. Both caller clones the copy runs
/// are armed in turn: the entry's span, and its lexer state.
#[test]
fn a_panicking_clone_at_entry_registers_nothing_and_moves_nothing() {
  type BombRef<'inp, 'cl> = InputRef<'inp, 'cl, BombLexer<'inp>, BombCtx<'inp>, ()>;
  fn read(
    inp: &mut BombRef<'_, '_>,
  ) -> (
    usize,
    usize,
    usize,
    u64,
    (usize, usize),
    usize,
    usize,
    usize,
  ) {
    (
      inp.token_budget().spent(),
      inp.token_budget().look_ahead_spent(),
      inp.cache().len(),
      inp.session.lineage.cache_pushes(),
      (inp.span().start, inp.span().end),
      inp.session.lineage.pinned_len(),
      inp.session.lineage.live_len(),
      inp.emitter().live_rows(),
    )
  }

  let arms: [(fn(usize), fn()); 2] = [
    (arm_span_clone, disarm_span_clone),
    (arm_tally_clone, disarm_tally_clone),
  ];
  for (which, (arm, disarm)) in arms.into_iter().enumerate() {
    let mut input = Input::<BombLexer<'_>, BombCtx<'_>, ()>::with_state_and_context(
      "ab cd ef",
      BombTally::default(),
      InputContext::new(
        BombEmitter::default(),
        DefaultCache::<'_, BombLexer<'_>>::default(),
      ),
    );
    let mut inp = input.as_ref();
    let _ = inp.peek_one().unwrap();
    let before = read(&mut inp);

    reset_bomb_fired();
    arm(1);
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      inp.look_ahead(|inp| inp.next().is_ok())
    }));
    disarm();
    assert!(caught.is_err(), "arm #{which}: the copy did not unwind");
    assert_eq!(
      bomb_fired(),
      1,
      "arm #{which}: the armed clone was not reached"
    );
    assert_eq!(
      read(&mut inp),
      before,
      "arm #{which}: an unwind out of the copy left a trace"
    );

    reset_lex_calls();
    let head = inp.next().unwrap().map(|t| (t.span.start, t.span.end));
    assert_eq!(
      head,
      Some((0, 2)),
      "arm #{which}: the head is still the front"
    );
    assert_eq!(lex_calls(), 0, "arm #{which}: and still resident");
  }
}

// ── T7: a budget met inside the look-ahead ────────────────────────────────────────────

/// A look-ahead's items are charged, so a ceiling it meets refuses the committed path sooner. The
/// restored window is served without a lex and the next lex is refused, so the parse stops after
/// the window: later than the head, where an `attempt` in its place leaves it, and earlier than
/// the run without either.
#[test]
fn a_budget_met_inside_a_look_ahead_serves_the_window_and_refuses_the_next_lex() {
  const SRC: &str = "a b c d e f g";

  let mut control = budgeted(SRC, 5);
  let mut ctl = control.as_ref();
  let _ = ctl.peek_one().unwrap();
  assert_eq!(consume(&mut ctl, 7), b"abcde".to_vec(), "refused at `f`");
  assert_eq!(*ctl.span(), SimpleSpan::new(8, 9));

  let mut attempted = budgeted(SRC, 5);
  let mut att = attempted.as_ref();
  let _ = att.peek_one().unwrap();
  let declined = att.attempt(|inp| {
    let _ = consume(inp, 7);
    None::<()>
  });
  assert!(declined.is_none());
  assert!(
    consume(&mut att, 7).is_empty(),
    "the head was dropped, and lexing it is refused"
  );
  assert_eq!(*att.span(), SimpleSpan::new(0, 0));

  let mut input = budgeted(SRC, 5);
  let mut inp = input.as_ref();
  let _ = inp.peek_one().unwrap();
  let inside = inp.look_ahead(|inp| consume(inp, 7));
  assert_eq!(
    inside,
    b"abcde".to_vec(),
    "the look-ahead is refused at `f`"
  );
  assert!(inp.token_budget().refused_an_item());
  assert_eq!(
    (
      inp.token_budget().spent(),
      inp.token_budget().look_ahead_spent()
    ),
    (5, 4),
    "the ceiling was met inside: five charged, the four behind the resident head excluded"
  );

  let lexes = la_lexes();
  assert_eq!(consume(&mut inp, 1), b"a".to_vec());
  assert_eq!(
    la_lexes(),
    lexes,
    "the restored head is served without a lex"
  );
  assert!(
    consume(&mut inp, 6).is_empty(),
    "and the next lex is refused"
  );
  assert_eq!(la_lexes(), lexes, "without the lexer running");
  assert!(inp.token_budget().refused_an_item());
  assert_eq!(
    *inp.span(),
    SimpleSpan::new(0, 1),
    "the stop lands after the restored window"
  );
}

// ── T8: a re-key inside the look-ahead ────────────────────────────────────────────────

/// A state re-key inside the look-ahead clears the cache and stamps a new regime. The rollback
/// puts the regime back, prunes what the new regime peeked, and the head comes back from the copy,
/// lexed under the entry regime, while the id source stays advanced.
#[test]
fn a_re_key_inside_a_look_ahead_is_undone_and_the_head_comes_back_from_the_copy() {
  let mut control = complete("a b c");
  let mut ctl = control.as_ref();
  let _ = ctl.peek_one().unwrap();
  let declined = ctl.attempt(|inp| {
    inp.set_state(LaState { mode: 1 });
    let _ = inp.peek_one().unwrap();
    None::<()>
  });
  assert!(declined.is_none());
  let lexes = la_lexes();
  assert_eq!(
    next_token(&mut ctl),
    Some(LaTok {
      byte: b'a',
      mode: 0
    })
  );
  assert_eq!(
    la_lexes(),
    lexes + 1,
    "the control: the attempt's rollback dropped the head, so it is lexed again"
  );

  let mut input = complete("a b c");
  let mut inp = input.as_ref();
  let _ = inp.peek_one().unwrap();
  let before = window(&inp);
  let (regime, seq) = (*inp.regime, *inp.regime_seq);
  let peeked = inp.look_ahead(|inp| {
    inp.set_state(LaState { mode: 1 });
    let _ = inp.peek_one().unwrap();
    window(inp)
  });
  assert_eq!(
    peeked.cache,
    std::vec![(
      SimpleSpan::new(0, 1),
      LaTok {
        byte: b'a',
        mode: 1
      }
    )],
    "inside, the head was lexed again under the new regime"
  );
  assert_eq!(*inp.regime, regime, "the regime is restored");
  assert_eq!(*inp.regime_seq, seq + 1, "the id source stays advanced");
  assert_eq!(window(&inp), before, "the window is the entry regime's");

  let lexes = la_lexes();
  let head = next_token(&mut inp);
  assert_eq!(head.map(|t| t.mode), Some(0));
  assert_eq!(
    head,
    Some(LaTok {
      byte: b'a',
      mode: 0
    })
  );
  assert_eq!(
    la_lexes(),
    lexes,
    "the head is served from the copy, without a lex"
  );
}

// ── T9: entries the context handed over ───────────────────────────────────────────────

/// A cache the context handed over holds entries lexed under another mode. A look-ahead that
/// consumed them puts them back verbatim; an `attempt` in its place re-lexes the region under the
/// input's own regime and gets other tokens.
#[test]
fn entries_the_context_handed_over_come_back_verbatim() {
  fn handed_over(src: &str) -> Input<'_, LaLexer<'_>, LaCtx<'_>, ()> {
    let mut cache = DefaultCache::<'_, LaLexer<'_>>::default();
    for (byte, start) in [(b'a', 0usize), (b'b', 2)] {
      let entry = CachedToken::new(
        Spanned::new(SimpleSpan::new(start, start + 1), LaTok { byte, mode: 1 }),
        LaState { mode: 1 },
      );
      assert!(
        cache.push_back(entry).is_none(),
        "the handed-over cache has room"
      );
    }
    Input::with_state_and_context(
      src,
      LaState::default(),
      InputContext::new(BombEmitter::default(), cache),
    )
  }
  let handed = LaTok {
    byte: b'a',
    mode: 1,
  };

  let mut control = handed_over("a b c");
  let mut ctl = control.as_ref();
  let declined = ctl.attempt(|inp| {
    let _ = consume(inp, 2);
    None::<()>
  });
  assert!(declined.is_none());
  assert_eq!(
    next_token(&mut ctl),
    Some(LaTok {
      byte: b'a',
      mode: 0
    }),
    "the control: the attempt's rollback dropped the entries, and the region re-lexes under the \
     input's own regime"
  );

  let mut input = handed_over("a b c");
  let mut inp = input.as_ref();
  let before = window(&inp);
  assert_eq!(before.pushes, 2, "the handed-over entries count as pushed");
  let inside = inp.look_ahead(|inp| consume(inp, 2));
  assert_eq!(inside, b"ab".to_vec());
  assert_eq!(
    window(&inp),
    before,
    "the handed-over entries are back, verbatim"
  );
  let lexes = la_lexes();
  assert_eq!(next_token(&mut inp), Some(handed));
  assert_eq!(la_lexes(), lexes, "served from the copy, never re-lexed");
}

// ── T10: a parked head ────────────────────────────────────────────────────────────────

/// Under a capacity-0 cache the head lives in the parked slot. A look-ahead that consumed it
/// re-parks the copy it took, under a new park id, and the read after it lexes nothing.
#[test]
fn a_parked_head_a_look_ahead_consumed_is_re_parked() {
  type Ref0<'inp, 'cl> = InputRef<'inp, 'cl, LaLexer<'inp>, (BombEmitter, ()), ()>;
  fn blackhole(src: &str) -> Input<'_, LaLexer<'_>, (BombEmitter, ()), ()> {
    Input::with_state_and_context(
      src,
      LaState::default(),
      InputContext::new(BombEmitter::default(), ()),
    )
  }
  fn park(inp: &mut Ref0<'_, '_>) {
    let declined = inp.try_expect(|_| false).unwrap();
    assert!(declined.is_none());
    assert!(
      inp.has_front_parked(),
      "a capacity-0 cache parks the declined head"
    );
  }
  fn take(inp: &mut Ref0<'_, '_>) -> Option<u8> {
    inp.next().unwrap().map(|t| t.data.byte)
  }

  let mut control = blackhole("a b c");
  let mut ctl = control.as_ref();
  park(&mut ctl);
  let declined = ctl.attempt(|inp| {
    let _ = take(inp);
    None::<()>
  });
  assert!(declined.is_none());
  assert!(
    !ctl.has_front_parked(),
    "the control: a rollback drops the parked token"
  );
  let lexes = la_lexes();
  assert_eq!(take(&mut ctl), Some(b'a'));
  assert_eq!(la_lexes(), lexes + 1, "so the control lexes it again");

  let mut input = blackhole("a b c");
  let mut inp = input.as_ref();
  park(&mut inp);
  #[cfg(feature = "pratt")]
  let parks = inp.session.lineage.parks();
  let spent = inp.token_budget().spent();
  let inside = inp.look_ahead(take);
  assert_eq!(
    inside,
    Some(b'a'),
    "the look-ahead consumed the parked head"
  );
  assert!(
    inp.has_front_parked(),
    "the parked head is back in the slot"
  );
  #[cfg(feature = "pratt")]
  assert_eq!(
    inp.session.lineage.parks(),
    parks + 1,
    "the copy entered the slot as a token it had not just handed out, so it took a new id"
  );
  assert_eq!(
    inp.token_budget().spent(),
    spent,
    "the look-ahead lexed nothing"
  );
  let lexes = la_lexes();
  assert_eq!(take(&mut inp), Some(b'a'));
  assert_eq!(
    la_lexes(),
    lexes,
    "and the read after it is served from the slot"
  );
}

// ── T11: a withheld head on a partial input ───────────────────────────────────────────

/// On a non-final partial input the look-ahead reaches the withheld head first, and the memo it
/// leaves answers the committed read without a lex. That read is the one produce event the
/// difference is short of the control by: exactly one, in the permissive direction.
#[test]
fn a_withheld_head_a_look_ahead_reached_is_answered_without_a_charge() {
  const SRC: &str = "a b";
  fn partial(src: &str) -> Input<'_, LaLexer<'_>, LaCtx<'_>, (), Partial> {
    Input::with_state_and_context(src, LaState::default(), context())
  }

  let mut control = partial(SRC);
  let mut ctl = control.as_ref();
  assert!(matches!(ctl.next(), Ok(Some(_))));
  assert!(matches!(ctl.next(), Err(BombErr::Incomplete)));
  let control_spent = ctl.token_budget().spent();
  assert_eq!(control_spent, 2);

  let mut input = partial(SRC);
  let mut inp = input.as_ref();
  assert!(matches!(inp.next(), Ok(Some(_))));
  let withheld = inp.look_ahead(|inp| matches!(inp.next(), Err(BombErr::Incomplete)));
  assert!(withheld, "the look-ahead met the withheld head");
  let lexes = la_lexes();
  assert!(matches!(inp.next(), Err(BombErr::Incomplete)));
  assert_eq!(
    la_lexes(),
    lexes,
    "the committed read is answered from the memo"
  );
  let budget = inp.token_budget();
  assert_eq!(budget.spent(), control_spent);
  assert_eq!(budget.look_ahead_spent(), 1);
  assert_eq!(
    budget.spent() - budget.look_ahead_spent(),
    control_spent - 1,
    "the difference is short of the control by the one read the memo answered"
  );
}

// ── T12: a session point opened and abandoned inside ──────────────────────────────────

/// A session point the closure opens and abandons is reconciled by the look-ahead's rollback: no
/// stranded point, pin, lineage entry or emitter mark.
#[test]
fn a_session_point_abandoned_inside_a_look_ahead_is_reconciled() {
  let mut input = complete("a b c");
  let mut inp = input.as_ref();
  let inside = inp.look_ahead(|inp| {
    let _abandoned = inp.begin_point();
    consume(inp, 2)
  });
  assert_eq!(inside, b"ab".to_vec());
  assert_eq!(inp.points(), 0, "the abandoned point is settled");
  assert_eq!(inp.session.lineage.pinned_len(), 0, "and its pin released");
  assert_eq!(inp.session.lineage.live_len(), 0, "and its lineage entry");
  assert_eq!(inp.emitter().live_rows(), 0, "and its emitter mark");
  assert_eq!(
    consume(&mut inp, 3),
    b"abc".to_vec(),
    "the progress was rolled back"
  );
}

// ── T13: inside an enclosing attempt ──────────────────────────────────────────────────

/// A look-ahead inside an attempt leaves the cache and the push count as the attempt found them,
/// so the attempt's own decline prunes exactly what it would have pruned without the look-ahead:
/// the entry it pushed, and not the one resident before its save.
#[test]
fn a_look_ahead_inside_an_attempt_leaves_the_attempt_s_restore_exact() {
  fn run(look: bool) -> (Window, std::vec::Vec<u8>, usize) {
    let mut input = complete("z a b c d");
    let mut inp = input.as_ref();
    let _ = inp.peek_one().unwrap();
    let declined = inp.attempt(|inp| {
      let _ = inp.peek::<U2>().unwrap();
      if look {
        let inside = inp.look_ahead(|inp| consume(inp, 3));
        assert_eq!(inside, b"zab".to_vec());
      }
      None::<()>
    });
    assert!(declined.is_none());
    let after = window(&inp);
    let lexes = la_lexes();
    let head = consume(&mut inp, 1);
    (after, head, la_lexes() - lexes)
  }

  let control = run(false);
  assert_eq!(
    control.0.cache.len(),
    1,
    "the control: the decline keeps `z`, resident before the save"
  );
  assert_eq!(control.2, 0, "so the read after it lexes nothing");
  assert_eq!(
    run(true),
    control,
    "the look-ahead inside changes nothing the attempt's decline restores"
  );
}
