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
/// A capacity-1 cache (`Option`) over the probe lexer, for the abandoned-lineage
/// truncation on the smallest non-trivial cache.
type ProbeOptionCache<'a> = Option<crate::cache::CachedTokenOf<'a, ProbeLexer<'a>>>;
type ProbeOptionVerboseCtx<'a> = (Verbose<ProbeErr>, ProbeOptionCache<'a>);

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
fn restore_drops_cache_entries_from_abandoned_lineage() {
  // A cached token prefilled BEFORE the save makes the checkpoint cursor equal the cache
  // front, so the cache rewind takes its no-op (cursor == front) branch and leaves the
  // cache untouched. A wider peek AFTER the save then crosses the malformed `@`, emitting
  // its lexer error and caching the tokens that follow it. Those post-save entries belong
  // to the abandoned continuation: restore rewinds the error's emission, and unless the
  // entries are dropped a later drain pops straight over the rewound error — so it is
  // never re-emitted. The token VALUES are faithfully memoized either way; only the scan
  // side effect (the error emission) is lost.
  //   1 @ 2 3
  //   0 . 4 6      (`@` spans [2, 3); high limit so only the plain lexer error is in play)
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::{U1, U3};

  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 @ 2 3",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );

  let drained: Vec<SimpleSpan> = {
    let mut inp = input.as_ref(&mut emitter);

    // Prefill exactly one cached token (`1`); the cursor now sits at its start, so the
    // save's cursor equals the cache front.
    let _ = inp.peek::<U1>().unwrap();

    // Save BEFORE the error is crossed: the checkpoint predates the `@` emission, so
    // restoring rewinds that emission.
    let ckp = inp.save();

    // Peek across the malformed `@`: emits its lexer error and caches the tokens that
    // follow it (`2`, `3`) — entries from the continuation we are about to abandon.
    let _ = inp.peek::<U3>().unwrap();

    // Abandon the continuation.
    inp.restore(ckp);

    // Drain to EOF on the committed path: it must re-lex the `@` region and re-emit the
    // error exactly once, then yield the full faithful token sequence.
    let mut toks = Vec::new();
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    toks
  };

  assert_eq!(
    drained,
    std::vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(4, 5),
      SimpleSpan::new(6, 7)
    ],
    "the drained stream is the full faithful token sequence"
  );
  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 1,
    "the rolled-back lexer error must be re-emitted exactly once after restore drops the abandoned cache entries"
  );
}

#[test]
fn restore_option_cache_capacity_one_reemits_error_once() {
  // The capacity-1 `Option` cache cannot express the abandoned-lineage hole: hitting the
  // rewind's no-op (cursor == front) branch needs a prefilled entry occupying the cache,
  // which at capacity 1 leaves no room to also cache a post-error token. The token that
  // follows the `@` overflows instead of being cached, so nothing from the abandoned
  // continuation survives the restore and the error region always re-lexes. This pins
  // that faithful behavior and guards the truncation against wrongly dropping the
  // surviving pre-save entry.
  //   1 @ 2 3      (`@` spans [2, 3))
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::{U1, U2};

  let cache: ProbeOptionCache<'_> = None;
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeOptionVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 @ 2 3",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );

  let drained: Vec<SimpleSpan> = {
    let mut inp = input.as_ref(&mut emitter);

    // Prefill the single slot with `1`; the cursor sits at its start.
    let _ = inp.peek::<U1>().unwrap();
    let ckp = inp.save();
    // Peek across `@`: the error is emitted, but `2` cannot be cached (slot full) and
    // overflows instead of surviving the restore.
    let _ = inp.peek::<U2>().unwrap();
    inp.restore(ckp);

    let mut toks = Vec::new();
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    toks
  };

  assert_eq!(
    drained,
    std::vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(4, 5),
      SimpleSpan::new(6, 7)
    ],
    "the capacity-1 cache still drains the full faithful token sequence"
  );
  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(total, 1, "the error re-lexes and is emitted exactly once");
}

#[test]
fn nested_restore_retains_pre_save_cache_entries() {
  // Nested LIFO over a prefilled cache entry. Prefill exactly one cached token, then
  // stack two saves on top of it (nothing consumed between them, so BOTH checkpoint
  // cursors equal the cache front and the rewind takes its no-op branch). Peek several
  // more tokens into the continuation the restores abandon, then restore inner and
  // restore outer. The prefilled token predates both saves, so it must survive both
  // restores and be served FROM CACHE on the drain — never re-lexed.
  //
  // The push count is per-lineage state that a restore copies back to its saved value,
  // exactly like the dedup watermark and the poison boundary. The inner restore drops
  // its post-save tail (`2`,`3`) and rewinds the count to the inner save's value; the
  // outer restore then computes zero post-save survivors and keeps the prefilled `1`.
  // At HEAD the count was never rewound, so the outer restore saw a stale-high count and
  // over-dropped `1`; re-consuming re-lexed it — the shared `ProbeLimiter` counter, which
  // observes every scan, makes that re-lex visible as a nonzero delta across one `next()`.
  //   1 2 3 4 5   (all valid Nums; a `usize::MAX` limit never trips)
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::{U1, U3};

  let limiter = ProbeLimiter::with_limit(usize::MAX);
  let scanned = limiter.counter();
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5",
    limiter,
    cache,
  );

  let drained: Vec<SimpleSpan> = {
    let mut inp = input.as_ref(&mut emitter);

    // Prefill exactly one cached token (`1`); the cursor now equals the cache front.
    let _ = inp.peek::<U1>().unwrap();
    // Stack two saves on the prefilled front.
    let outer = inp.save();
    let inner = inp.save();
    // Peek more: caches `2`,`3` — post-save entries — lifting the input-wide push count
    // above BOTH checkpoints' saved values.
    let _ = inp.peek::<U3>().unwrap();
    // Inner restore drops the post-inner tail; outer restore must NOT over-drop `1`.
    inp.restore(inner);
    inp.restore(outer);

    // (a) The prefilled `1` is served FROM CACHE: consuming it does no scan work, so the
    // shared counter is unchanged across this single `next()`. At HEAD the outer restore
    // over-drops `1`, so it re-lexes here and the counter ticks up.
    let before = scanned.get();
    let first = inp.next().unwrap().expect("first drained token");
    assert_eq!(
      *first.span_ref(),
      SimpleSpan::new(0, 1),
      "the first drained token is `1`"
    );
    assert_eq!(
      scanned.get(),
      before,
      "the pre-save cache entry must be served from cache, never re-lexed (scan counter unchanged)"
    );

    // (b) The full token stream is faithful.
    let mut toks = std::vec![*first.span_ref()];
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    toks
  };

  assert_eq!(
    drained,
    std::vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(2, 3),
      SimpleSpan::new(4, 5),
      SimpleSpan::new(6, 7),
      SimpleSpan::new(8, 9),
    ],
    "the drained stream is the full faithful token sequence"
  );
  // (c) No poison diagnostic — nor any diagnostic; the input is clean and never trips.
  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(total, 0, "a clean nested restore emits no diagnostic");
}

