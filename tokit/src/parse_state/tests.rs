//! Tests for [`ParseState`](super::ParseState) session points — the owned, non-lexical form of
//! speculation.
//!
//! [`begin_point`](super::ParseState::begin_point) saves a checkpoint onto an internal stack and
//! pins it like a guard base; [`commit_point`](super::ParseState::commit_point) keeps the progress
//! and [`rollback_point`](super::ParseState::rollback_point) returns to it, newest-first. Because
//! the checkpoints are values on the state rather than a borrowing guard, a driver can own the
//! state and step it across separate method calls — the shape the guards cannot express. The
//! observable facts a session point saves and restores (lexer state, emission log) are driven here
//! through the reachable [`state_mut`](super::ParseState::state_mut) /
//! [`emitter`](super::ParseState::emitter) surface, mirroring the guard test technique; the
//! remaining checkpoint facts (position, span, dedup watermark, poison boundary) ride in the same
//! [`Checkpoint`](crate::input::Checkpoint) and are covered end-to-end by the poison leg of
//! `session_point_rollback_restores`.

use crate::{
  Emitter, ParseState, Token,
  cache::DefaultCache,
  emitter::{Silent, Verbose},
  error::token::UnexpectedToken,
  input::Input,
  lexer::LogosLexer,
  span::{SimpleSpan, Spanned},
  state::token_tracker::{TokenLimitExceeded, TokenLimiter},
};

// ── Fixture: a number lexer over a by-value token limiter (as in the guard tests) ──────────────
//
// The by-value `TokenLimiter` travels inside the lexer state, so a session point taken before a
// limit trip saves a clean count and rolling back to it un-trips. `@` matches no rule, a plain
// lexer error between numbers.

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

/// A `ParseState` over the verbose (collecting) context — the driver's/session's handle.
type VerbosePs<'a, 'inp, 'closure> =
  ParseState<'a, 'inp, 'closure, NumLexer<'inp>, NumVerboseCtx<'inp>>;

/// Builds a `Silent` input over `src` with a limit high enough never to trip.
fn silent_input(src: &str) -> Input<'_, NumLexer<'_>, NumCtx<'_>, ()> {
  let cache = DefaultCache::<'_, NumLexer<'_>>::default();
  Input::<NumLexer<'_>, NumCtx<'_>, ()>::with_state_and_cache(
    src,
    TokenLimiter::with_limitation(usize::MAX),
    cache,
  )
}

/// Builds a `Verbose` input over `src` with a limit high enough never to trip.
fn verbose_input(src: &str) -> Input<'_, NumLexer<'_>, NumVerboseCtx<'_>, ()> {
  let cache = DefaultCache::<'_, NumLexer<'_>>::default();
  Input::<NumLexer<'_>, NumVerboseCtx<'_>, ()>::with_state_and_cache(
    src,
    TokenLimiter::with_limitation(usize::MAX),
    cache,
  )
}

/// Emits an application diagnostic through the state's emitter (the lexer type pins the blanket
/// `Verbose: Emitter<L>` impl, exactly as the emitter unit tests do).
fn emit(ps: &mut VerbosePs<'_, '_, '_>, at: usize, err: NumErr) {
  let span = SimpleSpan::new(at, at + 1);
  <Verbose<NumErr> as Emitter<'_, NumLexer<'_>>>::emit_error(ps.emitter(), Spanned::new(span, err))
    .expect("Verbose is a non-fatal emitter");
}

/// The number of diagnostics currently retained by the state's emitter.
fn diag_count(ps: &mut VerbosePs<'_, '_, '_>) -> usize {
  ps.emitter().errors().values().map(|g| g.len()).sum()
}

// ── 1. Both verbs restore/keep the session point ───────────────────────────────────────────────

#[test]
fn session_point_commit_keeps() {
  // Commit keeps the speculative work done through the point: the re-keyed lexer state and the
  // emitted diagnostic both survive, and the point leaves the stack.
  let mut input = verbose_input("1 2 3 4");
  let mut emitter = Verbose::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);
  let start = *inp.cursor();
  let mut ps = ParseState::new(&mut inp, start);

  ps.begin_point();
  assert_eq!(ps.points(), 1, "the point is live");

  // Drive speculative work reachable through the state: a fresh regime and a diagnostic.
  *ps.state_mut() = TokenLimiter::with_limitation(5);
  emit(&mut ps, 0, NumErr::Lex);

  ps.commit_point();
  assert_eq!(ps.points(), 0, "the point settled");
  assert_eq!(
    ps.state().limitation(),
    5,
    "commit keeps the re-keyed state"
  );
  assert_eq!(
    diag_count(&mut ps),
    1,
    "commit keeps the emitted diagnostic"
  );
}

