//! GATE_CENSUS — the section-4 never-recoverable gate, locked to one chokepoint.
//!
//! Every resilient emit-and-continue loop body in the try-driven collection families must refuse
//! to spend a frontier `Incomplete`, a tripped *scanner* limit, or a tripped *descent* budget as
//! a diagnostic. The three witnesses live in [`file_element_failure`](super::file_element_failure)
//! and are documented there; what this module pins is that **nothing can reach the swallow
//! without passing them**.
//!
//! # What the previous shape of this census could not see
//!
//! It counted the literal text `emit_error(Spanned::new(span,` in four hard-coded sources and
//! required one gate per match. Both halves could be defeated without noticing:
//!
//! * a swallow spelled `emit_error(Spanned::new(other_span, err))` matched nothing, so the tally
//!   stayed balanced and the census stayed green over an ungated site;
//! * a swallow in a *fifth* source was not read at all.
//!
//! A census that answers by not looking is worse than no census, because it is quoted as
//! evidence. So the swallow moved: there is exactly **one** `emit_error` call in the whole
//! `many`/`fold` tree, it is the chokepoint's, and the gate sits above it. The three tests below
//! pin, in order: that no driver files a diagnostic itself *whatever the spelling*; that each
//! driver's per-element baseline is taken inside the loop that hosts its chokepoint call; and
//! that no module of the tree has been added without being classified.
//!
//! # The other half of the law: the exits that carry no error
//!
//! A stop does not need an `Err` to be spent. An element that catches a *descent* trip itself and
//! then declines — or that declines on a window a *scanner* stop truncated — hands the driver an
//! `Ok`, and the driver's absence exit reads it as "no more elements" and succeeds. The failure
//! chokepoint above never sees it. [`absence_after_element`](super::absence_after_element) is the
//! second chokepoint and holds all three witnesses for those exits, and
//! [`every_absence_exit_carries_the_terminal_witnesses`] applies the same shape of proof to it:
//! **all twelve** guard-bearing drivers spell **none** of them themselves, so there is nothing
//! left for them to spell differently, and the count of chokepoint calls is pinned per source so a
//! new absence exit cannot land ungated.
//!
//! One of those three is a *counter*, not a boundary, and that is what lets this half reach one
//! nesting level deeper than a boundary can. A driver's two rollback-set readings — the poison
//! boundary at an absence exit, the committed cursor at a failure exit — are both erased by an
//! element that catches a stop inside an [`attempt`](crate::InputRef::attempt) of its own and
//! declines it, so both answered `false` over a live stop. `Input::scanner_trips` is outside the
//! rollback set by construction; [`every_driver_baselines_its_scanner_witness_once_per_collection`]
//! pins its baseline at the granularity the fact has, which is the *opposite* of the descent one's.
//!
//! This half reaches further than the swallow scan does, and deliberately so. Only the try-driven
//! four can spend an element's `Err`, so only they need a gate above a swallow; the four `*_while`
//! drivers and the four fold sources propagate an element failure with `?` and are terminal on
//! that path for free. What they shared with the four was the exit with no `Err` in it — the
//! element answering the trip itself and then reporting absence — and until the per-element
//! baseline [`every_driver_baselines_its_trip_witness_inside_its_element_loop`] pins, those eight
//! could not even ask the question. [`absence_exit_shapes`] records what that cost and how it was
//! measured.
//!
//! # The third half: the exits that close on a real token
//!
//! An exit that commits a REAL closer is not an absence exit, and for a long time that was read as
//! "no witness belongs there". It is true of the scanner ones. A committed pre-trip closer is a
//! fact about *position*: the construct ended ahead of whatever boundary the element's lookahead
//! went on to latch, so neither scanner witness is about it — the boundary reading and the counter
//! reading come from the same lookahead — and gating on either would fail a parse that a wider
//! scan window completes identically. It is not a fact about the *descent counter*: a trip the
//! element caught happened inside the attempt that concluded "no more elements", and a valid
//! closer arriving afterwards does not unmake it — the collection closes successfully having
//! silently spent a resource-limit stop. [`close_after_element`](super::close_after_element) is the
//! third chokepoint and holds the descent witness **alone**;
//! [`every_real_closer_exit_after_an_element_is_trip_gated`] pins that every close verdict in the
//! tree reaches it before committing — **every** one, with no per-site exemption, which is what
//! lets that test scan the verdict→commit region per verdict instead of comparing tallies — and
//! [`the_close_chokepoint_reads_the_counter_and_not_the_position`] pins the asymmetry in both
//! directions, since either scanner term smuggled into that body is as much a defect as a missing
//! descent one. The only exits it exempts are the two **direct** closers, committed from a
//! driver's own scan with no probe verdict at all, and it counts those rather than ignoring them.
//!
//! # The two things the chokepoints cannot centralize, and they pull opposite ways
//!
//! Both baselines are the caller's, and getting either one's granularity from the other's is a
//! defect. They are scanned per caller because placement is a per-caller property.
//!
//! `trips` must be `inp.trip_snapshot()` taken **inside the element loop**, once per element — not
//! hoisted out beside `latch_snapshot()`. The counter is a monotone session fact and is never
//! cleared, so a baseline hoisted above the loop is arithmetically a session-absolute read for
//! every element after the first: an ordinary syntax error in an unrelated construct then ends the
//! collection instead of being filed, and one deep expression early in a document suppresses every
//! diagnostic after it. Hoisting it also restores the defect one nesting level up — a trip inside
//! an *inner* collection that an element swallows charged to the enclosing driver's next ordinary
//! failure.
//!
//! `scans` must be `inp.scanner_trip_snapshot()` taken **outside** it, once per collection, beside
//! `latch_snapshot()` — the exact opposite, because the two facts decay differently. A descent trip
//! an element caught and parsed past stops being true of the input; a spent scanner budget does
//! not, and every element after it is reading a stream the stop truncated. Taken per element the
//! baseline is re-read after each accepted trip, and *element 1 tripped and accepted, element 2
//! declines* concludes cleanly over a spent budget — the case the latch beside it has always
//! refused. [`every_driver_baselines_its_scanner_witness_once_per_collection`] pins that placement
//! in both directions, as its sibling does for `trips`.

use super::end_state_census::{code_find, code_matches};

/// The swallow itself — matched on the **call**, not on the arguments, so no spelling of the
/// span or the error binding can slip past. This is the needle whose count must be zero
/// everywhere but the chokepoint.
const EMIT: &str = "emit_error(";

/// The chokepoint: its call in a driver, and its definition in this module's own source.
const CHOKEPOINT: &str = "file_element_failure(";
const CHOKEPOINT_DEF: &str = "fn file_element_failure";

/// The four witnesses the chokepoint must consult before it reaches [`EMIT`] — three facts, with
/// the scanner one read both positionally and as a session counter. Neither scanner reading
/// subsumes the other: the positional one sees a boundary latched *before* the driver started that
/// the element has now run onto, and the counter sees a stop an inner rollback erased.
const WITNESSES: [&str; 4] = [
  "Cmpl::is_incomplete_error(&",
  "inp.at_committed_boundary()",
  "inp.scanner_tripped_during_attempt(",
  "inp.tripped_during_attempt(",
];

/// The absence chokepoint: its call in a driver, and its definition in this module's own source.
const ABSENCE: &str = "absence_after_element(";
const ABSENCE_DEF: &str = "fn absence_after_element";

/// The terminal end-of-input the absence chokepoint produces — the needle that ends its guard
/// region, the way [`EMIT`] ends the failure chokepoint's.
const ABSENCE_STOP: &str = "into_terminal()";

/// The three witnesses the absence chokepoint must consult before it reaches [`ABSENCE_STOP`].
///
/// Neither scanner half is the failure path's `at_committed_boundary`: a restore over the latch
/// rewinds the cursor behind a boundary that survives, so every positional reading is clean at an
/// absence exit while the stop is live. The **counter** is the load-bearing one — a restore taken
/// *before* the trip puts the boundary back too, so the latch is clean there as well — and it is
/// listed first for that reason.
const ABSENCE_WITNESSES: [&str; 3] = [
  "inp.scanner_tripped_during_attempt(",
  "inp.latched_during_attempt(",
  "inp.tripped_during_attempt(",
];

/// The descent-only close chokepoint: its call in a driver, and its definition in this module's
/// own source. It shares [`ABSENCE_STOP`] as the needle that ends its guard region.
const CLOSE_GATE: &str = "close_after_element(";
const CLOSE_GATE_DEF: &str = "fn close_after_element";

