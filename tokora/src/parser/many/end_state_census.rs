//! END_STATE_CENSUS and its companions — the accounting laws of the repetition drivers,
//! locked by count.
//!
//! Every exit that returns `Ok` from a repetition driver runs that driver's end-state pass
//! exactly once before producing the success value; `Err` exits run it never. The defect this
//! guards against is a second `Ok`-returning exit that jumps over the pass — and because the
//! skipping exit is the one taken on WELL-FORMED input, the pass becomes dead code precisely
//! where it matters and every test that feeds conforming input stays green and blind.

/// Counts occurrences of `needle` on lines that are not whole-line comments, so prose
/// mentions of a counted name do not skew a tally.
///
/// Shared with [`gate_census`](super::gate_census) and with
/// [`parser::recovery_gate`](crate::parser::recovery_gate)'s `RECOVERY_GATE_CENSUS`, whose
/// baseline tallies need the same comment-blindness for the same reason.
pub(crate) fn code_matches(src: &str, needle: &str) -> usize {
  src
    .lines()
    .filter(|line| !line.trim_start().starts_with("//"))
    .map(|line| line.matches(needle).count())
    .sum()
}

/// The byte offset of `needle`'s first occurrence on a line that is not a whole-line comment,
/// or `None` when there is none.
///
/// The positional twin of [`code_matches`], and comment-blind for the same reason: a region scan
/// anchored on a needle a doc comment also spells would measure the wrong region, which is a
/// quieter failure than measuring nothing.
pub(crate) fn code_find(src: &str, needle: &str) -> Option<usize> {
  let mut at = 0;
  for line in src.split_inclusive('\n') {
    let hit = if line.trim_start().starts_with("//") {
      None
    } else {
      line.find(needle)
    };
    if let Some(k) = hit {
      return Some(at + k);
    }
    at += line.len();
  }
  None
}

/// This module's own file with the census modules cut off the end — several needles below
/// appear verbatim in census source and would otherwise be counted as production sites.
pub(super) fn many_mod_production() -> &'static str {
  let src = include_str!("mod.rs");
  src
    .split_once("#[cfg(test)]")
    .expect("the census marker must be present in its own source")
    .0
}

/// The eight collection drivers: source, the literal that spells the end-state pass, the
/// number of times it must appear, the literal that spells a success exit, and its count.
const SITES: &[(&str, &str, &str, usize, &str, usize)] = &[
  (
    "many/repeated/mod.rs",
    include_str!("repeated/mod.rs"),
    "rh.on_stop(",
    1,
    "rh.on_stop(",
    1,
  ),
  (
    "many/repeated_while/mod.rs",
    include_str!("repeated_while/mod.rs"),
    "rh.on_stop(",
    2,
    "rh.on_stop(",
    2,
  ),
  (
    "many/delim/repeated.rs",
    include_str!("delim/repeated/mod.rs"),
    "on_stop(nums, inp, &span)",
    2,
    "Ok(span)",
    2,
  ),
  (
    "many/delim/repeated_while.rs",
    include_str!("delim/repeated_while/mod.rs"),
    "on_stop(nums, inp, &span)",
    3,
    "Ok(span)",
    3,
  ),
  (
    "many/sep/parse/mod.rs",
    include_str!("sep/parse/mod.rs"),
    "self.handle_end(",
    3,
    "return Ok(span);",
    3,
  ),
  (
    "many/sep/delim/mod.rs",
    include_str!("sep/delim/mod.rs"),
    "parser.handle_end(",
    2,
    "Ok(inp.span_since(&anchor))",
    2,
  ),
  (
    "many/sep_while/parse/mod.rs",
    include_str!("sep_while/parse/mod.rs"),
    "self.handle_end(",
    3,
    "return Ok(span);",
    3,
  ),
  (
    "many/sep_while/delim/mod.rs",
    include_str!("sep_while/delim/mod.rs"),
    "parser.handle_end(",
    4,
    "Ok(inp.span_since(&anchor))",
    4,
  ),
];