#[test]
fn nested_restore_with_shared_limiter_no_spurious_poison() {
  // The same nested LIFO shape, but under a SHARED-counter limiter whose budget is one
  // scan short of tolerating the over-drop's re-lex. A faithful drain serves the prefilled
  // `1` from cache (no scan) and re-lexes only the post-save `2`,`3`,`4`,`5`, reaching a
  // count of 7 — untripped at the limit of 7. The HEAD over-drop also re-lexes `1`,
  // reaching 8 and tripping the limiter on the fifth drained token: a spurious poison latch
  // and a limit diagnostic this checkpoint's lineage never produced.
  //   1 2 3 4 5   (limit 7)
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::{U1, U3};

  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5",
    ProbeLimiter::with_limit(7),
    cache,
  );

  let drained: Vec<SimpleSpan> = {
    let mut inp = input.as_ref(&mut emitter);

    let _ = inp.peek::<U1>().unwrap(); // prefill `1`
    let outer = inp.save();
    let inner = inp.save();
    let _ = inp.peek::<U3>().unwrap(); // cache post-save `2`,`3`
    inp.restore(inner);
    inp.restore(outer);

    let mut toks = Vec::new();
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    // The faithful drain reaches EOF untripped: `1` came from cache, so the extra scan
    // that would trip the limiter is never spent.
    assert!(
      !inp.is_poisoned(),
      "serving the pre-save entry from cache must not spend the extra scan that spuriously trips the limiter"
    );
    toks
  };

  assert_eq!(
    drained,
    std::vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(2, 3),
      SimpleSpan::new(4, 5),
      SimpleSpan::new(6, 7),
      SimpleSpan::new(8, 9),
    ],
    "the full faithful stream drains — no token is lost to a spurious trip"
  );
  let limit_diags = emitter
    .errors()
    .values()
    .flatten()
    .filter(|e| **e == ProbeErr::Limit)
    .count();
  assert_eq!(
    limit_diags, 0,
    "no spurious limit diagnostic — the checkpoint's lineage never tripped"
  );
}

#[test]
fn consumed_pre_save_cache_entry_relexes_identically_on_restore() {
  // A legal last-in, first-out shape over a CONSUMED pre-save cache entry. `peek(1)`
  // stages token `T`; `save` snapshots the lineage; `next()` consumes `T` FROM CACHE;
  // `restore` returns to the save. The abandoned branch already drained `T` out of the
  // cache, so the cache no longer holds it and the first post-restore read RE-LEXES `T`
  // from source. That re-lex is the architecture — a restore replays a dropped or
  // consumed cached token on demand — and by the `Lexer` determinism contract it is
  // observationally identical: the same token, the same span, the diagnostics exactly
  // once, and an in-`State` limiter recounting the same total. Only instrumentation that
  // lives OUTSIDE the lexer state (here a shared scan counter) sees the extra scan.
  //
  // This pins the current behavior: a change that snapshotted consumed cache entries to
  // skip the re-lex would alter the scan counts or the stream below and trip it.
  //   1 @ 2 3      (`@` is a lexer error spanning [2, 3); the limit never trips)
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::U1;

  const SRC: &str = "1 @ 2 3";

  // A fresh single pass over the same source: the faithful stream, the scan count, and
  // the diagnostic count the replay must reproduce.
  let (oracle_spans, oracle_scans, oracle_diags): (Vec<SimpleSpan>, usize, usize) = {
    let limiter = ProbeLimiter::with_limit(usize::MAX);
    let scanned = limiter.counter();
    let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
    let mut emitter = Verbose::<ProbeErr>::new();
    let mut input =
      Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(SRC, limiter, cache);
    let spans = {
      let mut inp = input.as_ref(&mut emitter);
      let mut toks = Vec::new();
      while let Some(t) = inp.next().unwrap() {
        toks.push(*t.span_ref());
      }
      toks
    };
    let diags: usize = emitter.errors().values().map(|group| group.len()).sum();
    (spans, scanned.get(), diags)
  };
  assert_eq!(
    oracle_spans,
    std::vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(4, 5),
      SimpleSpan::new(6, 7)
    ],
    "single-pass oracle: the faithful token stream"
  );
  assert_eq!(
    oracle_scans, 3,
    "single-pass oracle: three Nums, each scanned once"
  );
  assert_eq!(oracle_diags, 1, "single-pass oracle: the `@` error, once");

  // ── Shared observer: the consumed `T` re-lexes with exactly one extra scan. ──
  let limiter = ProbeLimiter::with_limit(usize::MAX);
  let scanned = limiter.counter();
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input =
    Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(SRC, limiter, cache);
  let drained: Vec<SimpleSpan> = {
    let mut inp = input.as_ref(&mut emitter);

    // Stage `T` (`1`): exactly one scan.
    let _ = inp.peek::<U1>().unwrap();
    assert_eq!(scanned.get(), 1, "peek(1) scans `T` exactly once");

    let ckp = inp.save();

    // Consume `T` FROM CACHE: served from the cache, so no scan runs.
    let before_consume = scanned.get();
    let consumed = inp.next().unwrap().expect("consume `T` from cache");
    assert_eq!(
      *consumed.span_ref(),
      SimpleSpan::new(0, 1),
      "the consumed token is `T`"
    );
    assert_eq!(
      scanned.get(),
      before_consume,
      "consuming `T` from cache re-scans nothing"
    );

    // Abandon the branch. `T` is no longer cached, so the next read re-lexes it.
    inp.restore(ckp);

    // The first post-restore read RE-LEXES `T`: exactly one additional scan, and only
    // the shared counter (instrumentation outside the lexer state) observes it. This is
    // expected by design — the replay re-lexes on demand.
    let before_replay = scanned.get();
    let replayed = inp.next().unwrap().expect("re-lex `T`");
    assert_eq!(
      *replayed.span_ref(),
      SimpleSpan::new(0, 1),
      "`T` re-lexes to the same span"
    );
    assert_eq!(
      scanned.get(),
      before_replay + 1,
      "re-lexing `T` is exactly one additional scan"
    );

    let mut toks = std::vec![*replayed.span_ref()];
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    toks
  };

  // (i) The drained stream is the full faithful sequence — `T` appears exactly once.
  assert_eq!(
    drained, oracle_spans,
    "the drained stream is the full faithful token sequence — `T` appears exactly once"
  );
  // (ii) Exactly one scan beyond a single pass: the consumed `T`'s replay.
  assert_eq!(
    scanned.get(),
    oracle_scans + 1,
    "the replay costs exactly one scan beyond a single pass — only outside-state instrumentation observes it"
  );
  // (iv) Diagnostics are emitted exactly once, matching a single pass.
  let diags: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    diags, oracle_diags,
    "the `@` error is emitted exactly once, as in a single pass"
  );

  // ── By-value limiter: restore rewinds its state, so the replay recounts identically. ──
  // The knife-edge budget equals the single-pass scan count. The consumed `T`'s replay
  // re-lexes the whole source once, and because `restore` rewinds the in-`State` counter
  // the recount reaches exactly that total — not one past it — so the budget never trips.
  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    SRC,
    TokenLimiter::with_limitation(3),
    cache,
  );
  let drained: Vec<SimpleSpan> = {
    let mut inp = input.as_ref(&mut emitter);
    let _ = inp.peek::<U1>().unwrap();
    let ckp = inp.save();
    let _ = inp.next().unwrap().expect("consume `T` from cache");
    inp.restore(ckp);
    let mut toks = Vec::new();
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    // The rewound limiter recounts the replay identically to a first pass, reaching the
    // knife-edge exactly.
    assert!(
      !inp.is_poisoned(),
      "the restored by-value limiter recounts the replay identically; a single-pass budget never trips"
    );
    assert_eq!(
      inp.state().tokens(),
      3,
      "the replay counts exactly the single-pass token total"
    );
    toks
  };
  assert_eq!(
    drained,
    std::vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(4, 5),
      SimpleSpan::new(6, 7)
    ],
    "the by-value replay drains the full faithful stream"
  );
  let diags: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    diags, 1,
    "the by-value replay emits the `@` error exactly once"
  );
}

