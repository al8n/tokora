//! The never-recoverable law's chokepoint for the recovery combinators — **the one place** a
//! recovery attempt's failure is judged, and the one place its baselines are taken.
//!
//! [`Recover`](super::Recover), [`InplaceRecover`](super::InplaceRecover) and
//! [`SkipThenRetry`](super::SkipThenRetry) run four attempts between them, and each one has to
//! answer the same question: *did something no further input clears happen inside this attempt?*
//! Four hand-written copies of that guard is how the scanner-side witnesses came to be consulted
//! zero times at all four sites while `combinator::many`'s twelve drivers consulted one of them at
//! every one. Here there is no guard to spell: a combinator hands its attempt to
//! [`speculated_attempt`] or [`inplace_attempt`] and matches on an [`Attempted`], and `GATE_CENSUS`
//! pins that none of the four sources names a witness or a baseline of its own.
//!
//! [`SkipThenRetry`](super::SkipThenRetry) also does recovery work that is **not** a parse attempt
//! — the skip that finds a sync point and the advance that consumes a spent one — and that work
//! goes through [`recovery_step`] for the reason the [section below](#the-work-outside-an-attempt)
//! gives.
//!
//! # The five witnesses
//!
//! Two ride the error value and three ride the input, and none subsumes another *as a question*,
//! though the last two overlap heavily as answers:
//!
//! * `err.is_incomplete()` — the frontier [`Incomplete`](crate::error::Incomplete). Not a malformed
//!   construct, merely an unfinished one, so recovering past it fabricates output from input that
//!   has not finished arriving.
//! * `err.is_terminal()` — a terminal stop the grammar's error type **did** carry: the
//!   terminal-marked [`UnexpectedEot`](crate::error::UnexpectedEot) an accepting emitter's
//!   `Tripped` arm builds.
//! * `inp.tripped_during_attempt(trips)` — a **descent** budget trip. It latches no boundary (it
//!   has a control stack rather than a position), and the grammar's error type is free to discard
//!   [`RecursionLimitReached`](crate::error::RecursionLimitReached) on conversion — `()` does — so
//!   the session counter is what makes the verdict independent of the error sink.
//! * `inp.scanner_tripped_during_attempt(scans)` — a **scanner** limit trip, counted on the input
//!   session. This is the load-bearing scanner witness, and the
//!   [section below](#why-the-scanner-witness-is-a-counter-and-not-the-latch) is why it has to be a
//!   monotone counter rather than the boundary that trip latched.
//! * `inp.latched_during_attempt(latch)` — the scanner's **poison boundary**, compared against a
//!   per-attempt snapshot. The narrower of the two scanner readings: it answers *a different stop
//!   is standing at the end of this attempt*, where the counter answers *a stop happened inside
//!   it*. Every transition this crate can produce that moves the latch also moves the counter, so
//!   this term adds no verdict the counter does not already give; it is kept because it is the
//!   reading the boundary itself is the subject of, it costs one `Option<L::Offset>` clone on a
//!   path that already clones one, and a witness removed on a subsumption argument is a witness
//!   removed on an argument rather than on a measurement.
//!
//! # Why the error alone cannot answer, and why this is not a `Fatal` quirk
//!
//! One trip, one position, one committed leaf, two emitters:
//!
//! * an **accepting** emitter files the trip's diagnostic, `next_or_stop`/`try_expect_or_stop`
//!   reach their `Tripped` arm, and the caller gets an `UnexpectedEot` through
//!   [`into_terminal`](crate::error::UnexpectedEnd::into_terminal) — `is_terminal()` is `true`;
//! * a **rejecting** emitter reports the same trip by returning `Err` from
//!   [`emit_lexer_error`](crate::Emitter::emit_lexer_error) — that `Err` is not a refusal to
//!   report, it **is** the report — and the scanner propagates that value straight out of
//!   `scan_with`'s `Verdict::Trip` arm. It was built by
//!   [`FromEmitterError::from_lexer_error`](crate::emitter::FromEmitterError::from_lexer_error) out
//!   of the lexer's own `Token::Error`; no `UnexpectedEnd` exists on that path, so nothing ever
//!   calls `into_terminal` and `is_terminal()` is `false`.
//!
//! Every fail-fast emitter behaves that way, so this is a property of the *channel*, not of
//! [`Fatal`](crate::emitter::Fatal). Terminality is a fact about the **event**, this crate
//! enumerates **carriers** of it, and the two come apart in both directions — the same root cause
//! the descent counter was added for. The scanner's half of it is `Input::scanner_trips`, and
//! reading it here is what makes every gated site answer the same way for every sink.
//!
//! # Why the scanner witness is a COUNTER and not the latch
//!
//! The obvious scanner witness is the poison boundary the trip latched, and it is the wrong one at
//! every nesting depth. It is a **lineage memo**: a [`Checkpoint`](crate::input::Checkpoint)
//! carries it and [`restore`](crate::InputRef::restore) copies the saved value back verbatim.
//!
//! That defeats a comparison twice over, one level at a time.
//!
//! 1. A `latch_snapshot()` taken before [`try_attempt`](crate::InputRef::try_attempt) and compared
//!    after it compares a restored latch against the value it was restored *to*: always equal,
//!    always `false`, a dead term. Measured, not reasoned about: with the read placed after the
//!    attempt, `tokora/tests/recovery_terminal_stop.rs`'s in-place cell passed and all three
//!    speculating cells still failed with the scanner re-entered after the trip.
//! 2. Moving the read *inside* the attempt closes exactly that level. Grammar code is free to catch
//!    a scanner stop inside an **inner** attempt or transaction of its own, and that inner rollback
//!    restores the boundary before the outer read runs — so the outer verdict reads clean over a
//!    stop that is live, already diagnosed, and will re-trip on the next scan of the same prefix.
//!    Measured the same way: `recover_reraises_a_trip_caught_inside_a_nested_speculation` observed
//!    the recoverer running and the scan count moving 3 → 4 with the read placed inside the outer
//!    attempt.
//!
//! Relocating the read a third time would close level two and open level three. **A cell inside the
//! rollback set cannot witness an event across a rollback at any depth**, so the witness is a cell
//! no rollback reaches: `Input::scanner_trips`, a monotone counter bumped by
//! `InputRef::latch_if_limit_tripped` — the crate's sole terminal predicate, reached by both lexing
//! drivers — and outside the rollback set by the same taxonomy that puts the descent counter there.
//! This is the [descent side's](crate::InputRef::descend) answer applied to the scanner, and it is
//! the same root cause both times: this crate enumerates *carriers* of terminality, and a terminal
//! *event* can reach a caller through an unmarked or a rewound one.
//!
//! # The baselines are taken here, and the witnesses are read INSIDE the attempt
//!
//! With the counter in place the placement no longer decides the verdict — the counter reads the
//! same before or after any rollback — but [`judge`] still runs inside the closure the attempt
//! wraps, and the latch reading is why. Read outside, that term is the dead one described above;
//! read inside, it is at least the honest narrow reading it claims to be. One body, one place, and
//! the verdict rides out on the error channel beside the error.
//!
//! # The work outside an attempt
//!
//! A gate that wraps only the parse attempts says nothing about recovery work that never enters
//! one, and [`SkipThenRetry`](super::SkipThenRetry) does its actual recovering out there: the skip
//! to a sync point, and the advance over a sync point that did not admit a parse. Both are `Ok`-
//! returning primitives that fold a **terminal trip** into the same value as a genuine end of input
//! — `sync_balanced` gives `Ok(None)` for either, `next` gives `Ok(None)` for either — so a spent
//! scanner budget read as *"nothing left to skip to"* and the combinator surfaced its ordinary
//! trigger error over a stop.
//!
//! [`recovery_step`] is the same discipline for that shape: sample the input-side witnesses around
//! the operation and report a [`Stepped::Stopped`] when they move. It reads three witnesses rather
//! than five because the other two interrogate an error value and this shape has none — the step
//! returned `Ok`; an `Err` from it is already the emitter's own report and propagates untouched.
//!
//! # Granularity, and where the baselines belong
//!
//! One **attempt** is the unit, exactly as one *element* is `combinator::many`'s: within it the verdict
//! is "something no further input clears happened here", and it fails closed. An ordinary failure
//! that shares its attempt with a trip the grammar caught and parsed past is re-raised rather than
//! recovered; the reverse never happens. See `combinator::many`'s module docs for why the strong form —
//! "re-raise only when this very error *is* the trip" — is not implementable while the grammar's
//! error type may be `()`.
//!
//! Both baselines are per attempt, and taking them here is what makes that unspellable rather than
//! merely pinned. `SkipThenRetry`'s retry baseline in particular has to be **per retry cycle**: the
//! session counter is monotone and never cleared, so a baseline hoisted out of the retry loop
//! degrades into a session-absolute read and every retry after the parse's first trip re-raises,
//! ordinary syntax errors included. There is no longer a baseline at that call site to hoist.