/// END_STATE_CENSUS. What this actually pins, row by row — stated precisely, because the
/// equality is not equally informative everywhere:
///
/// * **Every row**: the *pinned totals*. Deleting an end-state pass, or adding a driver exit
///   without one, moves a count away from its literal and fails.
/// * `sep/delim` and `sep_while/delim` (the two rows where the defect actually lived): the
///   pass literal and the exit literal are **independent**, so pass-count == exit-count is a
///   real assertion. Reverting the mid-scan-closer arm moves both, independently.
/// * `sep{,_while}/parse`: the pass binds the span the exit returns, so equality asserts that
///   every `handle_end` call is the one an `Ok` exit is built on — a non-returning call fails.
/// * `repeated`, `repeated_while`, `delim/repeated{,_while}`: the exit and the pass are one
///   statement pair by construction, so equality holds structurally there and only the totals
///   bite.
///
/// The bare-`return Ok(` check below is what closes the gap the equality leaves: it catches a
/// *new* success exit of any shape, in any of the eight sources, including the six where the
/// equality cannot. Its allow-list is the two shapes the drivers actually return — the
/// whole-construct span, and the span an end-state pass bound.
#[test]
fn every_driver_ok_exit_runs_the_end_state_pass() {
  for (name, src, pass, pass_n, exit, exit_n) in SITES {
    let passes = code_matches(src, pass);
    let exits = code_matches(src, exit);
    assert_eq!(
      passes, *pass_n,
      "{name}: expected {pass_n} end-state pass call(s) (`{pass}`), found {passes}"
    );
    assert_eq!(
      exits, *exit_n,
      "{name}: expected {exit_n} success exit(s) (`{exit}`), found {exits}"
    );
    assert_eq!(
      passes, exits,
      "{name}: every `Ok`-returning exit runs the end-state pass exactly once"
    );

    // Any `return Ok(` outside the two recognized shapes is a success exit that was added
    // without an end-state pass.
    let bare = code_matches(src, "return Ok(");
    let anchored = code_matches(src, "return Ok(inp.span_since(&anchor))")
      + code_matches(src, "return Ok(span);");
    assert_eq!(
      bare, anchored,
      "{name}: a success exit returning something other than the whole-construct span or the \
       span its end-state pass bound has been added without that pass"
    );
  }
}

/// CAPACITY_REPORT_CENSUS — the destination's capacity report is emitted **at the refusal**,
/// inside [`admit_element`](super::admit_element)'s refusal arm, and nowhere else.
///
/// This is the *timing* law, made countable — and the countable thing is a **region**, not a
/// tally, because the defect this replaced was an emission that moved rather than one that
/// multiplied. The report was briefly withheld until each driver's end-state pass, which put
/// the capacity report after the construct's count bounds; it also moved *when the emitter is
/// consulted*, and in this driver the emitter is what decides whether the parse continues. A
/// rejecting emitter stopped one whole construct late, every later `Err` exit discarded the
/// withheld report, and an O(1)-prefix refusal became O(n) over the rest of the input.
/// [`admit_element`](super::admit_element) states why the ordering cannot be bought back.
///
/// So the pin is: exactly one `emit_full_container` in the tree, it is in `admit_element`, and
/// it is **ahead of the `*nums += 1` that ends the refusal arm** — a call relocated to any exit,
/// or to any code that runs after the element is counted, moves out of that region and fails.
/// The `code_find`s panic rather than pass when their anchor is gone, so a gutted or renamed
/// chokepoint reports that instead of quietly finding nothing to check.
#[test]
fn the_capacity_report_is_emitted_at_the_refusal() {
  let prod = many_mod_production();
  let mut total_emissions = code_matches(prod, "emit_full_container(");
  for (name, src, ..) in SITES {
    let direct = code_matches(src, "emit_full_container(");
    assert_eq!(
      direct, 0,
      "{name}: drivers report through `admit_element`, never `emit_full_container` directly — a \
       driver-side call is one the once-per-construct latch does not cover"
    );
    total_emissions += direct;
  }
  assert_eq!(
    total_emissions, 1,
    "`emit_full_container(` is called once in the whole `many` tree, inside `admit_element`"
  );

  let body = &prod[admit_element_body(prod)..];
  let counted_at = code_find(body, "*nums += 1;").unwrap_or_else(|| {
    panic!("`many/mod.rs`: `admit_element` no longer ends its refusal arm by counting the element")
  });
  assert_eq!(
    code_matches(&body[..counted_at], "emit_full_container("),
    1,
    "`many/mod.rs`: the capacity report is emitted on the refusal arm, before the element is \
     counted. Moved past it — to a driver exit, or anywhere the construct keeps running — a \
     rejecting emitter stops late, a later `Err` discards the report, and the refusal stops \
     being constant work"
  );
}

/// The byte offset at which [`admit_element`](super::admit_element)'s definition starts, or a
/// panic naming what went missing.
///
/// Shared by the two censuses that measure *regions* of that body rather than tallies over it.
/// A `code_find` that quietly returned `None` would make both of them pass by looking at
/// nothing, which is the failure mode a renamed or gutted chokepoint takes.
fn admit_element_body(prod: &str) -> usize {
  code_find(prod, "fn admit_element").unwrap_or_else(|| {
    panic!(
      "`many/mod.rs`: `fn admit_element` is gone. The element-admission chokepoint has been \
       renamed or removed; re-cut this census against whatever replaced it before trusting a \
       green run"
    )
  })
}

