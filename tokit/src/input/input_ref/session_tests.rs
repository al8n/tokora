//! Tests for [`InputRef`](super::InputRef) session points — the non-lexical form of speculation.
//!
//! [`begin_point`](super::InputRef::begin_point) saves a checkpoint onto the input's point stack
//! and pins it like a guard base; [`commit_point`](super::InputRef::commit_point) keeps the
//! progress and [`rollback_point`](super::InputRef::rollback_point) returns to it, newest-first.
//! Because a point is a value on the input rather than a borrow *of* it, the consume surface stays
//! callable while one is open — so unlike the guard suites these tests speculate over **real token
//! consumption**, and every rollback is watched to put the tokens back. The other facts a
//! checkpoint carries (lexer state, emission log, dedup watermark, poison boundary) ride along and
//! are asserted beside the cursor.

use crate::{
  Emitter, Token,
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
// limit trip saves a clean count and rolling back to it un-trips.

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

/// The input reference the session runs on, over the verbose (collecting) context.
type VerboseIr<'inp, 'closure> =
  super::InputRef<'inp, 'closure, NumLexer<'inp>, NumVerboseCtx<'inp>>;

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

/// Emits an application diagnostic through the input's emitter (the lexer type pins the blanket
/// `Verbose: Emitter<L>` impl, exactly as the emitter unit tests do).
fn emit(ir: &mut VerboseIr<'_, '_>, at: usize, err: NumErr) {
  let span = SimpleSpan::new(at, at + 1);
  <Verbose<NumErr> as Emitter<'_, NumLexer<'_>>>::emit_error(ir.emitter(), Spanned::new(span, err))
    .expect("Verbose is a non-fatal emitter");
}

/// The number of diagnostics currently retained by the input's emitter.
fn diag_count(ir: &mut VerboseIr<'_, '_>) -> usize {
  ir.emitter().errors().values().map(|g| g.len()).sum()
}

/// Consumes one token, asserting it is there, and returns its source text.
fn take<'inp>(ir: &mut VerboseIr<'inp, '_>) -> &'inp str {
  ir.next()
    .expect("complete + non-fatal")
    .expect("a token is there");
  ir.slice()
}

// ── 1. Both verbs settle a point that CONSUMED TOKENS ──────────────────────────────────────────

#[test]
fn session_point_commit_keeps_consumed_tokens() {
  // Commit keeps the speculative work done through the point — including the tokens it consumed
  // across separate calls, which is the work a `ParseState` could never have done.
  let mut input = verbose_input("1 2 3 4");
  let mut emitter = Verbose::<NumErr>::new();
  let mut ir = input.as_ref(&mut emitter);

  ir.begin_point();
  assert_eq!(ir.points(), 1, "the point is live");

  // Real parsing, through the open point, one separate call at a time.
  assert_eq!(take(&mut ir), "1");
  assert_eq!(take(&mut ir), "2");
  let after_two = *ir.cursor().as_inner();
  emit(&mut ir, 0, NumErr::Lex);

  ir.commit_point();
  assert_eq!(ir.points(), 0, "the point settled");
  assert_eq!(
    *ir.cursor().as_inner(),
    after_two,
    "commit keeps the consumed tokens: the cursor stays past `2`"
  );
  assert_eq!(
    diag_count(&mut ir),
    1,
    "commit keeps the emitted diagnostic"
  );
  // The stream resumes where the committed work left it.
  assert_eq!(take(&mut ir), "3", "the next token is the one after `2`");
}

#[test]
fn session_point_rollback_puts_the_tokens_back() {
  // THE capability: mark, consume several tokens across separate calls, emit, then roll back —
  // and the cursor, the token stream, and the emission log all return to the mark.
  let mut input = verbose_input("1 2 3 4");
  let mut emitter = Verbose::<NumErr>::new();
  let mut ir = input.as_ref(&mut emitter);

  assert_eq!(take(&mut ir), "1", "committed work before the session");
  let mark = *ir.cursor().as_inner();

  ir.begin_point();
  assert_eq!(take(&mut ir), "2");
  emit(&mut ir, 2, NumErr::Lex);
  assert_eq!(take(&mut ir), "3");
  assert_eq!(diag_count(&mut ir), 1, "a speculative diagnostic");
  assert_ne!(
    *ir.cursor().as_inner(),
    mark,
    "the session moved the cursor"
  );

  ir.rollback_point();
  assert_eq!(ir.points(), 0, "the point settled");
  assert_eq!(
    *ir.cursor().as_inner(),
    mark,
    "rollback returned the cursor to the mark"
  );
  assert_eq!(
    diag_count(&mut ir),
    0,
    "rollback dropped the speculative diagnostic"
  );
  // The tokens are genuinely back on the stream: the abandoned `2` is next again.
  assert_eq!(take(&mut ir), "2", "the rewound token re-lexes");
  assert_eq!(take(&mut ir), "3");
  assert_eq!(take(&mut ir), "4");
  assert!(
    ir.next().expect("complete + non-fatal").is_none(),
    "and then the stream ends"
  );
}

#[test]
fn session_point_rollback_restores_state_and_poison() {
  // The non-cursor facts a checkpoint carries. The point is taken at a limit-tripped position
  // (poison latched, the limit diagnostic emitted, the watermark lifted); state surgery through
  // the point re-keys those forward-scanning facts away and a speculative diagnostic is emitted;
  // the rollback returns state, poison, watermark, position, and the emission log to the trip.
  let cache = DefaultCache::<'_, NumLexer<'_>>::default();
  let mut emitter = Verbose::<NumErr>::new();
  let mut input = Input::<NumLexer<'_>, NumVerboseCtx<'_>, ()>::with_state_and_cache(
    "1 2 3 4 5 6",
    TokenLimiter::with_limitation(2),
    cache,
  );
  {
    let mut ir = input.as_ref(&mut emitter);

    // Trip the limiter: two tokens, then a poisoned `None`; the limit diagnostic emits once.
    assert!(ir.next().unwrap().is_some(), "1");
    assert!(ir.next().unwrap().is_some(), "2");
    assert!(ir.next().unwrap().is_none(), "the 3rd scan trips → None");
    let tripped = *ir.cursor().as_inner();

    ir.begin_point(); // saves the tripped lineage: state, poison, watermark, emission mark
    assert_eq!(ir.points(), 1);

    // Speculative work through the point: re-key the regime (dropping poison, resetting the
    // watermark) and emit a fresh diagnostic on top of the trip's.
    *ir.state_mut() = TokenLimiter::with_limitation(usize::MAX);
    emit(&mut ir, 0, NumErr::Lex);
    assert_eq!(ir.state().limitation(), usize::MAX, "the re-key took");
    assert_eq!(
      diag_count(&mut ir),
      2,
      "the speculative diagnostic joined the trip's"
    );
    // The re-key lifted the poison, so the stream flows again — real work, rolled back below.
    assert!(
      ir.next().unwrap().is_some(),
      "the un-poisoned stream yields again"
    );

    ir.rollback_point();
    assert_eq!(ir.points(), 0, "the point settled");
    assert_eq!(
      ir.state().limitation(),
      2,
      "state: the pre-surgery regime returned"
    );
    assert_eq!(
      ir.state().tokens(),
      2,
      "state: the saved token count returned"
    );
    assert_eq!(
      diag_count(&mut ir),
      1,
      "diagnostics: the speculative one was rolled back, the trip's kept"
    );
    assert_eq!(
      *ir.cursor().as_inner(),
      tripped,
      "position rolled back to the trip"
    );
  }

  // The input now sits at the restored (tripped) lineage: the restored poison boundary stops the
  // stream, and the limit diagnostic is retained exactly once — never duplicated by the rollback.
  {
    let mut ir = input.as_ref(&mut emitter);
    assert!(
      ir.next().unwrap().is_none(),
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
  // Three nested points, each opened after consuming one more token. Roll back the newest, commit
  // the middle, roll back the oldest — the stream stays faithful: the middle commit keeps the
  // current position but does not disturb the oldest point's saved one, reached by the final
  // rollback.
  let mut input = verbose_input("1 2 3 4 5");
  let mut emitter = Verbose::<NumErr>::new();
  let mut ir = input.as_ref(&mut emitter);

  assert_eq!(take(&mut ir), "1");
  let at1 = *ir.cursor().as_inner();
  ir.begin_point(); // P1 marks "after 1"

  assert_eq!(take(&mut ir), "2");
  ir.begin_point(); // P2 marks "after 2"

  assert_eq!(take(&mut ir), "3");
  let at3 = *ir.cursor().as_inner();
  ir.begin_point(); // P3 marks "after 3"

  assert_eq!(take(&mut ir), "4");
  assert_eq!(ir.points(), 3, "three live points");

  ir.rollback_point(); // newest: back to P3's mark
  assert_eq!(ir.points(), 2);
  assert_eq!(
    *ir.cursor().as_inner(),
    at3,
    "rolled back to the newest point's mark"
  );

  ir.commit_point(); // middle: keep the current position, release the point
  assert_eq!(ir.points(), 1);
  assert_eq!(
    *ir.cursor().as_inner(),
    at3,
    "commit keeps the current position"
  );

  ir.rollback_point(); // oldest: back to P1's mark, unaffected by the middle commit
  assert_eq!(ir.points(), 0);
  assert_eq!(
    *ir.cursor().as_inner(),
    at1,
    "rolled back to the oldest point's mark — a faithful stream"
  );
  assert_eq!(take(&mut ir), "2", "and the stream replays from there");
}

// ── 3. Misuse panics with the documented prefix ──────────────────────────────────────────────

#[test]
#[should_panic(expected = "no live session point")]
fn session_point_commit_misuse_panics() {
  let mut input = silent_input("1 2 3");
  let mut emitter = Silent::<NumErr>::new();
  let mut ir = input.as_ref(&mut emitter);

  ir.commit_point(); // zero live points → panic
}

#[test]
#[should_panic(expected = "no live session point")]
fn session_point_rollback_misuse_panics() {
  let mut input = silent_input("1 2 3");
  let mut emitter = Silent::<NumErr>::new();
  let mut ir = input.as_ref(&mut emitter);

  ir.rollback_point(); // zero live points → panic
}

// ── 4. A session point pins its base ─────────────────────────────────────────────────────────

#[test]
#[should_panic(
  expected = "restore would invalidate a live transaction guard or attempt (the target predates its begin point)"
)]
fn session_point_is_pinned() {
  // A rewind below a live session point's base tears the session's foundation out. `begin_point`
  // pins that base (like a guard), so the checked restore panics AT the restore with the existing
  // pin message — detect-at-cause, in every allocator build.
  let mut input = silent_input("1 2 3 4 5");
  let mut emitter = Silent::<NumErr>::new();
  let mut ir = input.as_ref(&mut emitter);

  let a = ir.save(); // raw checkpoint, below the session point
  let _ = ir.next().unwrap().expect("consume 1"); // advance past A
  ir.begin_point(); // pins the base, above A
  ir.restore(a); // panics: restoring A would pop the still-pinned base off the lineage
}

// ── 5. The depth accessor through the lifecycle ──────────────────────────────────────────────

#[test]
fn session_depth_accessor() {
  let mut input = silent_input("1 2 3 4");
  let mut emitter = Silent::<NumErr>::new();
  let mut ir = input.as_ref(&mut emitter);

  assert_eq!(ir.points(), 0, "a fresh reference has no points");
  ir.begin_point();
  assert_eq!(ir.points(), 1);
  ir.begin_point();
  assert_eq!(ir.points(), 2, "nesting deepens the stack");
  ir.commit_point();
  assert_eq!(ir.points(), 1, "committing the newest lowers the depth");
  ir.rollback_point();
  assert_eq!(ir.points(), 0, "rolling back the oldest empties the stack");
}

// ── 6. Settled points leave the lineage bounded ──────────────────────────────────────────────

#[test]
fn settled_points_do_not_grow_the_lineage() {
  // Every settle path — commit and rollback alike — releases the point's lineage entry, so a
  // driver that speculates in a loop does not grow the input's live-checkpoint stack.
  let mut input = silent_input("1 2 3 4 5 6 7 8");
  let mut emitter = Silent::<NumErr>::new();
  let mut ir = input.as_ref(&mut emitter);

  for i in 0..4 {
    ir.begin_point();
    let _ = ir.next().unwrap();
    if i % 2 == 0 {
      ir.commit_point();
    } else {
      ir.rollback_point();
    }
    assert_eq!(
      ir.live_checkpoints_len(),
      0,
      "a settled session point releases its lineage entry"
    );
  }
  assert_eq!(ir.points(), 0);
}
