//! The named-regression suite of the rewindable event sink: the failure-corpus scenarios
//! (F-A1/F-A2/F-A3/F-A5, T3) at the mechanism level, the unified-log exactness laws, and
//! the CST_FORWARD_CENSUS source lock.

use core::num::NonZeroU32;

use crate::{
  Lexer, SimpleSpan,
  cache::DefaultCache,
  cst::event::{Event, TOMBSTONE},
  emitter::{CstEmitter, Emitter, Fatal, Verbose},
  error::token::UnexpectedToken,
  input::{Cursor, Input},
  span::Spanned,
  token::Token,
};

use super::CstSink;

// ── A tiny real lexer: one byte per token, `!` is a lexer error ─────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MiniTok(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MiniErr;

impl Token<'_> for MiniTok {
  type Kind = u8;
  type Error = MiniErr;

  fn kind(&self) -> u8 {
    self.0
  }

  fn is_trivia(&self) -> bool {
    self.0 == b' '
  }
}

struct MiniLexer<'inp> {
  src: &'inp str,
  tok_start: usize,
  pos: usize,
  state: (),
}

impl<'inp> Lexer<'inp> for MiniLexer<'inp> {
  type State = ();
  type Source = str;
  type Token = MiniTok;
  type Span = SimpleSpan;
  type Offset = usize;

  fn new(src: &'inp str) -> Self {
    Self {
      src,
      tok_start: 0,
      pos: 0,
      state: (),
    }
  }

  fn with_state(src: &'inp str, state: ()) -> Self {
    Self {
      src,
      tok_start: 0,
      pos: 0,
      state,
    }
  }

  fn check(&self) -> Result<(), MiniErr> {
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

  fn span(&self) -> SimpleSpan {
    SimpleSpan::new(self.tok_start, self.pos)
  }

  fn slice(&self) -> &'inp str {
    &self.src[self.tok_start..self.pos]
  }

  fn lex(&mut self) -> Option<Result<MiniTok, MiniErr>> {
    let byte = *self.src.as_bytes().get(self.pos)?;
    self.tok_start = self.pos;
    self.pos += 1;
    if byte == b'!' {
      Some(Err(MiniErr))
    } else {
      Some(Ok(MiniTok(byte)))
    }
  }

  fn bump(&mut self, n: &usize) {
    self.pos += *n;
    self.tok_start = self.pos;
  }
}

// ── The test error type (FromEmitterError via the blanket) ─────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestErr {
  Lex,
  Unexpected,
  Custom(u8),
}

