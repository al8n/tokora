//! Partial-input (Sans-I/O) frontier-rule tests.
//!
//! Each of the three conservative rules at the scan chokepoint gets a focused case: frontier
//! holdback (a token touching the buffer end), frontier error (a lexer error touching the buffer
//! end), and non-final EOF. Plus the two boundary properties: `is_final == true` behaves exactly
//! like a complete parse, and a *mid-buffer* token or error (strictly before the buffer end) is
//! yielded / emitted normally even while partial.
//!
//! The last section covers the **terminal-dominance law**: a limit trip whose tripping token ends
//! exactly at a non-final buffer end is *not* a frontier item to be withheld — no refill can
//! un-trip a limiter, so the trip fires (diagnostic emitted, poison boundary latched) instead of
//! surfacing `Incomplete`. Its dual is the rule it must not swallow: a genuinely *non-terminal*
//! error at the same frontier is still withheld.

use core::cell::Cell;
use std::rc::Rc;

use crate::{
  InputRef, Parse, Parser, Token,
  cache::DefaultCache,
  emitter::{Fatal, Verbose},
  error::{Incomplete, MaybeIncomplete, token::UnexpectedToken},
  input::{Complete, Input, Partial, SurfaceIncomplete, parse_partial},
  lexer::LogosLexer,
  state::State,
};

// An error type that can carry the partial-input incomplete sentinel. `From<Incomplete>` is the
// exact construction path the frontier rules use (via `SurfaceIncomplete`), and `is_incomplete()`
// is what recovery keys the never-recoverable law off — the two must stay coherent.
#[derive(Debug, Clone, PartialEq)]
enum PErr {
  Lex,
  /// A resource-limit trip — the terminal outcome no amount of further input can clear.
  Limit,
  Incomplete(usize),
}

impl From<()> for PErr {
  fn from(_: ()) -> Self {
    PErr::Lex
  }
}

impl From<LimitExceeded> for PErr {
  fn from(_: LimitExceeded) -> Self {
    PErr::Limit
  }
}

impl From<Incomplete<usize>> for PErr {
  fn from(inc: Incomplete<usize>) -> Self {
    PErr::Incomplete(inc.into_offset())
  }
}

impl MaybeIncomplete for PErr {
  fn is_incomplete(&self) -> bool {
    matches!(self, PErr::Incomplete(_))
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for PErr {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    PErr::Lex
  }
}

#[derive(Debug, Clone, PartialEq, crate::logos::Logos)]
#[logos(crate = crate::logos, skip r"[ \t\r\n]+")]
enum PTok {
  #[regex(r"[a-z]+")]
  Word,
  #[regex(r"[0-9]+")]
  Num,
}

impl core::fmt::Display for PTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(match self {
      PTok::Word => "word",
      PTok::Num => "number",
    })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PKind {
  Word,
  Num,
}

impl core::fmt::Display for PKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(match self {
      PKind::Word => "word",
      PKind::Num => "number",
    })
  }
}

