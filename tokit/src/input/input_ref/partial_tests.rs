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
  InputRef, Token,
  cache::DefaultCache,
  emitter::{Fatal, Verbose},
  error::{Incomplete, MaybeIncomplete, token::UnexpectedToken},
  input::{Complete, Input, Partial, parse_partial},
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
  let mut emitter = Verbose::<PErr>::new();
  let (kinds, result) = {
    let mut inp = input.as_ref(&mut emitter);
    inp.set_final(is_final);
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
  let mut emitter = Verbose::<PErr>::new();
  let (tokens, result) = {
    let mut inp = input.as_ref(&mut emitter);
    inp.set_final(is_final);
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
  let mut emitter = Verbose::<PErr>::new();
  let (kinds, result, poisoned) = {
    let mut inp = input.as_ref(&mut emitter);
    inp.set_final(is_final);
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

    let mut inp = input.as_ref(&mut emitter);
    inp.set_final(false);

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