#[test]
fn consume_all_cached_then_restore_replays_faithfully() {
  // The same consumed-prefix architecture, but the abandoned branch drains the WHOLE
  // cached run at once through `consume_all_cached` before the restore. Nothing pre-save
  // survives in the cache, so the committed path re-lexes the entire run on demand. By
  // the `Lexer` determinism contract the replay is faithful: the full token stream
  // returns, the lexer error is emitted exactly once, and the by-value limiter — its
  // state rewound by the restore — recounts the replay identically, so a knife-edge
  // budget equal to the single-pass token count never trips.
  //
  // This pins the current behavior: a change that snapshotted the consumed run to skip
  // the re-lex would alter the stream, the diagnostics, or the recount below and trip it.
  //   1 @ 2 3 4 5      (`@` is a lexer error spanning [2, 3))
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::U3;

  const SRC: &str = "1 @ 2 3 4 5";

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    SRC,
    // Knife-edge: the single pass scans five Nums, so a budget of five tolerates the
    // faithful replay exactly and one less would trip.
    TokenLimiter::with_limitation(5),
    cache,
  );

  let drained: Vec<SimpleSpan> = {
    let mut inp = input.as_ref(&mut emitter);

    // Fill the cache with the first three tokens (crossing and sealing `@`).
    let _ = inp.peek::<U3>().unwrap();
    let ckp = inp.save();

    // Consume the ENTIRE cached run at once; it returns the last cached token (`3`).
    let last = inp.consume_all_cached().expect("consume the cached run");
    assert_eq!(
      *last.span_ref(),
      SimpleSpan::new(6, 7),
      "consume_all_cached returns the last cached token (`3`)"
    );

    // Abandon the branch. The run is gone from the cache, so the drain re-lexes it.
    inp.restore(ckp);

    let mut toks = Vec::new();
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    // The rewound by-value limiter recounts the whole replay from scratch, reaching the
    // knife-edge exactly — re-lexing the consumed run costs no spurious trip.
    assert!(
      !inp.is_poisoned(),
      "the restored by-value limiter recounts the replay identically; the knife-edge budget never trips"
    );
    assert_eq!(
      inp.state().tokens(),
      5,
      "the replay counts exactly the single-pass token total"
    );
    toks
  };

  // The whole run re-lexes, then the tokens past it follow — the full faithful stream.
  assert_eq!(
    drained,
    std::vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(4, 5),
      SimpleSpan::new(6, 7),
      SimpleSpan::new(8, 9),
      SimpleSpan::new(10, 11),
    ],
    "the drained stream is the full faithful token sequence"
  );
  // The `@` error is emitted exactly once across peek → consume-run → restore → drain.
  let diags: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    diags, 1,
    "the `@` error is emitted exactly once, as in a single pass"
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