impl Token<'_> for PTok {
  type Kind = PKind;
  type Error = ();

  fn kind(&self) -> PKind {
    match self {
      PTok::Word => PKind::Word,
      PTok::Num => PKind::Num,
    }
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

type Lex<'a> = LogosLexer<'a, PTok>;
type PartialCtx<'a> = (Verbose<PErr>, DefaultCache<'a, Lex<'a>>);
type CompleteCtx<'a> = (Verbose<PErr>, DefaultCache<'a, Lex<'a>>);

/// The observable outcome of draining an input to its first stop: the yielded token kinds, the
/// terminating result (`Ok(None)` for genuine end of input, `Err` otherwise), and how many
/// diagnostics the emitter collected.
struct Run {
  kinds: std::vec::Vec<PKind>,
  result: Result<Option<()>, PErr>,
  emitted: usize,
}

/// Drives a **partial** input over `src` with the given `is_final`, draining `next()` to its first
/// stop.
fn run_partial(src: &str, is_final: bool) -> Run {
  let mut input = Input::<Lex<'_>, PartialCtx<'_>, (), Partial>::with_state_and_cache(
    src,
    (),
    DefaultCache::<'_, Lex<'_>>::default(),
  );
  if is_final {
    input.seal();
  }
  let mut emitter = Verbose::<PErr>::new();
  let (kinds, result) = {
    let mut inp = input.as_ref(&mut emitter);
    let mut kinds = std::vec::Vec::new();
    let result = loop {
      match inp.next() {
        Ok(Some(t)) => kinds.push(t.data().kind()),
        Ok(None) => break Ok(None),
        Err(e) => break Err(e),
      }
    };
    (kinds, result)
  };
  let emitted = emitter.errors().values().map(|g| g.len()).sum();
  Run {
    kinds,
    result,
    emitted,
  }
}

/// Drives a **complete** input over `src` — the oracle the `is_final == true` partial run must
/// match.
fn run_complete(src: &str) -> Run {
  let mut input = Input::<Lex<'_>, CompleteCtx<'_>, (), Complete>::with_state_and_cache(
    src,
    (),
    DefaultCache::<'_, Lex<'_>>::default(),
  );
  let mut emitter = Verbose::<PErr>::new();
  let (kinds, result) = {
    let mut inp = input.as_ref(&mut emitter);
    let mut kinds = std::vec::Vec::new();
    let result = loop {
      match inp.next() {
        Ok(Some(t)) => kinds.push(t.data().kind()),
        Ok(None) => break Ok(None),
        Err(e) => break Err(e),
      }
    };
    (kinds, result)
  };
  let emitted = emitter.errors().values().map(|g| g.len()).sum();
  Run {
    kinds,
    result,
    emitted,
  }
}

// ── Rule 1: frontier holdback ───────────────────────────────────────────────────────

#[test]
fn holdback_token_touching_buffer_end() {
  // "foo" is one token spanning the whole buffer (0..3), so its end touches the buffer end.
  // Non-final: it may be a prefix of a longer word, so it is withheld and Incomplete surfaces.
  let run = run_partial("foo", false);
  assert!(run.kinds.is_empty(), "the frontier token is not yielded");
  assert_eq!(
    run.result,
    Err(PErr::Incomplete(3)),
    "Incomplete carries the frontier offset (the buffer end)"
  );
  assert!(
    run.result.unwrap_err().is_incomplete(),
    "the surfaced error reports itself incomplete (the never-recoverable law keys off this)"
  );
  assert_eq!(run.emitted, 0, "holdback emits nothing");
}

// ── Rule 2: frontier error ──────────────────────────────────────────────────────────

#[test]
fn holdback_error_touching_buffer_end() {
  // "foo @" — after the mid-buffer word "foo", the "@" is a lexer error at 4..5 whose span touches
  // the buffer end. Non-final: it may be a truncation artifact, so it is neither emitted nor
  // surfaced as an error — Incomplete surfaces instead.
  let run = run_partial("foo @", false);
  assert_eq!(
    run.kinds,
    std::vec![PKind::Word],
    "the mid-buffer word yields"
  );
  assert_eq!(
    run.result,
    Err(PErr::Incomplete(5)),
    "the frontier error surfaces Incomplete at the buffer end, not the lexer error"
  );
  assert_eq!(
    run.emitted, 0,
    "the frontier error is held back, not emitted"
  );
}

// ── Rule 3: non-final EOF ───────────────────────────────────────────────────────────

#[test]
fn nonfinal_eof_surfaces_incomplete() {
  // "foo " — "foo" ends at 3, strictly before the buffer end 4 (a trailing space), so it is NOT a
  // frontier token and yields normally. The whitespace tail then exhausts the lexer at a non-final
  // EOF, which surfaces Incomplete rather than genuine end of input.
  let run = run_partial("foo ", false);
  assert_eq!(
    run.kinds,
    std::vec![PKind::Word],
    "the mid-buffer token (end < buffer end) yields normally"
  );
  assert!(
    matches!(run.result, Err(PErr::Incomplete(_))),
    "a non-final EOF is Incomplete, not Ok(None)"
  );
  assert_eq!(run.emitted, 0);
}

#[test]
fn nonfinal_eof_on_empty_buffer() {
  // An empty non-final chunk is entirely Incomplete: nothing to yield, more may arrive.
  let run = run_partial("", false);
  assert!(run.kinds.is_empty());
  assert_eq!(run.result, Err(PErr::Incomplete(0)));
}

// ── Mid-buffer items are unaffected while partial ─────────────────────────────────────

#[test]
fn mid_buffer_error_is_emitted_normally_while_partial() {
  // "foo @ bar" non-final: "foo" (0..3) yields, the "@" error (4..5) is *mid-buffer* (before the
  // end) so it is emitted and skipped exactly as in complete mode, and only "bar" (6..9, touching
  // the end) is held back → Incomplete. The mid-buffer error must still reach the emitter.
  let run = run_partial("foo @ bar", false);
  assert_eq!(run.kinds, std::vec![PKind::Word]);
  assert_eq!(
    run.result,
    Err(PErr::Incomplete(9)),
    "the trailing word touches the end and is held back"
  );
  assert_eq!(
    run.emitted, 1,
    "the mid-buffer lexer error is emitted normally in partial mode"
  );
}

// ── `is_final == true` is exact parity with a complete parse ─────────────────────────

#[test]
fn is_final_matches_complete() {
  // With is_final == true, a partial input behaves exactly like a complete one: every token is
  // yielded (the frontier holdback is off), a trailing error is emitted, and EOF is genuine.
  for src in ["foo", "foo bar baz", "foo @ bar", "12 ab 34", "", "x"] {
    let partial = run_partial(src, true);
    let complete = run_complete(src);
    assert_eq!(
      partial.kinds, complete.kinds,
      "final partial and complete yield the same tokens for {src:?}"
    );
    assert_eq!(
      partial.result, complete.result,
      "final partial and complete end the same way for {src:?}"
    );
    assert_eq!(
      partial.emitted, complete.emitted,
      "final partial and complete emit the same diagnostics for {src:?}"
    );
  }
}

// ── The complete path is untouched: it never surfaces Incomplete ─────────────────────

#[test]
fn complete_never_surfaces_incomplete() {
  // The same "foo" that a non-final partial holds back is a genuine, whole token in complete mode.
  let run = run_complete("foo");
  assert_eq!(run.kinds, std::vec![PKind::Word]);
  assert_eq!(run.result, Ok(None), "complete mode reaches genuine EOF");
}

// ── Exhaustive chunked-equivalence oracle over every split point ──────────────────────

/// A full observation of a drain: each yielded token as `(kind, start, end)`, each emitted lexer
/// error's `(start, end)` in span order, and the terminating result.
struct Trace {
  tokens: std::vec::Vec<(PKind, usize, usize)>,
  errors: std::vec::Vec<(usize, usize)>,
  result: Result<Option<()>, PErr>,
}

/// Drains a partial input over `src` at the given `is_final`, capturing the full [`Trace`].
fn trace_partial(src: &str, is_final: bool) -> Trace {
  let mut input = Input::<Lex<'_>, PartialCtx<'_>, (), Partial>::with_state_and_cache(
    src,
    (),
    DefaultCache::<'_, Lex<'_>>::default(),
  );
  if is_final {
    input.seal();
  }
  let mut emitter = Verbose::<PErr>::new();
  let (tokens, result) = {
    let mut inp = input.as_ref(&mut emitter);
    let mut tokens = std::vec::Vec::new();
    let result = loop {
      match inp.next() {
        Ok(Some(t)) => tokens.push((t.data().kind(), *t.span().start_ref(), *t.span().end_ref())),
        Ok(None) => break Ok(None),
        Err(e) => break Err(e),
      }
    };
    (tokens, result)
  };
  let errors = collect_errors(&emitter);
  Trace {
    tokens,
    errors,
    result,
  }
}

/// Drains a complete input over `src`, capturing the full [`Trace`] — the oracle a chunked partial
/// run is checked against.
fn trace_complete(src: &str) -> Trace {
  let mut input = Input::<Lex<'_>, CompleteCtx<'_>, (), Complete>::with_state_and_cache(
    src,
    (),
    DefaultCache::<'_, Lex<'_>>::default(),
  );
  let mut emitter = Verbose::<PErr>::new();
  let (tokens, result) = {
    let mut inp = input.as_ref(&mut emitter);
    let mut tokens = std::vec::Vec::new();
    let result = loop {
      match inp.next() {
        Ok(Some(t)) => tokens.push((t.data().kind(), *t.span().start_ref(), *t.span().end_ref())),
        Ok(None) => break Ok(None),
        Err(e) => break Err(e),
      }
    };
    (tokens, result)
  };
  let errors = collect_errors(&emitter);
  Trace {
    tokens,
    errors,
    result,
  }
}

/// Collects every emitted lexer error's `(start, end)` in span order from a verbose emitter.
fn collect_errors(emitter: &Verbose<PErr>) -> std::vec::Vec<(usize, usize)> {
  emitter
    .errors()
    .iter()
    .flat_map(|(span, group)| {
      let se = (*span.start_ref(), *span.end_ref());
      group.iter().map(move |_| se)
    })
    .collect()
}

/// The correctness oracle (crate-side, exhaustive): for **every** split point `k` of each corpus
/// string, a non-final partial drain of the prefix `src[0..k]` must
///
/// 1. yield exactly the complete-parse tokens that lie strictly before `k` (the frontier holdback
///    withholds the one touching the cut),
/// 2. emit exactly the complete-parse lexer errors that lie strictly before `k` (the frontier error
///    is held back), and
/// 3. always terminate with an `Incomplete` (a non-final drain never reports genuine end of input),
///
/// while a *final* drain of the whole string reproduces the complete parse exactly (the "complete
/// over the full input" leg of the resumption loop). Together these are the chunked-equivalence
/// guarantee: reassembling the chunk-by-chunk prefixes yields the same tokens and emission log as a
/// single complete parse.
#[test]
fn chunked_equivalence_over_every_split_point() {
  // A corpus mixing words, numbers, trailing/leading/interior whitespace, and lexer errors (`@`).
  const CORPUS: &[&str] = &[
    "",
    "a",
    "foo bar baz",
    "12 ab 345 cd",
    "  lead",
    "trail  ",
    "ab@cd",
    "foo @ bar @ baz",
    "a b c d e f",
    "x1 y2 z3",
  ];

  for &src in CORPUS {
    let complete = trace_complete(src);

    // The "complete over the full input" leg: a final partial drain equals the complete parse.
    let final_partial = trace_partial(src, true);
    assert_eq!(
      final_partial.tokens, complete.tokens,
      "final partial tokens must equal complete for {src:?}"
    );
    assert_eq!(
      final_partial.errors, complete.errors,
      "final partial emission log must equal complete for {src:?}"
    );
    assert_eq!(
      final_partial.result, complete.result,
      "final partial terminal must equal complete for {src:?}"
    );

    for k in 0..=src.len() {
      if !src.is_char_boundary(k) {
        continue;
      }
      let prefix = trace_partial(&src[..k], false);

      let expected_tokens: std::vec::Vec<_> = complete
        .tokens
        .iter()
        .copied()
        .filter(|&(_, _, end)| end < k)
        .collect();
      assert_eq!(
        prefix.tokens, expected_tokens,
        "prefix tokens diverge from the complete prefix for {src:?} at k={k}"
      );

      let expected_errors: std::vec::Vec<_> = complete
        .errors
        .iter()
        .copied()
        .filter(|&(_, end)| end < k)
        .collect();
      assert_eq!(
        prefix.errors, expected_errors,
        "prefix emission log diverges from the complete prefix for {src:?} at k={k}"
      );

      match &prefix.result {
        Err(e) => assert!(
          e.is_incomplete(),
          "a non-final prefix must terminate Incomplete for {src:?} at k={k}, got {e:?}"
        ),
        Ok(none) => panic!(
          "a non-final prefix never reports genuine end of input for {src:?} at k={k}, got Ok({none:?})"
        ),
      }
    }
  }
}

// ── The terminal-dominance law: a trip at the frontier is NOT an incomplete ───────────
//
// The three frontier rules withhold an item that later input could still change. A limit trip is
// not such an item: a limiter's tally is monotone, so no refill can un-trip it. The tripping token
// landing exactly on the buffer end must therefore fire the limit — diagnostic emitted, poison
// boundary latched — rather than surface `Incomplete` and invite the caller to feed more bytes to a
// limit that will never fire (the streaming DoS).

/// A limiter whose tally is **shared** across every lexer the input rebuilds (an `Rc<Cell<_>>`,
/// exactly as `tests::ProbeLimiter`), so a test can watch whether the tripping token was scanned at
/// all — and, after the latch, that it is never rescanned.
#[derive(Debug, Clone)]
struct LimitTracker {
  scanned: Rc<Cell<usize>>,
  limit: usize,
}

impl Default for LimitTracker {
  fn default() -> Self {
    // A limit-free default: only an explicitly constructed tracker ever trips.
    Self::with_limit(usize::MAX)
  }
}

impl LimitTracker {
  fn with_limit(limit: usize) -> Self {
    Self {
      scanned: Rc::new(Cell::new(0)),
      limit,
    }
  }

  /// A shared handle on the scan counter, kept after the state is moved into the input.
  fn counter(&self) -> Rc<Cell<usize>> {
    self.scanned.clone()
  }

  fn increase(&self) {
    self.scanned.set(self.scanned.get() + 1);
  }
}

#[derive(Debug, Clone, PartialEq)]
struct LimitExceeded;

impl State for LimitTracker {
  type Error = LimitExceeded;

  fn check(&self) -> Result<(), Self::Error> {
    if self.scanned.get() > self.limit {
      Err(LimitExceeded)
    } else {
      Ok(())
    }
  }
}

/// A word lexer behind the limiter: every word bumps the tally, so the `(limit + 1)`-th word is the
/// tripping token — the Logos backend turns the post-token `check()` failure into a `Lexed::Error`
/// carrying that token's span. `@` is a plain (non-limit) lexer error, for the dual case.
#[derive(Debug, Clone, PartialEq, crate::logos::Logos)]
#[logos(crate = crate::logos, extras = LimitTracker, skip r"[ \t\r\n]+")]
enum LTok {
  #[regex(r"[a-z]+", |lex| { lex.extras.increase(); })]
  Word,
}

impl core::fmt::Display for LTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("word")
  }
}

