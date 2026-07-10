//! Input-level tests for `InputRef` scanning entry points.

use core::cell::Cell;
use std::rc::Rc;

use crate::{
  Token,
  cache::DefaultCache,
  emitter::{Silent, Verbose},
  error::token::UnexpectedToken,
  input::Input,
  lexer::LogosLexer,
  state::State,
};

// ── A limiter whose scan counter is SHARED across every cloned lexer ──────────
//
// `InputRef` builds a fresh lexer per operation by cloning the state, so a
// plain by-value counter (like `TokenLimiter`) hides re-scans: the temporary
// lexer's increments are discarded with it. This limiter shares its counter
// through an `Rc<Cell<_>>`, so every token a temporary lexer scans is
// observable — a frozen count across calls proves the input latched and stopped
// rebuilding lexers.

#[derive(Debug, Clone, Default)]
struct ProbeLimiter {
  /// Total tokens ever scanned, shared by every clone of this state.
  scanned: Rc<Cell<usize>>,
  limit: usize,
}

impl ProbeLimiter {
  fn with_limit(limit: usize) -> Self {
    Self {
      scanned: Rc::new(Cell::new(0)),
      limit,
    }
  }

  /// A shared handle to observe the scan counter after moving the state in.
  fn counter(&self) -> Rc<Cell<usize>> {
    self.scanned.clone()
  }

  fn increase(&self) {
    self.scanned.set(self.scanned.get() + 1);
  }
}

#[derive(Debug, Clone, PartialEq)]
struct ProbeLimitExceeded;

impl State for ProbeLimiter {
  type Error = ProbeLimitExceeded;

  fn check(&self) -> Result<(), Self::Error> {
    if self.scanned.get() > self.limit {
      Err(ProbeLimitExceeded)
    } else {
      Ok(())
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
enum ProbeErr {
  Lex,
  Limit,
}

impl From<()> for ProbeErr {
  fn from(_: ()) -> Self {
    ProbeErr::Lex
  }
}

impl From<ProbeLimitExceeded> for ProbeErr {
  fn from(_: ProbeLimitExceeded) -> Self {
    ProbeErr::Limit
  }
}

// Lets `ProbeErr` back a `Verbose` emitter (via the blanket `FromEmitterError`).
// The unexpected-token path is never exercised by these tests; only the plain
// lexer-error path matters.
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for ProbeErr {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    ProbeErr::Lex
  }
}

#[derive(Debug, Clone, PartialEq, crate::logos::Logos)]
#[logos(crate = crate::logos, extras = ProbeLimiter, skip r"[ \t\r\n]+")]
enum ProbeTok {
  #[regex(r"[0-9]+", |lex| { lex.extras.increase(); })]
  Num,
}

impl core::fmt::Display for ProbeTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "number")
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ProbeKind {
  Num,
}

impl core::fmt::Display for ProbeKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "number")
  }
}