#[test]
fn sync_through_trip_after_skips_commits_the_diagnosed_prefix() {
  // `sync_through` scans forward diagnosing every non-matching token; if a limit trips
  // after some are skipped, the diagnosed prefix must be COMMITTED at the durable
  // frontier — the end of the last skipped token — so the boundary latches there and a
  // later scan yields the poisoned outcome at that frontier, never rewinding to the
  // pre-call cursor and stranding tokens that were already diagnosed.
  //
  // A by-value in-`State` limiter makes the commit observable: the frontier snapshots
  // the lexer state at the moment it advances over a token, so the committed count is
  // the pre-trip prefix count (2), distinct from the total scanned (3, incl. the trip
  // token). `sync_through(|_| false, ..)` matches nothing, so it skips-and-diagnoses `1`
  // and `2`, then the 3rd scanned token (`3`, span [4, 5)) trips before any target.
  //   1 2 3 4 5 6   (limit 2 → the 3rd scanned token trips; `2` ends at offset 3)
  use crate::span::SimpleSpan;

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(2),
    cache,
  );

  {
    let mut inp = input.as_ref(&mut emitter);

    // Pre-call anchor: nothing consumed yet.
    assert_eq!(inp.span(), &SimpleSpan::new(0, 0), "pre-call span anchor");
    assert_eq!(inp.state().tokens(), 0, "pre-call token count");

    // No target is ever matched: the scan diagnoses `1` and `2`, then trips on `3`.
    assert!(
      inp.sync_through(|_| false, || None).unwrap().is_none(),
      "the trip yields the poisoned outcome — no matching token"
    );

    // (i) The diagnosed prefix is COMMITTED at the durable frontier (end of `2`,
    // offset 3), NOT stranded at the stale pre-call anchor (offset 0) that the old
    // `AtCursor` policy left behind. At HEAD this span was still `[0, 0)` and the
    // token count still 0 — the regression this test pins.
    assert!(inp.is_poisoned(), "the trip latches the poison boundary");
    assert_eq!(
      inp.span(),
      &SimpleSpan::new(2, 3),
      "committed span sits at the end of the last diagnosed token (`2`)"
    );
    assert_eq!(
      inp.state().tokens(),
      2,
      "committed state counts exactly the diagnosed prefix (`1`, `2`) — not the trip token"
    );

    // (ii) A subsequent `next()` yields the poisoned outcome AT that boundary without
    // rescanning the diagnosed tokens: the committed by-value counter stays frozen at 2
    // (no re-lex of `1`/`2`) and nothing past the boundary is scanned.
    assert!(
      inp.next().unwrap().is_none(),
      "next() stops at the committed boundary"
    );
    assert_eq!(
      inp.state().tokens(),
      2,
      "the committed lineage's scan counter is frozen — `1`/`2` are not rescanned"
    );
  }

  // (iii) Each skipped token is diagnosed exactly once (`1`, `2` → two unexpected-token
  // errors) and the limit trip exactly once (`3` → one limit error).
  let unexpected = emitter
    .errors()
    .values()
    .flatten()
    .filter(|e| **e == ByValErr::Lex)
    .count();
  let limit = emitter
    .errors()
    .values()
    .flatten()
    .filter(|e| **e == ByValErr::Limit)
    .count();
  assert_eq!(
    unexpected, 2,
    "each skipped token is diagnosed exactly once (`1`, `2`)"
  );
  assert_eq!(limit, 1, "the limit trip is diagnosed exactly once (`3`)");
}

#[test]
fn sync_through_then_peek_trip_after_skips_commits_the_diagnosed_prefix() {
  // The twin of `sync_through_trip_after_skips_commits_the_diagnosed_prefix` for the
  // separately-reachable `sync_through_then_peek` loop: the same commit-at-the-frontier
  // behavior must hold there too. `sync_through_then_peek(|_| false, ..)` matches
  // nothing, diagnoses `1` and `2`, then trips on `3` and returns no matched token and
  // an empty peek — committing the diagnosed prefix at the durable frontier (offset 3).
  //   1 2 3 4 5 6   (limit 2 → the 3rd scanned token trips; `2` ends at offset 3)
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::U1;

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(2),
    cache,
  );

  {
    let mut inp = input.as_ref(&mut emitter);

    let (matched, peeked) = inp
      .sync_through_then_peek::<_, _, U1>(|_| false, || None)
      .unwrap();
    assert!(matched.is_none(), "the trip yields no matched token");
    assert!(peeked.is_empty(), "the trip yields an empty peek");
    // `peeked` borrows `inp` for its lifetime; release it before reusing `inp`.
    drop(peeked);

    // The diagnosed prefix is committed at the durable frontier (end of `2`, offset 3),
    // not stranded at the pre-call anchor.
    assert!(inp.is_poisoned(), "the trip latches the poison boundary");
    assert_eq!(
      inp.span(),
      &SimpleSpan::new(2, 3),
      "committed span sits at the end of the last diagnosed token (`2`)"
    );
    assert_eq!(
      inp.state().tokens(),
      2,
      "committed state counts exactly the diagnosed prefix (`1`, `2`)"
    );

    // A subsequent `next()` stops at that boundary without rescanning `1`/`2`.
    assert!(
      inp.next().unwrap().is_none(),
      "next() stops at the committed boundary"
    );
    assert_eq!(
      inp.state().tokens(),
      2,
      "the committed lineage's scan counter is frozen"
    );
  }

  let unexpected = emitter
    .errors()
    .values()
    .flatten()
    .filter(|e| **e == ByValErr::Lex)
    .count();
  let limit = emitter
    .errors()
    .values()
    .flatten()
    .filter(|e| **e == ByValErr::Limit)
    .count();
  assert_eq!(
    unexpected, 2,
    "each skipped token is diagnosed exactly once (`1`, `2`)"
  );
  assert_eq!(limit, 1, "the limit trip is diagnosed exactly once (`3`)");
}

#[test]
fn failed_sync_through_leaves_no_diagnostics() {
  // A `sync_through` whose predicate never matches scans every valid token to EOF,
  // diagnosing each as unexpected, then takes the no-match EOF exit — which commits
  // nothing: the cursor stays at the pre-call anchor so the caller can fall back from
  // the original position. Diagnostics travel with progress, so a path that commits
  // nothing leaves no trace; the tokens the caller then consumes normally must carry
  // no stale unexpected-token noise.
  //   1 2 3   (high limit: the scan reaches EOF and never trips)
  use crate::span::SimpleSpan;

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3",
    TokenLimiter::with_limitation(usize::MAX),
    cache,
  );

  let drained: Vec<SimpleSpan> = {
    let mut inp = input.as_ref(&mut emitter);

    // Pre-call anchor: nothing consumed.
    assert_eq!(inp.span(), &SimpleSpan::new(0, 0), "pre-call span anchor");
    assert_eq!(inp.state().tokens(), 0, "pre-call token count");

    // Never matches: scans `1`, `2`, `3`, reaches EOF with no target.
    assert!(
      inp.sync_through(|_| false, || None).unwrap().is_none(),
      "the no-match scan to EOF yields None"
    );

    // The no-match EOF path commits nothing: cursor/span/state stay at the pre-call
    // anchor so the caller can fall back from the original position.
    assert_eq!(
      inp.span(),
      &SimpleSpan::new(0, 0),
      "span stays at the pre-call anchor — the failed sync commits no progress"
    );
    assert_eq!(
      inp.state().tokens(),
      0,
      "state stays at the pre-call count — the failed sync commits no progress"
    );

    // A subsequent drain consumes every token normally.
    let mut toks = Vec::new();
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    toks
  };

  assert_eq!(
    drained,
    std::vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(2, 3),
      SimpleSpan::new(4, 5)
    ],
    "the drain consumes the full token sequence normally"
  );

  // The failed sync left no diagnostics, and a normal drain of valid tokens emits
  // none. At HEAD the failed sync retained one unexpected-token diagnostic per scanned
  // token (three) — the stale, misleading noise this fix removes.
  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 0,
    "a failed sync_through leaves no diagnostics behind"
  );
}