impl Token<'_> for LTok {
  type Kind = PKind;
  type Error = PErr;

  fn kind(&self) -> PKind {
    PKind::Word
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

type LimLex<'a> = LogosLexer<'a, LTok>;
type LimCtx<'a> = (Verbose<PErr>, DefaultCache<'a, LimLex<'a>>);
type LimFatalCtx<'a> = (Fatal<PErr>, DefaultCache<'a, LimLex<'a>>);

/// What a limited partial drain is judged on: the yielded tokens, how it terminated, how many limit
/// diagnostics reached the emitter, and whether the input latched its poison boundary.
struct LimRun {
  kinds: std::vec::Vec<PKind>,
  result: Result<Option<()>, PErr>,
  limit_diags: usize,
  poisoned: bool,
}

/// Counts the limit diagnostics a verbose emitter collected.
fn limit_diags(emitter: &Verbose<PErr>) -> usize {
  emitter
    .errors()
    .values()
    .flatten()
    .filter(|e| **e == PErr::Limit)
    .count()
}

/// Drains `next()` over a **partial** input behind a `limit`-token limiter.
fn run_limited(src: &str, limit: usize, is_final: bool) -> LimRun {
  let tracker = LimitTracker::with_limit(limit);
  let mut input = Input::<LimLex<'_>, LimCtx<'_>, (), Partial>::with_state_and_cache(
    src,
    tracker,
    DefaultCache::<'_, LimLex<'_>>::default(),
  );
  if is_final {
    input.seal();
  }
  let mut emitter = Verbose::<PErr>::new();
  let (kinds, result, poisoned) = {
    let mut inp = input.as_ref(&mut emitter);
    let mut kinds = std::vec::Vec::new();
    let result = loop {
      match inp.next() {
        Ok(Some(t)) => kinds.push(t.data().kind()),
        Ok(None) => break Ok(None),
        Err(e) => break Err(e),
      }
    };
    (kinds, result, inp.is_poisoned())
  };
  LimRun {
    kinds,
    result,
    limit_diags: limit_diags(&emitter),
    poisoned,
  }
}

#[test]
fn frontier_limit_trip_is_terminal_not_incomplete() {
  // "a b c": three words; the limiter trips on the third. Its span is 4..5 — its end IS the buffer
  // end (len 5) — so it sits exactly on the non-final frontier, the alignment an attacker picks.
  //
  // Before the ordering fix the frontier holdback ran first and returned Incomplete here: no
  // diagnostic, no latch, and a streaming caller told to feed MORE bytes to a limit that had
  // already been exceeded.
  let run = run_limited("a b c", 2, false);

  assert_eq!(
    run.kinds,
    std::vec![PKind::Word, PKind::Word],
    "the two tokens under the limit yield"
  );
  assert_eq!(
    run.result,
    Ok(None),
    "the trip is TERMINAL: it stops the scan, and never masquerades as Incomplete"
  );
  assert!(
    !matches!(run.result, Err(ref e) if e.is_incomplete()),
    "a terminal trip must never surface on the Incomplete channel"
  );
  assert_eq!(
    run.limit_diags, 1,
    "the limit diagnostic IS emitted, even though the tripping token touches the buffer end"
  );
  assert!(
    run.poisoned,
    "the poison boundary IS latched, so no later operation can rescan past the trip"
  );
}

#[test]
fn frontier_limit_trip_is_terminal_on_the_peek_fill() {
  // The same alignment, reached through `peek` instead of `next`: the peek fill lexes the tripping
  // token at the frontier. A peek that merely withheld it would latch nothing and emit nothing, and
  // the caller's follow-up `next()` would then be told Incomplete — the same mask, one path over.
  let tracker = LimitTracker::with_limit(2);
  let scanned = tracker.counter();
  let mut input = Input::<LimLex<'_>, LimCtx<'_>, (), Partial>::with_state_and_cache(
    "a b c",
    tracker,
    DefaultCache::<'_, LimLex<'_>>::default(),
  );
  let mut emitter = Verbose::<PErr>::new();
  let (peeked, poisoned, after) = {
    use generic_arraydeque::typenum::U4;

    // Born open: a fresh `Partial` input is non-final until a driver seals it.
    let mut inp = input.as_ref(&mut emitter);

    // A window of 4 over a 3-token source: the fill runs into the tripping token.
    let peeked = inp
      .peek::<U4>()
      .expect("a peek never surfaces Incomplete")
      .len();
    let poisoned = inp.is_poisoned();

    // Draining afterwards must reach the same terminal stop — never Incomplete.
    let mut kinds = std::vec::Vec::new();
    let after = loop {
      match inp.next() {
        Ok(Some(t)) => kinds.push(t.data().kind()),
        Ok(None) => break Ok(kinds),
        Err(e) => break Err(e),
      }
    };
    (peeked, poisoned, after)
  };

  assert_eq!(peeked, 2, "the peek serves the two tokens under the limit");
  assert!(
    poisoned,
    "the peek fill latched the poison boundary on the frontier trip"
  );
  assert_eq!(
    limit_diags(&emitter),
    1,
    "the peek fill emitted the limit diagnostic rather than silently withholding it"
  );
  assert_eq!(
    after,
    Ok(std::vec![PKind::Word, PKind::Word]),
    "the follow-up drain serves the cached prefix and stops TERMINALLY, not Incomplete"
  );
  assert_eq!(
    scanned.get(),
    3,
    "the tripping token was scanned exactly once; the latch stops it being rescanned"
  );
}

#[test]
fn frontier_nonterminal_error_is_still_withheld() {
  // The rule the fix must NOT break. `@` is a plain lexer error at 2..3, touching the buffer end,
  // and the limiter has NOT tripped (limit 5, one word scanned). A truncated buffer really can make
  // a valid token look like a lex error, so this one IS withheld: Incomplete, nothing emitted,
  // nothing latched. The holdback was narrowed to non-terminal items, not removed.
  let run = run_limited("a @", 5, false);

  assert_eq!(
    run.kinds,
    std::vec![PKind::Word],
    "the mid-buffer word yields"
  );
  assert_eq!(
    run.result,
    Err(PErr::Incomplete(3)),
    "a NON-terminal frontier error is still held back as Incomplete"
  );
  assert_eq!(run.limit_diags, 0, "no limit was tripped");
  assert!(
    !run.poisoned,
    "a non-terminal frontier error latches no poison boundary"
  );
}

#[test]
fn mid_buffer_limit_trip_is_unaffected() {
  // The control: the same trip, but with the tripping token strictly before the buffer end (a
  // trailing space). This always worked — it is the alignment the attacker avoids — and it must
  // keep behaving identically to the frontier case, which is the point of the law.
  let run = run_limited("a b c ", 2, false);

  assert_eq!(run.kinds, std::vec![PKind::Word, PKind::Word]);
  assert_eq!(run.result, Ok(None), "the trip stops the scan");
  assert_eq!(run.limit_diags, 1);
  assert!(run.poisoned);
}

#[test]
fn final_and_complete_agree_with_the_frontier_trip() {
  // A final partial drain and the frontier-trip case observe the same terminal facts: `is_final`
  // changes nothing about a trip, because a trip was never about the frontier.
  let fin = run_limited("a b c", 2, true);
  assert_eq!(fin.kinds, std::vec![PKind::Word, PKind::Word]);
  assert_eq!(fin.result, Ok(None));
  assert_eq!(fin.limit_diags, 1);
  assert!(fin.poisoned);
}

// ── The driver-level DoS: the refill loop must TERMINATE ───────────────────────────────

/// The parser every driver round runs: drain to the first stop, counting tokens. `?` propagates
/// whatever the input surfaces — an `Incomplete` (refill and re-drive) or a terminal error (stop).
fn drain_all<'inp>(
  inp: &mut InputRef<'inp, '_, LimLex<'inp>, LimCtx<'inp>, (), Partial>,
) -> Result<usize, PErr> {
  let mut n = 0usize;
  while inp.next()?.is_some() {
    n += 1;
  }
  Ok(n)
}