#[test]
fn session_point_rollback_restores() {
  // Rolling back to a session point undoes every fact it captured. The point is taken at a
  // limit-tripped position (poison latched, the limit diagnostic emitted, the watermark lifted);
  // state surgery through the point re-keys those forward-scanning facts away and a speculative
  // diagnostic is emitted; the rollback returns state, poison, watermark, position, and the
  // emission log to the trip — mirroring the guard rollback technique.
  let cache = DefaultCache::<'_, NumLexer<'_>>::default();
  let mut emitter = Verbose::<NumErr>::new();
  let mut input = Input::<NumLexer<'_>, NumVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(2),
    cache,
  );
  {
    let mut inp = input.as_ref(&mut emitter);

    // Trip the limiter: two tokens, then a poisoned `None`; the limit diagnostic emits once.
    assert!(inp.next().unwrap().is_some(), "1");
    assert!(inp.next().unwrap().is_some(), "2");
    assert!(inp.next().unwrap().is_none(), "the 3rd scan trips → None");
    let tripped = *inp.cursor().as_inner();

    let start = *inp.cursor();
    let mut ps = ParseState::new(&mut inp, start);

    ps.begin_point(); // saves the tripped lineage: state, poison, watermark, emission mark
    assert_eq!(ps.points(), 1);

    // Speculative work through the point: re-key the regime (dropping poison, resetting the
    // watermark) and emit a fresh diagnostic on top of the trip's.
    *ps.state_mut() = TokenLimiter::with_limitation(usize::MAX);
    emit(&mut ps, 0, NumErr::Lex);
    assert_eq!(ps.state().limitation(), usize::MAX, "the re-key took");
    assert_eq!(
      diag_count(&mut ps),
      2,
      "the speculative diagnostic joined the trip's"
    );

    ps.rollback_point();
    assert_eq!(ps.points(), 0, "the point settled");
    assert_eq!(
      ps.state().limitation(),
      2,
      "state: the pre-surgery regime returned"
    );
    assert_eq!(
      ps.state().tokens(),
      2,
      "state: the saved token count returned"
    );
    assert_eq!(
      diag_count(&mut ps),
      1,
      "diagnostics: the speculative one was rolled back, the trip's kept"
    );

    // Position rolled back to the trip point (nothing was consumed through the state).
    assert_eq!(
      *inp.cursor().as_inner(),
      tripped,
      "position rolled back to the trip"
    );
  }

  // The input now sits at the restored (tripped) lineage: the restored poison boundary stops the
  // stream, and the limit diagnostic is retained exactly once — never duplicated by the rollback.
  {
    let mut inp = input.as_ref(&mut emitter);
    assert!(
      inp.next().unwrap().is_none(),
      "the restored poison boundary stops the stream"
    );
  }
  let total: usize = emitter.errors().values().map(|g| g.len()).sum();
  assert_eq!(
    total, 1,
    "the limit diagnostic is retained exactly once across the session"
  );
}

// ── 2. Nesting is last-in, first-out ─────────────────────────────────────────────────────────

#[test]
fn session_points_nest_lifo() {
  // Three nested points, each over a distinct lexer regime. Roll back the newest, commit the
  // middle, roll back the oldest — the stream stays faithful: the middle commit keeps the current
  // regime but does not disturb the oldest point's saved regime, reached by the final rollback.
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);
  let start = *inp.cursor();
  let mut ps = ParseState::new(&mut inp, start);

  *ps.state_mut() = TokenLimiter::with_limitation(10);
  ps.begin_point(); // P1 saves regime 10
  *ps.state_mut() = TokenLimiter::with_limitation(20);
  ps.begin_point(); // P2 saves regime 20
  *ps.state_mut() = TokenLimiter::with_limitation(30);
  ps.begin_point(); // P3 saves regime 30
  *ps.state_mut() = TokenLimiter::with_limitation(40);
  assert_eq!(ps.points(), 3, "three live points");

  ps.rollback_point(); // newest: back to P3's regime
  assert_eq!(ps.points(), 2);
  assert_eq!(
    ps.state().limitation(),
    30,
    "rolled back to the newest point's regime"
  );

  ps.commit_point(); // middle: keep the current regime, release the point
  assert_eq!(ps.points(), 1);
  assert_eq!(
    ps.state().limitation(),
    30,
    "commit keeps the current regime"
  );

  ps.rollback_point(); // oldest: back to P1's regime, unaffected by the middle commit
  assert_eq!(ps.points(), 0);
  assert_eq!(
    ps.state().limitation(),
    10,
    "rolled back to the oldest point's regime — a faithful stream"
  );
}

