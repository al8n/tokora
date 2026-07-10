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

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "non-LIFO checkpoint restore")]
fn non_lifo_watermark_restore_is_rejected_in_debug() {
  // Contract: restores are last-in, first-out. Restoring the older checkpoint A
  // invalidates every checkpoint saved after it, so restoring the younger B afterward
  // refers to a lineage that no longer exists. The debug witness rejects it. (This is
  // the dedup-watermark shape: A predates a sealed `@`, B postdates it.)
  //   1 @ 2 3      (`@` is a lexer error spanning [2, 3))
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 @ 2 3",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );

  use generic_arraydeque::typenum::U2;
  let mut inp = input.as_ref(&mut emitter);

  let a = inp.save(); // older, predates the sealed `@`
  let _ = inp.peek::<U2>().unwrap(); // seals `@`, lifts the watermark
  let b = inp.save(); // younger, postdates the sealed `@`
  while inp.next().unwrap().is_some() {}

  inp.restore(a); // invalidates b
  inp.restore(b); // ✗ non-LIFO — debug panic
}

#[test]
fn restore_before_overflow_trip_reemits_limit_diagnostic_exactly_once() {
  // A limit trip during an *overflow* peek latches poison AND emits the limit
  // diagnostic together. A caller that saved BEFORE that speculative peek and then
  // restores must not be left silently poisoned: `restore` un-latches the poison
  // (the AND-clamp lowers it toward the clean saved value) in lockstep with the
  // emitter rewind that removed the speculative diagnostic. The committed drain
  // then re-lexes the region, re-trips, re-latches, and RE-EMITS the diagnostic —
  // exactly once, never a diagnostic-less latch masquerading as clean EOF.
  //   1 2 3 4 5 6   (limit 5 → the 6th scanned token trips; U6 window > U3 cache)
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    ProbeLimiter::with_limit(5),
    cache,
  );

  {
    use generic_arraydeque::typenum::U6;
    let mut inp = input.as_ref(&mut emitter);

    // save BEFORE the speculative peek: the checkpoint is clean (poisoned = false).
    let ckp = inp.save();

    // Overflow peek (U6 > U3 cache) trips the limiter mid-overflow: poison latches
    // and the limit diagnostic is sealed into the emitter.
    let _ = inp.peek::<U6>().unwrap();
    assert!(inp.is_poisoned(), "the overflow trip must latch poison");

    // Restore the pre-peek checkpoint: the emitter rewinds the speculative
    // diagnostic AND the AND-clamp lowers poison back to the clean saved value, so
    // the latch is not left stranded without its diagnostic.
    inp.restore(ckp);
    assert!(
      !inp.is_poisoned(),
      "restoring a clean checkpoint must un-latch the speculative poison"
    );

    // Drain the committed path: it re-lexes the region and re-trips the limiter.
    while inp.next().unwrap().is_some() {}

    // The re-trip re-establishes the latch — poison stays paired with its diagnostic.
    assert!(
      inp.is_poisoned(),
      "the committed re-lex must re-latch poison"
    );
  }

  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 1,
    "the limit diagnostic must survive save → overflow-trip → restore → drain, reported exactly once"
  );
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "non-LIFO checkpoint restore")]
fn non_lifo_poison_boundary_restore_is_rejected_in_debug() {
  // Contract: restores are last-in, first-out — the poison-boundary analog of the
  // watermark case. A clean older A, an overflow trip that poisons the input, a
  // poisoned younger B; restoring A invalidates B, so restoring B afterward is a
  // violation the debug witness rejects.
  //   1 2 3 4 5 6   (limit 5 → the 6th scanned token trips; U6 window > U3 cache)
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    ProbeLimiter::with_limit(5),
    cache,
  );

  use generic_arraydeque::typenum::U6;
  let mut inp = input.as_ref(&mut emitter);

  let a = inp.save(); // older, clean
  let _ = inp.peek::<U6>().unwrap(); // overflow trip: poison + diagnostic
  assert!(inp.is_poisoned(), "the overflow trip must latch poison");
  let b = inp.save(); // younger, poisoned
  while inp.next().unwrap().is_some() {}

  inp.restore(a); // invalidates b
  inp.restore(b); // ✗ non-LIFO — debug panic
}

// ── The poison BOUNDARY: a drained cache prefix replays after a restore ────────
//
// These use a BY-VALUE limiter (`TokenLimiter`, checkpointed/restored with the
// lexer state) rather than the shared `ProbeLimiter`. The distinction is load
// bearing: an overflow peek never writes its temporary lexer's counter back into
// the input state, so a checkpoint taken *after* the trip still saves a clean
// count. Restoring it therefore lets the committed path re-lex the prefix from
// scratch, re-counting toward the same limit and re-tripping at the very position
// it would have — which is exactly what makes a positional boundary observable: a
// shared counter would instead re-trip on the first replayed token and hide the
// prefix again.

