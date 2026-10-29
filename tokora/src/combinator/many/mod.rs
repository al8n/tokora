//! The repetition drivers: `repeated`, `separated`, their `*_while` twins, and the delimited
//! forms of all four.
//!
//! # Resilience, and the granularity floor under it
//!
//! The four **try-driven** families are resilient by design: an element's `Err` is filed as a
//! diagnostic and the loop goes on to the next element. Three failures must never be spent that
//! way, because no further input clears any of them — a frontier `Incomplete`, a terminal
//! *scanner* stop, and a *descent* budget trip. [`file_element_failure`] is the **one** place an
//! element failure is either filed or re-raised, and it gates on all three before it files; the
//! drivers below carry no swallow of their own. The scanner stop is read there through **two**
//! witnesses rather than one, because the obvious reading of it lives inside the rollback set —
//! see "the two scanner readings" below.
//!
//! **A stop does not need an `Err` to be spent.** An element that meets a terminal stop and answers
//! it *itself* — catching the trip, or declining on a lookahead window the scanner truncated — hands
//! the driver an `Ok`, and the driver reads it as "no more elements" and **succeeds**. That is the
//! same defect through the exit the failure chokepoint does not cover: a resource-budget stop
//! becomes an accepted absence. [`absence_after_element`] is the second chokepoint, and the **one**
//! place a driver's *absence* exits consult either witness. Why the two are separate functions, and
//! why the two witnesses take different baselines, is on it.
//!
//! **The absence and real-closer chokepoints are not confined to the try-driven four.** The other
//! eight guard-bearing sources — the four `*_while` collection drivers and the four fold sources —
//! never file an element's `Err` at all, so a trip an element *hands back* propagates untouched and
//! is terminal there without any gate. What they could not see is the same `Ok`: an element that
//! answers the trip itself and then declines, or accepts consuming nothing, ended those collections
//! cleanly and successfully. All twelve now take the per-element trip baseline and route every
//! absence exit through [`absence_after_element`] and every probed closer through
//! [`close_after_element`]; `GATE_CENSUS`'s
//! `every_driver_baselines_its_trip_witness_inside_its_element_loop` is what keeps the baseline in
//! the loop that hosts each call, in all twelve.
//!
//! **The two FACTS are not the same kind, and a committed closer settles only one of them.** A
//! scanner stop is, in its positional aspect, a fact about a *token position*: a construct that
//! closed on a real pre-trip token closed **before** the boundary, so a boundary latched past that
//! closer says nothing about it, and gating a real close on it would fail a parse a wider window
//! completes identically. A descent trip is a *counter event that happened during the element
//! attempt*: a valid closer arriving afterwards does not unmake it. An element that caught one,
//! reported *no more elements*, and was then followed by a real closer therefore yields a
//! **successfully closed collection that silently spent a resource-limit stop** — the same defect,
//! through the one exit the absence chokepoint deliberately does not cover. So the real-closer
//! exits take the descent witness and **neither** scanner one, through [`close_after_element`].
//! Three chokepoints rather than one guard with a mode flag, because the three exits differ in what
//! they hold: an error to re-raise, every witness, or the descent counter alone.
//!
//! The descent witness is not carried by the error value. It is a **monotone session counter** on
//! the input (`Input::resource_trips`), bumped by [`InputRef::descend`](crate::InputRef::descend)
//! before the grammar's `From` runs, and the gate compares it against a baseline taken **once per
//! element** (`InputRef::trip_snapshot` / `InputRef::tripped_during_attempt`). That is what makes
//! the re-raise independent of the error type: a
//! [`RecursionLimitReached`](crate::error::RecursionLimitReached) that a discarding sink erases on
//! conversion — `()` does — still cannot be filed here.
//!
//! ## The two scanner readings, and why a latch alone cannot answer
//!
//! The scanner half has a counter of its own, for a reason the descent half never had to face. The
//! two readings the drivers *used* to have — the poison boundary's value at an absence exit, and
//! the committed cursor's position at a failure exit — are both inside the **rollback set**: a
//! [`Checkpoint`](crate::input::Checkpoint) carries the boundary and a restore copies it back
//! verbatim, along with the cursor, the cache and the emissions.
//!
//! Grammar code is entitled to catch a scanner stop inside an [`attempt`](crate::InputRef::attempt)
//! of its **own** and then decline it. That inner rollback puts the boundary back to the value the
//! *driver* snapshotted and the cursor back behind it, so at the exit that follows every reading of
//! either says clean — over a stop that is live, already diagnosed, and will re-trip on the next
//! scan of the same prefix. Relocating a read closes one nesting level and opens the next: **a cell
//! inside the rollback set cannot witness an event across a rollback at any depth.**
//!
//! So the load-bearing scanner witness is `Input::scanner_trips`, a monotone session counter bumped
//! by `InputRef::latch_if_limit_tripped` — the crate's sole terminal predicate, reached by both
//! lexing drivers — read through `InputRef::scanner_trip_snapshot` /
//! `InputRef::scanner_tripped_during_attempt`. It is outside the rollback set by construction, and
//! it is the same answer `combinator::recovery_gate` reached for the
//! recovery combinators. Its baseline here is **per collection**, beside `latch_snapshot()` and
//! deliberately unlike the descent counter's per-element one; [`absence_after_element`] says why
//! the two granularities differ.
//!
//! ## What a counter witnesses, and what it cannot
//!
//! `tripped_during_attempt` proves that **a** trip happened while the element ran. It does not
//! prove that the `Err` in hand **is** that trip. Element code that catches a trip itself, carries
//! on, and then fails *ordinarily inside the same element* hands the driver an ordinary error with
//! the counter already moved — and the driver re-raises it instead of filing it and continuing.
//! That is a real residual, and
//! `tokora/tests/collection_resource_trip.rs::a_caught_trip_inside_one_element_re_raises_its_ordinary_failure`
//! pins the behaviour rather than wishing it away.
//!
//! **The strong form — "re-raise only when this very error is the trip" — is not implementable**:
//! deciding it means interrogating the error value, and the grammar's error type may be `()`,
//! which discards the trip on conversion. A sink that discards is the whole premise of the change
//! that put the witness on the input, so a design that could tell the two errors apart would be
//! reading a payload that is not there.
//!
//! So the floor is **one element** here, and **one attempt** for
//! [`Recover`](crate::combinator::Recover), [`InplaceRecover`](crate::combinator::InplaceRecover) and
//! [`skip_then_retry`](crate::ParseInput::skip_then_retry). Within that unit the verdict is
//! "something no further input clears happened here", and it **fails closed** at the failure,
//! absence and real-closer chokepoints, and at those three recovery combinators: an ordinary
//! failure sharing its unit with a caught trip is re-raised, never the reverse, and a real trip
//! that reaches one of them is never filed as a diagnostic and never recovered from. `Accept`
//! reaches none of these gates — see "the channel neither chokepoint closes" below for the one
//! exit this floor does not cover.
//!
//! Finer granularity is reachable, but only with cooperation from whoever catches the trip: an
//! explicit **rebaseline** — code that deliberately catches a trip declaring it settled, so the
//! enclosing baselines move past it. That is the design if a consumer ever needs it. It is not
//! built, because the escape hatch has no consumer yet and this crate does not publish API on
//! speculation.
//!
//! ## The channel neither chokepoint closes, by design
//!
//! [`absence_after_element`] and [`close_after_element`] gate every exit that concludes the
//! collection from an element's **absence** — a decline, a no-progress stall, or a real closer
//! committed after either. An `Accept` concludes nothing of the driver's: the element produced a
//! value, and the driver is faithfully collecting what it was handed rather than manufacturing "no
//! more elements" out of a stop the caller never learns about. Neither chokepoint is called on
//! that path, and neither should be — refusing it would mean refusing a value the grammar actually
//! produced, on the strength of a stop the grammar already answered.
//!
//! So an element that catches a trip, still consumes, and still answers `Accept` spends it —
//! permanently, not only for that one element. The next cycle takes its own trip baseline
//! ([`InputRef::trip_snapshot`](crate::InputRef::trip_snapshot)) fresh, *after* the accepting
//! cycle's trip already happened, so by the time that next cycle's own absence or close exit runs,
//! the counter has not moved *during it* and there is nothing left for either chokepoint to see.
//! This is the same per-element granularity floor described above, not a special case carved out
//! of it: an accepting cycle was never a unit either chokepoint judges in the first place, so there
//! was never a baseline for a later cycle to inherit.
//!
//! **Gating `Accept` would not be a narrower version of this design; it would be a different one.**
//! Decline and the no-progress stall are gated because the *driver* is the one concluding the
//! construct ended, on evidence a stop may have truncated — that conclusion is the driver's to
//! guard. `Accept` is the element's own conclusion, carrying a value the driver did not manufacture
//! and has no independent reason to distrust. Refusing it would mean a value-producing element
//! could never recover from a budget it deliberately caught — true of every error a grammar is
//! free to catch and answer, not only this one, since tokora has no way to stop a grammar from
//! catching an error and returning a value without diagnosing it. That is a broader contract than
//! this crate makes for any other error, and nothing here proposes making it.
//!
//! `tokora/tests/collection_resource_trip.rs`'s section 6 pins this directly: an element that
//! catches a trip, consumes, and answers `Accept`, followed by a real closer, and the collection
//! succeeds identically whether or not the budget actually tripped. It exists so this boundary
//! cannot drift silently in either direction — narrower, if `Accept` starts being gated; or wider,
//! if the decline/stall/closer exits above stop being gated and the section's non-vacuity controls
//! stop noticing.
//!
//! # The eight element loops, and which of their differences are essential
//!
//! [#259](https://github.com/al8n/tokora/issues/259)'s progress-and-terminal-stop criterion
//! admits two answers — a **clearly identified shared engine** *or* **deliberately documented
//! exceptions**. This section is the per-loop verdict on which of the two each difference is. It
//! was decided against `tokora/tests/repetition_behavioural_matrix.rs` rather than by reading,
//! and the rule it was decided by is: a difference is **incidental** when a property can be
//! stated that all eight loops satisfy, and **essential** when stating it requires naming a
//! loop — the naming is then the documented exception, and the matrix carries it as an
//! assertion rather than as a skipped case.
//!
//! The eight are `repeated`, `repeated_while`, `sep/parse`, `sep/delim`, `sep_while/parse`,
//! `sep_while/delim`, `delim/repeated` and `delim/repeated_while`.
//!
//! ## What all eight already share
//!
//! Counted comment-blind, the way `END_STATE_CENSUS` counts, so a prose mention is not a site.
//! Counted per **loop**, which is no longer the same as per file: `repeated` and `delim/repeated`
//! run one loop text between them, `many/macros.rs`'s `try_element_cycle!`, so the rows below that
//! sit inside the element cycle — the stall test, the descent baseline, the element `Err` and the
//! admission — are spelled six times for eight drivers. `GATE_CENSUS` counts the sources
//! accordingly and reads that one body with needles written for macro text.
//!
//! | law | one mechanism, in every loop | sites per loop |
//! |---|---|---|
//! | progress | one stall test, `new_committed <= committed` over `span().end()` — never the cache-front cursor | 1, in all eight |
//! | terminal scanner stop | `latch_snapshot()` beside `scanner_trip_snapshot()`, taken once per collection | 1 + 1, in all eight |
//! | descent budget trip | `inp.trip_snapshot()`, inside the element loop, so once per element | 1, in all eight |
//! | an element's `Err` | [`file_element_failure`] | 1 in each try-driven loop, 0 in each `*_while` one — the resilience axis, and nothing else, decides who files one |
//! | an element's absence | [`absence_after_element`], on every decline and every stall exit | 1 to 6, one per absence exit; the delimited loops have more exits because a close probe follows each |
//! | a real closer | [`close_after_element`] | 1 to 3 in each delimited loop, 0 in each undelimited one |
//! | one admitted element | [`admit_element`] | 1 or 4, except `sep/delim` and `sep_while/delim`, which reach it through `sep{,_while}/parse`'s own `handle_continue` and carry no admission at all |
//! | what is collected | layer A7: one abstract fixture reaches one collection whichever of the four structural shapes runs it | — |
//! | where the input rests | layer B9: all eight come to rest at the lookahead front | — |
//!
//! ## The verdicts
//!
//! | loop | what it does that the others do not | verdict |
//! |---|---|---|
//! | `repeated` | files an element's `Err` as a diagnostic and goes on; stops on the element's own decline. Returns the span its [`RepeatedHandler::on_stop`] built | **essential** on resilience — layer A6 asserts it against the `*_while` twin. **Incidental** on the stop-hook return: `repeated_while` computes the same span itself and throws the handler's away |
//! | `repeated_while` | asks a caller condition over a `W`-wide decision window; never files an element failure; binds `span_since` before the stop hook. Like all four `*_while` loops it is written for `Complete` alone, where the try-driven four are generic over `Cmpl` | **essential** on resilience and on the window — that pair *is* what `while` means. The `Complete` restriction is essential by mechanism and **unmeasured**: a fixed-width decision window cannot tell a short read from a truncated one, and the matrix runs `Complete` only. **Incidental** on the stop-hook shape |
//! | `sep/parse` | peeks a separator slot before every element through `try_expect_or_stop`, and runs a four-arm `State` machine over it | **essential** — the slot is what `Separated` means. It contributes its own tokens and its own diagnostics and nothing else: over `junk_middle` it records one diagnostic where `repeated` records none, and A7 shows the collection is unchanged |
//! | `sep/delim` | the same state machine under an opener and a closer, with the slot probed by `try_expect_map` because the same position may hold the closer | **essential** on both counts. The three-way probe exists because a delimited list's separator slot is also a close position; the undelimited form has no closer there and needs the terminal re-raise instead |
//! | `sep_while/parse` | `sep/parse`'s slot with `repeated_while`'s condition | **essential**, as the union of the two axes above |
//! | `sep_while/delim` | `sep/delim`'s slot and closer with `repeated_while`'s condition | **essential**, as the union of the three axes above |
//! | `delim/repeated` | a delimited *plain* loop that **shares** `repeated`'s element cycle through `try_element_cycle!` and adds an opener, a close position in the cycle's decline hole, and a close-probe epilogue; takes its count hook and its stop hook as two separate arguments, a third stop-hook shape; implements **one** destination contract | **essential** on the delimiter alone, and that is now what the source shows: the cycle is one text and the delimiter is one hole in it. **Incidental** on the other three — `sep/delim` is delimited too and reuses its undelimited sibling's handlers, carries one hook, and implements four contracts |
//! | `delim/repeated_while` | the same three, with `repeated_while`'s condition — and, unlike its try-driven sibling, it still **re-states** the element loop rather than sharing one | same verdict on the three. The re-statement is the **documented exception** below: the delimiter enters that cycle at its core rather than at its edge, so there is no contiguous cycle to share |
//!
//! ## The one re-stated cycle, documented rather than removed
//!
//! [#259](https://github.com/al8n/tokora/issues/259)'s criterion admits "a clearly identified
//! shared engine **or deliberately documented exceptions**". Of the two plain pairs whose element
//! loop was re-stated, one is now shared and the other is that exception, and the split is
//! structural rather than a matter of effort.
//!
//! **The try pair shares.** `repeated` and `delim/repeated` run the same cycle — descent baseline,
//! attempt, `Accept →` [`admit_element`], `Err →` [`file_element_failure`], stall test,
//! bookkeeping — differing *only* in the decline arm, where the plain driver breaks and the
//! delimited one probes its close position. The delimiter enters that cycle **at its edge**, so
//! the cycle is a skeleton with one hole and both of the old texts were exact instances of it.
//! `many/macros.rs`'s `try_element_cycle!` is that skeleton, invoked once by each driver, and the
//! hole takes each driver's own decline arm verbatim — which is why
//! [`close_after_element`] is still reachable from delimited driver text alone.
//!
//! **The while pair does not, and the reason is where the delimiter enters.** In
//! `delim/repeated_while` the closer is probed **first, every cycle, before the decision window**,
//! because a while-driver must not ask the caller's condition about a closer token — so the probe
//! cannot follow the decision the way the try family's follows the element's own decline. The
//! delimiter enters that cycle **at its core**. What the two while loops actually share is the
//! window peek plus the terminal check (four lines), the `Continue` arm (two), and the stall tail
//! — non-contiguous fragments *nested inside* the delimited probe's third arm, with the stop arms
//! differing besides. Sharing them needs a mid-cycle hole whose tokens receive the probe's binding
//! and re-invoke the skeleton from inside the caller's own hole; the result would be the least
//! readable macro in the crate, to retire about eight scattered lines per driver.
//!
//! **Why the price is not paid anyway.** The extraction was measured, in the function shape, on
//! 2026-08-22: it preserved behaviour and moved three witnessed instruction-count rows —
//! `at_least` +2.383%, `at_most_shallow` +1.214% (aarch64) / +1.030% (x86_64), `at_most` +0.535% —
//! while nine rows read +0.000%. The cost is a codegen re-roll rather than a semantic tax
//! (`try_element_cycle!`'s own documentation carries the evidence), so it lands unpredictably, per
//! monomorphization and per ISA, and the only candidate whose cost is knowable in advance is the
//! token-identical one. That candidate is available for the try cycle and not for the while cycle,
//! which is exactly the split above.
//!
//! So this is the terminal state, not a retreat: every incidental duplication the verdicts above
//! found is now either consolidated — the sixteen destination impls, and the try cycle — or
//! documented here with the reason and the measurement that priced it. What holds the eight
//! drivers to one behaviour is `tokora/tests/repetition_behavioural_matrix.rs`'s seventeen
//! properties, not source resemblance, and that was true before either consolidation.
//!
//! ## Where the input rests, and why it is not a family difference
//!
//! Every driver builds its reported span with [`InputRef::span_since`](crate::InputRef::span_since),
//! which reads [`InputRef::cursor`](crate::InputRef::cursor) — the lookahead front, documented on
//! that method as *the start of the first cached token, otherwise the current position*. So a
//! repetition construct's reported end runs to the start of the next token, **trailing trivia
//! included**, whenever it stopped by looking at a token it did not consume, and to the end of the
//! last token it consumed when it looked no further.
//!
//! That is uniform, and stage 2 had to measure it because this crate's own prose said otherwise —
//! that `repeated` rests at the end of its last token while `separated` rests at the start of the
//! next one, so a caller's span would differ by which combinator it picked. It does not: both
//! reach the stop through a peek that declined, and a declined peek leaves the token cached
//! whether the peek came from the separator slot or from the element parser's own lookahead. On
//! `1 2 3    +` under `repeated` and on `1 , 2 , 3    +` under `separated`, both report an end
//! four bytes past the last element, at the `+`; both commit only as far as the `3`. Layer B9
//! states it over all forty-eight rows, and the difference is in the **arm** —
//! peeked-and-declined against consumed-to-the-end — not in the family.
//!
//! ## Two API-surface asymmetries, both incidental
//!
//! * **`delim/repeated` and `delim/repeated_while` implement only the owning
//!   `Collect<_, Container>`.** Their undelimited siblings implement three destination contracts
//!   (owning to `Container`, owning to `Spanned<Container>`, borrowed to `L::Span`) and the four
//!   separated loops implement four. Being delimited is not what stops them: `sep/delim` and
//!   `sep_while/delim` are delimited and implement all four. The borrowed contract is the only one
//!   that lets a caller see the container on the **failure** arm, which is layer B8's whole
//!   subject, so the two loops missing it are the two the matrix cannot ask that question of.
//! * **`Separated` and `SeparatedWhile` expose no `at_least(n).at_most(m)` chaining.** Ten
//!   [`Apply`](crate::parse::Apply) impls exist and all ten are `Repeated`'s or `RepeatedWhile`'s;
//!   the two cardinality builders reach `Bounded` through direct constructors instead. The missing
//!   pair is `Apply<Bounded<P>> for AtLeast<P>` and its `AtMost` mirror — three lines each,
//!   mentioning no separator, forwarding to a `Bounded<Separated<..>>` that already drives. It
//!   costs the matrix's A2 relation half its subject: `bounded(n, n)`, `at_least(n).at_most(n)`
//!   and `at_most(n).at_least(n)` can only be related where three paths exist.
//!
//! ## What a consolidation pass could take, in order of what each one buys
//!
//! Only the incidental findings are here; the essential ones above are the exceptions the
//! criterion asks to be documented rather than removed.
//!
//! 1. ~~**`delim/repeated` and `delim/repeated_while` re-state the plain element loop instead of
//!    wrapping it.**~~ **Taken for the try pair; documented for the while pair.** It was the
//!    largest incidental difference, and it paid twice — the loop spelling, plus the destination
//!    contracts that were missing *because* the loop was spelled
//!    `parse_repeated(inp, container, counts, on_stop)` rather than as `Repeated::parse` under a
//!    delimiter. The second half was banked by
//!    [#334](https://github.com/al8n/tokora/pull/334), which gave the eight delimited leaves the
//!    destination contracts directly. The first half splits by where the delimiter enters the
//!    cycle — `delim/repeated` shares
//!    `try_element_cycle!` with `repeated` and carries no stall test, admission or
//!    [`file_element_failure`] gate of its own, and `delim/repeated_while` is the one documented
//!    re-statement, for the reason and at the price recorded under "the one re-stated cycle" above.
//!    What remains here is the *wrapping*: both delimited drivers still spell their own
//!    `parse_repeated`-shaped entry point rather than driving the plain parser under a delimiter,
//!    which is what `sep/delim` and `sep_while/delim` do with their undelimited siblings' handlers.
//!    The instruction-count gate is what bounds the cost of taking it.
//! 2. **Three stop-hook shapes for one end-of-construct pass.** [`RepeatedHandler::on_stop`]
//!    returning the span (`repeated` uses it, `repeated_while` discards it and rebuilds the same
//!    span itself), `EndStateHandler`'s four state-dispatched methods, and a bare
//!    `FnOnce(usize, &mut InputRef, &L::Span) -> Result<(), _>` closure in the two delimited plain
//!    loops. `END_STATE_CENSUS` exists partly to pin which of two success-exit spellings each
//!    driver uses; one shape retires that half of it.
//! 3. **`Apply<Bounded<P>> for AtLeast<P>` and its `AtMost` mirror, for `Separated` and
//!    `SeparatedWhile`.** Four small impls. They buy the builder parity above and let layer A2 —
//!    the relation over three construction paths to one cardinality — cover eight drivers instead
//!    of four.
//! 4. **The descent baseline's placement**, beside the element attempt in `repeated` and
//!    `delim/repeated` and at the top of the cycle in the other six. Lowest: the widened window is
//!    already argued at each site (nothing between the two points can descend, so the reading is
//!    the one the element attempt would have given), so unifying it buys uniformity and no
//!    behaviour, and costs re-making that argument once per driver.

