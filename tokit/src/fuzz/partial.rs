//! The **partial-input** driver: fuzzes the `Partial` completeness typestate — `set_final`, the
//! frontier holdback, and the `Incomplete` channel — via chunked equivalence, the same law the
//! conformance kit's `run_partial` checks, but over fuzzed (error-bearing) streams and randomized
//! cuts rather than an exhaustive sweep.
//!
//! Two shapes are checked against the single-shot complete parse (the oracle):
//!
//! - **In-place two-phase.** One `Partial` input over the whole source: a non-final drain yields
//!   every token strictly before the buffer end and ends `Incomplete` (the frontier withholds the
//!   last token / trailing error); flipping [`set_final(true)`](crate::InputRef::set_final) and
//!   draining on reproduces the rest. The concatenation must equal the complete parse.
//! - **Chunked prefixes.** For random cut points `k`, a non-final drain of `src[..k]` yields exactly
//!   the complete tokens whose span ends strictly before `k` and always ends `Incomplete`.
//!
//! This is error-agnostic (it compares committed token streams, which skip lexer errors
//! identically in both modes), so — unlike the consume driver — the fuzzed streams here **do**
//! contain lexer-error bytes, exercising the error-skipping and dedup paths under truncation.

use std::vec::Vec;

use super::{
  fixtures::{CountEmitter, FuzzCtx, FuzzError, FuzzKind, ScriptLexer, cache, initial_state},
  ops::{Coverage, Op},
  rng::Rng,
};
use crate::{Token, input::Partial, span::SimpleSpan};

/// A committed token, reduced to the facts the oracle compares.
type Tok = (FuzzKind, SimpleSpan);

/// Drains a fresh **complete** input over `src`, returning the committed token stream — the oracle.
fn complete_stream(src: &[u8], budget: usize) -> Vec<Tok> {
  let cache = cache();
  let mut emitter = CountEmitter::new();
  let state = initial_state(src);
  let mut input = crate::input::Input::<'_, ScriptLexer<'_>, FuzzCtx<'_>, ()>::with_state_and_cache(
    src, state, cache,
  );
  let mut ir = input.as_ref(&mut emitter);
  let mut out = Vec::new();
  loop {
    assert!(
      out.len() <= budget,
      "complete drain exceeded budget (non-terminating?)"
    );
    match ir.next().expect("complete + non-fatal emitter never errs") {
      Some(sp) => {
        let (span, tok) = sp.into_components();
        out.push((tok.kind(), span));
      }
      None => return out,
    }
  }
}

/// Drains a fresh **partial** input over `src` at `is_final`, returning the committed token stream
/// and whether the drain ended `Incomplete` (rather than at genuine end of input).
fn partial_stream(src: &[u8], is_final: bool, budget: usize) -> (Vec<Tok>, bool) {
  let cache = cache();
  let mut emitter = CountEmitter::new();
  let state = initial_state(src);
  let mut input =
    crate::input::Input::<'_, ScriptLexer<'_>, FuzzCtx<'_>, (), Partial>::with_state_and_cache(
      src, state, cache,
    );
  let mut ir = input.as_ref(&mut emitter);
  ir.set_final(is_final);
  let mut out = Vec::new();
  loop {
    assert!(
      out.len() <= budget,
      "partial drain exceeded budget (non-terminating?)"
    );
    match ir.next() {
      Ok(Some(sp)) => {
        let (span, tok) = sp.into_components();
        out.push((tok.kind(), span));
      }
      Ok(None) => return (out, false),
      Err(e) => {
        assert_eq!(
          e,
          FuzzError::Incomplete,
          "partial drain surfaced a non-Incomplete error"
        );
        return (out, true);
      }
    }
  }
}

/// Drains one `Partial` input over the whole `src` in two phases across a live `set_final(true)`:
/// non-final until `Incomplete`, then final to genuine end of input. Returns the concatenated
/// committed stream.
fn two_phase_stream(src: &[u8], budget: usize) -> Vec<Tok> {
  let cache = cache();
  let mut emitter = CountEmitter::new();
  let state = initial_state(src);
  let mut input =
    crate::input::Input::<'_, ScriptLexer<'_>, FuzzCtx<'_>, (), Partial>::with_state_and_cache(
      src, state, cache,
    );
  let mut ir = input.as_ref(&mut emitter);
  let mut out = Vec::new();

  // Phase 1: non-final. Drains everything before the frontier, ending Incomplete (or genuine EOF
  // for an empty stream — either way, stop).
  ir.set_final(false);
  loop {
    assert!(out.len() <= budget, "two-phase drain exceeded budget");
    match ir.next() {
      Ok(Some(sp)) => {
        let (span, tok) = sp.into_components();
        out.push((tok.kind(), span));
      }
      Ok(None) => break,
      Err(e) => {
        assert_eq!(
          e,
          FuzzError::Incomplete,
          "phase-1 drain surfaced a non-Incomplete error"
        );
        break;
      }
    }
  }

  // Phase 2: mark final in place and drain the withheld remainder to genuine end of input.
  ir.set_final(true);
  loop {
    assert!(out.len() <= budget, "two-phase drain exceeded budget");
    match ir.next() {
      Ok(Some(sp)) => {
        let (span, tok) = sp.into_components();
        out.push((tok.kind(), span));
      }
      Ok(None) => break,
      Err(_) => panic!("a final input must never surface Incomplete"),
    }
  }
  out
}

/// Runs one partial case: builds a fuzzed (error-bearing) stream and checks both chunked-equivalence
/// shapes against the complete parse.
pub(crate) fn run(src: &[u8], seed: u64, cov: &mut Coverage) {
  cov.mark(Op::SetFinal);
  let budget = src.len() + 4;
  let complete = complete_stream(src, budget);

  // 1. A final partial drain of the whole source reproduces the complete parse.
  let (final_tokens, final_incomplete) = partial_stream(src, true, budget);
  assert!(
    !final_incomplete,
    "a FINAL drain must reach genuine end of input, not Incomplete"
  );
  assert_eq!(
    final_tokens, complete,
    "final partial drain diverged from the complete parse"
  );

  // 2. In-place two-phase: non-final then set_final(true) reproduces the complete parse.
  assert_eq!(
    two_phase_stream(src, budget),
    complete,
    "in-place set_final two-phase diverged from the complete parse"
  );

  // 3. Chunked prefixes at random cut points: a non-final prefix yields exactly the complete
  //    tokens ending strictly before the cut, and always ends Incomplete.
  let len = src.len();
  let mut rng = Rng::new(seed ^ 0x5015_5EED);
  let cuts = (len + 1).min(6);
  for _ in 0..cuts {
    let k = rng.below(len + 1);
    let (prefix_tokens, incomplete) = partial_stream(&src[..k], false, budget);
    let expected: Vec<Tok> = complete
      .iter()
      .filter(|(_, span)| span_end(span) < k)
      .copied()
      .collect();
    assert_eq!(
      prefix_tokens, expected,
      "chunked prefix (k={k}) diverged from the complete prefix"
    );
    assert!(
      incomplete,
      "a non-final prefix drain (k={k}) must end Incomplete"
    );
  }
}

/// The end offset of a span.
fn span_end(span: &SimpleSpan) -> usize {
  *span.end_ref()
}