/// The one witness [`CLOSE_GATE`] must read, and the two it must **not**.
///
/// The asymmetry is the whole point of there being a third chokepoint, so it is pinned in both
/// directions rather than only in the direction that is easy to check. A real pre-trip closer
/// settles the *scanner* fact — the construct ended ahead of any boundary a later lookahead
/// latched — and settles nothing about the *descent counter* fact, which is a trip that already
/// happened inside the element attempt this exit is concluding from.
///
/// **Both** scanner readings are non-witnesses here, not just the latch. The lookahead that
/// latched past the closer also bumped the session counter, so a counter reading in this body
/// refuses exactly the parses the latch reading would — every delimited parse a wider scan window
/// completes to the identical value. Adding a witness to the absence gate without adding it here
/// as a *non*-witness is the shape that quietly breaks that, so the list is a list.
const CLOSE_WITNESS: &str = "inp.tripped_during_attempt(";
const CLOSE_NON_WITNESSES: [&str; 2] = [
  "inp.latched_during_attempt(",
  "inp.scanner_tripped_during_attempt(",
];

/// A real-closer exit, in the two spellings a driver can reach one by: the probe verdict that
/// hands the closer over, and the by-value commit that settles it. `CLOSE_HANDOFF` is the
/// container call every closer of either origin passes through, so a *new* closer commit of any
/// shape moves its tally.
const CLOSE_VERDICT: &str = "CloseStatus::Close(";
const CLOSE_COMMIT: &str = "inp.commit_probed(";
const CLOSE_HANDOFF: &str = "on_close_delimiter(";

/// The element attempt whose failure a driver must route through the chokepoint, the descent
/// witness's baseline, and the loop opener that baseline must be taken after.
const ATTEMPT: &str = "try_parse_input(inp)";
const BASELINE: &str = "inp.trip_snapshot()";
const HOSTING_LOOP: &str = "loop {";

/// The shared try-driven cycle: its invocation in a driver, and its own source.
///
/// `repeated` and `delim/repeated` no longer spell the element cycle. They invoke
/// `try_element_cycle!`, whose single body carries the loop, the baseline, the attempt, the
/// admission and the chokepoint call for both — so for those two sources the invocation is what
/// [`HOSTING_LOOP`] is for the other ten, and every region scan anchors on it instead.
const CYCLE: &str = "try_element_cycle!(";
pub(super) const CYCLE_NAME: &str = "many/macros.rs";
pub(super) const CYCLE_SRC: &str = include_str!("macros.rs");

/// [`BASELINE`] and [`ATTEMPT`] as the macro spells them.
///
/// Written for macro text on purpose, because half of these needles substring-match through a
/// `$` sigil by accident and half do not: `inp.trip_snapshot()` *is* inside
/// `$inp.trip_snapshot()`, and `try_parse_input(inp)` is *not* inside `$f.try_parse_input($inp)`.
/// A census that let the first kind ride its accident would be counting a site it cannot
/// describe, and would silently report nothing for the second. So the cycle's own sites are
/// counted with needles that name the `$`, and the driver-facing needles above stay at zero in
/// the two sources that now invoke the macro.
const MACRO_BASELINE: &str = "$inp.trip_snapshot()";
const MACRO_ATTEMPT: &str = "$f.try_parse_input($inp)";

/// The **scanner** witness's baseline, and the latch baseline it must sit beside.
///
/// The mirror image of [`BASELINE`], and the reason it is scanned separately: this one is per
/// COLLECTION and must be taken *above* the element loop, exactly where hoisting the descent one
/// would be the defect. A spent scanner budget stays true of the input, so a per-element baseline
/// would be taken after the trip an earlier element caught and every later exit would read clean —
/// which is the case "element 1 tripped and accepted, element 2 declines".
const SCANNER_BASELINE: &str = "inp.scanner_trip_snapshot()";
const LATCH_BASELINE: &str = "inp.latch_snapshot()";

/// SWALLOW SCAN — every source in the `many` and `fold` trees that can reach an [`InputRef`]
/// inside a repetition: the eight collection drivers, the four folds, the three tree `mod.rs`
/// files that could host a driver of their own, and `many/macros.rs`, which hosts the try-driven
/// element cycle itself. Only these are read for [`EMIT`], and
/// [`the_gate_census_covers_every_driver_module`] is what keeps the list from silently falling
/// behind the tree.
///
/// `macros.rs` is here because driver-reachable *loop* code lives there now. It is not a driver
/// — it takes no baseline of its own per collection and carries no exit — so it is deliberately
/// not in [`progress_guard_sites`]; the swallow scan is the one list whose question ("does this
/// source file a diagnostic behind the gate's back?") the cycle's body can answer.
///
/// [`InputRef`]: crate::InputRef
fn swallow_scan_sites() -> [(&'static str, &'static str); 16] {
  let mut sites = [("", ""); 16];
  let (guarded, rest) = sites.split_at_mut(12);
  guarded.copy_from_slice(&progress_guard_sites());
  rest.copy_from_slice(&[
    ("many/delim/mod.rs", include_str!("delim/mod.rs")),
    ("many/sep/mod.rs", include_str!("sep/mod.rs")),
    ("many/sep_while/mod.rs", include_str!("sep_while/mod.rs")),
    (CYCLE_NAME, CYCLE_SRC),
  ]);
  sites
}

/// The four try-driven families: the only drivers that swallow, and therefore the only ones whose
/// element cycle reaches the chokepoint.
///
/// The third column is where that cycle's TEXT lives. The two `sep` drivers spell their own; the
/// two plain ones share `try_element_cycle!`, so their chokepoint call, attempt and baseline are
/// counted once in [`CYCLE_SRC`] rather than once each here.
fn try_driven_sites() -> [(&'static str, &'static str, bool); 4] {
  [
    (
      "many/repeated/mod.rs",
      include_str!("repeated/mod.rs"),
      false,
    ),
    (
      "many/delim/repeated.rs",
      include_str!("delim/repeated/mod.rs"),
      false,
    ),
    (
      "many/sep/parse/mod.rs",
      include_str!("sep/parse/mod.rs"),
      true,
    ),
    (
      "many/sep/delim/mod.rs",
      include_str!("sep/delim/mod.rs"),
      true,
    ),
  ]
}