#[test]
fn successful_sync_through_retains_skipped_token_diagnostics() {
  // The commit side of the rule: a `sync_through` that DOES match commits through the
  // target, so the unexpected-token diagnostics it emitted for the tokens it skipped on
  // the way persist — they describe real, committed progress.
  //   1 2 3   (match the third scanned token; `1` and `2` are skipped and diagnosed)
  use crate::span::SimpleSpan;

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3",
    TokenLimiter::with_limitation(usize::MAX),
    cache,
  );

  {
    let mut inp = input.as_ref(&mut emitter);

    // Match only the third scanned token (`3`); skip `1` and `2`, diagnosing each.
    let mut seen = 0;
    let matched = inp
      .sync_through(
        |_| {
          seen += 1;
          seen == 3
        },
        || None,
      )
      .unwrap();
    assert_eq!(
      matched.map(|t| *t.span_ref()),
      Some(SimpleSpan::new(4, 5)),
      "the matching token `3` is consumed and returned"
    );
    // The match commits through the skipped prefix: the cursor advances to the end of
    // `3`, and the two skipped tokens' diagnostics describe that committed progress.
    assert_eq!(inp.span(), &SimpleSpan::new(4, 5), "committed at the match");
  }

  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 2,
    "the skipped `1` and `2` stay diagnosed — the match committed through them"
  );
}

#[test]
fn failed_sync_through_reemits_scanned_lexer_error_once() {
  use crate::span::SimpleSpan;
  // A `sync_through` whose predicate never matches scans a region containing a lexer
  // error (`@`) to EOF. Crossing `@` emits it and lifts the dedup watermark past it; the
  // no-match EOF path commits nothing, so it unwinds this call's emissions AND restores
  // the watermark to its entry value. The error is therefore neither retained (the path
  // committed nothing) nor lost: the genuine consume that follows re-crosses `@` and
  // re-emits it exactly once. Without the watermark restore the rewound error would stay
  // watermark-covered and be silently deduplicated away on the re-scan (emitted zero
  // times); at HEAD the failed sync instead retains it plus two stale unexpected tokens.
  //   1 @ 2   (`@` is a lexer error spanning [2, 3); high limit so no trip)
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 @ 2",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    // Never matches: skips `1`, crosses `@` (emitting the lexer error and lifting the
    // watermark past it), skips `2`, reaches EOF. The no-match EOF path unwinds every
    // emission and restores the watermark.
    assert!(
      inp.sync_through(|_| false, || None).unwrap().is_none(),
      "the no-match scan to EOF yields None"
    );

    // With the watermark restored, the genuine consume re-crosses `@` and re-emits.
    while inp.next().unwrap().is_some() {}
  }

  // Exactly the one `@` lexer error is retained: the failed sync left no trace, and the
  // genuine consume re-emitted the error exactly once.
  let at = SimpleSpan::new(2, 3);
  assert_eq!(
    emitter
      .errors()
      .get(&at)
      .map(|group| group.len())
      .unwrap_or(0),
    1,
    "the scanned-past lexer error re-emits exactly once on the genuine consume"
  );
  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 1,
    "only the re-emitted lexer error is retained — no stale unexpected-token noise"
  );
}

#[test]
fn failed_sync_through_then_peek_leaves_no_diagnostics_and_position() {
  // The peek-variant sibling of `failed_sync_through_leaves_no_diagnostics`: a
  // `sync_through_then_peek` whose predicate never matches scans every valid token to EOF,
  // diagnosing each as unexpected, then takes the no-match EOF exit. That exit commits
  // nothing — the cursor stays at the pre-call anchor, the peek is empty, and the failed
  // scan's diagnostics are unwound — so a caller using the peek variant for recovery or
  // lookahead keeps the original position and carries no stale noise.
  //   1 2 3   (high limit: the scan reaches EOF and never trips)
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::U1;

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3",
    TokenLimiter::with_limitation(usize::MAX),
    cache,
  );

  let drained: Vec<SimpleSpan> = {
    let mut inp = input.as_ref(&mut emitter);

    // Pre-call anchor: nothing consumed.
    assert_eq!(inp.span(), &SimpleSpan::new(0, 0), "pre-call span anchor");
    assert_eq!(inp.state().tokens(), 0, "pre-call token count");

    // Never matches: scans `1`, `2`, `3`, reaches EOF with no target.
    let (matched, peeked) = inp
      .sync_through_then_peek::<_, _, U1>(|_| false, || None)
      .unwrap();
    assert!(
      matched.is_none(),
      "the no-match scan to EOF yields no token"
    );
    assert!(
      peeked.is_empty(),
      "the no-match scan to EOF yields an empty peek"
    );
    // `peeked` borrows `inp` for its lifetime; release it before reusing `inp`.
    drop(peeked);

    // The no-match EOF path commits nothing: cursor/span/state stay at the pre-call anchor
    // so the caller can fall back from the original position. At HEAD this loop instead
    // advanced span/state to the lexer EOF.
    assert_eq!(
      inp.span(),
      &SimpleSpan::new(0, 0),
      "span stays at the pre-call anchor — the failed peek-sync commits no progress"
    );
    assert_eq!(
      inp.state().tokens(),
      0,
      "state stays at the pre-call count — the failed peek-sync commits no progress"
    );

    // A subsequent drain consumes every token normally.
    let mut toks = Vec::new();
    while let Some(t) = inp.next().unwrap() {
      toks.push(*t.span_ref());
    }
    toks
  };

  assert_eq!(
    drained,
    std::vec![
      SimpleSpan::new(0, 1),
      SimpleSpan::new(2, 3),
      SimpleSpan::new(4, 5)
    ],
    "the drain consumes the full token sequence normally"
  );

  // The failed peek-sync left no diagnostics, and a normal drain of valid tokens emits
  // none. At HEAD the failed peek-sync retained one unexpected-token diagnostic per scanned
  // token (three) — the stale, misleading noise this fix removes.
  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 0,
    "a failed sync_through_then_peek leaves no diagnostics behind"
  );
}