/// ELEMENT_ADMISSION_CENSUS — **one admission, and it settles the count bound before the
/// destination is offered the element.**
///
/// This is the structural half of the repair for
/// [#277](https://github.com/al8n/tokora/issues/277), and it is the form the law has to take
/// because [#259](https://github.com/al8n/tokora/issues/259) will rewrite this subtree into
/// shared engines. An ordering kept by convention at twelve loop bodies is a property a rewrite
/// of those bodies inherits by luck; an ordering that is one function's two consecutive lines,
/// with every driver forced through it, is a property a rewrite has to actively destroy.
///
/// Four things are pinned, and each one is a way the divergence could come back:
///
/// 1. **No driver touches a container itself.** `container.push(` appears nowhere in the eight
///    driver sources — a direct push is an element admitted without a count verdict.
/// 2. **No driver runs the count hook itself.** `on_element(` appears nowhere in them either. A
///    driver-side call would either double-report a maximum or place it back on the wrong side
///    of the push, which is exactly what #277 was.
/// 3. **The admission sites are counted, per driver.** A new driver reaching a container moves
///    a number here, and the failure message sends whoever moved it to
///    `tokora/tests/repetition_diagnostic_order.rs` to declare its row.
/// 4. **Inside the chokepoint, the hook precedes the push.** Swapping the two lines is the
///    one-line regression this whole census exists for, and it is the only one a tally cannot
///    see — so it is measured as a region, like the capacity report above.
///
/// The two zero-count sources are not exemptions: `sep/delim` and `sep_while/delim` reach a
/// container through the `handle_continue` of their non-delimited twin, which is where their
/// four admissions are already counted. Wiring either of them to a container directly would
/// raise its count off zero and fail here.
#[test]
fn every_driver_admits_its_elements_through_the_one_chokepoint() {
  /// Admissions per driver source. The four separated engines are two `handle_continue`
  /// bodies of four state arms each, shared with their delimited twins.
  ///
  /// The two try-driven plain drivers read zero for the same kind of reason and a different
  /// mechanism: their admission is the one inside `try_element_cycle!`, counted once in
  /// `many/macros.rs` below and serving both. Wiring either of them to a container directly
  /// would raise its count off zero and fail here, exactly as it would for `sep{,_while}/delim`.
  const ADMISSIONS: &[(&str, usize)] = &[
    ("many/repeated/mod.rs", 0),
    ("many/repeated_while/mod.rs", 1),
    ("many/delim/repeated.rs", 0),
    ("many/delim/repeated_while.rs", 1),
    ("many/sep/parse/mod.rs", 4),
    ("many/sep/delim/mod.rs", 0),
    ("many/sep_while/parse/mod.rs", 4),
    ("many/sep_while/delim/mod.rs", 0),
  ];

  let mut total = 0;
  for (name, src, ..) in SITES {
    assert_eq!(
      code_matches(src, "container.push("),
      0,
      "{name}: a driver never pushes into the destination itself — `many::admit_element` is the \
       one admission, and it settles the element's count bound first"
    );
    assert_eq!(
      code_matches(src, "on_element("),
      0,
      "{name}: a driver never runs the count hook itself — `many::admit_element` runs it, in \
       front of the push, so that one element's maximum is reported before its refusal (#277)"
    );

    let want = ADMISSIONS
      .iter()
      .find(|(n, _)| n == name)
      .unwrap_or_else(|| panic!("{name}: not listed in ADMISSIONS"))
      .1;
    let got = code_matches(src, "admit_element(");
    assert_eq!(
      got, want,
      "{name}: expected {want} element admission(s), found {got}. A driver that gained or lost \
       one needs its row in `tokora/tests/repetition_diagnostic_order.rs` reviewed — that file \
       is what proves the order this census only makes structural"
    );
    total += got;
  }

  // The shared try cycle's own admission, counted with the same three needles, because it hosts
  // driver-reachable loop code and a push written there would reach two drivers at once.
  let (cycle_name, cycle_src) = (
    super::gate_census::CYCLE_NAME,
    super::gate_census::CYCLE_SRC,
  );
  for needle in ["container.push(", "on_element("] {
    assert_eq!(
      code_matches(cycle_src, needle),
      0,
      "{cycle_name}: the shared element cycle admits through `many::admit_element` like every \
       driver — `{needle}` here is the one place a bypass would serve two drivers at once"
    );
  }
  let cycle_admissions = code_matches(cycle_src, "admit_element(");
  assert_eq!(
    cycle_admissions, 1,
    "{cycle_name}: `try_element_cycle!` carries exactly one admission — the one `repeated` and \
     `delim/repeated` stopped spelling. A second is a second element admitted per cycle"
  );
  total += cycle_admissions;

  assert_eq!(
    total, 11,
    "eleven element admissions across the eight drivers and the one shared cycle; \
     `sep{{,_while}}/delim` reach a container through their non-delimited twin's \
     `handle_continue`, and `repeated`/`delim/repeated` through `try_element_cycle!` — which is \
     why the total is eleven for twelve driver-visible admissions"
  );

  // The order, as a region: the count hook is called before the container is asked.
  let prod = many_mod_production();
  let body = &prod[admit_element_body(prod)..];
  let pushed_at = code_find(body, "container.push(").unwrap_or_else(|| {
    panic!("`many/mod.rs`: `admit_element` no longer offers the element to the destination")
  });
  assert_eq!(
    code_matches(&body[..pushed_at], "on_element("),
    1,
    "`many/mod.rs`: `admit_element` settles the element's count bound BEFORE offering it to the \
     destination. Reversed, an element that both exceeds its maximum and is refused reports the \
     refusal first, and under `Fatal` the public error goes back to depending on which \
     repetition builder the caller picked (#277)"
  );
}