// `UnexpectedEot` reaches this family's descendants through their `use super::*`; the drivers
// under `repeated*/`, `sep*/` and `delim/` all name it in a `From` bound and none of them import
// it directly.
use crate::{
  Emitter, ParseContext, ParseInput,
  container::Container as ContainerT,
  emitter::FullContainerEmitter,
  error::{FullContainer, UnexpectedEot},
  input::{CloseStatus, Cursor, InputRef},
  lexer::Lexer,
  parse::{Decision, Window},
  span::{Span as _, Spanned},
};

use super::*;
use crate::input::{ResourceTripBaseline, ScannerTripBaseline};
use handler::*;

pub use delim::*;
pub use options::*;
pub use repeated::*;
pub use repeated_while::*;
pub use sep::*;
pub use sep_while::*;

#[macro_use]
mod macros;

mod delim;
// `pub(crate)` for `crate::parse`, which re-exports the two handler traits declared here.
pub(crate) mod handler;
mod repeated;
mod repeated_while;

mod options;
mod sep;
mod sep_while;

/// Admits one parsed element: settles its **count bound**, then offers it to the destination and
/// reports a refused push exactly once per construct, **at the refusal**.
///
/// The single chokepoint for both container-accounting laws; it replaced twelve separate
/// emission sites that had grown three different conventions between them. The count verdict was
/// folded in afterwards, for the reason under "Why the maximum is settled HERE" below — the two
/// steps are one function because their **order** is the property, and an order kept by a
/// convention at N call sites is a property that a rewrite of those sites does not inherit.
///
/// # Every number here is the driver's own
///
/// [`Container`](crate::container::Container) is implementable outside this crate, so it is
/// **untrusted input to the accounting**, not a participant in it. Two consequences, and they
/// are the same rule seen from two sides:
///
/// * **`nums` counts elements the driver PARSED**, not elements the container stored. Count
///   bounds are a property of the input: a container that ran out of room must not turn a
///   satisfied `at_least` into a `TooFew`, nor silently swallow a violated `at_most` by
///   clamping the count it is judged on. Container inadequacy has its own diagnostic.
/// * **`FullContainer`'s count is `nums` too.** It used to be `container.len() + 1`, which asked
///   a caller-implemented method for a number the driver already had — and believed the answer,
///   including that a refused push left `len()` alone, that a successful one moved it by one,
///   and that it never passes `max_capacity()`. None of that is stated anywhere a downstream
///   implementor would read it, and `len() + 1` on a `len()` of `usize::MAX` overflows. The
///   driver's own count needs none of it: `*nums + 1` is **which element of this construct the
///   destination refused**, a fact about this loop and nothing else.
///
/// # What the diagnostic may therefore say
///
/// A refusal, and not an exceedance. Asserting that the count passes `max_capacity()` is a claim
/// about the destination's **occupancy**, which the driver does not read and has no right to: a
/// destination handed to `collect_with` already holding elements is perfectly conforming and
/// refuses the construct's *first* element, so `*nums + 1` is `1` at any capacity. The obligation
/// the trait does keep — a refusal means the destination cannot hold the item — says nothing
/// about counts, so no count claim can rest on it.
///
/// The number is unchanged from the line above; the sentence built from it is not. The old
/// `len() + 1` reached a larger value on that seeded path only by trusting the occupancy this
/// function stopped reading, so restoring the arithmetic claim means restoring the dependency.
/// [`FullContainer`]'s `Display` carries the rendered form and the rest of the reasoning.
///
/// **The report is made once per construct as a matter of diagnostic policy** — one report per
/// construct, not one per dropped element — and *not* because a container that refused once is
/// assumed to refuse again. The reported numbers describe the refusal that was actually
/// witnessed; nothing here predicts, or needs, what a later push does.
///
/// # Why the report is made HERE, and why its ordering cannot be bought back
///
/// `TooMany` and `TooFew` say the *input* disagrees with the grammar. `FullContainer` says the
/// *caller's destination* was too small for an input that may be perfectly well formed. Those
/// are not peers, and it is tempting to make the second one land after the first, from each
/// driver's end pass.
///
/// **Withholding the report until the construct's end is what buys that, and it is not payable.**
/// In this driver the emitter is not a log — it is the thing that decides whether the parse
/// continues. `Ok(())` means *recovered, keep going*; `Err` means *stop now*. So moving **when** a
/// diagnostic is emitted moves **when** the parse stops, and a report withheld to the end costs
/// three things at once:
///
/// * a rejecting emitter — [`Fatal`](crate::emitter::Fatal), documented to stop at the first
///   error — does not stop at the refusal, because nothing has asked it yet;
/// * every later `Err` exit (a lexer error, an element failure, a delimiter, a recovery stop)
///   propagates past the withheld report and **discards a diagnostic that was witnessed**, so
///   the parse's error identity is the later failure and the refusal is never told;
/// * the refusal stops being constant work. A destination that refuses at element 2 is an
///   O(1)-prefix decision; parsing the whole remaining input before delivering it makes that
///   O(n) over input the caller does not choose.
///
/// **The two emitter classes want opposite things and the trait cannot tell them apart here.**
/// A rejecting emitter needs the call at the refusal; a recovering one would prefer it after the
/// count bounds. [`Emitter`] exposes no classification — no associated const, no marker trait,
/// nothing readable before the call — and it could not usefully expose one, because the class is
/// a property of the **call and its argument**, not of the emitter: an error-budget emitter
/// recovers until its budget runs out, and the crate's own docs make the point on
/// [`Severity`](crate::emitter::Severity) ("whether a diagnostic stops parsing is the
/// `Emitter`'s policy"). The only signal is the `Result` this call returns, and reading it means
/// having made the call.
///
/// So the capacity report stays at the refusal, and `capacity_report_timing.rs` holds the three
/// cells that were red while it did not.
///
/// # Why the maximum is settled HERE, one line earlier
///
/// Fixing the capacity report at the refusal leaves the *relative* order of the two refusals one
/// element can collect still to be decided, and for a while each driver decided it differently:
/// the maximum was detected mid-loop in the two plain families, from an end callback in the four
/// delimited ones, and from the end-state pass in the four separated ones. So `at_most(1)` with a
/// capacity-one destination and two parsed elements recorded `[TooMany, FullContainer]` under two
/// builders and `[FullContainer, TooMany]` under the other six — and under `Fatal`, which stops at
/// the first diagnostic, the **public error changed with the builder** (#277). `end_state_parity`
/// states the contrary invariant in one line: the same logical history must produce the same
/// diagnostic vector no matter which builder the user picked.
///
/// The two candidate orders are not symmetric, and three independent readings pick the same one:
///
/// * **Chronology.** "This element exceeds the maximum" is settled by the driver's own
///   parsed-element count the moment the element parses. "The destination refused it" cannot be
///   settled until the driver *offers* it, which is strictly later. There is no history in which
///   one element's refusal precedes its own count verdict, so the divergence was a placement
///   accident rather than a genuine tie.
/// * **Dependency.** The maximum is decidable without asking anyone. The refusal is a
///   caller-implemented [`Container`](crate::container::Container)'s answer, and the section
///   above holds that container to be untrusted input to this accounting rather than a
///   participant in it. Reporting the untrusted party's verdict first inverts that exactly where
///   it is most visible.
/// * **Which fact the parse turns on.** A same-element collision can only happen at element
///   `max + 1`, and only if elements `1..=max` were *accepted* — a destination that refused
///   earlier already spent this construct's one capacity report on that earlier element. So the
///   destination is provably large enough for every element the grammar allows: **enlarging it
///   never clears the parse**, while **trimming the input to `max` elements clears both**. The
///   maximum is the root and the refusal is a consequence of parsing an element the grammar had
///   already rejected.
///
/// Hence the count handler runs first, from inside this function rather than beside its call. The
/// hook is [`ElementCountHandler::on_element`], whose contract is that a construct exceeds its
/// maximum **iff** some element saw the pre-element count equal to it — so a maximum reported here
/// is reported exactly once, and the end passes stop re-checking it
/// ([`Maximum::check`](crate::combinator::Maximum), [`RepeatedHandler::on_stop`], and the four
/// delimited-repeated `on_stop` closures all say so at their own sites).
///
/// **The order is a property of one function, not of twelve call sites.** That is the whole point
/// of folding the hook in here: [#259](https://github.com/al8n/tokora/issues/259) will rewrite
/// this subtree into shared engines, and an ordering convention spread over twelve loop bodies is
/// exactly the kind of property a rewrite re-establishes by accident or not at all. There is now
/// one admission, it settles the count first, and `ELEMENT_ADMISSION_CENSUS` pins that no driver
/// reaches a container any other way. The rule a rewrite can check itself against is that
/// sentence.
///
/// # Cross-element order is untouched
///
/// The law is about **one** element. A destination that fills at element 3 while the maximum is
/// first exceeded at element 5 still records `[FullContainer, TooMany]`, under every driver,
/// because those are two elements and the refusal really did happen first.
/// `repetition_diagnostic_order.rs` carries that control beside the collision rows.
///
/// The latch is read only on the refusal arm, so the success path is the count hook — a
/// monomorphized no-op for `Unbounded` and for a minimum-only bound — plus the pre-existing `push`
/// and increment.
#[inline(always)]
pub(super) fn admit_element<'inp, 'closure, C, O, L, Ctx, Lang: ?Sized, Cmpl, EC>(
  counts: &EC,
  nums: &mut usize,
  full: &mut bool,
  container: &mut C,
  item: O,
  inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  anchor: &Cursor<'inp, 'closure, L>,
) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: crate::input::Completeness,
  Ctx::Emitter: FullContainerEmitter<'inp, L, Lang>,
  C: ContainerT<O>,
  EC: ElementCountHandler<'inp, 'closure, L, Ctx, Lang, Cmpl>,
{
  // `*nums` is the count BEFORE this element, which is what makes `== max` fire exactly once per
  // construct. It has to be read here, ahead of the push that increments it.
  counts.on_element(*nums, inp, anchor)?;

  if container.push(item).is_err() && !*full {
    *full = true;
    let span = inp.span_since(anchor);
    inp.emitter().emit_full_container(FullContainer::of(
      span,
      *nums + 1,
      container.max_capacity(),
    ))?;
  }
  *nums += 1;
  Ok(())
}