/// A diagnostic filed for an element failure passes the never-recoverable gate first, because
/// there is exactly one place a diagnostic can be filed and the gate is above it.
///
/// This is the spelling-independent half. The needle is `emit_error(` — the call, with no
/// argument text — so a swallow renamed, re-spanned or re-wrapped still counts, and the
/// assertion is that the count is **zero** in every driver rather than that it balances against
/// something. There is nothing left to balance: the drivers have no swallow of their own.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn every_element_failure_routes_through_the_gated_chokepoint() {
  for (name, src) in swallow_scan_sites() {
    assert_eq!(
      code_matches(src, EMIT),
      0,
      "{name}: a driver hands its element failure to `file_element_failure` or propagates it \
       with `?` — it never files a diagnostic itself. A swallow written here bypasses the \
       never-recoverable gate whatever it is spelled, which is the defect this census exists to \
       make unspellable"
    );
  }

  // …and the one place it can be filed is the chokepoint, whose own source is read here rather
  // than assumed. `code_matches` ignores whole-line comments, so the prose above and the
  // chokepoint's own doc comment cannot stand in for the call.
  let prod = super::end_state_census::many_mod_production();
  assert_eq!(
    code_matches(prod, EMIT),
    1,
    "`many/mod.rs`: the tree files element diagnostics through exactly one `emit_error` call, \
     inside `file_element_failure`"
  );

  // The three witnesses, in the chokepoint's own body and BEFORE the emission.
  //
  // WHAT THIS PROVES, EXACTLY: each witness is present, named, and textually ahead of the one
  // `emit_error`. Scanning the region from the definition to the call — rather than counting
  // needles over the whole file — is what makes that much true: three independent tallies are
  // equally satisfied by three witnesses scattered anywhere, and a region scan is not. Both
  // `find`s panic rather than pass when the thing being scanned for is absent, so a chokepoint
  // that has been renamed or gutted reports that instead of quietly finding nothing to check.
  //
  // WHAT IT DOES NOT PROVE is that the witnesses GATE the emission. Textual presence ahead of a
  // call is not control-flow domination. This exact body keeps the census green with the gate
  // entirely gone:
  //
  //     let _ = Cmpl::is_incomplete_error(&err);
  //     let _ = inp.at_committed_boundary();
  //     let _ = inp.tripped_during_attempt(trips);
  //     let span = inp.span_since(since);
  //     inp.emitter().emit_error(Spanned::new(span, err))
  //
  // Verified, not reasoned about: that body was compiled and the whole `--all-features` suite run
  // over it. This test passed. Proving domination from source text means writing a Rust parser
  // inside a test module that must also compile under `--no-default-features` with no `String`
  // and no `Vec`; the cost is out of proportion to a needle scan's value, and it is not what
  // makes the runtime behaviour safe.
  //
  // WHAT PROVES THE GATE GATES is behaviour, one suite per witness, each one confirmed by
  // neutering its term and watching the suite go red:
  //
  // * `Cmpl::is_incomplete_error(&err)` — `input::input_ref::tests::partial`
  //   (`gate_propagates_frontier_incomplete_out_of_repeated`,
  //   `lego_chain_runs_both_modes_to_equivalence`); 2 tests.
  // * `inp.at_committed_boundary()` — `tokora/tests/collection_terminal_stop.rs`, the five `r1b_*`
  //   cells; 5 tests. Without them the witness can be deleted with the entire suite still green (a
  //   mutation run showed it), because R1 only pins where it must stay quiet. R1b pins the other
  //   direction.
  // * `inp.tripped_during_attempt(trips)` — `tokora/tests/collection_resource_trip.rs`; 12 tests.
  //
  // So this scan is a fast tripwire on the source — it catches a witness dropped, renamed or
  // duplicated in one cheap unit test — and the three suites above are the proof. Delete a
  // witness and both fire; neuter one in place and only the suite does.
  //
  // TO RE-CHECK, in five minutes rather than by re-deriving it: replace one term of the guard
  // with `let _ = <that term>;` above the `if`, leaving the other two in the condition, and run
  // `cargo test -p tokora --all-features --no-fail-fast`. The suite named for that witness must
  // go red, and this test must stay green. Repeat per witness. Automating the loop needs
  // `cargo-mutants` — a full rebuild-and-test per mutant across a 100-binary suite, plus a
  // survivor baseline to triage — which is out of proportion to three terms in one function.
  let def_at = code_find(prod, CHOKEPOINT_DEF).unwrap_or_else(|| {
    panic!(
      "`many/mod.rs`: `{CHOKEPOINT_DEF}` is gone. The swallow chokepoint has been renamed or \
       removed; re-cut this census against whatever replaced it before trusting a green run"
    )
  });
  let body = &prod[def_at..];
  let emit_at = code_find(body, EMIT).unwrap_or_else(|| {
    panic!("`many/mod.rs`: the one `emit_error` call is not inside `file_element_failure`")
  });
  let guard = &body[..emit_at];
  for witness in WITNESSES {
    assert_eq!(
      code_matches(guard, witness),
      1,
      "`many/mod.rs`: the chokepoint must consult all four never-recoverable witnesses before \
       it files — `{witness}` is missing from, or duplicated in, the code ahead of its \
       `emit_error`. The two scanner readings are both required: the positional one for a \
       boundary latched before the driver started, the counter for a stop an inner rollback erased"
    );
  }
}

/// Every try-driven element cycle takes its trip baseline **inside the loop that hosts the
/// chokepoint call**, once per element — in the two sources that spell their own cycle, and in
/// the one macro body the other two share.
///
/// The half the chokepoint cannot own: `trips` is passed in, so the caller decides whether the
/// comparison means "this element tripped" or "this parse has tripped". The `rfind` panics
/// rather than passing when a call is not inside a loop at all, so this cannot be satisfied by a
/// source that has stopped looking like a driver.
///
/// Four drivers, three cycle texts. `repeated` and `delim/repeated` invoke `try_element_cycle!`,
/// so what is asserted of them is that they spell **none** of the cycle themselves and invoke it
/// exactly once — the "nothing left to balance" form, the same one the swallow scan uses. The
/// cycle's own body is then read once, with needles written for macro text, because
/// [`BASELINE`] rides a `$` accidentally there and [`ATTEMPT`] would silently match nothing.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn every_element_loop_baselines_its_trip_witness_per_element() {
  let mut routed = 0;
  let mut invocations = 0;
  for (name, src, own) in try_driven_sites() {
    if !own {
      assert_eq!(
        code_matches(src, CYCLE),
        1,
        "{name}: a driver that does not spell its own element cycle invokes `{CYCLE}` exactly \
         once. Two invocations are two cycles under one collection's baselines; none means the \
         element loop went somewhere this census cannot read"
      );
      for needle in [CHOKEPOINT, ATTEMPT, BASELINE, HOSTING_LOOP] {
        assert_eq!(
          code_matches(src, needle),
          0,
          "{name}: this driver shares the cycle, so it spells none of it — `{needle}` here is a \
           second, unshared element loop, or the shared one being re-stated beside its own \
           invocation"
        );
      }
      invocations += 1;
      continue;
    }

    let calls = code_matches(src, CHOKEPOINT);
    assert_eq!(
      calls, 1,
      "{name}: one element loop, one chokepoint call — found {calls}"
    );
    assert_eq!(
      code_matches(src, ATTEMPT),
      calls,
      "{name}: every element attempt's failure must reach the chokepoint. An attempt without \
       one either swallows behind the gate's back or propagates with `?`; if the second, move \
       it out of this list and say why"
    );
    assert_eq!(
      code_matches(src, BASELINE),
      calls,
      "{name}: exactly one `trip_snapshot()` baseline per chokepoint call — a second, stray read \
       of the counter is how a session-absolute test gets back in"
    );

    let mut checked = 0;
    let mut from = 0;
    while let Some(at) = code_find(&src[from..], CHOKEPOINT) {
      let call_at = from + at;
      let loop_at = src[..call_at].rfind(HOSTING_LOOP).unwrap_or_else(|| {
        panic!("{name}: a chokepoint call that is not inside a repetition loop")
      });
      assert_eq!(
        code_matches(&src[loop_at..call_at], BASELINE),
        1,
        "{name}: the descent witness is attempt-relative — take the `trip_snapshot()` baseline \
         INSIDE the element loop, once per element. Hoisted out of the loop it reads the \
         monotone session counter, and every element failure after the parse's first trip \
         re-raises, ordinary syntax errors included"
      );
      checked += 1;
      from = call_at + CHOKEPOINT.len();
    }
    assert_eq!(
      checked, calls,
      "{name}: every chokepoint call must have been inspected"
    );
    routed += calls;
  }
  assert_eq!(
    (routed, invocations),
    (2, 2),
    "the try-driven families carry exactly four gated element cycles: two spelled in their own \
     source, and two invocations of the one shared `try_element_cycle!`"
  );

  // The shared cycle, read once, with needles written for its own text. Everything the two
  // invoking drivers stopped spelling has to be here, exactly once, and in the same relation.
  for (needle, want) in [
    (HOSTING_LOOP, 1),
    (MACRO_BASELINE, 1),
    (MACRO_ATTEMPT, 1),
    (CHOKEPOINT, 1),
    (ABSENCE, 0),
    (CLOSE_GATE, 0),
  ] {
    assert_eq!(
      code_matches(CYCLE_SRC, needle),
      want,
      "{CYCLE_NAME}: the shared try cycle is one loop, one baseline, one attempt and one \
       chokepoint call, and it holds no absence or close gate at all — those stay in driver \
       text, which is what keeps `close_after_element` the delimited four's alone. `{needle}` \
       is not {want}"
    );
  }
  let call_at = code_find(CYCLE_SRC, CHOKEPOINT).expect("the shared cycle calls the chokepoint");
  let loop_at = CYCLE_SRC[..call_at]
    .rfind(HOSTING_LOOP)
    .unwrap_or_else(|| panic!("{CYCLE_NAME}: the chokepoint call is not inside the cycle's loop"));
  assert_eq!(
    code_matches(&CYCLE_SRC[loop_at..call_at], MACRO_BASELINE),
    1,
    "{CYCLE_NAME}: the descent witness is attempt-relative — the shared cycle takes its \
     `trip_snapshot()` baseline INSIDE its own loop, once per element, for both drivers at once. \
     Hoisting it out of the macro body would make it per collection in two drivers with one edit"
  );
}