/// The same drain under a **fatal** emitter, which rejects the limit diagnostic instead of
/// recovering from it.
fn drain_all_fatal<'inp>(
  inp: &mut InputRef<'inp, '_, LimLex<'inp>, LimFatalCtx<'inp>, (), Partial>,
) -> Result<usize, PErr> {
  let mut n = 0usize;
  while inp.next()?.is_some() {
    n += 1;
  }
  Ok(n)
}

/// One round of the documented Sans-I/O refill loop over `buffer`: build a `Partial` input, drain
/// it, and report what the caller would see.
fn drive_round(buffer: &str, limit: usize) -> Result<usize, PErr> {
  let ctx: LimCtx<'_> = (
    Verbose::<PErr>::new(),
    DefaultCache::<'_, LimLex<'_>>::default(),
  );
  parse_partial(
    ctx,
    buffer,
    LimitTracker::with_limit(limit),
    /* is_final */ false,
    drain_all,
  )
}

#[test]
fn parse_partial_refill_loop_terminates_on_a_frontier_trip() {
  // THE denial-of-service, driven end to end. The attacker extends the tripping token one byte at a
  // time, so it ends exactly at the buffer end on EVERY round: "a b c", "a b cc", "a b ccc", …
  //
  // If a frontier trip surfaced as Incomplete, each round would say "send more" — the caller would
  // refill forever, re-lexing an ever-growing buffer, and the token limit would never fire. The
  // loop below caps the rounds and fails if the cap is reached: it must stop on round ONE.
  let mut buffer = std::string::String::from("a b c");
  let mut rounds = 0usize;
  let outcome = loop {
    rounds += 1;
    assert!(
      rounds <= 8,
      "the refill loop never terminated: a frontier limit trip was masked as Incomplete, so the \
       limit never fired and an attacker aligned to the chunk boundary drives an unbounded refill"
    );
    match drive_round(&buffer, 2) {
      // The caller's own refill step: append the attacker's next byte and re-drive.
      Err(ref e) if e.is_incomplete() => buffer.push('c'),
      other => break other,
    }
  };

  assert_eq!(rounds, 1, "the trip terminates the loop on the FIRST round");
  assert_eq!(
    outcome,
    Ok(2),
    "the recovering emitter turns the trip into a bounded stop after the two tokens under the limit"
  );
}

