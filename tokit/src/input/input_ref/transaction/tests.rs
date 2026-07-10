//! Tests for the [`Transaction`](super::Transaction) guard.
//!
//! `begin` saves one checkpoint and wraps the input; `commit` keeps the parsed
//! progress, `rollback` returns to the begin point, and dropping an undecided guard
//! rolls back. Nested guards borrow their parent, so the last-in, first-out discipline
//! holds at compile time (see the `compile_fail` doctest on the type).

use crate::{
  InputRef, Token,
  cache::DefaultCache,
  emitter::{Silent, Verbose},
  error::token::UnexpectedToken,
  input::Input,
  lexer::LogosLexer,
  span::SimpleSpan,
  state::token_tracker::{TokenLimitExceeded, TokenLimiter},
};

// ── Fixture: a number lexer over a by-value token limiter ──────────────────────
//
// A by-value `TokenLimiter` (checkpointed and restored with the lexer state) is what
// makes a rolled-back limit trip re-tripable: an overflow peek never writes its
// temporary lexer's counter back, so a checkpoint taken before the trip saves a clean
// count and the committed path re-lexes and re-trips from scratch. `@` matches no rule,
// so it is a plain lexer error between numbers.

#[derive(Debug, Clone, PartialEq)]
enum NumErr {
  Lex,
  Limit,
}

impl From<()> for NumErr {
  fn from(_: ()) -> Self {
    NumErr::Lex
  }
}

impl From<TokenLimitExceeded> for NumErr {
  fn from(_: TokenLimitExceeded) -> Self {
    NumErr::Limit
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for NumErr {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    NumErr::Lex
  }
}

#[derive(Debug, Clone, PartialEq, crate::logos::Logos)]
#[logos(crate = crate::logos, extras = TokenLimiter, skip r"[ \t\r\n]+")]
enum NumTok {
  #[regex(r"[0-9]+", |lex| { lex.extras.increase(); })]
  Num,
}

impl core::fmt::Display for NumTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "number")
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NumKind {
  Num,
}

impl core::fmt::Display for NumKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "number")
  }
}

impl Token<'_> for NumTok {
  type Kind = NumKind;
  type Error = NumErr;

  fn kind(&self) -> NumKind {
    NumKind::Num
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

type NumLexer<'a> = LogosLexer<'a, NumTok>;
type NumCtx<'a> = (Silent<NumErr>, DefaultCache<'a, NumLexer<'a>>);
type NumVerboseCtx<'a> = (Verbose<NumErr>, DefaultCache<'a, NumLexer<'a>>);

/// Builds a `Silent` input over `src` with a limit high enough never to trip.
fn silent_input(src: &str) -> Input<'_, NumLexer<'_>, NumCtx<'_>, ()> {
  let cache = DefaultCache::<'_, NumLexer<'_>>::default();
  Input::<NumLexer<'_>, NumCtx<'_>, ()>::with_state_and_cache(
    src,
    TokenLimiter::with_limitation(usize::MAX),
    cache,
  )
}

// ── begin/commit/rollback ──────────────────────────────────────────────────────

#[test]
fn txn_commit_keeps_progress() {
  // begin, consume two tokens through the guard, commit: the progress sticks, so the
  // next token is the third.
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let start = *inp.cursor().as_inner();
  let mut txn = inp.begin();
  let _ = txn.next().unwrap().expect("first token");
  let _ = txn.next().unwrap().expect("second token");
  txn.commit();

  assert!(
    *inp.cursor().as_inner() > start,
    "commit keeps progress — the cursor advanced past the consumed tokens"
  );
  assert_eq!(
    *inp.next().unwrap().expect("third token").span_ref(),
    SimpleSpan::new(4, 5)
  );
}