#[test]
fn failed_sync_through_then_peek_reemits_crossed_lexer_error_once() {
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::U1;
  // The peek-variant sibling of `failed_sync_through_reemits_scanned_lexer_error_once`: a
  // `sync_through_then_peek` whose predicate never matches scans a region containing a lexer
  // error (`@`) to EOF. Crossing `@` emits it and lifts the dedup watermark past it; the
  // no-match EOF path commits nothing, so it unwinds this call's emissions AND restores the
  // watermark to its entry value. The error is therefore neither retained (the path
  // committed nothing) nor lost: the genuine consume that follows re-crosses `@` and
  // re-emits it exactly once. Without the watermark restore the rewound error would stay
  // watermark-covered and be silently deduplicated away on the re-scan (emitted zero times);
  // at HEAD the failed peek-sync instead retains it plus two stale unexpected tokens.
  //   1 @ 2   (`@` is a lexer error spanning [2, 3); high limit so no trip)
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 @ 2",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    // Never matches: skips `1`, crosses `@` (emitting the lexer error and lifting the
    // watermark past it), skips `2`, reaches EOF. The no-match EOF path unwinds every
    // emission and restores the watermark.
    let (matched, peeked) = inp
      .sync_through_then_peek::<_, _, U1>(|_| false, || None)
      .unwrap();
    assert!(
      matched.is_none(),
      "the no-match scan to EOF yields no token"
    );
    assert!(
      peeked.is_empty(),
      "the no-match scan to EOF yields an empty peek"
    );
    drop(peeked);

    // With the watermark restored, the genuine consume re-crosses `@` and re-emits.
    while inp.next().unwrap().is_some() {}
  }

  // Exactly the one `@` lexer error is retained: the failed peek-sync left no trace, and the
  // genuine consume re-emitted the error exactly once.
  let at = SimpleSpan::new(2, 3);
  assert_eq!(
    emitter
      .errors()
      .get(&at)
      .map(|group| group.len())
      .unwrap_or(0),
    1,
    "the scanned-past lexer error re-emits exactly once on the genuine consume"
  );
  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 1,
    "only the re-emitted lexer error is retained — no stale unexpected-token noise"
  );
}

#[test]
fn successful_sync_through_then_peek_retains_skipped_token_diagnostics() {
  // The commit side of the rule for the peek variant: a `sync_through_then_peek` that DOES
  // match commits through the target, so the unexpected-token diagnostics it emitted for the
  // tokens it skipped on the way persist — they describe real, committed progress. This pins
  // the no-trace change to the failure path only.
  //   1 2 3   (match the third scanned token; `1` and `2` are skipped and diagnosed)
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::U1;

  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3",
    TokenLimiter::with_limitation(usize::MAX),
    cache,
  );

  {
    let mut inp = input.as_ref(&mut emitter);

    // Match only the third scanned token (`3`); skip `1` and `2`, diagnosing each.
    let mut seen = 0;
    let (matched, peeked) = inp
      .sync_through_then_peek::<_, _, U1>(
        |_| {
          seen += 1;
          seen == 3
        },
        || None,
      )
      .unwrap();
    assert_eq!(
      matched.map(|t| *t.span_ref()),
      Some(SimpleSpan::new(4, 5)),
      "the matching token `3` is consumed and returned"
    );
    assert!(
      peeked.is_empty(),
      "`3` is the last token, so the peek after the match is empty"
    );
    drop(peeked);
    // The match commits through the skipped prefix: the cursor advances to the end of `3`,
    // and the two skipped tokens' diagnostics describe that committed progress.
    assert_eq!(inp.span(), &SimpleSpan::new(4, 5), "committed at the match");
  }

  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 2,
    "the skipped `1` and `2` stay diagnosed — the match committed through them"
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
    let mut r1 = in1.as_ref(&mut em1);
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
    let mut r = original.as_ref(&mut em1);
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
        match roll(&mut rng) % 6 {
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
          3 => {
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
          4 => {
            // The exact class the cache-lineage hole lived in: prefill the cache, save on
            // top of it (so the checkpoint cursor equals the cache front and the rewind
            // takes its no-op branch), then peek across a lexer-error character so
            // post-error tokens are cached into the continuation this save may later
            // abandon. The checkpoint joins `live`, so a subsequent restore op exercises
            // the post-save truncation over a cache that straddles a rolled-back error.
            let _ = inp.peek::<U1>().unwrap();
            let off = *inp.cursor().as_inner();
            live.push((inp.save(), off));
            let _ = inp.peek::<U3>().unwrap();
          }
          _ => {
            // A self-contained NESTED last-in, first-out restore over a prefilled cache entry
            // — the (save, save, peek, restore, restore) shape the input-wide push count made
            // unsound. Prefill one token so BOTH saves' cursors equal the cache front, stack
            // two saves, widen the peek to cache post-save entries, then restore inner and
            // restore outer. The prefilled entry predates both saves, so the drained token must
            // match a fresh parse of the same prefix — the outer restore must not over-drop it
            // (a stale push count did, re-lexing a token whose scan side effects belonged to the
            // abandoned lineage). Both checkpoints are resolved here and never join `live`, so
            // the other ops' LIFO discipline is untouched.
            let _ = inp.peek::<U1>().unwrap();
            let off = *inp.cursor().as_inner();
            let outer = inp.save();
            let inner = inp.save();
            let _ = inp.peek::<U3>().unwrap();
            inp.restore(inner);
            inp.restore(outer);
            assert_eq!(
              inp.next().unwrap().map(|t| *t.span_ref()),
              oracle_next(off),
              "a nested LIFO restore must retain the pre-save cache entry (matches a fresh parse)"
            );
          }
        }
      }

      // Final commit drain to EOF: a full pass from the current position crosses every
      // remaining error region. A rolled-back error that a stale post-save cache entry let
      // a drain skip would re-lex to ZERO emissions here, so together with the soundness
      // checks below this pins exactly-once COMPLETENESS — the direction the hole broke.
      drop(live);
      while inp.next().unwrap().is_some() {}
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

    // (c) after the final full drain to EOF, every real error span is retained exactly
    // once. With (b) this is exactly-once completeness on the committed lineage: a stale
    // post-save cache entry that skipped a rolled-back error would drop it to zero here.
    for diag in &oracle_diags {
      assert_eq!(
        em.errors().get(diag).map(|g| g.len()).unwrap_or(0),
        1,
        "after a full final drain, the error at {diag:?} must be retained exactly once"
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

// ── attempt / try_attempt: raw restore below the checkpoint panics AT THE RESTORE ─
//
// The closure receives `&mut InputRef` and can raw-restore to a checkpoint saved BEFORE the
// attempt began. `attempt` PINS its held checkpoint on entry, so such a restore — which would
// pop that pinned checkpoint off the live lineage — panics AT THE RESTORE, inside the closure,
// in every allocator build. A LIFO-clean raw pair taken and released above the attempt's own
// checkpoint is unaffected. (The former detect-at-use behavior — a stale panic in the decline
// arm — is now an unreachable backstop in allocator builds.)

#[test]
#[should_panic(
  expected = "restore would invalidate a live transaction guard or attempt (the target predates its begin point)"
)]
fn attempt_inner_raw_restore_below_checkpoint_panics_at_restore() {
  // Converted from `attempt_rollback_after_inner_raw_restore_below_checkpoint`. Inside the
  // attempt, raw-restore to a checkpoint older than the attempt's own. The attempt pins its
  // checkpoint on entry, so the raw restore panics AT THE RESTORE. At HEAD the raw restore
  // succeeded and the decline's rollback arm panicked as stale ("attempt checkpoint is
  // stale"); post-fix the pinned restore panics first.
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Silent::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );
  let mut inp = input.as_ref(&mut emitter);

  let a = inp.save(); // raw checkpoint, below the attempt's checkpoint
  let _ = inp.next().unwrap().expect("consume 1");

  let _out: Option<()> = inp.attempt(|inp| {
    let _ = inp.next().unwrap().expect("consume 2");
    inp.restore(a); // POST-FIX: panics here — restoring A would pop the attempt's pinned checkpoint
    None
  });
}