#[test]
fn parse_partial_frontier_trip_is_fatal_under_a_fatal_emitter() {
  // The same round under a `Fatal` emitter: the limit diagnostic is *rejected*, so the trip leaves
  // on the `Err` channel — as `PErr::Limit`, which reports itself NOT incomplete, so the refill
  // loop stops instead of asking for bytes that cannot help.
  let ctx: LimFatalCtx<'_> = (Fatal::of(), DefaultCache::<'_, LimLex<'_>>::default());
  let out: Result<usize, PErr> = parse_partial(
    ctx,
    "a b c",
    LimitTracker::with_limit(2),
    /* is_final */ false,
    drain_all_fatal,
  );

  assert_eq!(
    out,
    Err(PErr::Limit),
    "the fatal emitter surfaces the limit trip itself, not an Incomplete"
  );
  assert!(
    !out.unwrap_err().is_incomplete(),
    "a terminal trip never reports itself incomplete — the refill loop must not retry it"
  );
}

#[test]
fn refill_loop_still_refills_on_a_genuine_incomplete() {
  // The counterweight: an ordinary truncated word DOES resume. The loop refills until the word is
  // whole and the input is marked final — proving the fix narrowed the holdback to terminal items
  // rather than turning every frontier item into a stop.
  let chunks = ["foo b", "ar ", "baz"];
  let mut buffer = std::string::String::new();
  let mut incompletes = 0usize;
  let mut parsed = None;
  for (i, chunk) in chunks.iter().enumerate() {
    buffer.push_str(chunk);
    let is_final = i + 1 == chunks.len();
    let ctx: LimCtx<'_> = (
      Verbose::<PErr>::new(),
      DefaultCache::<'_, LimLex<'_>>::default(),
    );
    let out = parse_partial(
      ctx,
      buffer.as_str(),
      LimitTracker::with_limit(usize::MAX),
      is_final,
      drain_all,
    );
    match out {
      Ok(n) => {
        parsed = Some(n);
        break;
      }
      Err(e) if e.is_incomplete() => incompletes += 1,
      Err(e) => panic!("unexpected error {e:?}"),
    }
  }
  assert_eq!(
    parsed,
    Some(3),
    "the three words parse once the input is final"
  );
  assert_eq!(
    incompletes, 2,
    "the two non-final chunks each cut a word at the frontier and DID ask for more"
  );
}

// ── Finality is a WORLD fact: monotone, driver-owned, and outside the rollback set ────────────
//
// The two halves of one law, and they are mirrors of each other. Break either and a streaming
// parser is wrong in a way the type system used to permit:
//
//   * a parser that could END a stream would lose the frontier holdback — speculate,
//     `set_final(true)`, fail, roll back, and the rollback would not undo it (rollback rewinds the
//     PARSE, not the WORLD), so the next read hands back a token the frontier owed an `Incomplete`
//     for. That is the leak.
//   * a rollback that could UN-END a stream — the "obvious" fix of checkpointing the flag and
//     restoring it — would leave a parser asking for a refill that can never come. That is a hang,
//     and it is strictly worse than the leak it fixes.
//
// Both bugs share the premise that a parser can touch the bit at all. It cannot: `is_final` is
// settable only through the owning `Input` (`seal`, monotone), which an `InputRef` mutably borrows
// for its whole life. The compile-fail proof of the unreachability lives on `InputRef::is_final`;
// these two are the observable laws it buys.

/// LAW: a failed speculative branch cannot cost the frontier holdback.
///
/// The source ends mid-construct (`cd` touches the buffer end of a non-final buffer), so every read
/// past `ab` owes an `Incomplete`. A parser throws the crate's entire speculative surface at the
/// input and abandons all of it. Nothing it did — at any depth, through any guard — may leave the
/// input final, and the frontier must still owe `Incomplete` afterwards.
#[test]
fn speculation_cannot_end_the_stream() {
  let mut input = Input::<Lex<'_>, PartialCtx<'_>, (), Partial>::with_state_and_cache(
    "ab cd",
    (),
    DefaultCache::<'_, Lex<'_>>::default(),
  );
  // NOT sealed: the driver has not said the stream ended, so `cd` may yet become `cdef`.
  let mut emitter = Verbose::<PErr>::new();
  let mut inp = input.as_ref(&mut emitter);
  assert!(!inp.is_final(), "a fresh partial input is born open");

  // `ab` is clear of the frontier and yields normally.
  assert!(inp.next().expect("ab is clear of the frontier").is_some());

  // Every rollback shape the crate has, all of them abandoned.
  let declined: Option<()> = inp.attempt(|i| {
    let _ = i.next();
    None
  });
  assert!(declined.is_none());

  let errored: Result<(), ()> = inp.try_attempt(|i| {
    let _ = i.next();
    Err(())
  });
  assert!(errored.is_err());

  {
    let mut txn = inp.begin();
    let _ = txn.next();
    txn.rollback();
  }
  {
    // Undecided drop under the `Rollback` policy.
    let mut txn = inp.begin();
    let _ = txn.next();
  }
  {
    let mut txn = inp.begin_stacked();
    let _ = txn.next();
    let sp = txn.savepoint();
    let _ = txn.next();
    txn.rollback_to(sp);
    txn.rollback();
  }
  inp.begin_point();
  let _ = inp.next();
  inp.rollback_point();

  // The world did not move: no parser said the stream ended, so it did not end.
  assert!(
    !inp.is_final(),
    "a speculative branch ENDED THE STREAM: the frontier holdback is gone and the next read will \
     hand back a token that later input could still extend"
  );

  // And the observable half: the frontier still owes an Incomplete for `cd`.
  match inp.next() {
    Err(PErr::Incomplete(at)) => assert_eq!(at, 5, "the frontier is the buffer end"),
    other => panic!(
      "LAW VIOLATED: a rolled-back speculative branch cost the frontier holdback — `cd` touches a \
       NON-FINAL buffer end and is owed an Incomplete, but next() yielded {other:?}"
    ),
  }
}

