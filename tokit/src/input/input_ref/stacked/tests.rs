//! Tests for the [`StackedTransaction`](super::StackedTransaction) guard.
//!
//! `begin_stacked` saves a base checkpoint and wraps the input; `savepoint` marks a
//! position, `rollback_to` returns to a mark while destroying every younger one (SQL
//! `ROLLBACK TO`), `release` forgets marks while keeping progress (SQL `RELEASE`), and
//! `commit`/`rollback` decide the whole transaction. A foreign or destroyed
//! [`SavepointId`](super::SavepointId) panics in every build.

use crate::{
  Token,
  cache::DefaultCache,
  emitter::{Silent, Verbose},
  error::token::UnexpectedToken,
  input::Input,
  lexer::LogosLexer,
  span::SimpleSpan,
  state::token_tracker::{TokenLimitExceeded, TokenLimiter},
};

// ── Fixture: a number lexer over a by-value token limiter (mirrors the sibling) ──
//
// A by-value `TokenLimiter` (checkpointed and restored with the lexer state) is what
// makes a rolled-back limit trip re-tripable: an overflow peek never writes its
// temporary lexer's counter back, so a checkpoint taken before the trip saves a clean
// count and the committed path re-lexes and re-trips from scratch.

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

// ── rollback_to / savepoint ─────────────────────────────────────────────────────

#[test]
fn stacked_rollback_to_middle_destroys_younger_keeps_target() {
  // Three savepoints, then roll back to the middle one twice (SQL parity): the target
  // resumes the stream at exactly its position each time, and the younger savepoint is
  // destroyed structurally.
  let mut input = silent_input("1 2 3 4 5");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let mut txn = inp.begin_stacked();
  let _ = txn.next().unwrap().expect("1");
  let _sp1 = txn.savepoint();
  let _ = txn.next().unwrap().expect("2");
  let sp2 = txn.savepoint(); // fallback: the next token is "3"
  let _ = txn.next().unwrap().expect("3");
  let _sp3 = txn.savepoint();
  let _ = txn.next().unwrap().expect("4");

  txn.rollback_to(sp2);
  assert_eq!(
    *txn.next().unwrap().expect("resume at 3").span_ref(),
    SimpleSpan::new(4, 5),
    "rollback_to resumes exactly at the target savepoint"
  );

  // SQL parity: roll back to the same savepoint again after consuming past it.
  txn.rollback_to(sp2);
  assert_eq!(
    *txn.next().unwrap().expect("resume at 3 again").span_ref(),
    SimpleSpan::new(4, 5),
    "rolling back to the same savepoint twice works (SQL parity)"
  );

  assert_eq!(
    txn.saves.len(),
    2,
    "the first rollback_to destroyed the younger savepoint; only sp1 and sp2 remain"
  );
}

#[test]
fn stacked_release_keeps_progress_and_forgets() {
  // Release forgets the savepoint but keeps every parsed byte: the position does not move.
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let mut txn = inp.begin_stacked();
  let _ = txn.next().unwrap().expect("1");
  let sp1 = txn.savepoint();
  let _ = txn.next().unwrap().expect("2");
  let _ = txn.next().unwrap().expect("3");

  txn.release(sp1);
  assert!(
    txn.saves.is_empty(),
    "release forgets the savepoint (and any younger)"
  );
  assert_eq!(
    *txn.next().unwrap().expect("4").span_ref(),
    SimpleSpan::new(6, 7),
    "release keeps the parsed progress: the position does not move"
  );
  txn.commit();
}

#[test]
#[should_panic(expected = "stacked transaction: savepoint is stale")]
fn stacked_stale_id_after_rollback_panics() {
  // Rolling back to an older savepoint destroys the younger one; using the younger id
  // afterwards panics as stale.
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let mut txn = inp.begin_stacked();
  let _ = txn.next().unwrap().expect("1");
  let sp1 = txn.savepoint();
  let _ = txn.next().unwrap().expect("2");
  let sp2 = txn.savepoint();

  txn.rollback_to(sp1); // destroys sp2 (younger)
  txn.rollback_to(sp2); // sp2 is stale → panic
}

// Note: the same-input cross-transaction misuse (an id from one transaction on the same
// input used in a later one) is now a *compile* error — the id's lifetime brand keeps the
// input loan open, so the second `begin_stacked` cannot re-borrow it. It is pinned by a
// `compile_fail` doctest on `SavepointId`, so there is no runtime test for it here.