#[test]
#[should_panic(
  expected = "restore would invalidate a live transaction guard or attempt (the target predates its begin point)"
)]
fn try_attempt_inner_raw_restore_below_checkpoint_panics_at_restore() {
  // The `try_attempt` twin of the attempt test: the pinned restore panics inside the closure.
  // Converted from `try_attempt_err_after_inner_raw_restore_below_checkpoint`.
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Silent::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );
  let mut inp = input.as_ref(&mut emitter);

  let a = inp.save();
  let _ = inp.next().unwrap().expect("consume 1");

  let _out: Result<(), ()> = inp.try_attempt(|inp| {
    let _ = inp.next().unwrap().expect("consume 2");
    inp.restore(a); // POST-FIX: panics here
    Err(())
  });
}

#[test]
fn attempt_inner_lifo_clean_raw_pair_is_legal() {
  // Negative control: a raw save/restore pair taken and released entirely inside the attempt
  // (ABOVE the attempt's own pinned checkpoint) is LIFO-legal and must NOT trip the pin — the
  // attempt's checkpoint sits below it and is never popped. The attempt keeps its progress.
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Silent::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );
  let mut inp = input.as_ref(&mut emitter);

  let _ = inp.next().unwrap().expect("consume 1");
  let out: Option<u32> = inp.attempt(|inp| {
    let c = inp.save(); // raw checkpoint ABOVE the attempt's checkpoint
    let _ = inp.next().unwrap().expect("consume 2");
    inp.restore(c); // legal (LIFO): pops only c — the attempt's pinned checkpoint stays live
    Some(7)
  });
  assert_eq!(
    out,
    Some(7),
    "the attempt kept its progress after the legal inner raw pair"
  );
}

// ── A hand-rolled lexer that yields a ZERO-WIDTH token span ────────────────────
//
// The bundled Logos backend never produces an empty span, but the `Lexer` trait
// permits hand-written lexers that do. A zero-width token sitting at the poison
// boundary is excluded by the positional gate yet advances nothing, silently
// breaking replay and termination — so the contract forbids it and the single
// lexing chokepoint (`lex_within_boundary`) debug-asserts against it. This fixture
// yields one `[0, 0)` token to drive that assert.

#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq)]
struct ZeroWidthErr;

#[cfg(debug_assertions)]
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>>
  for ZeroWidthErr
{
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    ZeroWidthErr
  }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ZeroWidthTok;

#[cfg(debug_assertions)]
impl core::fmt::Display for ZeroWidthTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "zero-width")
  }
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ZeroWidthKind;

#[cfg(debug_assertions)]
impl core::fmt::Display for ZeroWidthKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "zero-width")
  }
}

#[cfg(debug_assertions)]
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
#[cfg(debug_assertions)]
struct ZeroWidthLexer<'inp> {
  src: &'inp str,
  state: (),
  yielded: bool,
}

#[cfg(debug_assertions)]
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

#[cfg(debug_assertions)]
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

// ── State surgery re-keys the cache, watermark, and poison boundary ─────────────
//
// `set_state`/`state_mut` document that replacing the lexer state re-keys every
// forward-scanning fact — the token cache, the lexer-error dedup watermark, and the
// poison boundary. These four pin that the re-key actually happens on BOTH public
// state-surgery APIs, keyed to the current committed cursor, and that it re-homes offset
// facts without rewriting emission history. (The re-key governs FORWARD scanning and is
// itself transactional — a checkpoint saved before the surgery restores across it,
// undoing it; the transaction/stacked guard suites pin that.)

#[test]
fn set_state_after_limit_trip_resumes_scanning() {
  // Trip a by-value limiter, then replace the state with a fresh, non-tripped one. The
  // re-key drops the poison boundary, so scanning resumes PAST the old boundary and the
  // stream completes; the old regime's limit diagnostic stays in the log exactly once (the
  // re-key re-homes offset facts, it never rewrites history that described a real event).
  //   1 2 3 4 5 6   (limit 2 → the 3rd scanned token trips; `2` ends at the frontier offset 3)
  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(2),
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    // Drive `next()` past the trip: `1` and `2` consume, the 3rd scan trips → None.
    assert!(inp.next().unwrap().is_some(), "first token");
    assert!(inp.next().unwrap().is_some(), "second token");
    assert!(
      inp.next().unwrap().is_none(),
      "the limit trip latches to None"
    );
    assert!(
      inp.is_poisoned(),
      "the limit trip latched the poison boundary"
    );

    // Replace the state with a fresh, non-tripped limiter — the documented limit-recovery
    // path. The re-key drops the poison boundary.
    inp.set_state(TokenLimiter::with_limitation(usize::MAX));
    assert!(
      !inp.is_poisoned(),
      "state replacement re-keys the poison boundary away (HEAD leaves it latched)"
    );

    // Scanning now resumes PAST the old boundary and the stream completes. At HEAD the
    // stale boundary made this first `next()` return `None` at the old frontier.
    let mut resumed = 0usize;
    while inp.next().unwrap().is_some() {
      resumed += 1;
    }
    assert_eq!(
      resumed, 4,
      "scans past the old boundary: 3, 4, 5, 6 all lex under the fresh state"
    );
  }

  // The old regime's limit diagnostic remains exactly once.
  let limit = emitter
    .errors()
    .values()
    .flatten()
    .filter(|e| **e == ByValErr::Limit)
    .count();
  assert_eq!(
    limit, 1,
    "the pre-replacement limit diagnostic stays in the log exactly once"
  );
}