/// LAW (the mirror): a rollback cannot un-end a stream the driver already ended.
///
/// The driver seals — the last chunk landed, the socket is closed, there are no more bytes. A parser
/// then speculates across that fact and rolls every bit of it back. The input must still be final,
/// and the drain must reach genuine end of input: an `Incomplete` here would be a request for a
/// refill that can never be satisfied, and the caller would loop forever.
///
/// This is the test that fails on the "obvious" fix (checkpoint the finality flag, restore it on
/// rollback) — it closes the leak the sibling test guards and opens this hang in its place.
#[test]
fn rollback_cannot_un_end_a_sealed_stream() {
  let mut input = Input::<Lex<'_>, PartialCtx<'_>, (), Partial>::with_state_and_cache(
    "ab cd",
    (),
    DefaultCache::<'_, Lex<'_>>::default(),
  );
  // The world fact: the stream has ENDED. Only the driver can say this, and only here — with no
  // handle alive, which is exactly when a driver can honestly know it.
  input.seal();

  let mut emitter = Verbose::<PErr>::new();
  let mut inp = input.as_ref(&mut emitter);
  assert!(inp.is_final(), "the driver sealed the stream");

  // The same full speculative surface, all of it abandoned.
  let declined: Option<()> = inp.attempt(|i| {
    let _ = i.next();
    let _ = i.next();
    None
  });
  assert!(declined.is_none());

  let errored: Result<(), ()> = inp.try_attempt(|i| {
    let _ = i.next();
    Err(())
  });
  assert!(errored.is_err());

  {
    let mut txn = inp.begin();
    let _ = txn.next();
    txn.rollback();
  }
  {
    let mut txn = inp.begin();
    let _ = txn.next();
  }
  {
    let mut txn = inp.begin_stacked();
    let _ = txn.next();
    let sp = txn.savepoint();
    let _ = txn.next();
    txn.rollback_to(sp);
    txn.rollback();
  }
  inp.begin_point();
  let _ = inp.next();
  inp.rollback_point();

  assert!(
    inp.is_final(),
    "A ROLLBACK UN-ENDED A SEALED STREAM: the parser will now wait forever for bytes that will \
     never arrive"
  );

  // The observable half: a full drain reaches genuine end of input, and NEVER Incomplete —
  // including on `cd`, which touches the buffer end but is no longer a frontier, because the
  // frontier is gone: the stream ended.
  let mut kinds = std::vec::Vec::new();
  loop {
    match inp.next() {
      Ok(Some(t)) => kinds.push(t.data().kind()),
      Ok(None) => break,
      Err(e) => panic!(
        "LAW VIOLATED: a SEALED stream surfaced {e:?} after a rollback — the refill it asks for can \
         never come, so the caller loops forever"
      ),
    }
  }
  assert_eq!(
    kinds,
    [PKind::Word, PKind::Word],
    "a sealed drain yields every token, including the one at the buffer end"
  );
}

/// The seal is **monotone**: sealing twice is a no-op, and there is no inverse anywhere in the
/// crate to un-seal with. The type system carries the law — this pins that `seal` itself does not
/// quietly toggle.
#[test]
fn the_seal_is_monotone() {
  let mut input = Input::<Lex<'_>, PartialCtx<'_>, (), Partial>::with_state_and_cache(
    "ab cd",
    (),
    DefaultCache::<'_, Lex<'_>>::default(),
  );
  let mut emitter = Verbose::<PErr>::new();
  assert!(!input.as_ref(&mut emitter).is_final(), "born open");

  input.seal();
  assert!(input.as_ref(&mut emitter).is_final(), "sealed");

  input.seal();
  assert!(
    input.as_ref(&mut emitter).is_final(),
    "sealing an already-sealed stream is a no-op, never a toggle"
  );
}

// ── Write once, run in both modes (0.3.0 §8.1/§8.2) ─────────────────────────────────
//
// ONE parser fn, generic over the completeness typestate, drives green under BOTH the
// complete combinator driver (`Parser…parse_str`, `Cmpl = Complete`) and the Sans-I/O
// partial driver (`parse_partial`, `Cmpl = Partial`) — the release's point, at runtime.

/// The write-once parser: collects every token kind to end of input under a
/// rollback-on-drop transaction. Generic over `Cmpl`; each drive site instantiates it.
fn kinds_generic<'inp, Cmpl>(
  inp: &mut InputRef<'inp, '_, Lex<'inp>, PartialCtx<'inp>, (), Cmpl>,
) -> Result<std::vec::Vec<PKind>, PErr>
where
  Cmpl: SurfaceIncomplete<'inp, Lex<'inp>, PartialCtx<'inp>, ()>,
{
  let mut txn = inp.begin();
  let mut kinds = std::vec::Vec::new();
  while let Some(t) = txn.next()? {
    kinds.push(t.data().kind());
  }
  txn.commit();
  Ok(kinds)
}

/// Drives `kinds_generic` COMPLETE through the public combinator driver.
fn drive_complete(src: &str) -> std::vec::Vec<PKind> {
  let ctx: PartialCtx<'_> = (Verbose::new(), DefaultCache::<'_, Lex<'_>>::default());
  Parser::with_context(ctx)
    .apply(kinds_generic)
    .parse_str(src)
    .expect("complete drive of the write-once parser succeeds")
}

/// Drives `kinds_generic` PARTIAL through `parse_partial` over a chunked buffer,
/// returning the parsed kinds and how many rounds surfaced `Incomplete` (refills).
fn drive_partial_chunked(chunks: &[&str]) -> (std::vec::Vec<PKind>, usize) {
  let mut buffer = std::string::String::new();
  let mut incompletes = 0;
  for (i, chunk) in chunks.iter().enumerate() {
    buffer.push_str(chunk);
    let is_final = i + 1 == chunks.len();
    let ctx: PartialCtx<'_> = (Verbose::new(), DefaultCache::<'_, Lex<'_>>::default());
    match parse_partial(ctx, buffer.as_str(), (), is_final, kinds_generic) {
      Ok(kinds) => return (kinds, incompletes),
      Err(e) if e.is_incomplete() => incompletes += 1,
      Err(e) => panic!("a real parse error: {e:?}"),
    }
  }
  panic!("the final chunk must complete the parse");
}

