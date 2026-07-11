//! Partial-input (Sans-I/O) frontier-rule tests.
//!
//! Each of the three conservative rules at the scan chokepoint gets a focused case: frontier
//! holdback (a token touching the buffer end), frontier error (a lexer error touching the buffer
//! end), and non-final EOF. Plus the two boundary properties: `is_final == true` behaves exactly
//! like a complete parse, and a *mid-buffer* token or error (strictly before the buffer end) is
//! yielded / emitted normally even while partial.

use crate::{
  Token,
  cache::DefaultCache,
  emitter::Verbose,
  error::{Incomplete, MaybeIncomplete, token::UnexpectedToken},
  input::{Complete, Input, Partial},
  lexer::LogosLexer,
};

// An error type that can carry the partial-input incomplete sentinel. `From<Incomplete>` is the
// exact construction path the frontier rules use (via `SurfaceIncomplete`), and `is_incomplete()`
// is what recovery keys the never-recoverable law off — the two must stay coherent.
#[derive(Debug, Clone, PartialEq)]
enum PErr {
  Lex,
  Incomplete(usize),
}

impl From<()> for PErr {
  fn from(_: ()) -> Self {
    PErr::Lex
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