#[test]
fn state_mut_applies_the_same_rekey() {
  // The `state_mut` twin of `set_state_after_limit_trip_resumes_scanning`: taking the
  // state mutably applies the same EAGER re-key, so resetting the tripped limiter through
  // the returned `&mut` resumes scanning past the old boundary.
  //   1 2 3 4 5 6   (limit 2 → the 3rd scanned token trips; `2` ends at the frontier offset 3)
  let cache = DefaultCache::<'_, ByValLexer<'_>>::default();
  let mut emitter = Verbose::<ByValErr>::new();
  let mut input = Input::<ByValLexer<'_>, ByValVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(2),
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    assert!(inp.next().unwrap().is_some(), "first token");
    assert!(inp.next().unwrap().is_some(), "second token");
    assert!(
      inp.next().unwrap().is_none(),
      "the limit trip latches to None"
    );
    assert!(
      inp.is_poisoned(),
      "the limit trip latched the poison boundary"
    );

    // `state_mut` re-keys EAGERLY on the call — before any mutation through the returned
    // `&mut`. Dropping the borrow without touching it already cleared the poison boundary.
    // At HEAD `state_mut` only cleared the checkpoint lineage, leaving the boundary latched.
    let _ = inp.state_mut();
    assert!(
      !inp.is_poisoned(),
      "state_mut eagerly re-keys the poison boundary away, before any mutation"
    );

    // Now reset the tripped limiter in place so scanning resumes under a fresh budget.
    *inp.state_mut() = TokenLimiter::with_limitation(usize::MAX);

    let mut resumed = 0usize;
    while inp.next().unwrap().is_some() {
      resumed += 1;
    }
    assert_eq!(
      resumed, 4,
      "the state_mut re-key lets scanning resume past the old boundary"
    );
  }

  let limit = emitter
    .errors()
    .values()
    .flatten()
    .filter(|e| **e == ByValErr::Limit)
    .count();
  assert_eq!(
    limit, 1,
    "the pre-replacement limit diagnostic stays in the log exactly once"
  );
}

#[test]
fn set_state_clears_stale_cache() {
  // Peek fills the cache under the old state; replacing the state must clear it, so the
  // next read RE-LEXES from the cursor instead of serving a dead cached token. The shared
  // scan counter makes the re-lex observable — at HEAD the cached token is served and the
  // counter does not move.
  //   1 2 3 4   (high limit: never trips; the point is the cache clear)
  use crate::span::SimpleSpan;
  use generic_arraydeque::typenum::U3;

  let limiter = ProbeLimiter::with_limit(usize::MAX);
  let scanned = limiter.counter();
  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Silent::<ProbeErr>::new();
  let mut input =
    Input::<ProbeLexer<'_>, ProbeCtx<'_>, ()>::with_state_and_cache("1 2 3 4", limiter, cache);
  let mut inp = input.as_ref(&mut emitter);

  // Peek three tokens: scans `1`, `2`, `3` into the cache.
  let _ = inp.peek::<U3>().unwrap();
  assert_eq!(scanned.get(), 3, "peek scanned 1, 2, 3 into the cache");

  // Replace the state with a fresh limiter that SHARES the same scan counter, so a re-lex
  // under the new state stays observable through `scanned`. The re-key empties the cache.
  inp.set_state(ProbeLimiter {
    scanned: scanned.clone(),
    limit: usize::MAX,
  });
  assert!(
    inp.cache().is_empty(),
    "state replacement clears the token cache (HEAD leaves the stale entries)"
  );

  // The next read re-lexes from the cursor — the shared counter climbs because the dead
  // cache no longer serves the token. At HEAD the counter would stay frozen at 3.
  let before = scanned.get();
  let tok = inp.next().unwrap().expect("re-lexed token");
  assert_eq!(
    *tok.span_ref(),
    SimpleSpan::new(0, 1),
    "the re-lex resumes from the cursor (token `1`)"
  );
  assert_eq!(
    scanned.get(),
    before + 1,
    "the token was re-scanned under the new state, not served from the dead cache"
  );
}

#[test]
fn set_state_resets_watermark_to_cursor() {
  // Peek across a malformed `@`: seals its lexer error and lifts the dedup watermark past
  // it. Replacing the state re-keys the watermark back to the committed cursor (behind the
  // error) AND clears the cache holding the tokens that skipped `@`, so draining re-lexes
  // the region under the NEW regime and the error reports AGAIN — one entry per regime (the
  // documented peek-ahead-speculation edge). At HEAD the stale cache/watermark suppress the
  // second report, so only one entry survives.
  //   1 @ 2 3
  //   0 2 4 6      (`@` spans [2, 3))
  use generic_arraydeque::typenum::U2;

  let cache = DefaultCache::<'_, ProbeLexer<'_>>::default();
  let mut emitter = Verbose::<ProbeErr>::new();
  let mut input = Input::<ProbeLexer<'_>, ProbeVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 @ 2 3",
    ProbeLimiter::with_limit(usize::MAX),
    cache,
  );

  {
    let mut inp = input.as_ref(&mut emitter);

    // Peek a window crossing `@`: seals its lexer error and lifts the watermark past
    // [2, 3). The cursor stays at 0 and the cache holds the valid tokens that skipped `@`.
    let _ = inp.peek::<U2>().unwrap();

    // Replace the state: the re-key resets the watermark to the committed cursor (0) and
    // clears the cache.
    inp.set_state(ProbeLimiter::with_limit(usize::MAX));

    // Drain: the cleared cache forces a re-lex from the cursor across `@`, and the reset
    // watermark lets the re-lexed error report again under the new regime.
    while inp.next().unwrap().is_some() {}
  }

  let total: usize = emitter.errors().values().map(|group| group.len()).sum();
  assert_eq!(
    total, 2,
    "the `@` lexer error reports once per regime: at the peek, then again after the re-key re-lexes it"
  );
}