// ── 3. Misuse panics with the pinned prefix ──────────────────────────────────────────────────

#[test]
#[should_panic(expected = "no live session point")]
fn session_point_misuse_panics() {
  // Settling with no point open is a driver bug — a clear panic, prefix `no live session point`.
  let mut input = silent_input("1 2 3");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);
  let start = *inp.cursor();
  let mut ps = ParseState::new(&mut inp, start);

  ps.commit_point(); // zero live points → panic
}

// ── 4. A session point pins its base ─────────────────────────────────────────────────────────

#[test]
#[should_panic(
  expected = "restore would invalidate a live transaction guard or attempt (the target predates its begin point)"
)]
fn session_point_is_pinned() {
  // A raw restore below a session point's base tears the session's foundation out. `begin_point`
  // pins that base (like a guard), so the checked restore panics AT the restore with the existing
  // pin message — the pre-demotion hazard, closed. The pin outlives even the state's drop (a
  // session ends explicitly, not on drop), so the input's lineage still refuses the restore.
  let mut input = silent_input("1 2 3 4 5");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);

  let a = inp.save(); // raw checkpoint, below the session point
  let _ = inp.next().unwrap().expect("consume 1"); // advance past A
  let start = *inp.cursor();
  {
    let mut ps = ParseState::new(&mut inp, start);
    ps.begin_point(); // pins the base, above A
    // `ps` drops here with a live point: its checkpoint is discarded, but the pin persists.
  }
  inp.restore(a); // panics: restoring A would pop the still-pinned base off the lineage
}

// ── 5. The capability proof: an owned, externally-driven session ─────────────────────────────

/// A driver that owns a `ParseState` and is stepped through separate method calls — the shape a
/// borrowing transaction guard cannot express (a guard beside the input it borrows is
/// self-referential). Speculation opened in one call is decided in a later one.
struct Stepper<'a, 'inp, 'closure> {
  ps: VerbosePs<'a, 'inp, 'closure>,
}

impl Stepper<'_, '_, '_> {
  /// Advance the "parse" by recording a diagnostic (observable, checkpoint-tracked work).
  fn step(&mut self) {
    emit(&mut self.ps, 0, NumErr::Lex);
  }

  /// Open a speculative session point.
  fn speculate(&mut self) {
    self.ps.begin_point();
  }

  /// Decide the newest open point: keep its work or discard it.
  fn decide(&mut self, keep: bool) {
    if keep {
      self.ps.commit_point();
    } else {
      self.ps.rollback_point();
    }
  }

  fn depth(&self) -> usize {
    self.ps.points()
  }

  fn diagnostics(&mut self) -> usize {
    diag_count(&mut self.ps)
  }
}

#[test]
fn external_driver_pattern_compiles_and_works() {
  let mut input = verbose_input("1 2 3 4");
  let mut emitter = Verbose::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);
  let start = *inp.cursor();
  let mut driver = Stepper {
    ps: ParseState::new(&mut inp, start),
  };

  // A committed step before any speculation.
  driver.step();
  assert_eq!(driver.depth(), 0);
  assert_eq!(driver.diagnostics(), 1);

  // Open speculation in one call, do work in another, abandon it in a third.
  driver.speculate();
  assert_eq!(
    driver.depth(),
    1,
    "the point persists across the call boundary"
  );
  driver.step();
  assert_eq!(driver.diagnostics(), 2, "a speculative diagnostic joined");
  driver.decide(false);
  assert_eq!(driver.depth(), 0);
  assert_eq!(
    driver.diagnostics(),
    1,
    "the abandoned step's diagnostic rolled back"
  );

  // Now speculate again and keep it.
  driver.speculate();
  driver.step();
  assert_eq!(driver.diagnostics(), 2);
  driver.decide(true);
  assert_eq!(driver.depth(), 0);
  assert_eq!(
    driver.diagnostics(),
    2,
    "the committed step's diagnostic stuck"
  );
}

// ── 6. The depth accessor through the lifecycle ──────────────────────────────────────────────

#[test]
fn session_depth_accessor() {
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut inp = input.as_ref(&mut emitter);
  let start = *inp.cursor();
  let mut ps = ParseState::new(&mut inp, start);

  assert_eq!(ps.points(), 0, "a fresh state has no points");
  ps.begin_point();
  assert_eq!(ps.points(), 1);
  ps.begin_point();
  assert_eq!(ps.points(), 2, "nesting deepens the stack");
  ps.commit_point();
  assert_eq!(ps.points(), 1, "committing the newest lowers the depth");
  ps.rollback_point();
  assert_eq!(ps.points(), 0, "rolling back the oldest empties the stack");
}