use crate::state::token_tracker::{TokenLimitExceeded, TokenLimiter};

#[derive(Debug, Clone, PartialEq)]
enum ByValErr {
  Lex,
  Limit,
}

impl From<()> for ByValErr {
  fn from(_: ()) -> Self {
    ByValErr::Lex
  }
}

impl From<TokenLimitExceeded> for ByValErr {
  fn from(_: TokenLimitExceeded) -> Self {
    ByValErr::Limit
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for ByValErr {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    ByValErr::Lex
  }
}

#[derive(Debug, Clone, PartialEq, crate::logos::Logos)]
#[logos(crate = crate::logos, extras = TokenLimiter, skip r"[ \t\r\n]+")]
enum ByValTok {
  #[regex(r"[0-9]+", |lex| { lex.extras.increase(); })]
  Num,
}

impl core::fmt::Display for ByValTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "number")
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ByValKind {
  Num,
}

impl core::fmt::Display for ByValKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "number")
  }
}

impl Token<'_> for ByValTok {
  type Kind = ByValKind;
  type Error = ByValErr;

  fn kind(&self) -> ByValKind {
    ByValKind::Num
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

type ByValLexer<'a> = LogosLexer<'a, ByValTok>;
type ByValVerboseCtx<'a> = (Verbose<ByValErr>, DefaultCache<'a, ByValLexer<'a>>);

#[test]
fn overflow_trip_peek_save_drain_restore_replays_prefix_and_stops_at_boundary() {
  // THE positional-boundary case. An overflow peek trips mid-window, truncates to
  // the cache-resident prefix, and latches the poison boundary at the DURABLE
  // FRONTIER — the end of the last cached token. A caller then SAVES, drains the
  // prefix speculatively, and RESTORES the same checkpoint. It must observe:
  //   (a) the prefix tokens are consumable AGAIN (same spans, same order) — the
  //       cache was drained, so the boundary lets lexing strictly before it
  //       replay the prefix from source;
  //   (b) after the prefix the stream ends AT the boundary — the trip token and
  //       everything past it are never re-scanned (frozen scan counter);
  //   (c) the limit diagnostic is retained exactly once.
  //
  // Under the old boolean latch the restore left the input fully latched, so the
  // prefix visible at save time became unreachable (the first replay `next()`
  // short-circuited to `None`): restore did not reproduce the saved state.
  //
  //   1 2 3 4 5 6   (limit 5 → the 6th scanned token trips; U6 window > U3 cache)
  //   ^0 ^2 ^4      (token 3 spans [4, 5): the durable frontier is offset 5)
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::U6;

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(5),
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    // Overflow peek (U6 > U3 cache): caches 1..=3, stages 4 & 5, trips on 6. The
    // result is truncated to the 3-token cache-resident prefix; the boundary
    // latches at the durable frontier (end of token 3, offset 5).
    {
      let peeked = inp.peek::<U6>().unwrap();
      assert_eq!(
        peeked.len(),
        3,
        "the overflow trip truncates the peek to the cache-resident prefix"
      );
    }
    assert!(
      inp.is_poisoned(),
      "the overflow trip latches the poison boundary"
    );

    // Save AFTER the trip: the checkpoint carries the boundary AND the retained
    // limit diagnostic (its emitter mark postdates the emission).
    let ckp = inp.save();

    // Speculatively drain the cached prefix. The three cached tokens come back;
    // then the boundary stops `next()` at the durable frontier (no phantom 4/5/6).
    assert_eq!(
      *inp.next().unwrap().expect("drain 1").span_ref(),
      SimpleSpan::new(0, 1)
    );
    assert_eq!(
      *inp.next().unwrap().expect("drain 2").span_ref(),
      SimpleSpan::new(2, 3)
    );
    assert_eq!(
      *inp.next().unwrap().expect("drain 3").span_ref(),
      SimpleSpan::new(4, 5)
    );
    assert!(
      inp.next().unwrap().is_none(),
      "the drain stops at the boundary — no phantom lookahead"
    );

    // Restore the SAME checkpoint: `boundary = max(saved, current)` keeps it intact.
    inp.restore(ckp);
    assert!(
      inp.is_poisoned(),
      "the boundary survives the restore (saved == current)"
    );

    // (a) The prefix is consumable AGAIN — same spans, same order — replayed from
    // source because the cache was drained.
    assert_eq!(
      *inp.next().unwrap().expect("replay 1").span_ref(),
      SimpleSpan::new(0, 1)
    );
    assert_eq!(
      *inp.next().unwrap().expect("replay 2").span_ref(),
      SimpleSpan::new(2, 3)
    );
    assert_eq!(
      *inp.next().unwrap().expect("replay 3").span_ref(),
      SimpleSpan::new(4, 5)
    );
    // (b) After the prefix the stream ends exactly at the boundary.
    assert!(
      inp.next().unwrap().is_none(),
      "the replay stops at the boundary — nothing past it is re-scanned"
    );
    // The frozen scan counter proves it: the replay re-scanned exactly the 3-token
    // prefix (the trip token past the boundary is never reached), so the current
    // lexer lineage's count is 3, not 4+.
    assert_eq!(
      inp.state().tokens(),
      3,
      "the replay scanned exactly the prefix (3), never the trip token past the boundary"
    );
  }

  // (c) The limit diagnostic is retained across save → drain → restore → replay,
  // exactly once.
  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 1,
    "the limit diagnostic survives save → drain → restore → replay, reported exactly once"
  );
}