/// How many element cycles each guard-bearing source runs, and how many of them it **spells
/// itself**. Index-aligned with [`progress_guard_sites`].
///
/// One row per source rather than a bare total, because the two fold sources that host three
/// driver impls each are exactly where a fourth impl could land without one. Each row's count is
/// asserted against **two** independent needles — the cycle openers (`loop {` for a source that
/// spells its own, `try_element_cycle!(` for one that shares) and the `trip_snapshot()` calls —
/// so a loop added without a baseline, or a baseline added outside a loop, breaks the equality
/// rather than being absorbed by it.
///
/// The second column is `0` for exactly two sources: `repeated` and `delim/repeated` invoke
/// `try_element_cycle!`, whose one body holds both their loops and both their baselines. Those
/// rows therefore assert the *absence* of both needles in driver text plus one invocation, and
/// the shared body is read once by
/// [`every_element_loop_baselines_its_trip_witness_per_element`]. A source that grew a second,
/// unshared loop moves its first column and fails here before it can be trusted.
fn element_loop_counts() -> [(&'static str, usize, usize); 12] {
  [
    ("many/repeated/mod.rs", 1, 0),
    ("many/repeated_while/mod.rs", 1, 1),
    ("many/delim/repeated.rs", 1, 0),
    ("many/delim/repeated_while.rs", 1, 1),
    ("many/sep/parse/mod.rs", 1, 1),
    ("many/sep/delim/mod.rs", 1, 1),
    ("many/sep_while/parse/mod.rs", 1, 1),
    ("many/sep_while/delim/mod.rs", 1, 1),
    ("fold/mod.rs", 3, 3),
    ("fold/rfold.rs", 1, 1),
    ("fold/fold_while.rs", 3, 3),
    ("fold/rfold_while.rs", 1, 1),
  ]
}

/// The needle that opens a source's element cycle: its own `loop {`, or the invocation of the
/// shared one. Every region scan below anchors on whichever of the two the source uses, so a
/// driver that shares the cycle is scanned from the invocation exactly as one that spells it is
/// scanned from its `loop {`.
const fn cycle_opener(own: usize) -> &'static str {
  if own == 0 { CYCLE } else { HOSTING_LOOP }
}

/// **All twelve** guard-bearing drivers take their trip baseline inside the element loop, once per
/// element, and every call to any of the three chokepoints reads one taken there.
///
/// [`every_element_loop_baselines_its_trip_witness_per_element`] is this test's narrower sibling:
/// it pins the *failure* chokepoint's placement in the four try-driven sources, where the extra
/// `try_parse_input`-per-chokepoint pairing is meaningful. This one covers the other two
/// chokepoints and the other eight sources, which is where the baseline was missing entirely
/// until the `*_while` drivers and the folds adopted it.
///
/// The half the chokepoints cannot own is the same one stated on
/// [`file_element_failure`](super::file_element_failure): `trips` is the caller's, so the caller
/// decides whether the comparison means "this element tripped" or "this parse has tripped". The
/// counter is a monotone session fact, never cleared, so a baseline hoisted above the loop is
/// arithmetically a session-absolute read for every element after the first — and an absence exit
/// reading that would end every later collection in the document over a trip some earlier element
/// legitimately caught and parsed past.
///
/// The `rfind` panics rather than passing when a chokepoint call is not inside a loop at all, so
/// this cannot be satisfied by a source that has stopped looking like a driver.
///
/// [`every_driver_baselines_its_scanner_witness_once_per_collection`] is the exact mirror of this
/// test for the *scanner* counter, whose baseline must be **outside** the loop this one requires
/// its baseline inside. The two are separate tests because they are opposite properties, and
/// writing them as one would need a flag naming which is which at every assertion.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn every_driver_baselines_its_trip_witness_inside_its_element_loop() {
  let sites = progress_guard_sites();
  let counts = element_loop_counts();
  let mut baselines = 0;
  let mut gates = 0;
  for i in 0..sites.len() {
    let (name, src) = sites[i];
    let (classified, loops, own) = counts[i];
    let opener = cycle_opener(own);
    assert_eq!(
      name, classified,
      "the guard-bearing source list and the element-loop classification have drifted apart at \
       row {i}: `{name}` against `{classified}`. They are index-aligned on purpose — a row that \
       names a source the scan does not read is a row that checks nothing"
    );
    assert_eq!(
      code_matches(src, HOSTING_LOOP),
      own,
      "{name}: expected {own} element loop(s) spelled in this source. A driver impl added here \
       needs its own baseline and its own classification row before it can be trusted, and a \
       driver that shares `try_element_cycle!` must not re-state the cycle beside its invocation"
    );
    assert_eq!(
      code_matches(src, CYCLE),
      loops - own,
      "{name}: expected {} invocation(s) of the shared `{CYCLE}`. The row says this source drives \
       {loops} element cycle(s) and spells {own} of them, so the rest are invocations — a count \
       that has drifted means a cycle is running under baselines nobody classified",
      loops - own
    );
    assert_eq!(
      code_matches(src, BASELINE),
      own,
      "{name}: exactly one `trip_snapshot()` baseline per element loop THIS SOURCE SPELLS — a \
       loop without one leaves its exits ungated, a second, stray read of the counter is how a \
       session-absolute test gets back in, and a baseline in a source that shares the cycle is a \
       second reading beside the one the macro already took"
    );

    // Every chokepoint call, of all three kinds, reads a baseline taken inside the cycle that
    // hosts it. The region is `loop {` → call for a source that spells its own cycle, so a
    // baseline hoisted above the loop leaves the region empty and fails here; and
    // `try_element_cycle!(` → call for one that shares, where the baseline is the macro's and
    // the region must therefore hold none — a reading taken between the invocation and a gate is
    // a second, per-collection one.
    for needle in [CHOKEPOINT, ABSENCE, CLOSE_GATE] {
      let mut from = 0;
      while let Some(at) = code_find(&src[from..], needle) {
        let call_at = from + at;
        let loop_at = src[..call_at].rfind(opener).unwrap_or_else(|| {
          panic!("{name}: a `{needle}` call that is not inside an element cycle (`{opener}`)")
        });
        assert_eq!(
          code_matches(&src[loop_at..call_at], BASELINE),
          usize::from(own > 0),
          "{name}: the descent witness is attempt-relative — the `trip_snapshot()` baseline is \
           taken INSIDE the element cycle, once per element. Hoisted out of it the comparison \
           degrades into a read of the monotone session counter, and every absence exit after \
           the parse's first trip refuses, ordinary ends of construct included"
        );
        gates += 1;
        from = call_at + needle.len();
      }
    }
    baselines += loops;
  }
  assert_eq!(
    (baselines, gates),
    (16, 40),
    "sixteen element cycles across the twelve sources, each with its own baseline, and forty \
     chokepoint calls in driver text reading one: two failure gates, thirty absence gates and \
     eight real-closer gates. The other two failure gates are the one `try_element_cycle!` body \
     serves, counted by \
     `every_element_loop_baselines_its_trip_witness_per_element`. The totals are pinned so a \
     gate cannot move between sources unnoticed"
  );
}