/// LIMIT_PAYLOAD_CENSUS 2a — `FullContainer` has exactly one emission site in the whole
/// `many` tree: the `admit_element` chokepoint. Twelve separate sites is how three different
/// counting conventions grew between them, and how the once-per-construct latch was lost.
#[test]
fn full_container_has_one_emission_site() {
  let mut total = code_matches(many_mod_production(), "FullContainer::of(");
  for (name, src, ..) in SITES {
    let n = code_matches(src, "FullContainer::of(");
    assert_eq!(
      n, 0,
      "{name}: drivers must admit through `admit_element`, never emit `FullContainer` directly"
    );
    total += n;
  }
  assert_eq!(
    total, 1,
    "`FullContainer::of(` is constructed once, inside `admit_element`"
  );
}

/// LIMIT_PAYLOAD_CENSUS 2b — every `TooMany` names a count that actually exceeds its limit, and
/// there are exactly **two** places that can name one.
///
/// `TooMany` renders as "found {nums}, but maximum is {limit}", so a `nums` equal to the limit
/// renders a self-contradicting sentence. Each emission site therefore passes `limit + 1`,
/// which is also the smallest count every driver can produce at the element that broke the
/// limit — the only value that makes one history yield one payload whichever builder produced
/// it.
///
/// The count is two, not eight, and the five sources that lost theirs stay in the list on
/// purpose. A maximum is settled at the element that broke it, from the two
/// `ElementCountHandler::on_element` hooks that `admit_element` runs; the two end checks in
/// `with.rs` and the four delimited-repeated `on_stop` closures used to re-decide it one whole
/// construct later, which is where the builder-dependent diagnostic order in #277 came from.
/// Scanning them and finding nothing is the assertion: a `TooMany` that reappears in an end pass
/// fails here even though the total on its own would still look plausible.
///
/// The requirement is `TooMany`'s alone. `FullContainer` once shared the "found … exceeds …"
/// wording and so looked like it shared the law; it does not, because its two numbers describe
/// different things — one construct's parsed elements against a destination's total capacity —
/// and it states a refusal rather than an exceedance.
///
/// The per-line conjunction assumes the site fits on one line, which both do at the current
/// rustfmt width; a future wrap would need the needle re-cut rather than dropped.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn every_too_many_payload_exceeds_its_limit() {
  let sites: [(&str, &str); 7] = [
    ("parser/with.rs", include_str!("../with.rs")),
    (
      "many/handler/maximum.rs",
      include_str!("handler/maximum.rs"),
    ),
    (
      "many/handler/bounded.rs",
      include_str!("handler/bounded.rs"),
    ),
    (
      "many/delim/repeated/at_most.rs",
      include_str!("delim/repeated/at_most.rs"),
    ),
    (
      "many/delim/repeated/bounded.rs",
      include_str!("delim/repeated/bounded.rs"),
    ),
    (
      "many/delim/repeated_while/at_most.rs",
      include_str!("delim/repeated_while/at_most.rs"),
    ),
    (
      "many/delim/repeated_while/bounded.rs",
      include_str!("delim/repeated_while/bounded.rs"),
    ),
  ];
  // The two sources allowed to name a `TooMany`: the element hooks, which run inside
  // `admit_element` in front of the push. Every other entry above must be silent.
  const HOOKS: [&str; 2] = ["many/handler/maximum.rs", "many/handler/bounded.rs"];

  let mut total = 0;
  for (name, src) in sites {
    let mut here = 0;
    for line in src
      .lines()
      .filter(|l| !l.trim_start().starts_with("//") && l.contains("TooMany::of("))
    {
      assert!(
        line.contains("+ 1,"),
        "{name}: `TooMany`'s count must exceed the limit it names (`limit + 1`): {}",
        line.trim()
      );
      here += 1;
    }
    assert_eq!(
      here,
      usize::from(HOOKS.contains(&name)),
      "{name}: a maximum is reported once, at the element that broke it, from \
       `ElementCountHandler::on_element`. An end pass that reports one again re-decides a fact \
       a whole construct later, which is how the diagnostic order came to depend on the \
       builder (#277)"
    );
    total += here;
  }
  assert_eq!(
    total, 2,
    "the two `TooMany` emission sites, both `ElementCountHandler::on_element`: `Maximum`'s and \
     `With<Minimum, Maximum>`'s"
  );
}