#[test]
fn txn_rollback_restores_everything() {
  // ── position, span, lexer state, emission log, and the dedup watermark ─────────
  // "1 @ 2": crossing the malformed `@` through the guard emits its lexer error and
  // lifts the watermark. `rollback` must return every one of those.
  {
    let cache = DefaultCache::<'_, NumLexer<'_>>::default();
    let mut emitter = Verbose::<NumErr>::new();
    let mut input = Input::<NumLexer<'_>, NumVerboseCtx<'_>, ()>::with_state_and_cache(
      "1 @ 2",
      TokenLimiter::with_limitation(usize::MAX),
      cache,
    );

    {
      let mut inp = input.as_ref(&mut emitter);

      let cur0 = *inp.cursor().as_inner();
      let span0 = *inp.span();
      let tokens0 = inp.state().tokens();

      let mut txn = inp.begin();
      // Consume `1`, cross `@` (emits the lexer error, lifts the watermark), consume
      // `2`, then abandon the branch.
      while txn.next().unwrap().is_some() {}
      txn.rollback();

      assert_eq!(*inp.cursor().as_inner(), cur0, "position rolled back");
      assert_eq!(*inp.span(), span0, "last-consumed span rolled back");
      assert_eq!(inp.state().tokens(), tokens0, "lexer state rolled back");
    }

    // The emission log was truncated by the rollback: nothing the guard emitted survives.
    let after_rollback: usize = emitter.errors().values().map(|g| g.len()).sum();
    assert_eq!(
      after_rollback, 0,
      "diagnostics emitted inside the transaction are rolled back (empty emission log)"
    );

    // The watermark rolled back too, so the committed path re-crosses `@` and the
    // rewound lexer error becomes re-emittable — exactly once.
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
  // An overflow peek inside the transaction trips the limiter (latching poison and
  // emitting the diagnostic); `rollback` un-latches it, and the committed path re-trips
  // — the diagnostic surviving exactly once, never a diagnostic-less latch.
  {
    use generic_arraydeque::typenum::U6;
    let cache = DefaultCache::<'_, NumLexer<'_>>::default();
    let mut emitter = Verbose::<NumErr>::new();
    let mut input = Input::<NumLexer<'_>, NumVerboseCtx<'_>, ()>::with_state_and_cache(
      "1 2 3 4 5 6",
      TokenLimiter::with_limitation(5),
      cache,
    );
    {
      let mut inp = input.as_ref(&mut emitter);

      let mut txn = inp.begin();
      let _ = txn.peek::<U6>().unwrap(); // overflow trip: poison + diagnostic
      assert!(
        txn.is_poisoned(),
        "the overflow trip latches poison inside the guard"
      );
      txn.rollback();
      assert!(
        !inp.is_poisoned(),
        "the rollback un-latches the speculative poison boundary"
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
fn txn_drop_without_commit_rolls_back() {
  // A guard dropped without deciding rolls back — uncommitted work is discarded.
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let start = *inp.cursor().as_inner();
  {
    let mut txn = inp.begin();
    let _ = txn.next().unwrap().expect("first token");
    let _ = txn.next().unwrap().expect("second token");
    // `txn` drops here without commit/rollback → rollback on drop.
  }
  assert_eq!(
    *inp.cursor().as_inner(),
    start,
    "dropping an undecided guard rolls back to the begin point"
  );
  assert_eq!(
    *inp.next().unwrap().expect("token 1 again").span_ref(),
    SimpleSpan::new(0, 1),
    "the consumed tokens are replayable after the drop-rollback"
  );
}

#[test]
fn txn_nested_inner_commit_outer_rollback() {
  // A committed child's progress is discarded when its parent rolls back (savepoint
  // semantics: rolling back a parent discards everything its children committed).
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let start = *inp.cursor().as_inner();
  let mut outer = inp.begin();
  let _ = outer.next().unwrap().expect("outer consumes 1");

  let mut inner = outer.begin(); // borrows `outer` through DerefMut
  let _ = inner.next().unwrap().expect("inner consumes 2");
  inner.commit(); // keep the inner progress — within the still-open outer

  outer.rollback(); // discards everything, including the inner's committed 2

  assert_eq!(
    *inp.cursor().as_inner(),
    start,
    "the outer rollback discards the child's committed progress"
  );
  assert_eq!(
    *inp.next().unwrap().expect("token 1 again").span_ref(),
    SimpleSpan::new(0, 1)
  );
}

#[test]
fn txn_nested_inner_rollback_outer_commit() {
  // The mirror image: the inner rolls back its own work, the outer commits and keeps
  // only its own progress.
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let mut outer = inp.begin();
  let _ = outer.next().unwrap().expect("outer consumes 1");
  let after_one = *outer.cursor().as_inner();

  let mut inner = outer.begin();
  let _ = inner.next().unwrap().expect("inner consumes 2");
  inner.rollback(); // back to just after token 1

  outer.commit(); // keep the outer progress: position stays just after token 1

  assert_eq!(
    *inp.cursor().as_inner(),
    after_one,
    "the inner rolled back; the outer kept its own progress"
  );
  assert_eq!(
    *inp.next().unwrap().expect("token 2").span_ref(),
    SimpleSpan::new(2, 3)
  );
}

#[test]
fn txn_over_limit_trip_rollback_reemits_exactly_once() {
  // Inside a transaction, an overflow peek trips the limiter (emitting the diagnostic);
  // rolling back un-emits it, and the committed path re-reaches the trip and re-emits —
  // exactly once in total, never zero.
  //   1 2 3 4 5 6   (limit 5 → the 6th scanned token trips; U6 window > U3 cache)
  use generic_arraydeque::typenum::U6;

  let cache = DefaultCache::<'_, NumLexer<'_>>::default();
  let mut emitter = Verbose::<NumErr>::new();
  let mut input = Input::<NumLexer<'_>, NumVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(5),
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    let mut txn = inp.begin();
    let _ = txn.peek::<U6>().unwrap(); // overflow trip: emits the limit diagnostic
    txn.rollback();
    assert!(
      !inp.is_poisoned(),
      "the rollback un-poisons and un-emits the speculative diagnostic"
    );

    while inp.next().unwrap().is_some() {}
    assert!(inp.is_poisoned(), "the committed re-lex re-latches poison");
  }

  let errs: Vec<&NumErr> = emitter.errors().values().flatten().collect();
  assert_eq!(
    errs.len(),
    1,
    "the limit diagnostic is emitted exactly once in total"
  );
  assert_eq!(*errs[0], NumErr::Limit, "and it is the limit diagnostic");
}

/// A plain `&mut InputRef` consumer: the guard must deref-coerce into it.
fn consume_all<'inp>(inp: &mut InputRef<'inp, '_, NumLexer<'inp>, NumCtx<'inp>>) -> usize {
  let mut n = 0;
  while inp.next().unwrap().is_some() {
    n += 1;
  }
  n
}

#[test]
fn txn_passes_as_input_ref() {
  // `&mut Transaction` coerces to `&mut InputRef` via `DerefMut`, so every combinator
  // and helper written against `InputRef` composes with a guard unchanged.
  let mut input = silent_input("1 2 3");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let mut txn = inp.begin();
  let consumed = consume_all(&mut txn); // deref coercion into fn(&mut InputRef)
  assert_eq!(consumed, 3, "the helper drove the input through the guard");
  txn.commit();

  assert!(inp.is_eoi(), "progress kept — every token was consumed");
}

#[cfg(all(debug_assertions, any(feature = "std", feature = "alloc")))]
#[test]
fn txn_commit_removes_id_from_live_stack() {
  // Committing drops a checkpoint that was never restored; its debug-witness id must be
  // forgotten so the live stack does not grow across commit-heavy loops.
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let baseline = inp.live_checkpoints_len();
  for _ in 0..100 {
    let txn = inp.begin();
    txn.commit();
  }
  assert_eq!(
    inp.live_checkpoints_len(),
    baseline,
    "each commit forgets its id — the live stack returns to its baseline length"
  );
}