/// **All twelve** guard-bearing drivers take their scanner-trip baseline once per COLLECTION,
/// beside the latch and **outside** the element loop — and every gate that reads one is a gate
/// that has one hoisted above it.
///
/// The exact mirror of
/// [`every_driver_baselines_its_trip_witness_inside_its_element_loop`], and the reason the two
/// cannot be one test: the descent counter's baseline must be *inside* the loop and the scanner
/// counter's must be *outside* it, because the two facts decay differently. A descent trip an
/// element caught and legitimately parsed past stops being true of the input; a spent scanner
/// budget does not — the token stream ends where it ended.
///
/// What a per-element scanner baseline would cost, stated as the case it drops: element 1 trips,
/// catches the stop and *accepts*; element 2 declines. Element 2's baseline would be taken after
/// element 1's trip, the counter would not have moved during element 2, and the collection would
/// conclude cleanly over a spent budget. Hoisted, it refuses — which is what the latch beside it
/// has always done, and what the counter now also does when a rollback erased the latch.
///
/// Both directions are pinned. The count per source stops a second, stray baseline from being
/// added (which would silently re-narrow one gate), and the region scan — the source's cycle
/// opener → call, which must contain **zero** of them — stops the baseline being moved into the
/// loop. The `rfind` panics rather than passing when a call is not inside a cycle at all.
///
/// This one counts per COLLECTION, so its per-source count is the row's cycle count and not the
/// number of cycles the source spells: `repeated` and `delim/repeated` share
/// `try_element_cycle!` and still take their own scanner baseline and their own latch, above
/// their own invocation, because those are the collection's and not the cycle's. That is the
/// same asymmetry stated as a diff — the descent baseline moved into the shared body and these
/// two did not.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn every_driver_baselines_its_scanner_witness_once_per_collection() {
  let sites = progress_guard_sites();
  let counts = element_loop_counts();
  let mut baselines = 0;
  let mut gates = 0;
  for i in 0..sites.len() {
    let (name, src) = sites[i];
    let (classified, loops, own) = counts[i];
    let opener = cycle_opener(own);
    assert_eq!(
      name, classified,
      "the guard-bearing source list and the element-loop classification have drifted apart at \
       row {i}: `{name}` against `{classified}`. They are index-aligned on purpose — a row that \
       names a source the scan does not read is a row that checks nothing"
    );
    assert_eq!(
      code_matches(src, SCANNER_BASELINE),
      loops,
      "{name}: exactly one `{SCANNER_BASELINE}` baseline per collection — the same count as the \
       `{LATCH_BASELINE}` it sits beside, because the two answer the same question at the same \
       granularity. A collection without one leaves its absence and failure exits blind to a stop \
       an element caught inside an `attempt` of its own"
    );
    assert_eq!(
      code_matches(src, SCANNER_BASELINE),
      code_matches(src, LATCH_BASELINE),
      "{name}: the scanner counter's baseline and the latch's are taken together, once per \
       collection. A count that has drifted apart means one gate is reading a per-collection fact \
       and its neighbour is not"
    );

    // The two gates that read it, and the negative half: NOTHING between the hosting cycle's
    // opener and the call may take this baseline, because taking it there makes it per element.
    for needle in [CHOKEPOINT, ABSENCE] {
      let mut from = 0;
      while let Some(at) = code_find(&src[from..], needle) {
        let call_at = from + at;
        let loop_at = src[..call_at].rfind(opener).unwrap_or_else(|| {
          panic!("{name}: a `{needle}` call that is not inside an element cycle (`{opener}`)")
        });
        assert_eq!(
          code_matches(&src[loop_at..call_at], SCANNER_BASELINE),
          0,
          "{name}: the scanner witness is COLLECTION-relative — take `{SCANNER_BASELINE}` above \
           the element loop, beside the latch. Taken inside it, the baseline is re-read after \
           every trip an earlier element caught and accepted, and the very next exit reads clean \
           over a spent budget"
        );
        gates += 1;
        from = call_at + needle.len();
      }
    }
    baselines += loops;
  }
  assert_eq!(
    (baselines, gates),
    (16, 32),
    "sixteen collections across the twelve sources, each with its own scanner baseline, and \
     thirty-two gates in driver text reading one: two failure gates and thirty absence gates. \
     The other two failure gates are the shared `try_element_cycle!` body's, which takes no \
     scanner baseline at all — that one is the collection's, and stays above each driver's \
     invocation. The eight real-closer gates are deliberately NOT among them — see \
     `close_after_element` — so this total is the previous test's forty minus exactly those eight"
  );

  // The shared cycle takes NEITHER collection-relative baseline: both stay in driver text, above
  // the invocation, one per collection. A baseline moved into the macro would become per element
  // in two drivers at once, which is the defect this test exists to make unspellable.
  for needle in [SCANNER_BASELINE, LATCH_BASELINE] {
    assert_eq!(
      code_matches(CYCLE_SRC, needle),
      0,
      "{CYCLE_NAME}: the shared try cycle is per ELEMENT, so it takes no `{needle}` — that \
       baseline is the collection's and belongs above each driver's invocation, beside the other"
    );
  }
}

/// The bound arrangements every driver family re-exposes, and the separator arrangements the
/// four separated families add. Named once: the leaves are formulaic, and the point of listing
/// them is that a name **not** in the pattern shows up.
const BOUNDS: &[&str] = &["at_least", "at_most", "bounded", "unbounded"];
const ARRANGEMENTS: &[&str] = &[
  "allow_leading",
  "allow_leading_require_trailing",
  "allow_surrounded",
  "allow_trailing",
  "require_leading",
  "require_leading_allow_trailing",
  "require_surrounded",
  "require_trailing",
];

/// The census's own frontier, stated as data: every `mod` declaration in the `many` and `fold`
/// driver trees, grouped.
///
/// One row for every source [`swallow_scan_sites`] reads, plus `many/mod.rs` itself, which
/// declares them. A row with no groups declares no modules today, and a `mod` added to it reds
/// this test rather than quietly widening the tree past the scan.
///
/// **What this closes and what it does not.** It closes the "fifth file" hole for the trees the
/// drivers live in: a new module beside a driver, or a new driver family, changes a declaration
/// list here. It does **not** descend into `many/handler` or `many/options`, which hold no
/// [`InputRef`](crate::InputRef) and run no loop; if one ever does, it belongs in the swallow
/// scan and its parent belongs here.
fn driver_tree() -> [(
  &'static str,
  &'static str,
  &'static [&'static [&'static str]],
); 17] {
  [
    (CYCLE_NAME, CYCLE_SRC, &[]),
    (
      "many/mod.rs",
      super::end_state_census::many_mod_production(),
      &[&[
        "delim",
        "handler",
        "macros",
        "options",
        "repeated",
        "repeated_while",
        "sep",
        "sep_while",
      ]],
    ),
    (
      "many/delim/mod.rs",
      include_str!("delim/mod.rs"),
      &[&["repeated", "repeated_while"]],
    ),
    (
      "many/sep/mod.rs",
      include_str!("sep/mod.rs"),
      &[&["delim", "parse"]],
    ),
    (
      "many/sep_while/mod.rs",
      include_str!("sep_while/mod.rs"),
      &[&["delim", "parse"]],
    ),
    (
      "fold/mod.rs",
      include_str!("../fold/mod.rs"),
      &[&["fold_while", "rfold", "rfold_while"]],
    ),
    ("fold/rfold.rs", include_str!("../fold/rfold.rs"), &[]),
    (
      "fold/fold_while.rs",
      include_str!("../fold/fold_while.rs"),
      &[],
    ),
    (
      "fold/rfold_while.rs",
      include_str!("../fold/rfold_while.rs"),
      &[],
    ),
    (
      "many/repeated/mod.rs",
      include_str!("repeated/mod.rs"),
      &[BOUNDS],
    ),
    (
      "many/repeated_while/mod.rs",
      include_str!("repeated_while/mod.rs"),
      &[BOUNDS],
    ),
    (
      "many/delim/repeated.rs",
      include_str!("delim/repeated/mod.rs"),
      &[BOUNDS],
    ),
    (
      "many/delim/repeated_while.rs",
      include_str!("delim/repeated_while/mod.rs"),
      &[BOUNDS],
    ),
    (
      "many/sep/parse/mod.rs",
      include_str!("sep/parse/mod.rs"),
      &[BOUNDS, ARRANGEMENTS],
    ),
    (
      "many/sep/delim/mod.rs",
      include_str!("sep/delim/mod.rs"),
      &[BOUNDS, ARRANGEMENTS],
    ),
    (
      "many/sep_while/parse/mod.rs",
      include_str!("sep_while/parse/mod.rs"),
      &[BOUNDS, ARRANGEMENTS],
    ),
    (
      "many/sep_while/delim/mod.rs",
      include_str!("sep_while/delim/mod.rs"),
      &[BOUNDS, ARRANGEMENTS],
    ),
  ]
}

/// No module of the driver trees goes unclassified.
///
/// The other half of "the census cannot pass by not looking": the swallow scan reads a fixed
/// list of sources, and a list is only as good as the guarantee that it is complete. This reads
/// the `mod` declarations that *define* the tree and requires each one to be accounted for, so a
/// new file next to a driver cannot be invisible to the scan — it fails here first, naming
/// itself.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn the_gate_census_covers_every_driver_module() {
  for (name, src, groups) in driver_tree() {
    let mut declared = 0;
    let mut known = 0;
    for group in groups {
      known += group.len();
    }
    for line in src.lines() {
      let line = line.trim();
      if line.starts_with("//") {
        continue;
      }
      // Visibility first, so a `pub mod` cannot slip through a prefix test written for `mod`.
      let line = line
        .strip_prefix("pub(crate) ")
        .or_else(|| line.strip_prefix("pub(super) "))
        .or_else(|| line.strip_prefix("pub "))
        .unwrap_or(line);
      let rest = match line.strip_prefix("mod ") {
        Some(rest) => rest,
        None => continue,
      };
      // `mod x;` and `mod x {` alike.
      let module = rest
        .split(|c: char| c == ';' || c == '{' || c.is_whitespace())
        .next()
        .unwrap_or("");
      assert!(
        groups.iter().any(|group| group.contains(&module)),
        "{name}: `mod {module};` is new to the driver tree and this census does not know it. If \
         it can reach an `InputRef` inside a repetition, add its source to `swallow_scan_sites` \
         so its element failures are read; if it is an `Apply`/options leaf, add its name here. \
         Do not do neither — the swallow scan reads a fixed list, and an unlisted source is a \
         source nothing checks"
      );
      declared += 1;
    }
    assert_eq!(
      declared, known,
      "{name}: this census names {known} module(s) and the source declares {declared}. A name \
       that has been removed or renamed leaves the census scanning for something that is not \
       there, which is how a list stops meaning anything"
    );
  }
}