/// MID_LOOP_PAIRING_CENSUS — no end pass re-checks the maximum, which is sound *only* because
/// every element goes through an admission that checks it: a construct exceeds `max` iff some
/// element saw the pre-element count equal to it, so the hook has already reported the
/// violation exactly once by the time any end pass runs.
///
/// The pairing used to be stated per driver — "a `RepeatedHandler` consumer must call
/// `on_element` for every element" — and it held, for the two consumers it could see, while the
/// other six drivers went on deciding their maximum from an end pass. That is the divergence
/// #277 reports, and it is why a per-consumer pairing is the wrong shape for this law: it is
/// satisfiable by the drivers that already agree. The pairing is now
/// [`admit_element`](super::admit_element)'s, pinned structurally by
/// `every_driver_admits_its_elements_through_the_one_chokepoint`, so what is left here is the
/// **other** half — that the end passes stay silent about the maximum.
#[test]
fn no_end_pass_re_checks_the_maximum() {
  for (name, src) in [
    (
      "many/handler/maximum.rs",
      include_str!("handler/maximum.rs"),
    ),
    (
      "many/handler/bounded.rs",
      include_str!("handler/bounded.rs"),
    ),
  ] {
    let def_at = code_find(src, "fn on_stop(").unwrap_or_else(|| {
      panic!(
        "{name}: `RepeatedHandler::on_stop` is gone; re-cut this census against its replacement"
      )
    });
    assert_eq!(
      code_matches(&src[def_at..], "emit_too_many("),
      0,
      "{name}: `on_stop` must not re-check the maximum — the admission has already reported it, \
       at the element that broke it and ahead of that element's push (#277)"
    );
  }

  // `progress_guard_sites` covers all twelve guard-bearing sources; none may report a maximum
  // itself. `ELEMENT_ADMISSION_CENSUS` says as much of the eight collection drivers, and this
  // widens it to the four fold sources, which carry no cardinality concept at all.
  for (name, src) in super::gate_census::progress_guard_sites() {
    assert_eq!(
      code_matches(src, "emit_too_many("),
      0,
      "{name}: a driver never reports a maximum itself; the count handler does, from inside \
       `admit_element`"
    );
  }
}

/// SEPARATOR_DELIVERY_CENSUS — every separator a driver consumes goes through
/// `observe_separator`, whose clone lives inside the `OBSERVES_SEPARATORS` guard. Four arms
/// per file is the whole of `handle_separator`'s `State` match: leading, happy path,
/// duplicate, and (via the state it leaves behind) trailing. A driver calling `on_separator`
/// directly would bypass the guard and clone a token for a container that ignores it.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn every_separator_arm_delivers_through_the_guard() {
  let sites = [
    ("many/sep/parse/mod.rs", include_str!("sep/parse/mod.rs")),
    (
      "many/sep_while/parse/mod.rs",
      include_str!("sep_while/parse/mod.rs"),
    ),
  ];
  for (name, src) in sites {
    assert_eq!(
      code_matches(src, "observe_separator("),
      4,
      "{name}: all four `handle_separator` arms deliver their separator"
    );
    assert_eq!(
      code_matches(src, "on_separator("),
      0,
      "{name}: drivers deliver through `observe_separator`, never `on_separator` directly — \
       the direct call clones unconditionally"
    );
  }
}
