use crate::{
  Token,
  error::{MaybeIncomplete, MaybeTerminal, UnexpectedEot},
  input::DelimClass,
  span::Span as _,
};

use super::{
  recovery_gate::{Attempted, Stepped, recovery_step, speculated_attempt},
  *,
};

/// A recovery combinator that skips to a synchronization point and retries the inner parser.
///
/// On failure of the inner parser, the input is rolled back to where the attempt began (the
/// failed attempt leaves no trace), then
/// [`sync_balanced`](InputRef::sync_balanced) skips forward — nesting-aware, using the held
/// [`DelimClass`] classifier and depth-0 sync predicate — and the inner parser runs again from
/// the sync point. Each successful skip is committed forward progress described by exactly one
/// skipped-region diagnostic (see [`Hole`](crate::input::Hole)); if an enclosing
/// [`attempt`](InputRef::attempt) or transaction rolls the whole recovery back, those
/// emissions unwind with the log like any other entry.
///
/// # The retry loop and the progress guard
///
/// A retry *cycle* is: sync to the next depth-0 sync point, then re-run the inner parser.
/// Cycles repeat while they make progress:
///
/// - a retry that **succeeds** ends the loop with its value;
/// - a cycle that **consumes nothing** — the sync point was already at hand and the retry
///   failed without net consumption — bails out with the error that triggered it (for the
///   first cycle, the original error), so a zero-consumption cycle can never loop;
/// - a retry that fails after real progress records its error as the next cycle's trigger and
///   **consumes the sync token** before re-syncing — that sync point did not admit a
///   successful parse, so the next cycle scans strictly past it. Every continuing cycle
///   therefore consumes at least one token, and the loop terminates: at the latest, a sync
///   that finds no further sync point (end of input, which itself leaves no trace) surfaces
///   the last recorded error.
///
/// # The never-recoverable law, and its terminal dual
///
/// An [`Incomplete`](crate::error::Incomplete) error, and a **terminal stop**, are re-raised
/// untouched — before any skip, and from any retry — exactly as [`Recover`] does, off the same five
/// witnesses it documents. Recovery synthesizes progress over a *malformed* construct: an
/// incomplete one is merely unfinished, so skipping would drop input that has not finished
/// arriving, and no quantity of skipped input un-trips a resource limit. See [`MaybeIncomplete`]
/// and [`MaybeTerminal`].
///
/// Each retry cycle is judged as its own attempt, against its own baselines. That is what keeps an
/// ordinary syntax error in cycle *n* recoverable after cycle *n − 1* legitimately caught and
/// parsed past a budget — the session counter is monotone, so a baseline shared across cycles would
/// refuse every retry after the parse's first stop.
///
/// ## The skip and the advance are covered too, and they are where the law used to leak
///
/// The law is about *recovery*, and this combinator's recovery is not only its retries: it is the
/// **skip** to a sync point and the **advance** over a sync point that did not admit a parse.
/// Neither is a parse attempt, and both used to run outside the gate entirely.
///
/// Both are `Ok`-returning primitives that fold a terminal trip into the value they use for
/// genuine exhaustion — [`sync_balanced`](InputRef::sync_balanced) answers `Ok(None)` whether it
/// found no sync point or the scanner tripped mid-skip, and [`next`](InputRef::next) answers
/// `Ok(None)` whether the input ended or the scan tripped. So a spent scanner budget read as
/// *"nothing left to skip to"*, and the combinator surfaced its ordinary trigger error for it.
/// Through a **rejecting** emitter that never showed, because the rejection propagates as an `Err`
/// from the skip itself; through an **accepting** one the trip's diagnostic is filed and the `Ok`
/// comes back looking exactly like end of input.
///
/// The witnesses are now sampled across both, and a stop inside either surfaces the same
/// terminal-marked [`UnexpectedEot`](crate::error::UnexpectedEot) every other committed exit in
/// this crate surfaces for a trip an accepting emitter took — so *"the scanner stopped"* and
/// *"there was nowhere to sync to"* are two different answers again, and only the second is
/// recoverable.
///
/// # Example
///
/// ```ignore
/// use tokora::{ParseInput, input::Balance};
///
/// // Parse a statement; on failure skip (nesting-aware) to the next `;` and retry.
/// let parser = parse_statement().skip_then_retry(
///     |kind: &TokenKind| match kind {
///         TokenKind::LBrace => Balance::Open('{'),
///         TokenKind::RBrace => Balance::Close('{'),
///         _ => Balance::Neutral,
///     },
///     |tok| matches!(tok.data(), Token::Semi),
/// );
/// // Input: "### ; let x = 1;" → skips `###` (one hole), retries at `;`…
/// ```
///
/// # See Also
///
/// - [`sync_balanced`](InputRef::sync_balanced) — the skip primitive and its contract
/// - [`Recover`] — recovery by an alternative parser from the original position
/// - [`InplaceRecover`] — recovery continuing from the error position
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SkipThenRetry<P, D, F, O, L, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  parser: P,
  classifier: D,
  pred: F,
  _m: PhantomData<O>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _l: PhantomData<L>,
  _cmpl: PhantomData<Cmpl>,
}