impl From<MiniErr> for TestErr {
  fn from(_: MiniErr) -> Self {
    Self::Lex
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for TestErr {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    Self::Unexpected
  }
}

// The remaining conversions the atomic emitter traits' blanket `From*Error` impls ask of a
// bundle's error type — the full-family conformance test below names every trait.
const _: () = {
  use crate::error::{
    UnexpectedEoLhs, UnexpectedEoRhs,
    syntax::{FullContainer, MissingSyntax, TooFew, TooMany},
    token::{MissingToken, SeparatedError},
  };

  impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for TestErr {
    fn from(_: TooFew<S, Lang>) -> Self {
      Self::Unexpected
    }
  }

  impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for TestErr {
    fn from(_: TooMany<S, Lang>) -> Self {
      Self::Unexpected
    }
  }

  impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for TestErr {
    fn from(_: FullContainer<S, Lang>) -> Self {
      Self::Unexpected
    }
  }

  impl<'a, Kind: Clone, O, Lang: ?Sized> From<MissingToken<'a, Kind, O, Lang>> for TestErr {
    fn from(_: MissingToken<'a, Kind, O, Lang>) -> Self {
      Self::Unexpected
    }
  }

  impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for TestErr {
    fn from(_: MissingSyntax<O, Lang>) -> Self {
      Self::Unexpected
    }
  }

  impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, Kind, S, Lang>> for TestErr {
    fn from(_: SeparatedError<'a, T, Kind, S, Lang>) -> Self {
      Self::Unexpected
    }
  }

  impl<O, Lang: ?Sized> From<UnexpectedEoLhs<O, Lang>> for TestErr {
    fn from(_: UnexpectedEoLhs<O, Lang>) -> Self {
      Self::Unexpected
    }
  }

  impl<O, Lang: ?Sized> From<UnexpectedEoRhs<O, Lang>> for TestErr {
    fn from(_: UnexpectedEoRhs<O, Lang>) -> Self {
      Self::Unexpected
    }
  }
};

// ── Dialect fixture: the unified kind space and the mapper ─────────────────────

const K_NODE: u16 = 2;
const K_LIST: u16 = 3;
const K_WRAP: u16 = 4;
const K_TOK: u16 = 10;
const K_ERR: u16 = 90;
const K_GAP: u16 = 91;

fn map_tok(_: &MiniTok) -> u16 {
  K_TOK
}

type VerboseSink<'inp> = CstSink<'inp, MiniLexer<'inp>, Verbose<TestErr>>;
type FatalSink<'inp> = CstSink<'inp, MiniLexer<'inp>, Fatal<TestErr>>;

fn verbose_sink<'inp>() -> VerboseSink<'inp> {
  CstSink::new(Verbose::new(), map_tok, K_ERR, K_GAP)
}

fn span(start: usize, end: usize) -> SimpleSpan {
  SimpleSpan::new(start, end)
}

/// Drives the sink's `Emitter::rewind` directly, the way the input layer does at a restore.
fn rewind(sink: &mut VerboseSink<'_>, mark: u64) {
  let origin = 0usize;
  Emitter::<MiniLexer<'_>>::rewind(sink, Cursor::from_ref(&origin), mark);
}

fn emit_error(sink: &mut VerboseSink<'_>, at: usize, tag: u8) {
  Emitter::<MiniLexer<'_>>::emit_error(sink, Spanned::new(span(at, at + 1), TestErr::Custom(tag)))
    .expect("verbose emitters collect");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Emission shapes
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn emissions_buffer_in_order() {
  let mut sink = verbose_sink();
  sink.cst_start(K_NODE);
  sink.cst_token(&MiniTok(b'a'), &span(0, 1));
  sink.cst_finish();
  assert_eq!(
    sink.events(),
    &[
      Event::StartNode {
        kind: K_NODE,
        forward_parent: None
      },
      Event::Token {
        kind: K_TOK,
        span: span(0, 1)
      },
      Event::FinishNode,
    ]
  );
}

#[test]
fn mark_appends_an_inert_tombstone() {
  let mut sink = verbose_sink();
  let mark = sink.cst_mark();
  assert_eq!(mark.index(), 0);
  assert_eq!(
    sink.events(),
    &[Event::StartNode {
      kind: TOMBSTONE,
      forward_parent: None
    }]
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// F-A5 — stale marks panic in every build (the savepoint posture)
// ═══════════════════════════════════════════════════════════════════════════════

/// A rewind truncates the tombstone; unrelated events regrow over its index; spending the
/// mark must panic, not wrap the regrown region.
#[test]
#[should_panic(expected = "stale EventMark")]
fn stale_mark_spend_panics_after_truncate_and_regrow() {
  let mut sink = verbose_sink();
  let ckp = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  let mark = sink.cst_mark();
  sink.cst_token(&MiniTok(b'a'), &span(0, 1));
  rewind(&mut sink, ckp);
  // Regrow: a token now occupies the mark's old index.
  sink.cst_token(&MiniTok(b'b'), &span(0, 1));
  sink.cst_start_at(mark, K_WRAP);
}

/// The sharpest alias: the regrown event at the mark's index is ANOTHER tombstone, so the
/// positional check alone would validate it — only the era distinguishes the histories.
#[test]
#[should_panic(expected = "stale EventMark")]
fn stale_mark_panics_even_over_a_regrown_tombstone() {
  let mut sink = verbose_sink();
  let ckp = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  let dead = sink.cst_mark();
  rewind(&mut sink, ckp);
  // Regrow a fresh tombstone at the very same index.
  let live = sink.cst_mark();
  assert_eq!(live.index(), dead.index());
  sink.cst_start_at(dead, K_WRAP);
}

/// An inert mark (a diagnostics-only emitter's defaulted `cst_mark`) can never spend on a
/// recording sink.
#[test]
#[should_panic(expected = "EventMark")]
fn inert_mark_spend_panics() {
  let mut fatal = Fatal::<TestErr>::new();
  let inert = CstEmitter::<MiniLexer<'_>>::cst_mark(&mut fatal);
  let mut sink = verbose_sink();
  sink.cst_token(&MiniTok(b'a'), &span(0, 1));
  sink.cst_start_at(inert, K_WRAP);
}

/// A mark minted by one sink must not validate on another, even when the foreign sink has
/// a live tombstone at the same index and era (debug witness).
#[cfg(all(
  debug_assertions,
  any(feature = "std", feature = "alloc"),
  target_has_atomic = "ptr"
))]
#[test]
#[should_panic(expected = "different sink")]
fn foreign_sink_mark_panics() {
  let mut a = verbose_sink();
  let mut b = verbose_sink();
  let mark_a = a.cst_mark();
  let mark_b = b.cst_mark();
  assert_eq!(mark_a.index(), mark_b.index());
  assert_eq!(mark_a.era(), mark_b.era());
  b.cst_start_at(mark_a, K_WRAP);
}

/// The legal counterpart: rewinds strictly above a mark leave it spendable forever (the
/// pratt shape — an entry mark surviving per-iteration rollbacks).
#[test]
fn mark_survives_rewinds_strictly_above_it() {
  let mut sink = verbose_sink();
  let mark = sink.cst_mark();
  sink.cst_token(&MiniTok(b'a'), &span(0, 1));
  let ckp = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  sink.cst_token(&MiniTok(b'b'), &span(1, 2));
  rewind(&mut sink, ckp);
  sink.cst_token(&MiniTok(b'c'), &span(1, 2));
  sink.cst_start_at(mark, K_WRAP);
  sink.cst_finish();
  assert_eq!(
    sink.forward_parent_at(0),
    NonZeroU32::new(3),
    "the wrap landed on the surviving tombstone (StartAt at index 3, target 0)"
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// F-A2 / F-A3 — the forward_parent write dies to journal + era
// ═══════════════════════════════════════════════════════════════════════════════

/// F-A2 (the dangle): a wrap inside a to-be-declined branch writes the tombstone's
/// forward_parent; the decline truncates the StartAt but the write targets a pre-mark
/// slot — only the journal's reverse-replay restores it. Without it, the pointer dangles
/// above the truncation.
#[test]
fn rewind_reverses_the_forward_parent_write() {
  let mut sink = verbose_sink();
  sink.cst_token(&MiniTok(b'a'), &span(0, 1));
  let mark = sink.cst_mark();
  sink.cst_token(&MiniTok(b'b'), &span(1, 2));
  let ckp = Emitter::<MiniLexer<'_>>::checkpoint(&sink);

  // The speculative wrap: StartAt + finish, with the journaled fp write onto index 1.
  sink.cst_start_at(mark, K_WRAP);
  sink.cst_token(&MiniTok(b'c'), &span(2, 3));
  sink.cst_finish();
  assert_eq!(sink.forward_parent_at(1), NonZeroU32::new(2));
  assert_eq!(sink.journal_len(), 1);

  // The decline: truncation must reverse the interior write, not just drop the suffix.
  rewind(&mut sink, ckp);
  assert_eq!(
    sink.forward_parent_at(1),
    None,
    "the journaled forward_parent write survived the rewind (F-A2's dangling pointer)"
  );
  assert_eq!(sink.journal_len(), 0);
  assert_eq!(sink.events().len(), ckp as usize);
}

/// F-A3 (the steal): after the decline, the retry opens an unrelated node and parses on.
/// With the write reversed, nothing ties the retry's events to the abandoned wrap — the
/// tombstone is pristine and no StartAt names it.
#[test]
fn regrown_branch_cannot_inherit_a_dead_wrap() {
  let mut sink = verbose_sink();
  sink.cst_token(&MiniTok(b'a'), &span(0, 1));
  let mark = sink.cst_mark();
  sink.cst_token(&MiniTok(b'b'), &span(1, 2));
  let ckp = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  sink.cst_start_at(mark, K_WRAP);
  sink.cst_token(&MiniTok(b'c'), &span(2, 3));
  sink.cst_finish();
  rewind(&mut sink, ckp);

  // The retry: an unrelated List over the next token.
  sink.cst_start(K_LIST);
  sink.cst_token(&MiniTok(b'd'), &span(2, 3));
  sink.cst_finish();

  assert_eq!(
    sink.forward_parent_at(1),
    None,
    "the dead wrap leaked into the retry's timeline (F-A3's stolen start)"
  );
  assert!(
    !sink
      .events()
      .iter()
      .any(|ev| matches!(ev, Event::StartAt { .. })),
    "no StartAt survives the decline"
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// T3 — release pops the kept capture: the mark stack holds exactly the live rows
// ═══════════════════════════════════════════════════════════════════════════════

/// Through the public `attempt` API (the T3 repro shape): committed attempts release
/// their capture, declined attempts rewind it — the stack never grows across a
/// commit-heavy loop, so a stale row can never alias a fresh capture at the same length.
#[test]
fn release_keeps_the_mark_stack_at_live_captures() {
  type Ctx<'inp> = (VerboseSink<'inp>, DefaultCache<'inp, MiniLexer<'inp>>);
  let mut sink = verbose_sink();
  let mut input = Input::<'_, MiniLexer<'_>, Ctx<'_>, ()>::new("abcdef");
  {
    let mut inp = input.as_ref(&mut sink);

    // T3's alias shape first: a declined attempt and a committed attempt capture at the
    // SAME buffer length (the u64s alias); the stack must spend each capture at its
    // settle, leaving no row either time.
    let declined: Option<()> = inp.attempt(|inp| {
      let _ = inp.next();
      None
    });
    assert!(declined.is_none());
    let committed: Option<()> = inp.attempt(|inp| inp.next().ok().flatten().map(|_| ()));
    assert!(committed.is_some());

    // Then the commit-heavy loop.
    for _ in 0..4 {
      let _ = inp.attempt(|inp| inp.next().ok().flatten().map(|_| ()));
    }
  }
  assert_eq!(
    sink.rows_len(),
    0,
    "kept captures must be released (T3: a stranded row is a stale alias for the next \
     same-length capture)"
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// F-A1 — the orphan finish is detected at cause (debug) and never absorbed
// ═══════════════════════════════════════════════════════════════════════════════

/// A finish with no open node above the innermost live capture is the combo-C shape at
/// the moment it happens: detect at cause in debug builds.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "no open node")]
fn orphan_finish_debug_asserts_at_emission() {
  let mut sink = verbose_sink();
  sink.cst_token(&MiniTok(b'a'), &span(0, 1));
  sink.cst_finish();
}

/// The same shape across a live checkpoint: the start was rolled back, the leaked finish
/// would close the enclosing node — the innermost-live-row baseline catches it.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "no open node")]
fn finish_crossing_a_live_capture_debug_asserts() {
  let mut sink = verbose_sink();
  sink.cst_start(K_NODE);
  let ckp = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  sink.cst_start(K_LIST);
  rewind(&mut sink, ckp);
  let _ckp2 = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  // The List start died with the rewind; this finish would cross the live capture and
  // close K_NODE instead.
  sink.cst_finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// The unified log — one mark governs both channels
// ═══════════════════════════════════════════════════════════════════════════════

/// Rewind recovers the inner emitter's state from the last surviving Diag slot — exactly
/// the diagnostics recorded below the mark survive, on values not guesses.
#[test]
fn rewind_recovers_the_inner_mark_from_diag_slots() {
  let mut sink = verbose_sink();
  emit_error(&mut sink, 0, 1);
  let ckp = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  emit_error(&mut sink, 1, 2);
  emit_error(&mut sink, 2, 3);
  assert_eq!(sink.inner_ref().errors().len(), 3);

  rewind(&mut sink, ckp);
  let errors = sink.inner_ref().errors();
  assert_eq!(
    errors.values().map(|group| group.len()).sum::<usize>(),
    1,
    "exactly the pre-mark diagnostic survives"
  );
  assert!(errors.contains_key(&span(0, 1)));
}

/// With no Diag slot below the mark, recovery falls back to the sink's base reading — the
/// inner returns to its construction-time state.
#[test]
fn rewind_to_origin_recovers_the_base_inner_mark() {
  let mut sink = verbose_sink();
  emit_error(&mut sink, 0, 1);
  emit_error(&mut sink, 1, 2);
  rewind(&mut sink, 0);
  assert!(sink.inner_ref().errors().is_empty());
  assert_eq!(sink.events().len(), 0);
}

/// Record-then-propagate: the Diag slot lands on the `Err` edge too, so a fatal unwind's
/// guard-driven rewind still sees an exact log.
#[test]
fn diag_slot_lands_on_the_err_edge_too() {
  let mut sink: FatalSink<'_> = CstSink::new(Fatal::new(), map_tok, K_ERR, K_GAP);
  let verdict =
    Emitter::<MiniLexer<'_>>::emit_error(&mut sink, Spanned::new(span(0, 1), TestErr::Custom(9)));
  assert!(verdict.is_err(), "fatal emitters reject");
  assert_eq!(
    sink.events().len(),
    1,
    "the forwarded diagnostic occupies its Diag slot even when the inner verdict is Err"
  );
  assert!(matches!(sink.events()[0], Event::Diag { .. }));
}

/// Labels forward to the inner emitter (they are scope state, not emissions — no Diag
/// slot), and the inner snapshots them into its own entries as usual.
#[test]
fn labels_forward_without_diag_slots() {
  let mut sink = verbose_sink();
  Emitter::<MiniLexer<'_>>::enter_label(&mut sink, "field");
  assert_eq!(sink.events().len(), 0, "a label is not an emission");
  emit_error(&mut sink, 0, 1);
  Emitter::<MiniLexer<'_>>::exit_label(&mut sink);
  let labels = sink.inner_ref().labels();
  assert_eq!(labels[&span(0, 1)][0], std::vec!["field"]);
}

/// Out-of-contract marks clamp (the Verbose posture) instead of panicking.
#[test]
fn rewind_clamps_out_of_range_marks() {
  let mut sink = verbose_sink();
  sink.cst_token(&MiniTok(b'a'), &span(0, 1));
  rewind(&mut sink, u64::MAX);
  assert_eq!(sink.events().len(), 1, "a clamped rewind truncates nothing");
}

/// A rewind to the current length spends the capture's row but truncates nothing — the
/// era does not bump, so previously issued marks stay live.
#[test]
fn rewind_to_current_mark_is_truncation_free() {
  let mut sink = verbose_sink();
  let mark = sink.cst_mark();
  let ckp = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  assert_eq!(sink.rows_len(), 1);
  rewind(&mut sink, ckp);
  assert_eq!(sink.rows_len(), 0, "the capture was spent");
  // The mark predates the (no-op) rewind and must still spend cleanly.
  sink.cst_start_at(mark, K_WRAP);
  sink.cst_finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Checkpoint rows — the frozen depth ledger
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn checkpoint_rows_freeze_derived_depth() {
  let mut sink = verbose_sink();
  let first = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  sink.cst_start(K_NODE);
  let second = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  assert_eq!(sink.rows_len(), 2);

  // Kept captures release newest-first (the LIFO settle order).
  Emitter::<MiniLexer<'_>>::release(&mut sink, second);
  Emitter::<MiniLexer<'_>>::release(&mut sink, first);
  assert_eq!(sink.rows_len(), 0);

  // The released rows became the derived-depth floor; the balance still closes.
  sink.cst_finish();
  assert_eq!(sink.events().len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// The hole wrap — recovery skips become error nodes over the REAL tokens
// ═══════════════════════════════════════════════════════════════════════════════

/// The wrap brackets exactly the hole's buffered token events — interleaved Diag slots
/// ride inside, tokens outside the hole span stay outside.
#[test]
fn hole_wrap_brackets_the_buffered_suffix() {
  let mut sink = verbose_sink();
  // A committed token BEFORE the hole: outside the wrap.
  sink.cst_token(&MiniTok(b'x'), &span(0, 1));
  // The hole's tokens, with a crossed lexer error between them (a Diag slot).
  sink.cst_token(&MiniTok(b'a'), &span(1, 2));
  Emitter::<MiniLexer<'_>>::emit_lexer_error(&mut sink, Spanned::new(span(2, 3), MiniErr))
    .expect("verbose collects");
  sink.cst_token(&MiniTok(b'b'), &span(3, 4));

  Emitter::<MiniLexer<'_>>::emit_skipped_region(&mut sink, span(1, 4), 2)
    .expect("verbose collects");

  assert_eq!(
    sink.events(),
    &[
      Event::Token {
        kind: K_TOK,
        span: span(0, 1)
      },
      Event::StartNode {
        kind: K_ERR,
        forward_parent: None
      },
      Event::Token {
        kind: K_TOK,
        span: span(1, 2)
      },
      Event::Diag {
        inner_mark_after: 1
      },
      Event::Token {
        kind: K_TOK,
        span: span(3, 4)
      },
      Event::FinishNode,
      Event::Diag {
        inner_mark_after: 2
      },
    ]
  );
}

/// A zero-skip hole produces no node (and the crate's caller never even emits one).
#[test]
fn zero_skip_hole_makes_no_node() {
  let mut sink = verbose_sink();
  sink.cst_token(&MiniTok(b'a'), &span(0, 1));
  Emitter::<MiniLexer<'_>>::emit_skipped_region(&mut sink, span(1, 1), 0).expect("collects");
  assert_eq!(sink.events().len(), 2, "one token, one Diag — no wrap");
  assert!(matches!(sink.events()[1], Event::Diag { .. }));
}

/// A hole with no buffered token events (no auto-emission wired, or a direct call) has
/// nothing to wrap: no node, just the forwarded diagnostic.
#[test]
fn tokenless_hole_makes_no_node() {
  let mut sink = verbose_sink();
  Emitter::<MiniLexer<'_>>::emit_skipped_region(&mut sink, span(0, 4), 3).expect("collects");
  assert_eq!(sink.events().len(), 1);
  assert!(matches!(sink.events()[0], Event::Diag { .. }));
}

/// The wrap survives a later rewind like any other events: a checkpoint below the hole
/// unwinds wrap and tokens together.
#[test]
fn hole_wrap_rewinds_with_the_log() {
  let mut sink = verbose_sink();
  sink.cst_token(&MiniTok(b'x'), &span(0, 1));
  let ckp = Emitter::<MiniLexer<'_>>::checkpoint(&sink);
  sink.cst_token(&MiniTok(b'a'), &span(1, 2));
  sink.cst_token(&MiniTok(b'b'), &span(2, 3));
  Emitter::<MiniLexer<'_>>::emit_skipped_region(&mut sink, span(1, 3), 2).expect("collects");
  assert_eq!(sink.events().len(), 6);
  rewind(&mut sink, ckp);
  assert_eq!(sink.events().len(), 1, "wrap, tokens, and Diag all unwind");
}

// ═══════════════════════════════════════════════════════════════════════════════
// The &mut threading shape — events flow through the blanket impl
// ═══════════════════════════════════════════════════════════════════════════════

/// The parse_partial round-threading configuration: a `&mut CstSink<E>` in the emitter
/// seat. Every event lands in the sink through the blanket forward.
#[test]
fn mut_ref_sink_records_events() {
  let mut sink = verbose_sink();
  {
    let mut threaded: &mut VerboseSink<'_> = &mut sink;
    CstEmitter::<MiniLexer<'_>>::cst_start(&mut threaded, K_NODE);
    let tok = MiniTok(b'a');
    let sp = span(0, 1);
    CstEmitter::<MiniLexer<'_>>::cst_token(&mut threaded, &tok, &sp);
    let mark = CstEmitter::<MiniLexer<'_>>::cst_mark(&mut threaded);
    CstEmitter::<MiniLexer<'_>>::cst_start_at(&mut threaded, mark, K_WRAP);
    CstEmitter::<MiniLexer<'_>>::cst_finish(&mut threaded);
    CstEmitter::<MiniLexer<'_>>::cst_finish(&mut threaded);
  }
  assert_eq!(sink.events().len(), 6);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CST_FORWARD_CENSUS — the source lock on the one-helper discipline
// ═══════════════════════════════════════════════════════════════════════════════

/// Counts occurrences of `needle` on the non-comment lines of the sink source.
fn count(hay: &str, needle: &str) -> usize {
  hay
    .lines()
    .filter(|line| !line.trim_start().starts_with("//"))
    .map(|line| line.matches(needle).count())
    .sum()
}

/// CST_FORWARD_CENSUS — every forwarded diagnostic of every implemented emitter trait
/// routes through the ONE helper (`forward_diag`), which records the Diag slot on Ok and
/// Err alike. A new atomic emitter trait that bypasses it is the one bug class this law
/// exists to prevent: its diagnostics would reach the inner emitter without occupying a
/// log position, skewing every later rewind recovery.
#[test]
fn cst_forward_census_one_helper_carries_every_channel() {
  let src = include_str!("../sink.rs");

  // The forwarded channels: 5 core Emitter + TooFew/TooMany/FullContainer +
  // SeparatedEmitter (2) + the 4 leading/trailing refinements + PrattEmitter (2).
  let calls = count(src, "self.forward_diag::<");
  assert!(
    calls == 16,
    "CST_FORWARD_CENSUS drift: {calls} forward_diag call sites, expected 16. A new \
     forwarded channel must route through the one helper AND bump this census in the \
     same commit (grep CST_FORWARD_CENSUS)."
  );
  assert!(
    count(src, "fn forward_diag") == 1,
    "CST_FORWARD_CENSUS drift: the helper must be defined exactly once"
  );

  // No emit bypasses the helper: the only `self.inner` touches are the helper's own
  // closure seam, the two label scope calls, the rewind recovery, and the read-only
  // reaches (base reading, accessor, Debug).
  assert!(
    count(src, "self.inner.emit") == 0,
    "CST_FORWARD_CENSUS drift: a diagnostic is forwarded outside the census helper"
  );
  assert!(
    count(src, "self.inner.rewind(") == 1,
    "CST_FORWARD_CENSUS drift: the inner emitter rewinds exactly once, in the sink's \
     rewind recovery"
  );
  assert!(
    count(src, "self.inner.enter_label(") == 1 && count(src, "self.inner.exit_label(") == 1,
    "CST_FORWARD_CENSUS drift: labels forward directly (scope state, not emissions) — \
     exactly once each"
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// The forwarding matrix — CstSink satisfies every bound its inner emitter does
// ═══════════════════════════════════════════════════════════════════════════════

/// The `ComposableEmitter`-shaped conformance (consumers R5): a context bound naming the
/// full emitter trait family — core + the six atomic capability traits + the separated
/// refinements + pratt — accepts `CstSink<E>` (and `&mut CstSink<E>`, the parse_partial
/// threading shape) wherever it accepts `E`.
#[test]
fn sink_satisfies_the_full_emitter_family() {
  use crate::emitter::{
    FullContainerEmitter, MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter,
    PrattEmitter, SeparatedEmitter, TooFewEmitter, TooManyEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  };

  fn composable<'inp, T>(_: &T)
  where
    T: Emitter<'inp, MiniLexer<'inp>>
      + CstEmitter<'inp, MiniLexer<'inp>>
      + TooFewEmitter<'inp, MiniLexer<'inp>>
      + TooManyEmitter<'inp, MiniLexer<'inp>>
      + FullContainerEmitter<'inp, MiniLexer<'inp>>
      + SeparatedEmitter<'inp, MiniLexer<'inp>>
      + MissingLeadingSeparatorEmitter<'inp, MiniLexer<'inp>>
      + MissingTrailingSeparatorEmitter<'inp, MiniLexer<'inp>>
      + UnexpectedLeadingSeparatorEmitter<'inp, MiniLexer<'inp>>
      + UnexpectedTrailingSeparatorEmitter<'inp, MiniLexer<'inp>>
      + PrattEmitter<'inp, MiniLexer<'inp>>,
  {
  }

  let mut sink = verbose_sink();
  composable(&sink);
  composable(&&mut sink);
}