/// Every `*_while` driver's decision-window peek is terminal-aware: it uses the
/// terminal-reporting `peek_with_emitter_terminal` (never the bare `peek_with_emitter`, whose
/// short window a mid-window trip would hide) and surfaces the stop with `into_terminal`, so a
/// resource-limit trip during the decision peek is never read as a clean end of list.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn every_while_decision_gate_is_terminal_aware() {
  let sites = [
    (
      "many/repeated_while/mod.rs",
      include_str!("repeated_while/mod.rs"),
    ),
    (
      "many/delim/repeated_while.rs",
      include_str!("delim/repeated_while/mod.rs"),
    ),
    (
      "many/sep_while/parse/mod.rs",
      include_str!("sep_while/parse/mod.rs"),
    ),
    (
      "many/sep_while/delim/mod.rs",
      include_str!("sep_while/delim/mod.rs"),
    ),
    ("fold/fold_while.rs", include_str!("../fold/fold_while.rs")),
    (
      "fold/rfold_while.rs",
      include_str!("../fold/rfold_while.rs"),
    ),
  ];
  for (name, src) in sites {
    assert!(
      src.contains("peek_with_emitter_terminal::<"),
      "{name}: the decision-window peek must use the terminal-reporting variant \
       (`peek_with_emitter_terminal`)"
    );
    assert!(
      !src.contains("peek_with_emitter::<"),
      "{name}: no bare decision-window peek may remain — a mid-window trip would hide in its \
       short window; use `peek_with_emitter_terminal`"
    );
    assert!(
      src.contains("into_terminal()"),
      "{name}: a decision-window terminal stop must be surfaced (`into_terminal`)"
    );
    // Exactly one eager terminal gate per decision peek: the flag is consulted immediately after
    // the peek and nowhere else. Per-arm re-scatter — a fallible or emitting call between the peek
    // and the check — reopens the terminal-precedence hole, so the gate count must track the peek
    // count (the turbofish form keeps prose mentions from skewing either tally).
    assert_eq!(
      src.matches("peek_with_emitter_terminal::<").count(),
      src.matches("if terminal").count(),
      "{name}: exactly one terminal gate per decision peek — the eager gate immediately after the \
       peek; per-arm re-scatter reopens the terminal-precedence hole"
    );
  }
}

/// Every non-delimited separated driver's separator-slot decision gate is terminal-aware: it
/// probes with `try_expect_or_stop` (never the bare `try_expect`, whose `Ok(None)` folds a trip
/// together with genuine absence and ends the list cleanly). The delimited separated drivers
/// route their separator-slot `None` through `probe_close`, whose `Tripped` arm surfaces the stop
/// instead — so they are exempt here and covered by that path.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn every_nondelim_separator_slot_surfaces_terminal() {
  let sites = [
    ("many/sep/parse/mod.rs", include_str!("sep/parse/mod.rs")),
    (
      "many/sep_while/parse/mod.rs",
      include_str!("sep_while/parse/mod.rs"),
    ),
  ];
  for (name, src) in sites {
    assert!(
      src.contains("try_expect_or_stop(|t"),
      "{name}: the separator-slot decision gate must probe with `try_expect_or_stop`, so a \
       terminal stop surfaces instead of folding into a clean end"
    );
  }
}

/// The absence-exit shape of every guard-bearing driver, as data: how many [`ABSENCE`] calls the
/// source must carry.
///
/// Index-aligned with [`progress_guard_sites`], and
/// [`every_absence_exit_carries_the_terminal_witnesses`] checks the two lists name the same
/// sources in the same order, so neither can drift.
///
/// **Every row is nonzero, and that is the property.** A zero would mark an *inline* shape that
/// spells `latched_during_attempt` at each exit and reads no descent witness at all — which has
/// the same hole the chokepoint closes for the try-driven four, reached the same way: an element
/// that catches a trip itself and then declines, or accepts consuming nothing, ends the collection
/// cleanly and successfully. Measured rather than inferred: with that shape, `fold` over such an
/// element returns the same `Ok(6)` on `"1 2 3"` under a budget the element exceeded as under one
/// it did not, and `repeated_while` returns the same `Ok([1, 2, 3, -1])` over the stalling shape.
/// Closing it needs the per-element trip baseline
/// [`every_driver_baselines_its_trip_witness_inside_its_element_loop`] pins in all twelve
/// sources — which is why three of the folds are not `while let` loops, a `while let` having
/// nowhere to snapshot the counter before the element attempt its own condition runs.
///
/// So there is no two-shape classification here, and re-introducing one means adding a zero,
/// which fails the per-row assertion rather than passing quietly.
fn absence_exit_shapes() -> [(&'static str, usize); 12] {
  [
    ("many/repeated/mod.rs", 1),
    ("many/repeated_while/mod.rs", 2),
    ("many/delim/repeated.rs", 4),
    ("many/delim/repeated_while.rs", 3),
    ("many/sep/parse/mod.rs", 2),
    ("many/sep/delim/mod.rs", 2),
    ("many/sep_while/parse/mod.rs", 2),
    ("many/sep_while/delim/mod.rs", 6),
    ("fold/mod.rs", 3),
    ("fold/rfold.rs", 1),
    ("fold/fold_while.rs", 3),
    ("fold/rfold_while.rs", 1),
  ]
}

/// Every guard-bearing driver's absence exits reach **every** never-recoverable witness, through
/// one chokepoint no driver can spell around.
///
/// A driver's absence exits — the no-progress stall, the element-decline break, a condition's
/// `Action::Stop`, and in the delimited drivers the close probe's `WrongToken`/`Eof` arms —
/// conclude "no more elements" from what the last element attempt did. Two facts make that
/// attempt's evidence unrepresentative while it still returns `Ok`, so neither reaches
/// [`file_element_failure`](super::file_element_failure):
///
/// * a terminal **scanner** stop the element's own lookahead latched, which leaves the pre-trip
///   tokens cached and the window short (the eager decision gate does not see it: the next window
///   is served whole from that cache and carries no terminal flag). Two witnesses, both baselined
///   per collection: `scanner_tripped_during_attempt` against a `scanner_trip_snapshot`, and
///   `latched_during_attempt` — presence-plus-change against a `latch_snapshot`, never a positional
///   reading, because a rollback over the latch moves the offset back behind a boundary that
///   survives. The counter is the load-bearing one, and the latch is the narrower reading it
///   subsumes: a rollback taken *before* the trip puts the boundary back as well as the offset, so
///   the latch is clean there too, and only a cell outside the rollback set answers;
/// * a **descent** budget trip the element caught itself. It latches no boundary and bumps no
///   scanner counter, so both witnesses above read `false` for one, and the absence exit spends the
///   stop as a *success*. The witness is `tripped_during_attempt` against the per-**element**
///   baseline, for the reason [`every_element_loop_baselines_its_trip_witness_per_element`] gives.
///
/// In the delimited drivers **this** gate belongs *inside* the close probe's `WrongToken`/`Eof`
/// arms, never ahead of the probe: the probe is cache-first, so a `Close` verdict rests on a real
/// pre-trip token, and *neither* scanner witness may reach it — a boundary latched past the closer
/// is not about a construct that ended before it, the counter was bumped by the very lookahead
/// that latched it, and reading either there would fail a parse a wider window completes
/// identically. The *descent* witness is a different kind of fact and does reach it, through the
/// separate [`close_after_element`](super::close_after_element) gate that
/// [`every_real_closer_exit_after_an_element_is_trip_gated`] counts.
///
/// What this pins, per source: the baseline is taken; the source carries the count
/// [`absence_exit_shapes`] classifies it with; and the source spells **none** of the witnesses
/// itself, which is the "nothing left to balance" form — the strongest of these the codebase has,
/// and the one the swallow scan above uses. What it does not pin is that the chokepoint's own guard
/// *gates*, which is
/// [`the_absence_chokepoint_consults_every_witness_before_it_stops`]'s region scan for presence
/// and `tokora/tests/collection_resource_trip.rs`'s sections 4 and 7 for behaviour.
#[test]
fn every_absence_exit_carries_the_terminal_witnesses() {
  let sites = progress_guard_sites();
  let shapes = absence_exit_shapes();
  let mut chokepointed = 0;
  for i in 0..sites.len() {
    let (name, src) = sites[i];
    let (classified, calls) = shapes[i];
    assert_eq!(
      name, classified,
      "the guard-bearing source list and the absence-exit classification have drifted apart at \
       row {i}: `{name}` against `{classified}`. They are index-aligned on purpose — a row that \
       names a source the scan does not read is a row that checks nothing"
    );
    assert!(
      src.contains("latch_snapshot()"),
      "{name}: the absence witness is attempt-relative — take the `latch_snapshot()` baseline once \
       per attempt, or a boundary an enclosing lookahead latched is mis-charged to this driver"
    );
    assert!(
      calls > 0,
      "{name}: a repetition driver concludes the construct ended somewhere, and every such exit \
       goes through `absence_after_element`. A zero here is the old inline shape coming back — see \
       `absence_exit_shapes` for what that shape spent, and how it was measured"
    );
    assert_eq!(
      code_matches(src, ABSENCE),
      calls,
      "{name}: expected {calls} absence-exit gate(s) (`{ABSENCE}`). An exit added without one \
       concludes the construct ended on evidence a terminal stop truncated — and an exit whose \
       gate was removed is the same defect, which is why this is a count and not a presence test"
    );
    for witness in ABSENCE_WITNESSES {
      assert_eq!(
        code_matches(src, witness),
        0,
        "{name}: a driver hands its absence exits to `absence_after_element` — it never reads \
         `{witness}` itself. A hand-spelled gate can consult two witnesses of three and look \
         complete, which is exactly the defect the chokepoint exists to make unspellable"
      );
    }
    chokepointed += calls;
  }
  assert_eq!(
    chokepointed, 30,
    "the twelve guard-bearing sources carry exactly thirty chokepointed absence exits between \
     them. The total is pinned so that adding an exit, or losing one, has to be deliberate — and \
     so that a source re-cut to a smaller count cannot be balanced by a larger one next door"
  );
}

/// The absence chokepoint consults **every** never-recoverable witness before it produces its
/// terminal end-of-input, and it is the only place in the tree that reads the scanner latch.
///
/// The absence twin of the region scan in
/// [`every_element_failure_routes_through_the_gated_chokepoint`], and it proves exactly as much
/// and as little: each witness is present, named, and textually ahead of the stop. That is a
/// tripwire on the source — a witness dropped, renamed or duplicated reds one cheap unit test —
/// and **not** a proof of control-flow domination, which a body reading them all into `let _ =`
/// bindings would satisfy with the gate gone. Verified, not reasoned about: with the descent term
/// so neutered this test and the whole census stayed green while
/// `tokora/tests/collection_resource_trip.rs` red 16 cells. What proves each gate gates is that
/// suite — section 4's 16 cells for the descent term,
/// `tokora/tests/absence_terminal_stop.rs`'s 10 (of 47) for the latch, and that file's
/// `a_trip_caught_inside_an_elements_own_attempt_still_surfaces_terminal` for the scanner counter.
/// Both `find`s panic rather than pass when what they scan for is absent.
///
/// The two tree-wide tallies below are the "counted globally" form of the asymmetry
/// [`the_close_chokepoint_reads_the_counter_and_not_the_position`] pins locally: the latch is read
/// **once** in the whole tree and the scanner counter **twice** — once at each of the two gates
/// that judge from an element attempt, and never at the third.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn the_absence_chokepoint_consults_every_witness_before_it_stops() {
  let prod = super::end_state_census::many_mod_production();
  assert_eq!(
    code_matches(prod, "inp.latched_during_attempt("),
    1,
    "`many/mod.rs`: the scanner latch is read once in this tree, inside `absence_after_element`"
  );
  assert_eq!(
    code_matches(prod, "inp.scanner_tripped_during_attempt("),
    2,
    "`many/mod.rs`: the scanner counter is read exactly twice in this tree — in \
     `absence_after_element` and in `file_element_failure`, the two gates that judge from an \
     element attempt. A third reading is `close_after_element` having grown one, which fails a \
     delimited parse a wider scan window completes identically; a count of one is the witness \
     dropped from a gate whose remaining witnesses answer the same way on every case anyone \
     thought to write"
  );
  let def_at = code_find(prod, ABSENCE_DEF).unwrap_or_else(|| {
    panic!(
      "`many/mod.rs`: `{ABSENCE_DEF}` is gone. The absence chokepoint has been renamed or \
       removed; re-cut this census against whatever replaced it before trusting a green run"
    )
  });
  let body = &prod[def_at..];
  let stop_at = code_find(body, ABSENCE_STOP).unwrap_or_else(|| {
    panic!(
      "`many/mod.rs`: `absence_after_element` no longer surfaces the stop as a terminal \
       end-of-input (`{ABSENCE_STOP}`)"
    )
  });
  let guard = &body[..stop_at];
  for witness in ABSENCE_WITNESSES {
    assert_eq!(
      code_matches(guard, witness),
      1,
      "`many/mod.rs`: the absence chokepoint must consult all three never-recoverable witnesses \
       before it stops — `{witness}` is missing from, or duplicated in, the code ahead of its \
       terminal end-of-input. One witness alone leaves the others' stops spendable as an accepted \
       absence"
    );
  }
}