// ── Last-in, first-out contract: the debug witness ─────────────────────────────
//
// Restoring a checkpoint invalidates every checkpoint saved after it, so restores
// must be LIFO. Debug builds track the live checkpoints exactly and panic on any
// out-of-order restore, and on any restore into a foreign input. These tests pin
// that witness; the LIFO-legal tests above pin the pure-copy behavior it protects.

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "non-LIFO checkpoint restore")]
fn alias_interleave_stale_restore_detected() {
  // THE witness test. After restoring the older A (which invalidates B), one
  // committed emission regrows the emission log to exactly B's saved mark, so a
  // length-based validity check (`B.mark <= emitter.len()`) would pass — yet B's
  // lineage is gone. The live-checkpoint id stack still rejects the restore.
  //   1 2 3 4 5 6   (limit 2 → the 3rd scanned token trips, emitting the diagnostic)
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    ProbeLimiter::with_limit(2),
    cache,
  );

  use generic_arraydeque::typenum::U6;
  let mut inp = input.as_ref(&mut emitter);

  let a = inp.save(); // older, clean, mark 0
  let _ = inp.peek::<U6>().unwrap(); // trips: emits the limit diagnostic (mark → 1)
  let b = inp.save(); // younger, mark 1
  inp.restore(a); // invalidates b; the emitter rewinds back to mark 0

  // One committed emission regrows the log to length 1 == B's saved mark.
  while inp.next().unwrap().is_some() {}

  inp.restore(b); // ✗ non-LIFO — only the id stack catches this
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "non-LIFO checkpoint restore")]
fn stale_poisoned_restore_never_exposes_tokens_past_saved_boundary() {
  // Clean older A, an overflow trip that latches an early boundary, poisoned younger
  // B, restore A (un-poisoning and invalidating B), consume through to a later
  // committed trip, then restore B. B's boundary belongs to a lineage restoring A
  // destroyed; the witness rejects the restore rather than exposing tokens between
  // the two frontiers.
  //   1 2 3 4 5 6   (limit 5 → the 6th scanned token trips; U6 window > U3 cache)
  use generic_arraydeque::typenum::U6;

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(5),
    cache,
  );

  let mut inp = input.as_ref(&mut emitter);

  let a = inp.save(); // clean older
  let _ = inp.peek::<U6>().unwrap(); // overflow trip: early boundary
  assert!(inp.is_poisoned(), "the overflow trip must latch poison");
  let b = inp.save(); // poisoned younger
  inp.restore(a); // invalidates b, un-poisons
  while inp.next().unwrap().is_some() {} // committed path re-lexes to a later trip

  inp.restore(b); // ✗ non-LIFO — debug panic
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "checkpoint restored into a foreign input")]
fn restore_with_foreign_checkpoint_rejected_in_debug() {
  // A checkpoint may only be restored into the input that created it.
  let cache1 = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let cache2 = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut em1 = Silent::<ProbeErr>::new();
  let mut em2 = Silent::<ProbeErr>::new();
  let mut in1 = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
    "1 2 3",
    ProbeLimiter::with_limit(usize::MAX),
    cache1,
  );
  let mut in2 = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
    "1 2 3",
    ProbeLimiter::with_limit(usize::MAX),
    cache2,
  );

  let foreign = {
    let r1 = in1.as_ref(&mut em1);
    r1.save()
  };
  let mut r2 = in2.as_ref(&mut em2);
  r2.restore(foreign); // ✗ created by a different input — debug panic
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "checkpoint restored into a foreign input")]
fn restore_with_clone_sibling_checkpoint_rejected_in_debug() {
  // A clone is a NEW input: a checkpoint from the original may not be restored into
  // the clone — their checkpoints must never cross.
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut em1 = Silent::<ProbeErr>::new();
  let mut em2 = Silent::<ProbeErr>::new();
  let mut original = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
    "1 2 3",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );
  let mut sibling = original.clone();

  let from_original = {
    let r = original.as_ref(&mut em1);
    r.save()
  };
  let mut r2 = sibling.as_ref(&mut em2);
  r2.restore(from_original); // ✗ the clone is a foreign input — debug panic
}