use crate::error::{MaybeIncomplete, MaybeTerminal};
use crate::input::{ResourceTripBaseline, ScannerTripBaseline};

use super::*;

/// The emitter error of a recovery attempt's context, spelled once.
type Failure<'inp, L, Ctx, Lang> =
  <<Ctx as ParseContext<'inp, L, Lang>>::Emitter as Emitter<'inp, L, Lang>>::Error;

/// What one recovery attempt yielded, already judged.
pub(super) enum Attempted<O, E> {
  /// The attempt succeeded, and its progress is kept.
  Done(O),
  /// The attempt failed with something no further input clears — an `Incomplete`, or a terminal
  /// stop in either of the places terminality is stored. Re-raise it **untouched**: recovery
  /// synthesizes progress over a *malformed* construct, and neither an unfinished construct nor a
  /// tripped budget is that.
  Reraise(E),
  /// The attempt failed with an ordinary error. Recovery may run.
  Recoverable(E),
}

/// Runs `f` as a **speculating** attempt — [`try_attempt`](InputRef::try_attempt), so a failure
/// rolls back to the pre-attempt state (position, lexer state, emissions, dedup watermark, poison
/// boundary) — and judges its failure before that rollback happens.
///
/// The shape [`Recover`](super::Recover) and both of
/// [`SkipThenRetry`](super::SkipThenRetry)'s attempts take.
#[inline(always)]
pub(super) fn speculated_attempt<'inp, 'closure, L, O, Ctx, Lang: ?Sized, Cmpl, F>(
  inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  f: F,
) -> Attempted<O, Failure<'inp, L, Ctx, Lang>>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
  Failure<'inp, L, Ctx, Lang>: MaybeIncomplete + MaybeTerminal,
  F: FnOnce(
    &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  ) -> Result<O, Failure<'inp, L, Ctx, Lang>>,
{
  let trips = inp.trip_snapshot();
  let scans = inp.scanner_trip_snapshot();
  let latch = inp.latch_snapshot();
  settle(inp.try_attempt(|input| judge(input, f, &latch, trips, scans)))
}

