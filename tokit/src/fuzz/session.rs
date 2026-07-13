//! The **session-point** driver: fuzzes [`InputRef`] session points — the non-lexical speculation
//! the transaction guards cannot express.
//!
//! A session point lives on the input handle itself, so — unlike a borrowing guard — it can be
//! opened, *parsed through*, and settled in unrelated steps. This driver exercises exactly that:
//! the generated sequence is a well-formed last-in, first-out stream of `begin_point` /
//! `commit_point` / `rollback_point` (never settling with no live point) interleaved with real
//! **token consumption**, diagnostic emission, and state re-keying, with a shadow stack asserting
//! the depth, the cursor, the emission count, and the state tag after every step.
//!
//! # Oracles
//!
//! - **LIFO depth** — `points()` always equals the shadow stack depth.
//! - **Commit keeps** — after `commit_point`, the current cursor, emission count, and state tag
//!   survive.
//! - **Rollback restores** — after `rollback_point`, the cursor, emission count, and state tag all
//!   return to the values captured when that point opened.

use std::vec::Vec;

use super::{
  fixtures::{CountEmitter, FuzzCtx, FuzzError, ScriptLexer, ScriptState, cache, initial_state},
  ops::{Coverage, Op},
  rng::Rng,
};
use crate::{
  InputRef,
  emitter::Emitter,
  span::{SimpleSpan, Spanned},
};

/// The session handle this driver steps: the input reference the session points live on.
type Ir<'a, 'inp> = InputRef<'a, 'inp, ScriptLexer<'a>, FuzzCtx<'a>>;

/// A session sub-operation. Only `Commit`/`Rollback` map to census ops; `Begin`/`Consume`/`Emit`/
/// `Rekey` are the checkpoint-tracked "work" a point speculates over.
#[derive(Debug, Clone, Copy)]
enum SOp {
  Begin,
  Consume,
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
      match rng.below(4) {
        0 | 1 => {
          out.push(SOp::Begin);
          depth += 1;
        }
        2 => out.push(SOp::Consume),
        _ => out.push(SOp::Emit),
      }
    } else {
      match rng.below(7) {
        0 => {
          out.push(SOp::Begin);
          depth += 1;
        }
        1 => out.push(SOp::Emit),
        2 => out.push(SOp::Rekey(rng.next_u64())),
        3 | 4 => out.push(SOp::Consume),
        5 => {
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

/// Records a diagnostic through the input's emitter (bumps the emission count).
fn emit<'inp>(ir: &mut Ir<'inp, '_>) {
  <CountEmitter as Emitter<'inp, ScriptLexer<'inp>>>::emit_error(
    ir.emitter(),
    Spanned::new(SimpleSpan::new(0, 1), FuzzError::Diagnostic),
  )
  .expect("CountEmitter is non-fatal");
}

/// The live emission count observed through the input's emitter.
fn count(ir: &mut Ir<'_, '_>) -> u64 {
  ir.emitter().count()
}

/// The live cursor offset — the fact only real token consumption moves.
fn at(ir: &Ir<'_, '_>) -> usize {
  *ir.cursor().as_inner()
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

  // Each live point remembers the emission count, state tag, and cursor captured when it opened.
  let mut stack: Vec<(u64, u64, usize)> = Vec::new();
  let mut cur_tag: u64 = ir.state().tag;

  for sop in seq {
    match sop {
      SOp::Begin => {
        let snap = (count(&mut ir), cur_tag, at(&ir));
        ir.begin_point();
        stack.push(snap);
        assert_eq!(
          ir.points(),
          stack.len(),
          "session: points() diverged from the shadow depth"
        );
      }
      // Real parsing *through* an open point — the thing a borrowing guard cannot be held across.
      SOp::Consume => {
        let _ = ir.next().expect("complete + non-fatal");
      }
      SOp::Emit => emit(&mut ir),
      SOp::Rekey(t) => {
        *ir.state_mut() = ScriptState { tag: t };
        cur_tag = t;
      }
      SOp::Commit => {
        cov.mark(Op::SessionCommit);
        let before = at(&ir);
        ir.commit_point();
        stack.pop();
        assert_eq!(
          ir.points(),
          stack.len(),
          "session: commit_point left the wrong depth"
        );
        // Commit keeps: the progress made through the point stays exactly where it was.
        assert_eq!(
          at(&ir),
          before,
          "session: commit_point moved the cursor instead of keeping the progress"
        );
      }
      SOp::Rollback => {
        cov.mark(Op::SessionRollback);
        let (saved_count, saved_tag, saved_at) =
          stack.pop().expect("generator keeps the stream well-formed");
        ir.rollback_point();
        cur_tag = saved_tag;
        assert_eq!(
          ir.points(),
          stack.len(),
          "session: rollback_point left the wrong depth"
        );
        assert_eq!(
          at(&ir),
          saved_at,
          "session: rollback did not restore the cursor"
        );
        assert_eq!(
          count(&mut ir),
          saved_count,
          "session: rollback did not restore the emission count"
        );
        assert_eq!(
          ir.state().tag,
          saved_tag,
          "session: rollback did not restore the state tag"
        );
      }
    }
  }
  assert_eq!(
    ir.points(),
    0,
    "session: points left open at the end of the case"
  );
}