/// Files one element failure as a diagnostic, or re-raises it — **the single place either can
/// happen** in the try-driven collection families.
///
/// The section-4 never-recoverable law, moved off the four loop bodies and into one chokepoint.
/// It used to be four hand-written copies of a three-way match guard, pinned by a census that
/// counted one exact spelling of the swallow beneath it; a fifth loop written with a different
/// spelling, or written in a file the census did not name, swallowed ungated and the census stayed
/// green. Here there is no second swallow to spell: a driver hands its element failure to this
/// function or propagates it with `?`, and the gate is upstream of the only `emit_error` in the
/// tree.
///
/// # The four witnesses, in order
///
/// Three facts, and two readings of the scanner one — neither of which subsumes the other, because
/// one is positional and inside the rollback set while the other is a counter outside it:
///
/// * `Cmpl::is_incomplete_error(&err)` — the frontier `Incomplete`. A constant `false` under
///   [`Complete`](crate::input::Complete), so the complete path keeps its codegen.
/// * `inp.at_committed_boundary()` — the **scanner** stop, *positionally*. Reads the *committed*
///   cursor, so it is attempt-relative: a boundary a prior lookahead already latched does not
///   mis-charge an ordinary element failure short of it. Never the lex-offset
///   `at_latched_boundary`, which a prefilled cache would make false-positive on that same ordinary
///   failure. **Not redundant** with the counter below and the reverse of the absence gate's
///   situation: this one sees a boundary latched *before the driver started* that the element has
///   now run onto, which no per-collection counter baseline can see.
/// * `inp.scanner_tripped_during_attempt(scans)` — the **scanner** stop, as a *session counter
///   event*. The reading no rollback reaches. An element that catches a trip inside an
///   [`attempt`](crate::InputRef::attempt) of its own and then fails ordinarily has that inner
///   restore rewind the cursor, the cache, the emissions **and** the poison boundary together — so
///   the positional witness above reads clean, the trip's own diagnostic is gone from the log, and
///   without this term the driver files a spent scanner budget as an ordinary syntax error and
///   keeps looping. See the module docs' "the two scanner readings" for why relocating a
///   rollback-set read cannot fix this at any depth.
/// * `inp.tripped_during_attempt(trips)` — the **descent** budget trip, which latches no boundary
///   (it has a control stack, not a position) and so is invisible to both witnesses above.
///
/// All four are positional/session facts read on the failure path, so a successful element does
/// zero terminal work and none of them costs a trait bound: no `MaybeTerminal` appears in these
/// families.
///
/// # What holds each witness in the guard
///
/// One behavioural suite per witness, each confirmed by neutering the term and watching the suite
/// go red. `GATE_CENSUS` also scans this body for all four, but a needle scan proves presence and
/// ordering, never that the term gates — the census says so itself, and says what it measured.
/// Editing the guard means keeping these green:
///
/// * `Cmpl::is_incomplete_error(&err)` — `input::input_ref::tests::partial`, 2 cells.
/// * `inp.at_committed_boundary()` — `tokora/tests/collection_terminal_stop.rs`, the `r1b_*` cells,
///   5 of them. Its `r1_*` sibling pins only the negative direction (the witness must not fire on
///   a boundary the cursor has not reached), which is why the term was for a while deletable with
///   the whole suite still green.
/// * `inp.scanner_tripped_during_attempt(scans)` — `tokora/tests/collection_terminal_stop.rs`'s
///   `r1c_*` cell, which no other witness in this guard can answer.
/// * `inp.tripped_during_attempt(trips)` — `tokora/tests/collection_resource_trip.rs`, 12 cells.
///
/// # `trips` is the caller's per ELEMENT, and `scans` is the caller's per COLLECTION
///
/// The descent baseline belongs to the *attempt this call judges*, so the caller takes
/// `inp.trip_snapshot()` **inside its element loop**, once per element — not hoisted out beside
/// `latch_snapshot()`. Hoisted, the comparison degrades into a read of the monotone session
/// counter, and every element failure after the parse's first trip re-raises — ordinary syntax
/// errors included. `GATE_CENSUS` pins the placement; the module docs state the granularity floor
/// this leaves, and why the floor cannot be lowered without cooperation from whoever catches a
/// trip.
///
/// The scanner baseline is the opposite, and takes the *latch's* granularity rather than the
/// descent counter's, because it answers the latch's question: `inp.scanner_trip_snapshot()` beside
/// `latch_snapshot()`, once per collection. A caught descent trip stops being true of the input; a
/// spent scanner budget does not — the token stream ends where it ended, and an element failure
/// after that point is a failure on a view the stop truncated. Per element it would also miss the
/// case an inner rollback creates: element *n* trips and Accepts, element *n+1* then fails against
/// the truncated remainder having tripped nothing itself. [`absence_after_element`] states the same
/// asymmetry for the exits with no error in hand.
#[inline(always)]
pub(super) fn file_element_failure<'inp, 'closure, L, Ctx, Lang: ?Sized, Cmpl>(
  inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  err: <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  since: &Cursor<'inp, 'closure, L>,
  scans: ScannerTripBaseline<'closure>,
  trips: ResourceTripBaseline<'closure>,
) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: crate::input::SurfaceIncomplete<'inp, L, Ctx, Lang>,
{
  if Cmpl::is_incomplete_error(&err)
    || inp.at_committed_boundary()
    || inp.scanner_tripped_during_attempt(scans)
    || inp.tripped_during_attempt(trips)
  {
    return Err(err);
  }
  let span = inp.span_since(since);
  inp.emitter().emit_error(Spanned::new(span, err))
}

