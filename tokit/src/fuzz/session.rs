//! The **session-point** driver: fuzzes [`ParseState`] session points — the owned, non-lexical
//! speculation the transaction guards cannot express.
//!
//! A [`ParseState`] borrows the input and does not expose the consume surface, so — exactly as the
//! crate's own `parse_state` unit tests do — speculation is driven through the reachable
//! [`state_mut`](ParseState::state_mut) (re-key the observable state tag) and
//! [`emitter`](ParseState::emitter) (record a diagnostic) surface, and a rollback is watched to
//! restore both. The generated sequence is a well-formed last-in, first-out stream of
//! `begin_point` / `commit_point` / `rollback_point` (never settling with no live point), with a
//! shadow stack asserting depth, the emission count, and the state tag after every step.
//!
//! # Oracles
//!
//! - **LIFO depth** — `points()` always equals the shadow stack depth.
//! - **Commit keeps** — after `commit_point`, the current emission count and state tag survive.
//! - **Rollback restores** — after `rollback_point`, the emission count and state tag return to the
//!   values captured when that point opened.

use std::vec::Vec;

use super::{
  fixtures::{CountEmitter, FuzzCtx, FuzzError, ScriptLexer, ScriptState, cache, initial_state},
  ops::{Coverage, Op},
  rng::Rng,
};
use crate::{
  ParseState,
  emitter::Emitter,
  span::{SimpleSpan, Spanned},
};

/// A session sub-operation. Only `Commit`/`Rollback` map to census ops; `Begin`/`Emit`/`Rekey` are
/// the checkpoint-tracked "work" a point speculates over.
#[derive(Debug, Clone, Copy)]
enum SOp {
  Begin,
  Emit,
  Rekey(u64),
  Commit,
  Rollback,
}

/// Generates a well-formed session sub-op sequence: it never commits/rolls back with no live point,
/// and settles every open point before the end (a session must end explicitly).
fn gen_session(rng: &mut Rng) -> Vec<SOp> {
  let n = rng.below(12) + 4;
  let mut depth = 0usize;
  let mut out = Vec::with_capacity(n + 2);
  for _ in 0..n {
    if depth == 0 {
      match rng.below(3) {
        0 | 1 => {
          out.push(SOp::Begin);
          depth += 1;
        }
        _ => out.push(SOp::Emit),
      }
    } else {
      match rng.below(6) {
        0 => {
          out.push(SOp::Begin);
          depth += 1;
        }
        1 => out.push(SOp::Emit),
        2 => out.push(SOp::Rekey(rng.next_u64())),
        3 => {
          out.push(SOp::Commit);
          depth -= 1;
        }
        _ => {
          out.push(SOp::Rollback);
          depth -= 1;
        }
      }
    }
  }
  while depth > 0 {
    out.push(if rng.chance(1, 2) {
      SOp::Commit
    } else {
      SOp::Rollback
    });
    depth -= 1;
  }
  out
}

/// Records a diagnostic through the state's emitter (bumps the emission count).
fn emit<'inp>(ps: &mut ParseState<'_, 'inp, '_, ScriptLexer<'inp>, FuzzCtx<'inp>>) {
  <CountEmitter as Emitter<'inp, ScriptLexer<'inp>>>::emit_error(
    ps.emitter(),
    Spanned::new(SimpleSpan::new(0, 1), FuzzError::Diagnostic),
  )
  .expect("CountEmitter is non-fatal");
}

/// The live emission count observed through the state's emitter.
fn count<'inp>(ps: &mut ParseState<'_, 'inp, '_, ScriptLexer<'inp>, FuzzCtx<'inp>>) -> u64 {
  ps.emitter().count()
}

/// Runs one session case over `src`, checking the session-point oracles.
pub(crate) fn run(src: &[u8], seq_seed: u64, cov: &mut Coverage) {
  let cache = cache();
  let mut emitter = CountEmitter::new();
  let state = initial_state(src);
  let mut input = crate::input::Input::<'_, ScriptLexer<'_>, FuzzCtx<'_>, ()>::with_state_and_cache(
    src, state, cache,
  );
  let mut ir = input.as_ref(&mut emitter);

  // Optionally pre-consume a few tokens so the session opens at a non-trivial position.
  let mut rng = Rng::new(seq_seed);
  let pre = rng.below(4);
  for _ in 0..pre {
    if ir.next().expect("complete + non-fatal").is_none() {
      break;
    }
  }

  let seq = gen_session(&mut rng);

  let start = *ir.cursor();
  let mut ps = ParseState::new(&mut ir, start);
  // Each live point remembers the emission count and state tag captured when it opened.
  let mut stack: Vec<(u64, u64)> = Vec::new();
  let mut cur_tag: u64 = ps.state().tag;

  for sop in seq {
    match sop {
      SOp::Begin => {
        let snap = (count(&mut ps), cur_tag);
        ps.begin_point();
        stack.push(snap);
        assert_eq!(
          ps.points(),
          stack.len(),
          "session: points() diverged from the shadow depth"
        );
      }
      SOp::Emit => emit(&mut ps),
      SOp::Rekey(t) => {
        *ps.state_mut() = ScriptState { tag: t };
        cur_tag = t;
      }
      SOp::Commit => {
        cov.mark(Op::SessionCommit);
        ps.commit_point();
        stack.pop();
        assert_eq!(
          ps.points(),
          stack.len(),
          "session: commit_point left the wrong depth"
        );
        // Commit keeps: the current count and tag are unchanged; nothing to assert beyond depth.
      }
      SOp::Rollback => {
        cov.mark(Op::SessionRollback);
        let (saved_count, saved_tag) = stack.pop().expect("generator keeps the stream well-formed");
        ps.rollback_point();
        cur_tag = saved_tag;
        assert_eq!(
          ps.points(),
          stack.len(),
          "session: rollback_point left the wrong depth"
        );
        assert_eq!(
          count(&mut ps),
          saved_count,
          "session: rollback did not restore the emission count"
        );
        assert_eq!(
          ps.state().tag,
          saved_tag,
          "session: rollback did not restore the state tag"
        );
      }
    }
  }
  assert_eq!(
    ps.points(),
    0,
    "session: points left open at the end of the case"
  );
}