impl Token<'_> for ProbeTok {
  type Kind = ProbeKind;
  type Error = ProbeErr;

  fn kind(&self) -> ProbeKind {
    ProbeKind::Num
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

type ProbeLexer<'a> = LogosLexer<'a, ProbeTok>;
type ProbeCtx<'a> = (Silent<ProbeErr>, DefaultCache<'a, ProbeLexer<'a>>);
type ProbeVerboseCtx<'a> = (Verbose<ProbeErr>, DefaultCache<'a, ProbeLexer<'a>>);

/// Builds an input over `src` behind a limit-2 [`ProbeLimiter`], returning the
/// input and a shared handle to observe its scan counter. The third scanned
/// token trips the limiter.
fn probe_input(src: &str) -> (Input<'_, ProbeLexer<'_>, ProbeCtx<'_>, ()>, Rc<Cell<usize>>) {
  let limiter = ProbeLimiter::with_limit(2);
  let scanned = limiter.counter();
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let input = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(src, limiter, cache);
  (input, scanned)
}

#[test]
fn poisoned_input_latches_no_rescan_across_next_and_peek() {
  // Limit of 2: the third scanned token trips the limiter. A recovering
  // (`Silent`) emitter keeps going, so the trip surfaces as a bounded stop.
  let limiter = ProbeLimiter::with_limit(2);
  let scanned = limiter.counter();

  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Silent::<ProbeErr>::new();
  let mut input =
    Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache("1 2 3 4 5 6", limiter, cache);
  let mut inp = input.as_ref(&mut emitter);

  // Drive `next()` past the trip.
  assert!(inp.next().unwrap().is_some(), "first token");
  assert!(inp.next().unwrap().is_some(), "second token");
  // The third `next()` trips the limiter; the recovering emitter turns it into a
  // bounded stop (`None`) and latches the input.
  assert!(inp.next().unwrap().is_none(), "trip latches to None");

  let frozen = scanned.get();
  assert_eq!(frozen, 3, "scanned exactly 1, 2, 3 before latching");

  // (a) No further scanning work: repeated `next()`/`peek()` must NOT rebuild a
  // lexer or rescan the tripping token, so the shared counter stays frozen; and
  // (b) returns stay None/empty.
  for _ in 0..5 {
    assert!(inp.next().unwrap().is_none(), "poisoned next() stays None");
  }
  for _ in 0..5 {
    assert!(
      inp.peek_one().unwrap().is_none(),
      "poisoned peek() stays empty"
    );
  }

  assert_eq!(
    scanned.get(),
    frozen,
    "no lexer was rebuilt after the latch — the token counter is frozen"
  );
}

#[test]
fn poisoned_input_latches_no_rescan_across_try_expect() {
  // `try_expect(|_| true)` consumes one token per call; the third rebuilds a
  // lexer that scans the tripping token and latches.
  let (mut input, scanned) = probe_input("1 2 3 4 5 6");
  let mut emitter = Silent::<ProbeErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  assert!(inp.try_expect(|_| true).unwrap().is_some(), "first token");
  assert!(inp.try_expect(|_| true).unwrap().is_some(), "second token");
  // The third scan trips the limiter; the recovering emitter turns it into a
  // bounded `None` and latches the input.
  assert!(
    inp.try_expect(|_| true).unwrap().is_none(),
    "trip latches to None"
  );

  let frozen = scanned.get();
  assert_eq!(frozen, 3, "scanned exactly 1, 2, 3 before latching");

  // Repeated calls must NOT rebuild a lexer or rescan the tripping token.
  for _ in 0..5 {
    assert!(
      inp.try_expect(|_| true).unwrap().is_none(),
      "poisoned try_expect stays None"
    );
  }
  assert_eq!(
    scanned.get(),
    frozen,
    "no lexer was rebuilt after the latch — the token counter is frozen"
  );
}

#[test]
fn poisoned_input_latches_no_rescan_across_skip_while() {
  // `skip_while(|_| true)` drains every matching token in a single call, so the
  // first call scans through the tripping token and latches.
  let (mut input, scanned) = probe_input("1 2 3 4 5 6");
  let mut emitter = Silent::<ProbeErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  inp.skip_while(|_| true).unwrap();

  let frozen = scanned.get();
  assert_eq!(frozen, 3, "scanned exactly 1, 2, 3 before latching");

  // Repeated calls must short-circuit on the latch without rescanning.
  for _ in 0..5 {
    inp.skip_while(|_| true).unwrap();
  }
  assert_eq!(
    scanned.get(),
    frozen,
    "no lexer was rebuilt after the latch — the token counter is frozen"
  );
}

#[test]
fn poisoned_input_latches_no_rescan_across_sync_to() {
  // `sync_to(|_| false, ..)` never matches, so it skips through the whole input;
  // the first call scans the tripping token and latches.
  let (mut input, scanned) = probe_input("1 2 3 4 5 6");
  let mut emitter = Silent::<ProbeErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  assert!(
    inp.sync_to(|_| false, || None).unwrap().is_none(),
    "no matching token before the trip"
  );

  let frozen = scanned.get();
  assert_eq!(frozen, 3, "scanned exactly 1, 2, 3 before latching");

  for _ in 0..5 {
    assert!(
      inp.sync_to(|_| false, || None).unwrap().is_none(),
      "poisoned sync_to stays None"
    );
  }
  assert_eq!(
    scanned.get(),
    frozen,
    "no lexer was rebuilt after the latch — the token counter is frozen"
  );
}

#[test]
fn poisoned_input_latches_no_rescan_across_sync_through() {
  // `sync_through(|_| false, ..)` never matches, so it skips through the whole
  // input; the first call scans the tripping token and latches.
  let (mut input, scanned) = probe_input("1 2 3 4 5 6");
  let mut emitter = Silent::<ProbeErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  assert!(
    inp.sync_through(|_| false, || None).unwrap().is_none(),
    "no matching token before the trip"
  );

  let frozen = scanned.get();
  assert_eq!(frozen, 3, "scanned exactly 1, 2, 3 before latching");

  for _ in 0..5 {
    assert!(
      inp.sync_through(|_| false, || None).unwrap().is_none(),
      "poisoned sync_through stays None"
    );
  }
  assert_eq!(
    scanned.get(),
    frozen,
    "no lexer was rebuilt after the latch — the token counter is frozen"
  );
}

#[test]
fn restore_after_peek_across_lexer_error_reemits_error_exactly_once() {
  // `@` is a lexer error between two numbers (high limit: the limiter never
  // trips, so only the plain lexer error is in play).
  //   1 @ 2 3
  //   0 2 4 6      (`@` spans [2, 3))
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 @ 2 3",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );

  {
    use generic_arraydeque::typenum::U2;
    let mut inp = input.as_ref(&mut emitter);

    // Peek a window that crosses the malformed `@`; this emits (seals) its lexer
    // error and advances the dedup watermark past it. The cursor stays at 0.
    let _ = inp.peek::<U2>().unwrap();

    // Checkpoint is captured AFTER the error's emission, so `Verbose` retains the
    // error across the restore (the emission log keeps everything up to the mark).
    let ckp = inp.save();

    // Speculatively consume forward, draining the cache and lexing past the error
    // region, then abandon the branch.
    while inp.next().unwrap().is_some() {}
    inp.restore(ckp);

    // The commit path re-lexes from the checkpoint, crossing the malformed span a
    // second time. With the watermark restored to its saved value (past the
    // error), the re-lex must NOT re-emit the retained error.
    while inp.next().unwrap().is_some() {}
  }

  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 1,
    "the malformed span's lexer error must appear exactly once after peek → save → restore → re-consume"
  );
}
