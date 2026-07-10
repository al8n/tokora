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
fn stale_younger_restore_keeps_rewound_lexer_error_reemittable() {
  // Out-of-order (non-LIFO) restores must never RAISE the dedup watermark past a
  // diagnostic the emission log has already unwound.
  //   1 @ 2 3      (`@` is a lexer error spanning [2, 3))
  //   0 2 4 6
  //
  // save A (BEFORE the error) → peek across `@` (seals it, watermark → 3) →
  // save B (AFTER the error, so its saved watermark 3 sits ABOVE the error) →
  // drain the speculative branch (empties the cache so a later re-lex actually
  // re-crosses `@`) → restore A (older: unwinds `@` from the emission log and
  // drops the watermark to 0) → restore B (STALE and younger). Restoring B must
  // NOT raise the watermark back to 3: the emission log can no longer resurrect
  // `@`, so a raised watermark would dedupe the re-lex and lose the diagnostic
  // forever. The min-clamp keeps it at 0, so re-lexing forward re-emits `@`.
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

    // save A predates the error emission (its saved watermark is below `@`).
    let a = inp.save();

    // Peek across `@`: seals its lexer error and lifts the watermark past it.
    let _ = inp.peek::<U2>().unwrap();

    // save B is captured after the error, so its saved watermark sits above it.
    let b = inp.save();

    // The (inner) speculative branch consumes forward, draining the cache so the
    // committed re-lex below actually re-crosses the malformed region.
    while inp.next().unwrap().is_some() {}

    // Out-of-order restores: the OLDER checkpoint first (unwinds `@`, drops the
    // watermark), then the STALE younger one (whose saved watermark is stale).
    inp.restore(a);
    inp.restore(b);

    // Commit path: re-lex forward, crossing `@` a second time.
    while inp.next().unwrap().is_some() {}
  }

  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 1,
    "a stale younger restore must not raise the watermark past a rewound error — it stays re-emittable and is reported exactly once"
  );
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
fn stale_younger_restore_never_leaves_poison_without_its_diagnostic() {
  // Out-of-order (non-LIFO) restores must never leave the input latched while the
  // emission log has already unwound the limit diagnostic. The end-state invariant
  // is "poisoned implies a retained limit diagnostic".
  //   1 2 3 4 5 6   (limit 5 → the 6th scanned token trips; U6 window > U3 cache)
  //
  // save A (clean, BEFORE the trip) → overflow peek trips (poison + diagnostic) →
  // save B (AFTER the trip: poisoned, its saved watermark ABOVE the diagnostic) →
  // drain the speculative branch (empties the cache) → restore A (older: un-latches
  // poison, drops the watermark, unwinds the diagnostic) → restore B (STALE younger:
  // the AND-clamp keeps poison LOW — it cannot resurrect a latch whose diagnostic the
  // log can no longer produce). The committed drain re-lexes, re-trips, and re-emits.
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

    // save A predates the trip: a clean checkpoint (poisoned = false).
    let a = inp.save();

    // Overflow peek trips: poison latches, the limit diagnostic is sealed.
    let _ = inp.peek::<U6>().unwrap();
    assert!(inp.is_poisoned(), "the overflow trip must latch poison");

    // save B is captured AFTER the trip: poisoned, its saved watermark above the error.
    let b = inp.save();

    // Drain the speculative branch so the committed re-lex actually re-crosses the
    // tripping region (the cache prefix is emptied here).
    while inp.next().unwrap().is_some() {}

    // Out-of-order restores: the OLDER checkpoint first (un-latches poison, unwinds
    // the diagnostic and drops the watermark), then the STALE younger one.
    inp.restore(a);
    assert!(
      !inp.is_poisoned(),
      "restoring the clean older checkpoint un-latches poison"
    );
    inp.restore(b);
    // A stale younger restore must NOT resurrect the latch: the AND-clamp keeps it
    // low because `current` is already false — poison cannot outlive its diagnostic.
    assert!(
      !inp.is_poisoned(),
      "a stale younger restore must not raise poison past a rewound diagnostic"
    );

    // Commit path: re-lex forward, re-trip, re-emit.
    while inp.next().unwrap().is_some() {}

    // Invariant: poisoned implies a retained limit diagnostic (never a silent latch).
    assert!(inp.is_poisoned(), "the committed re-lex re-latches poison");
  }

  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 1,
    "poisoned implies a retained diagnostic: the limit error is re-emitted exactly once, never lost"
  );
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