/// The real-closer shape of every guard-bearing driver, as data — index-aligned with
/// [`progress_guard_sites`].
///
/// `(source, probed closers, direct closers)`.
///
/// * a **probed** closer is a `CloseStatus::Close` verdict, committed by value with
///   [`CLOSE_COMMIT`]. **Every one of them carries [`CLOSE_GATE`]**, with no per-site exemption —
///   which is what lets [`every_real_closer_exit_after_an_element_is_trip_gated`] scan the
///   verdict→commit region *per verdict* instead of comparing two tallies that any arrangement of
///   needles would satisfy. Where a probe sits at the TOP of a cycle the gate is a constant
///   `false` (this cycle's baseline is taken a few lines above it and nothing since can trip), and
///   it is kept anyway: a `u64` comparison is cheaper than an exemption table, and it fails
///   closed if a later refactor moves an element attempt above it;
/// * a **direct** closer is one the driver committed straight from its own scan, with no probe
///   verdict at all. There are exactly two in the tree — `sep/delim`'s and `sep_while/delim`'s
///   mid-scan arms — and they are exempt for a structural reason, not because they "rest on a real
///   token": each is reached from the top of a cycle, so only an *accepting* element can precede
///   it (a decline and a stall each leave for an exit that gates) and the descent term would be
///   the same constant `false`. They are counted rather than ignored, so a third one cannot land
///   unnoticed.
///
/// The eight non-delimited sources carry no closer at all, and that `(0, 0)` is checked rather
/// than skipped: it is what stops a closer landing in a source this census reads but does not
/// expect one in.
fn close_exit_shapes() -> [(&'static str, usize, usize); 12] {
  [
    ("many/repeated/mod.rs", 0, 0),
    ("many/repeated_while/mod.rs", 0, 0),
    ("many/delim/repeated.rs", 2, 0),
    ("many/delim/repeated_while.rs", 2, 0),
    ("many/sep/parse/mod.rs", 0, 0),
    ("many/sep/delim/mod.rs", 1, 1),
    ("many/sep_while/parse/mod.rs", 0, 0),
    ("many/sep_while/delim/mod.rs", 3, 1),
    ("fold/mod.rs", 0, 0),
    ("fold/rfold.rs", 0, 0),
    ("fold/fold_while.rs", 0, 0),
    ("fold/rfold_while.rs", 0, 0),
  ]
}

/// Every exit that commits a real closer **after an element attempt** consults the descent
/// witness first.
///
/// The absence census above pins the exits that conclude the construct ended from *nothing*. This
/// pins the ones that conclude it from *a real token* — the shape it is tempting to leave ungated
/// on the reasoning that a committed pre-trip closer settles the question. It settles the
/// **position** question. The element attempt that declined, or stalled, may have caught a descent
/// trip on its way there, and a valid closer arriving afterwards does not unmake a counter event
/// that already happened: without a gate that is a successfully closed collection that spent a
/// resource-limit stop in silence.
///
/// What this pins, per source: the counted shape [`close_exit_shapes`] classifies it as, in three
/// independent tallies — the verdict, the by-value commit, and the container handoff that closers
/// of *both* origins pass through — plus a **region** scan requiring each verdict to reach
/// [`CLOSE_GATE`] before its commit. The region scan is what makes this more than three tallies
/// that three needles scattered anywhere would satisfy, and its `code_find` panics rather than
/// passing when a verdict has no commit after it at all.
///
/// What it does not pin is that the gate *gates* — see
/// [`the_close_chokepoint_reads_the_counter_and_not_the_position`] for the same limit stated at
/// length, and `tokora/tests/collection_resource_trip.rs`'s section 5 for the behaviour.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn every_real_closer_exit_after_an_element_is_trip_gated() {
  let sites = progress_guard_sites();
  let shapes = close_exit_shapes();
  let mut gated = 0;
  let mut exempt = 0;
  for i in 0..sites.len() {
    let (name, src) = sites[i];
    let (classified, probed, direct) = shapes[i];
    assert_eq!(
      name, classified,
      "the guard-bearing source list and the real-closer classification have drifted apart at row \
       {i}: `{name}` against `{classified}`. They are index-aligned on purpose — a row that names \
       a source the scan does not read is a row that checks nothing"
    );
    assert_eq!(
      code_matches(src, CLOSE_VERDICT),
      probed,
      "{name}: expected {probed} close verdict(s) (`{CLOSE_VERDICT}`). A new one is a new exit \
       that concludes the construct ended, and it has to be classified here before it can be \
       trusted"
    );
    assert_eq!(
      code_matches(src, CLOSE_COMMIT),
      probed,
      "{name}: every close verdict is committed by value exactly once (`{CLOSE_COMMIT}`) — a \
       verdict committed by re-scanning instead would leave this census measuring the wrong \
       program point"
    );
    assert_eq!(
      code_matches(src, CLOSE_HANDOFF),
      probed + direct,
      "{name}: expected {} closer handoff(s) (`{CLOSE_HANDOFF}`), {probed} probed and {direct} \
       direct. A closer committed by any other route is an exit this census cannot see",
      probed + direct
    );
    assert_eq!(
      code_matches(src, CLOSE_GATE),
      probed,
      "{name}: every probed closer reached after an element attempt gates on the descent witness \
       (`{CLOSE_GATE}`). An exit added without one closes the construct successfully over a \
       resource-limit stop the element already answered — and an exit whose gate was removed is \
       the same defect, which is why this is a count and not a presence test"
    );

    // Each verdict reaches the gate before it commits. The region is verdict → commit, so a gate
    // sitting anywhere else in the file — including inside a close-MISS arm, where the absence
    // chokepoint belongs instead — does not satisfy it.
    let mut checked = 0;
    let mut from = 0;
    while let Some(at) = code_find(&src[from..], CLOSE_VERDICT) {
      let verdict_at = from + at;
      let commit_at = code_find(&src[verdict_at..], CLOSE_COMMIT).unwrap_or_else(|| {
        panic!(
          "{name}: a `{CLOSE_VERDICT}` verdict with no `{CLOSE_COMMIT}` after it. The driver no \
           longer commits the probed closer by value; re-cut this census against whatever \
           replaced it before trusting a green run"
        )
      });
      assert_eq!(
        code_matches(&src[verdict_at..verdict_at + commit_at], CLOSE_GATE),
        1,
        "{name}: the descent gate belongs INSIDE the close verdict's arm, ahead of the commit. \
         Found none there, or more than one — a gate in a neighbouring arm leaves this exit \
         closing over a trip the element answered"
      );
      checked += 1;
      from = verdict_at + CLOSE_VERDICT.len();
    }
    assert_eq!(
      checked, probed,
      "{name}: every close verdict must have been inspected"
    );
    gated += probed;
    exempt += direct;
  }
  assert_eq!(
    (gated, exempt),
    (8, 2),
    "the four delimited families carry exactly eight gated real-closer exits — every \
     `CloseStatus::Close` verdict in the tree, whether or not its own descent term can be nonzero \
     — and exactly two exempt DIRECT closers, `sep/delim`'s and `sep_while/delim`'s mid-scan arms, \
     which commit from their own scan with no probe verdict at all. Both halves are pinned so that \
     exempting a third direct closer, or losing a gate from one of the eight, has to be deliberate"
  );
}