#[test]
fn write_once_runs_both_modes() {
  // The probe's exact §10.3 shape: ["foo b", "ar ", "baz"] — the first two non-final
  // rounds each cut a token at the frontier, the sealed round parses the whole sentence.
  let complete = drive_complete("foo bar baz");
  let (partial, incompletes) = drive_partial_chunked(&["foo b", "ar ", "baz"]);
  assert_eq!(complete, std::vec![PKind::Word, PKind::Word, PKind::Word]);
  assert_eq!(partial, complete, "one parser fn, two modes, one answer");
  assert_eq!(
    incompletes, 2,
    "each non-final chunk surfaced exactly one Incomplete"
  );
}

#[test]
fn write_once_chunk_sweep_equivalence() {
  // §8.2 as a deterministic sweep (the fuzz oracle's "chunked prefixes" shape): for EVERY
  // cut point of the corpus, the two-chunk partial drive must (1) surface Incomplete on
  // the non-final round — this parser drains to end of input, and rule 3 makes a
  // non-final end Incomplete — and (2) end with output identical to the complete drive.
  let corpus = "alpha beta42 gamma 7delta epsilon";
  let oracle = drive_complete(corpus);
  for cut in 1..corpus.len() {
    let (kinds, incompletes) = drive_partial_chunked(&[&corpus[..cut], &corpus[cut..]]);
    assert_eq!(
      kinds, oracle,
      "chunked equivalence must hold at cut point {cut}"
    );
    assert_eq!(
      incompletes, 1,
      "the non-final round at cut {cut} surfaces exactly one Incomplete"
    );
  }
}

#[test]
fn typed_local_fn_at_parse_partial() {
  // The concrete-`Partial` doctest pattern (a typed local fn, not a generic one) stays
  // the supported spelling under the trait bound.
  fn local<'inp>(
    inp: &mut InputRef<'inp, '_, Lex<'inp>, PartialCtx<'inp>, (), Partial>,
  ) -> Result<usize, PErr> {
    let mut txn = inp.begin();
    let mut n = 0;
    while txn.next()?.is_some() {
      n += 1;
    }
    txn.commit();
    Ok(n)
  }
  let ctx: PartialCtx<'_> = (Verbose::new(), DefaultCache::<'_, Lex<'_>>::default());
  assert_eq!(parse_partial(ctx, "foo bar", (), true, local), Ok(2));
}

// ── The §4 gate mechanism: `SurfaceIncomplete::is_incomplete_error` ──────────────────

#[test]
fn is_incomplete_error_constant_false_at_complete() {
  // Complete's arm is a constant, bound-free `false` — even for an error value that
  // *would* read incomplete: a complete input never constructs one, so the atom-layer
  // gate `if Cmpl::is_incomplete_error(&err)` const-folds away on the complete path.
  assert!(!<Complete as SurfaceIncomplete<
    '_,
    Lex<'_>,
    PartialCtx<'_>,
    (),
  >>::is_incomplete_error(&PErr::Incomplete(0)));
  assert!(!<Complete as SurfaceIncomplete<
    '_,
    Lex<'_>,
    PartialCtx<'_>,
    (),
  >>::is_incomplete_error(&PErr::Lex));
}

#[test]
fn is_incomplete_error_routes_through_maybe_incomplete_at_partial() {
  // Partial routes through `MaybeIncomplete`: exactly the incomplete sentinel reads true;
  // a plain error and a terminal limit trip both read false (terminal outranks incomplete
  // and must never be re-raised as one).
  assert!(<Partial as SurfaceIncomplete<
    '_,
    Lex<'_>,
    PartialCtx<'_>,
    (),
  >>::is_incomplete_error(&PErr::Incomplete(3)));
  assert!(!<Partial as SurfaceIncomplete<
    '_,
    Lex<'_>,
    PartialCtx<'_>,
    (),
  >>::is_incomplete_error(&PErr::Lex));
  assert!(!<Partial as SurfaceIncomplete<
    '_,
    Lex<'_>,
    PartialCtx<'_>,
    (),
  >>::is_incomplete_error(&PErr::Limit));
}

// ── §8.5: the generalized scanner drivers under Partial ─────────────────────────────
//
// Each G-class driver (0.3.0: `try_expect`, `skip_while`, `sync_to`, `sync_through`,
// `sync_balanced`, `fold`/`foldn`, `consume_cached_*`) runs under `Partial` through its
// now-generic header. The matrix per driver: non-final ⇒ `Incomplete` at the frontier
// (conservative — later input could extend what it stopped on); final ⇒ identical to the
// complete-mode outcome. Plus the poison-at-frontier precedence case for `try_expect`.

/// Inlines a partial-input drive: builds the input over `$src`, seals per `$is_final`,
/// hands the handle to `$body`, and evaluates to `(body_output, emitted_diagnostics)`.
macro_rules! with_partial {
  ($src:expr, $is_final:expr, |$inp:ident| $body:expr) => {{
    let mut input = Input::<Lex<'_>, PartialCtx<'_>, (), Partial>::with_state_and_cache(
      $src,
      (),
      DefaultCache::<'_, Lex<'_>>::default(),
    );
    if $is_final {
      input.seal();
    }
    let mut emitter = Verbose::<PErr>::new();
    let out = {
      #[allow(unused_mut)]
      let mut $inp = input.as_ref(&mut emitter);
      $body
    };
    let emitted: usize = emitter.errors().values().map(|g| g.len()).sum();
    (out, emitted)
  }};
}

#[test]
fn try_expect_partial_matrix() {
  // Non-final, the only token touches the buffer end: withheld, Incomplete.
  let (r, emitted) = with_partial!("foo", false, |inp| inp.try_expect(|_| true));
  assert_eq!(r, Err(PErr::Incomplete(3)));
  assert_eq!(emitted, 0);
  // Final: consumed, exactly as complete mode.
  let (r, _) = with_partial!("foo", true, |inp| {
    inp.try_expect(|_| true).map(|t| t.map(|s| s.data().kind()))
  });
  assert_eq!(r, Ok(Some(PKind::Word)));
  // The decline path puts a MID-BUFFER token back without any incomplete: "foo" ends
  // strictly before the trailing space, so it lexes even non-final, and the declining
  // predicate leaves it cached for the next consume.
  let (r, emitted) = with_partial!("foo ", false, |inp| {
    let declined = inp
      .try_expect(|t| matches!(t.data(), PTok::Num))
      .expect("the decline path is not an error");
    assert!(declined.is_none(), "the word is not a number");
    inp
      .try_expect(|t| matches!(t.data(), PTok::Word))
      .map(|t| t.map(|s| s.data().kind()))
  });
  assert_eq!(r, Ok(Some(PKind::Word)));
  assert_eq!(emitted, 0);
}