/// Refuses an **absence** conclusion that a terminal stop produced — **the single place any of its
/// witnesses is consulted** on every guard-bearing driver's non-`Err` exits.
///
/// All twelve reach it: the four try-driven collection families, the four `*_while` ones, and the
/// four fold sources. [`file_element_failure`] is this function's twin on the `Err` exits, and it
/// is the narrower of the two — only the try-driven four swallow an element failure, so only they
/// need a gate above the swallow. The other eight propagate an element's `Err` with `?`, which
/// makes a trip they are *handed* terminal without any gate; what they needed was this one, for the
/// trip an element answers and hands back as `Ok`. They are deliberately **two** functions, not one:
///
/// * the failure path has an error in hand and **re-raises that value**; there is none here, so this
///   synthesizes a terminal end-of-input instead — the same value the scanner-stop exits have always
///   produced;
/// * the failure path's positional scanner witness ([`at_committed_boundary`]) is
///   load-bearing there, because it is discriminating a *failure* that reached the poison from one
///   that did not — including a poison latched before the driver even started. An absence exit
///   cannot use a positional reading at all: a restore over the latch rewinds the cursor behind a
///   boundary that survives, so every positional reading is clean there while the stop is live;
/// * and the scanner witnesses take different baselines between the two — the positional one none
///   at all, the counter a per-collection snapshot. A single helper would have to carry each and
///   pick, which is a `match` on which caller it has, not a chokepoint.
///
/// [`at_committed_boundary`]: crate::InputRef::at_committed_boundary
///
/// # The three witnesses, the two facts, and the two granularities
///
/// An absence exit concludes "no more elements" from what the last element attempt did. Two facts
/// make that attempt's evidence unrepresentative of the input, and an element that hits either can
/// still return `Ok`, which is exactly why neither reaches the failure chokepoint:
///
/// * `inp.scanner_tripped_during_attempt(scans)` — a terminal **scanner** stop, as a *counter
///   event*. This is the load-bearing scanner reading, and the only one of the three that no
///   rollback can erase. An element's own lookahead ([`peek`](crate::InputRef::peek) and friends)
///   latches a stop and still hands back `Ok` with a short window, so the element may decline, or
///   accept consuming nothing, on a truncated view — and it may equally catch that stop inside an
///   [`attempt`](crate::InputRef::attempt) of its own, decline the attempt, and have the restore put
///   the boundary back to the value this driver snapshotted. Its baseline is **per collection**, for
///   the reason the latch's is.
/// * `inp.latched_during_attempt(latch)` — the same stop as a *boundary value*, compared against a
///   per-collection snapshot. The narrower reading: it answers *a different stop is standing at this
///   exit*, where the counter answers *a stop happened inside this collection*. **It is subsumed**
///   — every transition this crate can produce that moves the latch goes through
///   `latch_if_limit_tripped`, which bumps the counter first, so this term adds no verdict the
///   counter does not already give. It is kept for the reason
///   `combinator::recovery_gate`'s equivalent term is: it is the reading the
///   boundary itself is the subject of, it costs one `Option<L::Offset>` clone per collection on a
///   path that already takes one, and a witness removed on a subsumption argument is a witness
///   removed on an argument rather than on a measurement.
/// * `inp.tripped_during_attempt(trips)` — a **descent** budget trip the element caught itself and
///   answered with `Decline`, or with an `Accept` that consumed nothing. It latches no boundary and
///   bumps no scanner counter — it has a control stack rather than a position — so both witnesses
///   above read `false` for one, and without this term a resource-budget stop is spent as an
///   *accepted absence*: the collection ends clean and succeeds. Its baseline is **per element**,
///   for the reason [`file_element_failure`] gives at length: the counter is a monotone session
///   fact, so a collection-wide baseline would charge every later absence exit in the parse with a
///   trip some earlier element caught and legitimately parsed past.
///
/// The asymmetry in the baselines is therefore a property of the two facts, not an oversight: a
/// spent scanner budget stays true of the input, a caught descent trip does not. Concretely, the
/// per-collection scanner baseline is what keeps *element 1 tripped and accepted, element 2
/// declines* refused; per element, element 2's baseline would be taken after element 1's trip and
/// the exit would read clean.
///
/// # Where it does **not** belong
///
/// Only on the exits that conclude absence *from an element attempt*. Not, **as a group**, on an
/// exit resting on a **real token**: the delimited drivers' `CloseStatus::Close` arms and the
/// mid-scan closer in `sep/delim` read a committed pre-trip token. That token settles the *position*
/// question and only the position question — the construct closed before any boundary a later
/// lookahead latched, so *neither* scanner term belongs there: the counter would refuse the same
/// parse the latch would, and a wider scan window completes it to the identical value. It settles
/// nothing about the *descent counter*: a trip the element already caught is not unmade by a closer
/// arriving after it. Those exits therefore take the descent witness alone, through
/// [`close_after_element`], and this function stays off them.
///
/// Not on a separator-slot probe that runs *before* this cycle's element either: nothing between the
/// cycle's baseline and that probe can trip, so the descent term would be a constant `false` there.
///
/// And not, ever, on an `Accept` — the third exemption, and the one worth naming rather than
/// leaving to be inferred from "this function is about absence exits". An `Accept` is not an
/// absence conclusion, so there is nothing here for any witness to refuse: the element produced
/// a value, and the driver collects it. That holds even when the element caught this very trip to
/// produce it. This is not a hole left open by accident — gating a value a grammar legitimately
/// produced would forbid recovering from a caught budget, for this error alone among every error a
/// grammar may catch and answer. See the module docs' "the channel neither chokepoint closes"
/// section for the reasoning in full, and `tokora/tests/collection_resource_trip.rs`'s section 6 for
/// the pin.
///
/// The exemption's *reach* differs per witness, and this is the one place to say so plainly. For the
/// **descent** counter it is permanent: the next element's own baseline is taken *after* the
/// accepting one's trip, so no later absence exit in this collection ever sees it either. For the
/// two **scanner** readings it is local to the accepting call, because both are baselined per
/// collection — an element that trips, accepts, and is followed by a declining element is refused at
/// *that* element's absence exit. That has always been the latch's behaviour; the counter now
/// answers the same way when a rollback erased the latch.
///
/// # What holds each witness in the guard
///
/// `GATE_CENSUS` pins that all twelve guard-bearing drivers spell none of these witnesses themselves
/// and that all three appear in this body ahead of the stop — but a needle scan proves presence and
/// ordering, never that a term gates. One behavioural suite per witness does, each confirmed by
/// neutering the term in place and watching the suite go red:
///
/// * `inp.scanner_tripped_during_attempt(scans)` —
///   `tokora/tests/absence_terminal_stop.rs::a_trip_caught_inside_an_elements_own_attempt_still_surfaces_terminal`.
///   The one cell no other witness here can answer: with the term neutered it returns `Ok([])` over
///   a spent budget, and the other 46 cells stay green.
/// * `inp.latched_during_attempt(latch)` — **nothing**. Measured, not assumed: with this term
///   replaced by `let _ = inp.latched_during_attempt(latch);` above the `if` and the counter left in
///   the condition, the whole `--all-features` suite passes — all 105 test binaries, including the
///   10 `absence_terminal_stop.rs` cells this term used to be the sole holder of. That is the
///   subsumption argument above turned into an observation, and it is the honest reading of this
///   row: the term is kept for the reasons given, not because a test would notice its absence.
/// * `inp.tripped_during_attempt(trips)` — `tokora/tests/collection_resource_trip.rs`'s section 4,
///   all 16 cells, and its section 7, all 32.
///
/// To re-check in five minutes rather than by re-deriving it: replace one term with
/// `let _ = <that term>;` above the `if`, leaving the others in the condition, and run
/// `cargo test -p tokora --all-features --no-fail-fast`. The suite named for that witness must go
/// red and the census must stay green. Repeat per witness.
#[inline(always)]
pub(super) fn absence_after_element<'inp, 'closure, L, Ctx, Lang: ?Sized, Cmpl>(
  inp: &InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  latch: &Option<L::Offset>,
  scans: ScannerTripBaseline<'closure>,
  trips: ResourceTripBaseline<'closure>,
) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: crate::input::Completeness,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
{
  if inp.scanner_tripped_during_attempt(scans)
    || inp.latched_during_attempt(latch)
    || inp.tripped_during_attempt(trips)
  {
    return Err(
      UnexpectedEot::eot_of(inp.span().end())
        .into_terminal()
        .into(),
    );
  }
  Ok(())
}