#[test]
#[should_panic(expected = "stacked transaction: savepoint belongs to a different transaction")]
fn stacked_foreign_input_savepoint_panics() {
  // Two simultaneously-live inputs. Both transactions are opened *before* either savepoint
  // is taken, so their brand regions coincide and the compiler unifies them — passing input
  // A's id to input B's transaction type-checks. (Opening B's input after A's savepoint
  // would instead be a compile error, the same brand-region mismatch the nesting doctests
  // pin.) Each id carries the address of its input's `poison_boundary` field, and the two
  // live inputs are distinct structs at distinct addresses, so the foreign id is caught at
  // runtime in every build.
  let mut input_a = silent_input("1 2 3 4");
  let mut emitter_a = Silent::<NumErr>::new();
  let mut input_b = silent_input("1 2 3 4");
  let mut emitter_b = Silent::<NumErr>::new();

  let mut inp_a = input_a.as_ref(&mut emitter_a);
  let mut inp_b = input_b.as_ref(&mut emitter_b);

  let mut txn_a = inp_a.begin_stacked();
  let mut txn_b = inp_b.begin_stacked();

  let sp_a = txn_a.savepoint();
  let _sp_b = txn_b.savepoint();

  // `sp_a` belongs to input A's transaction → foreign to `txn_b` → panic.
  txn_b.rollback_to(sp_a);
}

#[test]
fn stacked_drop_rolls_back_to_begin() {
  // An undecided stacked transaction rolls back to its begin point on drop, discarding
  // every savepoint.
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let start = *inp.cursor().as_inner();
  {
    let mut txn = inp.begin_stacked();
    let _ = txn.next().unwrap().expect("1");
    let _sp = txn.savepoint();
    let _ = txn.next().unwrap().expect("2");
    // `txn` drops here undecided → rollback to begin.
  }
  assert_eq!(
    *inp.cursor().as_inner(),
    start,
    "dropping an undecided stacked transaction rolls back to the begin point"
  );
  assert_eq!(
    *inp.next().unwrap().expect("1 again").span_ref(),
    SimpleSpan::new(0, 1),
    "the consumed tokens replay after the drop-rollback"
  );
}

#[test]
fn stacked_savepoint_over_limit_trip_reemits_exactly_once() {
  // A savepoint taken before an overflow trip, rolled back to, then re-reached on the
  // committed path: the limit diagnostic is emitted exactly once in total, never zero.
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

    let mut txn = inp.begin_stacked();
    let sp = txn.savepoint(); // before the trip
    let _ = txn.peek::<U6>().unwrap(); // overflow trip inside the guard
    assert!(
      txn.is_poisoned(),
      "the overflow trip latches poison inside the guard"
    );

    txn.rollback_to(sp);
    assert!(
      !txn.is_poisoned(),
      "rollback_to un-latches the speculative poison boundary"
    );

    // The committed path re-reaches the trip and re-latches.
    while txn.next().unwrap().is_some() {}
    assert!(txn.is_poisoned(), "the committed re-lex re-latches poison");
    txn.commit();
  }

  let total: usize = emitter.errors().values().map(|g| g.len()).sum();
  assert_eq!(
    total, 1,
    "the limit diagnostic is emitted exactly once in total"
  );
}

#[test]
fn stacked_best_match_selection() {
  // The motivating pattern end-to-end: parse several stages, keep a fallback savepoint
  // after each, score them, then roll back to the best-scoring one and resume from
  // exactly there — the younger fallbacks die with the rollback.
  let mut input = silent_input("1 2 3 4 5 6");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let mut txn = inp.begin_stacked();

  // Three stages; the middle one scores highest.
  let scores = [1i32, 5, 2];
  let mut candidates = Vec::new();
  for s in scores {
    let _ = txn.next().unwrap().expect("stage token");
    candidates.push((s, txn.savepoint()));
  }

  // Select the best-scoring savepoint (the middle stage, sitting after "2").
  let (_, best) = candidates
    .iter()
    .copied()
    .max_by_key(|(s, _)| *s)
    .expect("three candidates");

  txn.rollback_to(best);

  assert_eq!(
    *txn.next().unwrap().expect("resume at 3").span_ref(),
    SimpleSpan::new(4, 5),
    "the stream resumes exactly at the best-scoring savepoint"
  );
  assert_eq!(
    txn.saves.len(),
    2,
    "rolling back to the middle savepoint destroyed the younger one"
  );

  txn.commit();
}

#[cfg(all(
  debug_assertions,
  any(feature = "std", feature = "alloc"),
  target_has_atomic = "ptr"
))]
#[test]
fn stacked_commit_removes_all_ids_from_live_stack() {
  // Committing keeps the parsed progress but forgets the base and every savepoint id, so
  // the debug live-checkpoint stack does not grow across commit-heavy loops.
  let mut input = silent_input("1 2 3 4 5 6");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let baseline = inp.live_checkpoints_len();
  for _ in 0..100 {
    let mut txn = inp.begin_stacked();
    let _ = txn.next().unwrap();
    let _ = txn.savepoint();
    let _ = txn.next().unwrap();
    let _ = txn.savepoint();
    txn.commit();
  }
  assert_eq!(
    inp.live_checkpoints_len(),
    baseline,
    "commit forgets the base and every savepoint id — the live stack returns to baseline"
  );
}