// ── Pure copy replays the saved lineage exactly (LIFO-legal) ───────────────────

#[test]
fn twin_checkpoint_restore_after_partial_drain_replays_identically() {
  use crate::span::SimpleSpan;
  // Two checkpoints at the same position. Draining one token then restoring the
  // younger, re-draining, then restoring the elder and re-draining yields identical
  // span sequences both times — pure copy replays the lineage exactly.
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Silent::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );
  let mut inp = input.as_ref(&mut emitter);

  let elder = inp.save();
  let younger = inp.save(); // same position as elder

  // Drain one token, then restore the younger and replay the full stream.
  let _ = inp.next().unwrap().expect("first token");
  inp.restore(younger);
  let mut seq_young = Vec::new();
  while let Some(tok) = inp.next().unwrap() {
    seq_young.push(*tok.span_ref());
  }

  // Restore the elder (still LIFO: the younger has been consumed) and replay again.
  inp.restore(elder);
  let mut seq_old = Vec::new();
  while let Some(tok) = inp.next().unwrap() {
    seq_old.push(*tok.span_ref());
  }

  assert_eq!(
    seq_young, seq_old,
    "both restores replay the same lineage identically"
  );
  assert_eq!(
    seq_young,
    vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(2, 3),
      SimpleSpan::new(4, 5),
      SimpleSpan::new(6, 7),
    ]
  );
}

#[test]
fn save_exactly_at_boundary_restores_empty_stream() {
  // Trip the limit by draining past it so the cursor sits exactly at the poison
  // boundary; save there; restore. Every scanner entry point yields its poisoned
  // outcome, the shared scan counter stays frozen, and the limit diagnostic is
  // retained exactly once.
  //   1 2 3 4 5 6   (limit 2 → the 3rd scanned token trips)
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let limiter = ProbeLimiter::with_limit(2);
  let scanned = limiter.counter();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    limiter,
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    // Drain to the trip: 1, 2, then the 3rd next() trips and latches at the cursor.
    assert!(inp.next().unwrap().is_some(), "first token");
    assert!(inp.next().unwrap().is_some(), "second token");
    assert!(inp.next().unwrap().is_none(), "trip latches to None");
    assert!(inp.is_poisoned(), "the trip latches the boundary");
    let frozen = scanned.get();

    let ckp = inp.save(); // saved with the cursor exactly at the boundary
    inp.restore(ckp); // LIFO restore of the poisoned checkpoint

    // Every scanner entry point returns its poisoned (empty) outcome, never rescanning.
    assert!(inp.next().unwrap().is_none(), "next() stays None");
    assert!(inp.peek_one().unwrap().is_none(), "peek stays empty");
    assert!(
      inp.try_expect(|_| true).unwrap().is_none(),
      "try_expect stays None"
    );
    assert!(
      inp.sync_to(|_| false, || None).unwrap().is_none(),
      "sync_to stays None"
    );
    assert!(
      inp.sync_through(|_| false, || None).unwrap().is_none(),
      "sync_through stays None"
    );
    inp.skip_while(|_| true).unwrap();

    assert_eq!(
      scanned.get(),
      frozen,
      "the scan counter stays frozen — nothing past the boundary is re-scanned"
    );
  }

  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(total, 1, "the limit diagnostic is retained exactly once");
}