impl<P, D, F, O, L, Ctx, Lang: ?Sized, Cmpl> SkipThenRetry<P, D, F, O, L, Ctx, Lang, Cmpl> {
  /// Creates a new `SkipThenRetry` parser.
  #[inline(always)]
  pub(crate) const fn new(parser: P, classifier: D, pred: F) -> Self {
    Self {
      parser,
      classifier,
      pred,
      _m: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _l: PhantomData,
      _cmpl: PhantomData,
    }
  }
}

impl<'inp, P, D, F, L, O, Ctx, Lang, Cmpl> ParseInput<'inp, L, O, Ctx, Lang, Cmpl>
  for SkipThenRetry<P, D, F, O, L, Ctx, Lang, Cmpl>
where
  P: ParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
  D: DelimClass<<L::Token as Token<'inp>>::Kind>,
  F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
  // `From<UnexpectedEot<..>>` is what lets the skip and the advance report a terminal stop the way
  // every other committed exit in this crate reports one. It is the same conversion
  // `next_or_stop`, `try_expect_or_stop`, the peek family and both pratt engines already require —
  // the crate's universal end-of-input carrier, and one fifth of `FromTokenErrors` — so a grammar
  // that can reach end of input at all already has it.
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    MaybeIncomplete + MaybeTerminal + From<UnexpectedEot<L::Offset, Lang>>,
  Lang: ?Sized,
  Cmpl: SurfaceIncomplete<'inp, L, Ctx, Lang>,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    // First attempt, exactly `Recover`'s shape and through the same chokepoint: speculate so a
    // failure rolls back to the pre-parse state (position, lexer state, emissions), and re-raise an
    // `Incomplete` — or a terminal stop — untouched before any skip, since no skipping clears
    // either (the never-recoverable law and its terminal dual).
    //
    // Terminality is read from all five places it is stored, and this is the arm where the
    // difference cost committed input: through a sink that does not carry it, the cycles below used
    // to skip the whole tripping construct and its sync token, buying progress that cannot help,
    // because no quantity of skipped input makes the next descent shallower or un-trips a scanner
    // budget. See [`recovery_gate`](super::recovery_gate) for the five witnesses and their
    // baselines.
    let mut err = match speculated_attempt(inp, |input| self.parser.parse_input(input)) {
      Attempted::Done(output) => return Ok(output),
      Attempted::Reraise(e) => return Err(e),
      Attempted::Recoverable(e) => e,
    };

    loop {
      // The cycle's progress anchor: the committed position before this sync.
      let before = inp.cursor().as_inner().clone();

      // Skip to the next depth-0 sync point, THROUGH THE CHOKEPOINT. The classifier is re-borrowed
      // through a closure (any `DelimClass` is reusable across cycles that way); a fatal emitter
      // rejection mid-skip propagates per the sync family's fatal-exit discipline.
      //
      // The skip is recovery work, so it is gated like one — see
      // [`recovery_gate`](super::recovery_gate)'s "the work outside an attempt". `sync_balanced`
      // answers `Ok(None)` both for "no sync point before end of input" (which leaves no trace,
      // and for which this cycle's trigger error is the right thing to surface: there is nowhere
      // left to retry from) and for "the scanner tripped mid-skip" (which commits the skipped
      // prefix at the durable frontier and is a terminal stop). The step tells them apart.
      let classifier = &mut self.classifier;
      let pred = &mut self.pred;
      let synced = recovery_step(inp, |input| {
        input.sync_balanced(
          |kind: &<L::Token as Token<'inp>>::Kind| classifier.classify(kind),
          pred,
        )
      })?;
      match synced {
        // A terminal stop inside the skip. No quantity of further skipping clears it, so this is
        // not "nowhere to sync to" — it is the parse stopping, and it says so in the carrier every
        // other committed exit uses for a trip an accepting emitter took.
        Stepped::Stopped => return Err(terminal_stop(inp)),
        // Genuine exhaustion: no sync point before the end of input, and the skip left no trace.
        // This cycle's trigger error is the answer — there is nowhere left to retry from.
        Stepped::Went(found) => {
          if found.is_none() {
            return Err(err);
          }
        }
      }

      // Each retry cycle is its own attempt, and the chokepoint takes its baselines per call — so
      // this cycle's are taken here, on this iteration, and a stop an earlier cycle caused is
      // already an `Err` this loop returned rather than something to charge to this one. That
      // per-cycle granularity used to be a hand-written `trip_snapshot()` inside the loop, load
      // bearing and unguarded: hoisted out, the monotone session counter would make every retry
      // after the parse's first trip re-raise. There is no longer a baseline here to hoist.
      match speculated_attempt(inp, |input| self.parser.parse_input(input)) {
        Attempted::Done(output) => return Ok(output),
        // The law applies to every raise: an `Incomplete`, a terminal scanner stop in either of the
        // places it is stored, or a resource budget trip this retry caused re-raises unchanged,
        // with no further skipping.
        Attempted::Reraise(e) => return Err(e),
        Attempted::Recoverable(e) => {
          // The progress guard: a cycle that consumed nothing — zero-skip sync, and the
          // failed retry rolled back to the same spot — must not loop. Bail with the error
          // that triggered this cycle (for the first cycle, the original error).
          if *inp.cursor().as_inner() <= before {
            return Err(err);
          }
          err = e;
          // This sync point did not admit a successful retry: consume it so the next cycle
          // scans strictly past it — the guarantee that every continuing cycle consumes at
          // least one token. Through the chokepoint for the same reason the skip is: `next`
          // folds a fresh trip into the same `Ok(None)` it uses for genuine exhaustion, so
          // "nothing left to consume" and "the scanner stopped" arrived as one value.
          match recovery_step(inp, InputRef::next)? {
            Stepped::Stopped => return Err(terminal_stop(inp)),
            Stepped::Went(consumed) => {
              if consumed.is_none() {
                return Err(err);
              }
            }
          }
        }
      }
    }
  }
}