/// Refuses a **close** that a descent trip the element already answered stands behind — the
/// descent-only third chokepoint, for the delimited drivers' exits that commit a REAL closer after
/// an element attempt.
///
/// [`absence_after_element`] holds every witness because all of them bear on an absence conclusion.
/// Here exactly one does, and the difference is not a matter of degree — the **scanner** stop and
/// the **descent** trip are facts of different kinds, and a committed pre-trip closer settles one of
/// them and not the other:
///
/// * the **scanner** stop is, for this exit, a fact about a *token position*. A `CloseStatus::Close`
///   verdict is cache-first, so it rests on a real token that was read before any stop; the
///   construct closed **ahead of** whatever boundary the element's lookahead went on to latch, and
///   that boundary is simply not about this construct. Reading it here would fail a parse that a
///   wider scan window completes to the identical value. That is why **both** of the absence gate's
///   scanner readings are deliberately absent from this body — the latch
///   (`latched_during_attempt`) and the session counter (`scanner_tripped_during_attempt`) alike,
///   since a counter bumped by the very lookahead that latched past the closer answers exactly as
///   wrongly. `GATE_CENSUS` pins both absences rather than leaving them to be noticed;
/// * the **descent** trip is a *counter event that happened inside the element attempt*. Nothing
///   arriving afterwards unmakes it. An element that catches a
///   [`RecursionLimitReached`](crate::error::RecursionLimitReached), reports *no more elements*, and
///   is then followed by a real closer produces a **successfully closed collection that silently
///   spent a resource-limit stop** — the position was fine and the budget was not.
///
/// So the verdict "a real token closed this construct" is a true statement about *where* the parse
/// is and says nothing about *what the attempt before it cost*. One gate per fact.
///
/// # Baseline, and the exits this is for
///
/// `trips` is the caller's, per **element**, exactly as [`file_element_failure`] and
/// [`absence_after_element`] take it, and for the same reason: the counter is a monotone session
/// fact, so a collection-wide baseline would charge every later close in the parse with a trip some
/// earlier element caught and legitimately parsed past.
///
/// It belongs on **every probed closer** — every `CloseStatus::Close` verdict in the tree — and the
/// rule is deliberately that flat, with no per-site exemption. Where the probe follows an element
/// attempt whose baseline is still in hand the term is the whole point of this function:
/// `delim/repeated`'s decline arm and its stall epilogue, `delim/repeated_while`'s epilogue,
/// `sep/delim`'s epilogue, and `sep_while/delim`'s stall arm. Where a probe instead sits at the TOP
/// of a cycle — `delim/repeated_while`'s mid-scan arm, and `sep_while/delim`'s front-empty and
/// `Action::Stop` arms — only an *accepting* element can precede it, this cycle's baseline is taken
/// a few lines above, and the term is a constant `false`; the call stays anyway, because a `u64`
/// comparison costs less than an exemption table, and because it lets `GATE_CENSUS` scan the
/// verdict→commit region **per verdict** rather than compare tallies that any arrangement of
/// needles would satisfy. It also fails closed if a later refactor moves an element attempt above
/// one of those probes.
///
/// The two exempt exits are **direct** closers, committed straight from a driver's own scan with no
/// probe verdict at all: `sep/delim`'s and `sep_while/delim`'s mid-scan arms. They are exempt for
/// the same structural reason, and are counted by the census as `direct` so a third cannot land
/// unnoticed.
///
/// Not on an `Accept` either — deliberately, and permanently, not merely "not yet reached". An
/// element that catches a trip and still produces a value has answered it, not concluded absence:
/// the driver is faithfully collecting what it was handed, not manufacturing a stop of its own, so
/// there is nothing here for this gate to refuse. The very next cycle's baseline is taken *after*
/// the accepting cycle's trip, which is why this stays true even when that next cycle is the one
/// that reaches a real closer: by then the counter has not moved *during* the cycle this call is
/// judging, and gating it anyway would mean refusing a value the grammar legitimately produced.
/// This is the same granularity floor the module docs state, applied to the one exit that never
/// takes a baseline for it to apply to — see the module docs' "the channel neither chokepoint
/// closes" section for the reasoning at length.
///
/// # What holds the witness in the guard
///
/// `GATE_CENSUS`'s `every_real_closer_exit_after_an_element_is_trip_gated` pins that every
/// `CloseStatus::Close` verdict in the tree reaches this call before it commits, and
/// `the_close_chokepoint_reads_the_counter_and_not_the_position` scans this body in **both**
/// directions — the descent witness present, and **both** scanner witnesses absent. A needle scan
/// proves presence and ordering, never that a term gates; the behaviour is
/// `tokora/tests/collection_resource_trip.rs`'s section 5, where an element catches a trip and then
/// declines — or accepts consuming nothing — with the closer genuinely present, over both sinks.
/// Neuter the term (`let _ = inp.tripped_during_attempt(trips);` above the `if`) and that section
/// reds while the census stays green.
#[inline(always)]
pub(super) fn close_after_element<'inp, 'closure, L, Ctx, Lang: ?Sized, Cmpl>(
  inp: &InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
  trips: ResourceTripBaseline<'closure>,
) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: crate::input::Completeness,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
{
  if inp.tripped_during_attempt(trips) {
    return Err(
      UnexpectedEot::eot_of(inp.span().end())
        .into_terminal()
        .into(),
    );
  }
  Ok(())
}

#[cfg(test)]
mod gate_census;

#[cfg(test)]
pub(super) mod end_state_census;