#[test]
fn sink_emitter_trip_bounds_work_and_survives_restore() {
  // With a non-collecting (Silent) emitter the poison boundary is input-owned: it
  // survives an attempt-style rollback even though the emitter retains nothing to
  // derive it from, and no rescan ever crosses it.
  //   1 2 3 4 5 6   (limit 2 → the 3rd scanned token trips)
  let (mut input, scanned) = probe_input("1 2 3 4 5 6");
  let mut emitter = Silent::<ProbeErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  // Trip the limit.
  assert!(inp.next().unwrap().is_some(), "first token");
  assert!(inp.next().unwrap().is_some(), "second token");
  assert!(inp.next().unwrap().is_none(), "trip latches to None");
  assert!(inp.is_poisoned(), "the trip latches the boundary");
  let frozen = scanned.get();

  // An attempt speculatively re-enters scanners, then declines and rolls back. The
  // boundary is checkpointed and copied back verbatim — the Silent emitter keeps no
  // log, so this proves the boundary is input-owned.
  let outcome = inp.attempt(|inp| {
    for _ in 0..5 {
      let _ = inp.next().unwrap();
    }
    None::<()>
  });
  assert!(outcome.is_none(), "the attempt declines and rolls back");
  assert!(
    inp.is_poisoned(),
    "the boundary survives the attempt rollback (input-owned)"
  );
  assert_eq!(
    scanned.get(),
    frozen,
    "no unbounded rescan — the shared scan counter stays frozen"
  );
}

#[test]
fn attempt_backtrack_over_trip_reemits_diagnostic_exactly_once() {
  // Inside an attempt, an overflow peek trips the limit (emitting the diagnostic);
  // the closure declines, rolling the speculative diagnostic back. The committed path
  // then re-reaches the trip and re-emits — exactly once in total, never zero.
  //   1 2 3 4 5 6   (limit 5 → the 6th scanned token trips; U6 window > U3 cache)
  use generic_arraydeque::typenum::U6;

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(5),
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    let outcome = inp.attempt(|inp| {
      let _ = inp.peek::<U6>(); // overflow trip: emits the limit diagnostic
      None::<()> // decline → rollback
    });
    assert!(outcome.is_none(), "the attempt declines and rolls back");
    assert!(
      !inp.is_poisoned(),
      "the rollback un-poisons and un-emits the speculative diagnostic"
    );

    // The committed path re-reaches the trip and re-emits.
    while inp.next().unwrap().is_some() {}
    assert!(inp.is_poisoned(), "the committed re-lex re-latches poison");
  }

  let errs: Vec<&ByValErr> = emitter.errors().values().flatten().collect();
  assert_eq!(
    errs.len(),
    1,
    "the limit diagnostic is emitted exactly once in total"
  );
  assert_eq!(*errs[0], ByValErr::Limit, "and it is the limit diagnostic");
}

#[test]
fn restore_after_interleaved_emissions_keeps_rewound_lexer_error_reemittable() {
  use crate::span::SimpleSpan;
  // A lexer error emitted interleaved with unexpected-token emissions (from sync_to),
  // then a restore of a checkpoint that predates them all. Pure copy returns the
  // watermark to its saved value, so the rewound lexer error re-emits exactly once
  // when the committed path re-reaches it — never zero times.
  //   1 @ 2   (`@` is a lexer error spanning [2, 3))
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 @ 2",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    let a = inp.save(); // predates every emission

    // sync_to(never-match) skips `1` (an unexpected token), crosses `@` (a lexer
    // error, lifting the watermark past it), then skips `2` — interleaved emissions.
    assert!(inp.sync_to(|_| false, || None).unwrap().is_none());

    inp.restore(a); // LIFO: rolls the log back and the watermark to 0

    // The committed path re-crosses `@`; with the watermark restored, it re-emits.
    while inp.next().unwrap().is_some() {}
  }

  // Exactly the one `@` lexer error is retained — re-emitted once, never lost.
  let at = SimpleSpan::new(2, 3);
  assert_eq!(
    emitter
      .errors()
      .get(&at)
      .map(|group| group.len())
      .unwrap_or(0),
    1,
    "the rewound lexer error re-emits exactly once when re-reached"
  );
  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(total, 1, "only the re-emitted lexer error is retained");
}