/// The terminal stop a sync or advance step ran into, in the carrier this crate uses for one.
///
/// Byte for byte what [`next_or_stop`](InputRef::next_or_stop) and
/// [`try_expect_or_stop`](InputRef::try_expect_or_stop) build on their `Tripped` arms, and what
/// `combinator::many::absence_after_element` builds when a collection's absence gate sees a stop — an
/// end-of-input error at the committed position, marked terminal. So a trip an accepting emitter
/// filed reaches the caller the same way from a recovery phase as it does from a leaf or from a
/// collection, which is the whole of what "terminality is a property of the event" buys. A
/// *rejecting* emitter never reaches here: its rejection is an `Err` out of the step itself.
///
/// The `From<UnexpectedEot<..>>` bound this needs is the one that absence gate already carries, and
/// the one every `_or_stop` primitive in the crate carries. There is no bound-free way to do this:
/// [`MaybeTerminal`] is a *predicate*, with nothing to construct from, and leaving the stop
/// unmarked is the defect rather than the alternative to it.
#[inline(always)]
fn terminal_stop<'inp, L, Ctx, Lang: ?Sized, Cmpl>(
  inp: &InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
{
  UnexpectedEot::eot_of(inp.span().end())
    .into_terminal()
    .into()
}

// ── RECOVERY_GATE_CENSUS_END — production above, tests below ──────────────────
//
// `RECOVERY_GATE_CENSUS` (in `combinator/recovery_gate.rs`) reads this file's source and counts
// needles that its own test fixtures also spell. The split is this marker rather than the first
// `#[cfg(test)]`, because the suites below are feature-gated and their attribute is not that
// literal; the census `expect()`s the marker, so deleting it fails loudly instead of silently
// widening the scan over test code.

// Recovery behavior needs a lexer that actually runs, which pins the suite to `logos` + `std` —
// the same gate as the `Recover` tests.
#[cfg(all(test, feature = "logos_0_16", feature = "std"))]
mod tests;