/// Runs `f` **in place** — no checkpoint, no rollback, the input stays where the failure left it —
/// and judges its failure.
///
/// The shape [`InplaceRecover`](super::InplaceRecover) takes. Nothing rewinds the latch here, so
/// the read inside [`judge`] is the same read an after-the-fact one would be; it is inside for the
/// same reason both baselines are taken here, which is that there is then one body to check.
#[inline(always)]
pub(super) fn inplace_attempt<'inp, 'closure, L, O, Ctx, Lang: ?Sized, Cmpl, F>(
  inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  f: F,
) -> Attempted<O, Failure<'inp, L, Ctx, Lang>>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
  Failure<'inp, L, Ctx, Lang>: MaybeIncomplete + MaybeTerminal,
  F: FnOnce(
    &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  ) -> Result<O, Failure<'inp, L, Ctx, Lang>>,
{
  let trips = inp.trip_snapshot();
  let scans = inp.scanner_trip_snapshot();
  let latch = inp.latch_snapshot();
  settle(judge(inp, f, &latch, trips, scans))
}

/// Runs one attempt and, on failure, reads all five never-recoverable witnesses **before returning
/// to whatever wraps the attempt** — which for a speculating one is the rollback that would erase
/// the scanner latch.
///
/// The verdict rides out on the error channel beside the error, so the wrapper needs no
/// out-parameter and no cell: a `try_attempt` whose `Err` type is `(E, bool)` rolls back exactly as
/// one whose `Err` type is `E`.
#[inline(always)]
fn judge<'inp, 'closure, L, O, Ctx, Lang: ?Sized, Cmpl, F>(
  inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  f: F,
  latch: &Option<L::Offset>,
  trips: ResourceTripBaseline<'closure>,
  scans: ScannerTripBaseline<'closure>,
) -> Result<O, (Failure<'inp, L, Ctx, Lang>, bool)>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
  Failure<'inp, L, Ctx, Lang>: MaybeIncomplete + MaybeTerminal,
  F: FnOnce(
    &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  ) -> Result<O, Failure<'inp, L, Ctx, Lang>>,
{
  match f(inp) {
    Ok(output) => Ok(output),
    Err(err) => {
      let never = err.is_incomplete()
        || err.is_terminal()
        || inp.tripped_during_attempt(trips)
        || inp.scanner_tripped_during_attempt(scans)
        || inp.latched_during_attempt(latch);
      Err((err, never))
    }
  }
}

/// What one recovery **side step** yielded — the skip that looks for a sync point, and the advance
/// over a sync point that did not admit a parse.
///
/// Neither is a parse attempt: nothing speculates, nothing rolls back, and an `Err` out of one is
/// the emitter's own report rather than a grammar failure to be judged. What they share with an
/// attempt is the hazard: each folds a **terminal stop** into the same `Ok` value it uses for a
/// genuine end of input.
pub(super) enum Stepped<T> {
  /// The step completed, and no terminal stop happened inside it. Its value means what it says.
  Went(T),
  /// A terminal stop happened **inside the step**. Whatever `Ok` it produced, it is not the end of
  /// input it looks like: the budget is spent, and no further skipping or advancing clears it.
  Stopped,
}

/// Runs one recovery side step and samples the three input-side witnesses across it.
///
/// The error-value witnesses have no subject here — the step returned `Ok`, and its `Err` is
/// propagated untouched by the `?` at the call site, being already a report rather than something
/// to judge.
///
/// Reading **after** the step, rather than inside it, is correct for this shape and for one reason:
/// a step performs no speculation of its own. `sync_balanced`'s trip exit deliberately *keeps* its
/// entry snapshot (the skipped prefix is committed progress), and `next` never rewinds. Even where
/// that were not so, the counter reading would survive it — which is precisely the property the
/// latch reading does not have, and why the latch is the last term here rather than the first.
#[inline(always)]
pub(super) fn recovery_step<'inp, 'closure, L, T, Ctx, Lang: ?Sized, Cmpl, F>(
  inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  f: F,
) -> Result<Stepped<T>, Failure<'inp, L, Ctx, Lang>>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
  F: FnOnce(
    &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  ) -> Result<T, Failure<'inp, L, Ctx, Lang>>,
{
  let trips = inp.trip_snapshot();
  let scans = inp.scanner_trip_snapshot();
  let latch = inp.latch_snapshot();
  let went = f(inp)?;
  if inp.tripped_during_attempt(trips)
    || inp.scanner_tripped_during_attempt(scans)
    || inp.latched_during_attempt(&latch)
  {
    return Ok(Stepped::Stopped);
  }
  Ok(Stepped::Went(went))
}

/// Turns [`judge`]'s carried verdict into the three-way answer the combinators match on.
#[inline(always)]
fn settle<O, E>(out: Result<O, (E, bool)>) -> Attempted<O, E> {
  match out {
    Ok(output) => Attempted::Done(output),
    Err((err, true)) => Attempted::Reraise(err),
    Err((err, false)) => Attempted::Recoverable(err),
  }
}

// ── RECOVERY_GATE_CENSUS_END — production above, tests below ──────────────────
//
// `RECOVERY_GATE_CENSUS` reads this file's source and counts needles that its own constants also
// spell. The split is this marker rather than the first `#[cfg(test)]`, because the recovery
// suites are feature-gated and their attribute is not that literal; the census `expect()`s the
// marker, so deleting it fails loudly instead of silently widening the scan over test code.

#[cfg(all(test, feature = "many"))]
mod gate_census;