#[test]
#[cfg(debug_assertions)]
fn property_random_lifo_scripts_stay_faithful_and_bounded() {
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::{U1, U2, U3};

  const SRC: &str = "1 @ 2 3 @ 4"; // Num tokens at 0,4,6,10; `@` errors at [2,3),[8,9)

  // Oracle: one fresh single pass over SRC.
  let oracle_tokens: Vec<SimpleSpan> = {
    let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
    let mut em = Verbose::<ProbeErr>::new();
    let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
      SRC,
      ProbeLimiter::with_limit(usize::MAX),
      cache,
    );
    let mut inp = input.as_ref(&mut em);
    let mut toks = Vec::new();
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    toks
  };
  let oracle_next =
    |off: usize| -> Option<SimpleSpan> { oracle_tokens.iter().copied().find(|s| s.start() >= off) };
  // The `@` lexer errors a full pass would emit.
  let oracle_diags: Vec<SimpleSpan> = vec![SimpleSpan::new(2, 3), SimpleSpan::new(8, 9)];

  // Deterministic linear congruential generator (no external dev-deps).
  let mut rng: u64 = 0x0123_4567_89ab_cdef;
  let roll = |rng: &mut u64| -> u64 {
    *rng = rng
      .wrapping_mul(6364136223846793005)
      .wrapping_add(1442695040888963407);
    *rng >> 33
  };

  for _script in 0..200u32 {
    let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
    let limiter = ProbeLimiter::with_limit(usize::MAX);
    let scanned = limiter.counter();
    let mut em = Verbose::<ProbeErr>::new();
    let mut input =
      Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(SRC, limiter, cache);

    let num_ops = 8 + (roll(&mut rng) % 12) as usize; // 8..=19 ops
    {
      let mut inp = input.as_ref(&mut em);
      // Live checkpoints as a stack of (checkpoint, saved cursor offset).
      let mut live: Vec<(crate::input::Checkpoint<'_, '_, ProbeLexer<'_>>, usize)> = Vec::new();

      for _ in 0..num_ops {
        match roll(&mut rng) % 4 {
          0 => {
            // save
            let off = *inp.cursor().as_inner();
            live.push((inp.save(), off));
          }
          1 => {
            // drain k
            let k = 1 + (roll(&mut rng) % 3) as usize;
            for _ in 0..k {
              if inp.next().unwrap().is_none() {
                break;
              }
            }
          }
          2 => {
            // peek w
            match roll(&mut rng) % 3 {
              0 => {
                let _ = inp.peek::<U1>().unwrap();
              }
              1 => {
                let _ = inp.peek::<U2>().unwrap();
              }
              _ => {
                let _ = inp.peek::<U3>().unwrap();
              }
            }
          }
          _ => {
            // restore the most-recent live checkpoint (always LIFO), then verify the
            // next drained span matches a fresh parse of the same prefix.
            if let Some((ckp, off)) = live.pop() {
              inp.restore(ckp);
              let got = inp.next().unwrap().map(|t| *t.span_ref());
              assert_eq!(
                got,
                oracle_next(off),
                "after a restore to offset {off}, the next token must match a fresh parse"
              );
            }
          }
        }
      }
    }

    // (a) total scans bounded by a generous linear budget.
    let budget = (num_ops + 1) * (oracle_tokens.len() + oracle_diags.len()) * 8;
    assert!(
      scanned.get() <= budget,
      "scans {} exceeded the linear budget {budget}",
      scanned.get()
    );

    // (b) every retained diagnostic span appears at most once and is a real error.
    for (span, group) in em.errors() {
      assert!(
        group.len() <= 1,
        "diagnostic span {span:?} retained more than once"
      );
      assert!(
        group.is_empty() || oracle_diags.contains(span),
        "retained an unexpected diagnostic span {span:?}"
      );
    }
  }
}

// ── try_attempt: Result-shaped speculation ─────────────────────────────────────
//
// `try_attempt` is the fallible sibling of `attempt`: on `Ok` progress is kept, on
// `Err` the input rolls back exactly as `restore` would and the error is returned.
// The save/restore pair is closure-scoped, so it is LIFO by construction.

#[test]
fn try_attempt_ok_keeps_progress() {
  use crate::span::SimpleSpan;
  // On `Ok`, the closure's progress is kept and the value is passed through.
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Silent::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );
  let mut inp = input.as_ref(&mut emitter);

  let start = *inp.cursor().as_inner();
  let out: Result<i64, ()> = inp.try_attempt(|inp| {
    let _ = inp.next().unwrap().expect("first token");
    let _ = inp.next().unwrap().expect("second token");
    Ok(7)
  });
  assert_eq!(out, Ok(7), "the Ok value is returned");
  assert!(
    *inp.cursor().as_inner() > start,
    "progress is kept — the cursor advanced past the consumed tokens"
  );
  // The two consumes stuck, so the next token is the third.
  assert_eq!(
    *inp.next().unwrap().expect("third token").span_ref(),
    SimpleSpan::new(4, 5)
  );
}