/// The close chokepoint reads the **counter** and not the **position**, and the census says so in
/// both directions.
///
/// The third of the region scans, and it proves exactly as much and as little as its two
/// siblings: the witness is present, named, and textually ahead of the stop. That is a tripwire on
/// the source and **not** a proof of control-flow domination — a body reading the counter into a
/// `let _ =` binding satisfies it with the gate gone, exactly as
/// [`the_absence_chokepoint_consults_every_witness_before_it_stops`] records for its own. What
/// proves this one gates is `tokora/tests/collection_resource_trip.rs`'s section 5.
///
/// The negative half has no sibling: **neither** scanner reading may appear here — not the latch
/// and not the session counter. Either one smuggled into this body would fail every delimited
/// parse whose element lookahead latched a boundary past a closer that legitimately closed (the
/// lookahead that latched also bumped the counter, so the two refuse the same parses) — a
/// regression a wider scan window makes invisible, and the exact reason it is tempting to leave
/// the whole arm ungated. Pinning the absence is cheap; noticing it later is not.
/// `many/mod.rs`'s tree-wide tallies for both, asserted next door, are the same claim counted
/// globally.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn the_close_chokepoint_reads_the_counter_and_not_the_position() {
  let prod = super::end_state_census::many_mod_production();
  let def_at = code_find(prod, CLOSE_GATE_DEF).unwrap_or_else(|| {
    panic!(
      "`many/mod.rs`: `{CLOSE_GATE_DEF}` is gone. The real-closer chokepoint has been renamed or \
       removed; re-cut this census against whatever replaced it before trusting a green run"
    )
  });
  let body = &prod[def_at..];
  let stop_at = code_find(body, ABSENCE_STOP).unwrap_or_else(|| {
    panic!(
      "`many/mod.rs`: `close_after_element` no longer surfaces the stop as a terminal \
       end-of-input (`{ABSENCE_STOP}`)"
    )
  });
  let guard = &body[..stop_at];
  assert_eq!(
    code_matches(guard, CLOSE_WITNESS),
    1,
    "`many/mod.rs`: the close chokepoint must consult the descent witness before it stops — \
     `{CLOSE_WITNESS}` is missing from, or duplicated in, the code ahead of its terminal \
     end-of-input. Without it a closer that arrives after a trip the element answered closes the \
     collection successfully"
  );
  for non_witness in CLOSE_NON_WITNESSES {
    assert_eq!(
      code_matches(guard, non_witness),
      0,
      "`many/mod.rs`: the close chokepoint must read NEITHER scanner witness (`{non_witness}` is \
       present). A committed pre-trip closer settles the scanner fact — the construct ended ahead \
       of any boundary a later lookahead latched, and that same lookahead is what bumped the \
       session counter — so reading either here fails a parse a wider scan window completes to \
       the identical value. The asymmetry between the scanner facts and the descent one is why \
       this chokepoint exists apart from `absence_after_element`"
    );
  }
}

/// Every no-progress guard measures committed consumption (`span().end()`), never a cache-front
/// cursor comparison. A lookahead fill moves the cursor across skipped trivia without consuming
/// anything, which a cursor-keyed guard reads as false progress — a zero-width element behind a
/// trivia gap then runs an extra cycle. This pins the metric across every guard-bearing driver so
/// the class cannot regrow.
///
/// [`CYCLE_SRC`] is scanned beside them because two of those guards moved there: the stall test
/// `repeated` and `delim/repeated` used to spell each is now the shared cycle's one. A census
/// that kept reading only the twelve driver sources would have stopped looking at the site it
/// was written for, and would still have passed.
#[test]
fn every_progress_guard_reads_committed_progress() {
  for (name, src) in progress_guard_sites()
    .into_iter()
    .chain([(CYCLE_NAME, CYCLE_SRC)])
  {
    assert!(
      !src.contains(".as_inner() == ") && !src.contains(".as_inner() != "),
      "{name}: no-progress guards must compare committed consumption (`span().end()`), never a \
       cache-front cursor (`.as_inner()` equality) — a lookahead fill moves the cursor without \
       committing"
    );
  }
}

/// The guard-bearing driver sources: every collection and fold driver that runs a repetition loop
/// and therefore needs both the committed-progress metric and the absence witness.
pub(super) fn progress_guard_sites() -> [(&'static str, &'static str); 12] {
  [
    ("many/repeated/mod.rs", include_str!("repeated/mod.rs")),
    (
      "many/repeated_while/mod.rs",
      include_str!("repeated_while/mod.rs"),
    ),
    (
      "many/delim/repeated.rs",
      include_str!("delim/repeated/mod.rs"),
    ),
    (
      "many/delim/repeated_while.rs",
      include_str!("delim/repeated_while/mod.rs"),
    ),
    ("many/sep/parse/mod.rs", include_str!("sep/parse/mod.rs")),
    ("many/sep/delim/mod.rs", include_str!("sep/delim/mod.rs")),
    (
      "many/sep_while/parse/mod.rs",
      include_str!("sep_while/parse/mod.rs"),
    ),
    (
      "many/sep_while/delim/mod.rs",
      include_str!("sep_while/delim/mod.rs"),
    ),
    ("fold/mod.rs", include_str!("../fold/mod.rs")),
    ("fold/rfold.rs", include_str!("../fold/rfold.rs")),
    ("fold/fold_while.rs", include_str!("../fold/fold_while.rs")),
    (
      "fold/rfold_while.rs",
      include_str!("../fold/rfold_while.rs"),
    ),
  ]
}