#[test]
fn skip_while_partial_matrix() {
  // Non-final: the trivia run's last word touches the buffer end — it may extend, so the
  // run is not finishable yet. (This is what hands `padded` its partial semantics.)
  let (r, _) = with_partial!("foo bar", false, |inp| {
    inp.skip_while(|t| matches!(t.data(), PTok::Word))
  });
  assert_eq!(r, Err(PErr::Incomplete(7)));
  // Final: the run completes and the stopper is left at the cache front.
  let (r, _) = with_partial!("foo 42", true, |inp| {
    inp
      .skip_while(|t| matches!(t.data(), PTok::Word))
      .expect("the final-mode skip completes");
    inp.next().map(|t| t.map(|s| s.data().kind()))
  });
  assert_eq!(r, Ok(Some(PKind::Num)));
}

#[test]
fn sync_to_partial_matrix() {
  // Non-final: the would-be sync token touches the buffer end — the next chunk could
  // extend it ("42" → "425"), so recovery-sync is not decidable yet: Incomplete.
  let (r, emitted) = with_partial!("foo bar 42", false, |inp| {
    inp
      .sync_to(|t| matches!(t.data(), PTok::Num), || None)
      .map(|s| s.is_some())
  });
  assert_eq!(r, Err(PErr::Incomplete(10)));
  assert_eq!(
    emitted, 2,
    "the mid-buffer skipped words are diagnosed as they settle (emit-as-you-go, exactly \
     as complete mode); only the frontier item itself is withheld"
  );
  // Final: syncs to the number, diagnosing the two skipped words — complete-identical.
  let (r, emitted) = with_partial!("foo bar 42", true, |inp| {
    inp
      .sync_to(|t| matches!(t.data(), PTok::Num), || None)
      .map(|s| s.is_some())
  });
  assert_eq!(r, Ok(true));
  assert_eq!(
    emitted, 2,
    "each skipped token is diagnosed, as in complete mode"
  );
}

#[test]
fn sync_through_partial_matrix() {
  let (r, _) = with_partial!("foo bar 42", false, |inp| {
    inp
      .sync_through(|t| matches!(t.data(), PTok::Num), || None)
      .map(|s| s.map(|t| t.data().kind()))
  });
  assert_eq!(r, Err(PErr::Incomplete(10)));
  let (r, emitted) = with_partial!("foo bar 42", true, |inp| {
    inp
      .sync_through(|t| matches!(t.data(), PTok::Num), || None)
      .map(|s| s.map(|t| t.data().kind()))
  });
  assert_eq!(
    r,
    Ok(Some(PKind::Num)),
    "through-sync consumes the sync token"
  );
  assert_eq!(emitted, 2);
}

#[test]
fn sync_balanced_partial_matrix() {
  use crate::input::Balance;
  // Non-final: same conservatism through the balanced scanner.
  let (r, _) = with_partial!("foo bar 42", false, |inp| {
    inp
      .sync_balanced(
        |_k: &PKind| Balance::<char>::Neutral,
        |t| matches!(t.data(), PTok::Num),
      )
      .map(|h| h.is_some())
  });
  assert_eq!(r, Err(PErr::Incomplete(10)));
  // Final: the hole over the skipped words is produced, with no per-token diagnostics.
  let (r, _) = with_partial!("foo bar 42", true, |inp| {
    inp
      .sync_balanced(
        |_k: &PKind| Balance::<char>::Neutral,
        |t| matches!(t.data(), PTok::Num),
      )
      .map(|h| h.is_some())
  });
  assert_eq!(r, Ok(true));
}

#[test]
fn fold_partial_matrix() {
  // Non-final: the last folded word touches the end — the fold is not finishable.
  let (r, _) = with_partial!("foo bar baz", false, |inp| {
    inp.fold(|t| matches!(t.data(), PTok::Word), || 0usize, |n, _| n + 1)
  });
  assert_eq!(r, Err(PErr::Incomplete(11)));
  // Final: all three words fold — complete-identical.
  let (r, _) = with_partial!("foo bar baz", true, |inp| {
    inp.fold(|t| matches!(t.data(), PTok::Word), || 0usize, |n, _| n + 1)
  });
  assert_eq!(r, Ok(3));
}

#[test]
fn foldn_stops_mid_buffer_without_incomplete() {
  // `foldn(2)` consumes exactly the two MID-BUFFER words and never reaches the frontier
  // token, so even a non-final run completes: the frontier rules hold back only what the
  // drive actually touches.
  let (r, _) = with_partial!("foo bar baz", false, |inp| {
    inp.foldn(|| 0usize, |n, _| n + 1, 2)
  });
  assert_eq!(r, Ok(2));
}

#[test]
fn consume_cached_never_surfaces_incomplete() {
  // `consume_cached_*` never lexes (bound-only generalization, no `SurfaceIncomplete`):
  // it pops what peeking already cached, and the peek fill never caches a frontier token,
  // so the mid-buffer word is consumable and the frontier word simply is not there.
  let (r, emitted) = with_partial!("foo bar", false, |inp| {
    let peeked = inp
      .peek_one()
      .expect("the partial peek never errs — a short window, not an error")
      .is_some();
    let first = inp.consume_cached_one().map(|t| t.data().kind());
    let second = inp.consume_cached_one().map(|t| t.data().kind());
    (peeked, first, second)
  });
  assert_eq!(r, (true, Some(PKind::Word), None));
  assert_eq!(emitted, 0);
}

#[test]
fn try_expect_poison_at_frontier_beats_incomplete() {
  // Terminal beats incomplete THROUGH the generalized driver: "a b c" behind a 2-word
  // limiter, non-final — the tripping third word ends exactly at the buffer end, and the
  // trip must fire as `Limit` (diagnostic + latch), never be re-raised as `Incomplete`.
  let tracker = LimitTracker::with_limit(2);
  let mut input = Input::<LimLex<'_>, LimCtx<'_>, (), Partial>::with_state_and_cache(
    "a b c",
    tracker,
    DefaultCache::<'_, LimLex<'_>>::default(),
  );
  let mut emitter = Verbose::<PErr>::new();
  let (results, poisoned) = {
    let mut inp = input.as_ref(&mut emitter);
    let a = inp.try_expect(|_| true).map(|t| t.is_some());
    let b = inp.try_expect(|_| true).map(|t| t.is_some());
    let c = inp.try_expect(|_| true).map(|t| t.is_some());
    ((a, b, c), inp.is_poisoned())
  };
  assert_eq!(results.0, Ok(true));
  assert_eq!(results.1, Ok(true));
  // Under the collecting emitter the trip's diagnostic is EMITTED (not returned), the
  // poison boundary latches, and the drive reads the terminal stop — a decline at the
  // boundary, exactly as the `next()`-driven twin above — and NEVER `Incomplete`.
  assert_eq!(
    results.2,
    Ok(false),
    "the frontier-aligned trip is a terminal stop, never re-raised as incomplete"
  );
  assert!(poisoned, "the poison boundary latched at the trip");
  assert_eq!(
    limit_diags(&emitter),
    1,
    "the trip was diagnosed exactly once"
  );
}