#[test]
fn try_attempt_err_rolls_back_everything() {
  use crate::span::SimpleSpan;

  // ── position, span, lexer state, emission log, and the dedup watermark ─────────
  // "1 @ 2": crossing the malformed `@` inside the attempt emits its lexer error and
  // lifts the watermark. Returning `Err` must roll every one of those back.
  {
    let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
    let mut emitter = Verbose::<ByValErr>::new();
    let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
      "1 @ 2",
      TokenLimiter::with_limitation(usize::MAX),
      cache,
    );

    // Phase 1: the attempt consumes across `@` (emitting the lexer error), then
    // abandons. Position, span, and lexer state must all return to their saved values.
    {
      let mut inp = input.as_ref(&mut emitter);

      let cur0 = *inp.cursor().as_inner();
      let span0 = *inp.span();
      let tokens0 = inp.state().tokens();

      let out: Result<(), ()> = inp.try_attempt(|inp| {
        // Consume `1`, cross `@` (emits the lexer error, lifts the watermark),
        // consume `2`, then abandon the branch.
        while inp.next().unwrap().is_some() {}
        Err(())
      });
      assert_eq!(out, Err(()), "the error is returned to the caller");

      assert_eq!(*inp.cursor().as_inner(), cur0, "position rolled back");
      assert_eq!(*inp.span(), span0, "last-consumed span rolled back");
      assert_eq!(inp.state().tokens(), tokens0, "lexer state rolled back");
    }

    // The emission log was truncated by the rollback: nothing the attempt emitted
    // survives.
    let after_rollback: usize = emitter.errors().values().map(|g| g.len()).sum();
    assert_eq!(
      after_rollback, 0,
      "diagnostics emitted inside the attempt are rolled back (empty emission log)"
    );

    // Phase 2: the watermark rolled back too, so the committed path re-crosses `@`
    // and the rewound lexer error becomes re-emittable — exactly once.
    {
      let mut inp = input.as_ref(&mut emitter);
      while inp.next().unwrap().is_some() {}
    }
    let at = SimpleSpan::new(2, 3);
    assert_eq!(
      emitter.errors().get(&at).map(|g| g.len()).unwrap_or(0),
      1,
      "the rewound lexer error re-emits exactly once when re-reached"
    );
    let total: usize = emitter.errors().values().map(|g| g.len()).sum();
    assert_eq!(total, 1, "only the re-emitted lexer error is retained");
  }

  // ── the poison boundary, via a limit-trip variant ─────────────────────────────
  // An overflow peek inside the attempt trips the limiter (latching poison and
  // emitting the diagnostic); the `Err` rollback un-latches it, and the committed
  // path re-trips — the diagnostic surviving exactly once, never a diagnostic-less
  // latch.
  {
    use generic_arraydeque::typenum::U6;
    let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
    let mut emitter = Verbose::<ByValErr>::new();
    let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
      "1 2 3 4 5 6",
      TokenLimiter::with_limitation(5),
      cache,
    );
    {
      let mut inp = input.as_ref(&mut emitter);

      let out: Result<(), ()> = inp.try_attempt(|inp| {
        let _ = inp.peek::<U6>().unwrap(); // overflow trip: poison + diagnostic
        Err(())
      });
      assert_eq!(out, Err(()));
      assert!(
        !inp.is_poisoned(),
        "the Err rollback un-latches the speculative poison boundary"
      );

      // The committed path re-reaches the trip and re-latches.
      while inp.next().unwrap().is_some() {}
      assert!(inp.is_poisoned(), "the committed re-lex re-latches poison");
    }
    let total: usize = emitter.errors().values().map(|g| g.len()).sum();
    assert_eq!(
      total, 1,
      "the limit diagnostic is emitted exactly once in total"
    );
  }
}

#[test]
fn try_attempt_nested_lifo() {
  use crate::span::SimpleSpan;

  // A `try_attempt` nested inside an `attempt`: the inner `Err` rollback is fully
  // contained, and the outer keeps its own progress. The closure-scoped save/restore
  // pairs nest as a stack, so the LIFO witness never fires.
  {
    let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
    let mut emitter = Silent::<ProbeErr>::new();
    let mut input = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
      "1 2 3 4",
      ProbeLimiter::with_limit(usize::MAX),
      cache,
    );
    let mut inp = input.as_ref(&mut emitter);

    let out = inp.attempt(|inp| {
      let _ = inp.next().unwrap().expect("outer consumes 1");
      let inner: Result<(), ()> = inp.try_attempt(|inp| {
        let _ = inp.next().unwrap().expect("inner consumes 2");
        Err(()) // inner rolls back to just after 1
      });
      assert!(inner.is_err(), "the inner try_attempt returned Err");
      Some(()) // outer keeps its own progress (only 1 consumed)
    });
    assert!(out.is_some(), "the outer attempt kept progress");
    // The inner's consume of 2 was rolled back; the next token is 2.
    assert_eq!(
      *inp.next().unwrap().expect("token 2").span_ref(),
      SimpleSpan::new(2, 3)
    );
  }

  // The mirror image: an `attempt` nested inside a `try_attempt`.
  {
    let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
    let mut emitter = Silent::<ProbeErr>::new();
    let mut input = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
      "1 2 3 4",
      ProbeLimiter::with_limit(usize::MAX),
      cache,
    );
    let mut inp = input.as_ref(&mut emitter);

    let out: Result<(), ()> = inp.try_attempt(|inp| {
      let _ = inp.next().unwrap().expect("outer consumes 1");
      let inner = inp.attempt(|inp| {
        let _ = inp.next().unwrap().expect("inner consumes 2");
        None::<()> // inner rolls back to just after 1
      });
      assert!(inner.is_none(), "the inner attempt returned None");
      Ok(()) // outer keeps its own progress (only 1 consumed)
    });
    assert!(out.is_ok(), "the outer try_attempt kept progress");
    assert_eq!(
      *inp.next().unwrap().expect("token 2").span_ref(),
      SimpleSpan::new(2, 3)
    );
  }
}

// ── A hand-rolled lexer that yields a ZERO-WIDTH token span ────────────────────
//
// The bundled Logos backend never produces an empty span, but the `Lexer` trait
// permits hand-written lexers that do. A zero-width token sitting at the poison
// boundary is excluded by the positional gate yet advances nothing, silently
// breaking replay and termination — so the contract forbids it and the single
// lexing chokepoint (`lex_within_boundary`) debug-asserts against it. This fixture
// yields one `[0, 0)` token to drive that assert.

#[derive(Debug, Clone, PartialEq)]
struct ZeroWidthErr;

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>>
  for ZeroWidthErr
{
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    ZeroWidthErr
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ZeroWidthTok;

impl core::fmt::Display for ZeroWidthTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "zero-width")
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ZeroWidthKind;

impl core::fmt::Display for ZeroWidthKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "zero-width")
  }
}

impl Token<'_> for ZeroWidthTok {
  type Kind = ZeroWidthKind;
  type Error = ZeroWidthErr;

  fn kind(&self) -> ZeroWidthKind {
    ZeroWidthKind
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

/// A lexer that yields exactly one zero-width `[0, 0)` token, then end of input.
struct ZeroWidthLexer<'inp> {
  src: &'inp str,
  state: (),
  yielded: bool,
}

impl<'inp> crate::Lexer<'inp> for ZeroWidthLexer<'inp> {
  type State = ();
  type Source = str;
  type Token = ZeroWidthTok;
  type Span = crate::SimpleSpan;
  type Offset = usize;

  fn new(src: &'inp str) -> Self {
    Self {
      src,
      state: (),
      yielded: false,
    }
  }

  fn with_state(src: &'inp str, state: ()) -> Self {
    Self {
      src,
      state,
      yielded: false,
    }
  }

  fn check(&self) -> Result<(), ZeroWidthErr> {
    Ok(())
  }

  fn state(&self) -> &Self::State {
    &self.state
  }

  fn state_mut(&mut self) -> &mut Self::State {
    &mut self.state
  }

  fn into_state(self) -> Self::State {
    self.state
  }

  fn source(&self) -> &'inp str {
    self.src
  }

  fn span(&self) -> Self::Span {
    // The zero-width span the contract forbids.
    crate::SimpleSpan::new(0, 0)
  }

  fn slice(&self) -> <Self::Source as crate::Source<Self::Offset>>::Slice<'inp> {
    ""
  }

  fn lex(&mut self) -> Option<Result<ZeroWidthTok, ZeroWidthErr>> {
    if self.yielded {
      return None;
    }
    self.yielded = true;
    Some(Ok(ZeroWidthTok))
  }

  fn bump(&mut self, _n: &usize) {}
}

type ZeroWidthVerboseCtx<'a> = (Verbose<ZeroWidthErr>, DefaultCache<'a, ZeroWidthLexer<'a>>);

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "lexer contract violation")]
fn boundary_replay_zero_width_token_contract() {
  // A hand-rolled lexer that yields a zero-width token trips the debug assert at the
  // single lexing chokepoint before the empty span can corrupt any positional fact.
  let cache = DefaultCache::<'_, ZeroWidthLexer<'_>>::default();
  let mut emitter = Verbose::<ZeroWidthErr>::new();
  let mut input = Input::<ZeroWidthLexer<'_>, ZeroWidthVerboseCtx<'_>, ()>::with_state_and_cache(
    "abc",
    (),
    cache,
  );
  let mut inp = input.as_ref(&mut emitter);
  let _ = inp.next();
}
