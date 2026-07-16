//! SETTLE_CENSUS — the source census of every place a committed token settles.
//!
//! # Why a census, and why here
//!
//! A token becomes *committed* at a handful of settle sites spread across the consume
//! surface: the `next` arms, the `consume_cached` family, the `try_expect` accept arms,
//! the shared scanner's skip/stop settles. Any side channel that must observe committed
//! tokens exactly once (a CST event stream riding the emitter, a lossless trivia view)
//! has to hook **all** of them — an invariant enforced at N call sites instead of one
//! chokepoint, with N ≈ a dozen on day one. The crate's answer is the same as everywhere
//! else: one primitive ([`InputRef::commit_token`](super::InputRef)), every consume
//! settle routed through it, and a census that fails the build the moment a settle
//! appears outside it.
//!
//! Rust has no call-site reflection, so — exactly like `OP_SURFACE_CENSUS` in
//! `src/fuzz/ops.rs` — the residual "notice a new site exists" step is anchored by a
//! greppable marker and a consciously-maintained count. These tests are that count: they
//! read the `input_ref` sources with `include_str!` and assert the exact number of
//! settle-surface calls per file. Adding a consume path without routing it through the
//! primitive moves a count and fails a named test whose message carries the checklist.
//! `grep SETTLE_CENSUS` finds every anchor.
//!
//! # The census (what must stay true)
//!
//! **Every 1:1 consume settle goes through `commit_token`** — the eleven sites:
//! `next()`'s cached and fresh-lex arms (2), `consume_cached_one` and
//! `consume_cached_to`'s loop body (2 — `consume_all_cached` drains *through*
//! `consume_cached_to`, per token), the `try_expect`/`try_expect_map`/
//! `try_expect_and_then` cached and on-input accept arms (6), and
//! `SyncThrough::on_stop` (1).
//!
//! **The one skip settle is `AtFrontier::adopt`**, called only by `skip_and_report`: a
//! token a scan skips settles behind the frontier — not into the input's span — exactly
//! once, whichever origin fed it. It is the settle surface's second (and last) member.
//!
//! **The span funnel is not a settle.** `set_span_after_consume` is also written by
//! three non-token paths, and they must **never** grow a settle hook: `settle_fatal`
//! writes a *rejected lexer error's* span; `SyncTo::on_eof` writes the lexer's span at
//! exhaustion; `commit_at` batch-writes a whole skipped run's frontier (each skipped
//! token already settled via `adopt`). Peeks, declines, `unconsume`, and the
//! position-write surgeries (`set_state`, restores) touch no settle at all.
//!
//! Counting is line-based and skips `//`-prefixed lines, so doc references to these
//! names do not count; only code does. Keep code mentions of the counted names off
//! comment-trailing positions and out of string literals in these files.

/// The `input_ref` sources under census. Every non-test source file of the module tree
/// is listed, so a settle added in a *new* neighbour file still lands in the sweep once
/// that file is registered here — and the `mod.rs` module list makes an unregistered
/// neighbour impossible to miss in review.
const SOURCES: &[(&str, &str)] = &[
  ("mod.rs", include_str!("mod.rs")),
  ("scan.rs", include_str!("scan.rs")),
  ("try_expect.rs", include_str!("try_expect.rs")),
  (
    "consume_cached/mod.rs",
    include_str!("consume_cached/mod.rs"),
  ),
  ("peek/mod.rs", include_str!("peek/mod.rs")),
  ("sync_to.rs", include_str!("sync_to.rs")),
  ("sync_through.rs", include_str!("sync_through.rs")),
  ("sync_balanced.rs", include_str!("sync_balanced.rs")),
  ("skip_while.rs", include_str!("skip_while.rs")),
  ("fold.rs", include_str!("fold.rs")),
  ("pratt.rs", include_str!("pratt.rs")),
  ("session.rs", include_str!("session.rs")),
  ("drop_policy.rs", include_str!("drop_policy.rs")),
  ("trace.rs", include_str!("trace.rs")),
  ("transaction/mod.rs", include_str!("transaction/mod.rs")),
  ("stacked/mod.rs", include_str!("stacked/mod.rs")),
];

/// Fetches a censused source by name.
fn source(name: &str) -> &'static str {
  SOURCES
    .iter()
    .find(|(n, _)| *n == name)
    .map(|(_, s)| *s)
    .unwrap_or_else(|| panic!("SETTLE_CENSUS: `{name}` is not a censused source"))
}

/// Counts occurrences of `needle` on the non-comment lines of `hay`.
///
/// Line-based on purpose: `//`-prefixed lines (`//`, `///`, `//!`) are documentation and
/// may name the censused methods freely. The censused files keep code mentions of these
/// names off comment-trailing positions, so this is exact for them.
fn count(hay: &str, needle: &str) -> usize {
  hay
    .lines()
    .filter(|line| !line.trim_start().starts_with("//"))
    .map(|line| line.matches(needle).count())
    .sum()
}

/// Counts call sites of a method: `self.<name>(` + `ir.<name>(` receivers.
fn calls(hay: &str, name: &str) -> usize {
  let self_form = std::format!("self.{name}(");
  let ir_form = std::format!("ir.{name}(");
  count(hay, &self_form) + count(hay, &ir_form)
}

/// SETTLE_CENSUS — the eleven 1:1 consume settles, each a `commit_token` call, and no
/// call anywhere else. A new consume path must route through the primitive **and** bump
/// its file's expected count here, in the same commit.
#[test]
fn settle_census_commit_token_routes_every_consume_settle() {
  // (file, expected `commit_token` call sites)
  let expected: &[(&str, usize)] = &[
    // `next()`: the cached arm and the fresh-lex arm.
    ("mod.rs", 2),
    // `SyncThrough::on_stop`: the consumed stopper.
    ("scan.rs", 1),
    // The cached and on-input accept arms of `try_expect`/`_map`/`_and_then`.
    ("try_expect.rs", 6),
    // `consume_cached_one` + `consume_cached_to`'s loop body; `consume_all_cached`
    // drains through `consume_cached_to`, so every cached token settles per token.
    ("consume_cached/mod.rs", 2),
  ];

  for (name, want) in expected {
    let got = calls(source(name), "commit_token");
    assert!(
      got == *want,
      "SETTLE_CENSUS drift: `{name}` has {got} `commit_token` call sites, expected {want}. \
       A consume settle moved. Route the new/changed site through `InputRef::commit_token` \
       (never a raw span/state write), then update this census in the same commit \
       (grep SETTLE_CENSUS)."
    );
  }

  // Everywhere else: zero. A settle outside the censused files means a new consume
  // surface grew without joining the census.
  for (name, src) in SOURCES {
    if expected.iter().any(|(n, _)| n == name) {
      continue;
    }
    let got = calls(src, "commit_token");
    assert!(
      got == 0,
      "SETTLE_CENSUS drift: `{name}` gained {got} `commit_token` call site(s) outside the \
       census. Add the file's expected count to this test in the same commit \
       (grep SETTLE_CENSUS)."
    );
  }

  // The primitive is defined exactly once, in `mod.rs`.
  assert!(
    count(source("mod.rs"), "fn commit_token(") == 1,
    "SETTLE_CENSUS drift: `commit_token` must be defined exactly once, in `mod.rs`"
  );
}

/// SETTLE_CENSUS — the span funnel's callers are locked. `commit_token` is the only
/// *settle* that writes it; the three non-token writers (`settle_fatal`, `commit_at`,
/// `SyncTo::on_eof`) must never gain a settle hook, and no fourth caller may appear:
/// a new consume path writing the funnel directly would bypass the settle primitive.
#[test]
fn settle_census_span_funnel_callers_are_locked() {
  // (file, expected `set_span_after_consume` call sites, who they are)
  let expected: &[(&str, usize, &str)] = &[
    (
      "mod.rs",
      3,
      "`commit_token`'s body, `settle_fatal` (a rejected error's span — NOT a token), \
       and `commit_at` (a scan's batch frontier write — its tokens settled via `adopt`)",
    ),
    (
      "scan.rs",
      1,
      "`SyncTo::on_eof` (the lexer's span at exhaustion — NOT a token)",
    ),
  ];

  for (name, want, who) in expected {
    let got = calls(source(name), "set_span_after_consume");
    assert!(
      got == *want,
      "SETTLE_CENSUS drift: `{name}` has {got} `set_span_after_consume` call sites, \
       expected {want} ({who}). A consume settle must route through `commit_token`; \
       a position write that is not a token settle must be listed here, in the same \
       commit (grep SETTLE_CENSUS)."
    );
  }

  for (name, src) in SOURCES {
    if expected.iter().any(|(n, _, _)| n == name) {
      continue;
    }
    let got = calls(src, "set_span_after_consume");
    assert!(
      got == 0,
      "SETTLE_CENSUS drift: `{name}` writes the span funnel directly ({got} site(s)). \
       Token settles go through `InputRef::commit_token`; non-token position writes \
       must be registered here (grep SETTLE_CENSUS)."
    );
  }
}

/// SETTLE_CENSUS — `AtFrontier::adopt` is the one skip settle, with exactly one caller
/// (`skip_and_report`). Every token a scan skips — trivia and recovery garbage alike,
/// cached or freshly lexed — settles behind the frontier through this single call, so a
/// side channel that hooks skipped tokens has exactly one home too.
#[test]
fn settle_census_adopt_is_the_single_skip_settle() {
  assert!(
    count(source("scan.rs"), ".adopt(") == 1,
    "SETTLE_CENSUS drift: `AtFrontier::adopt` must have exactly one caller \
     (`skip_and_report` in scan.rs). A second skip settle splits the surface the census \
     exists to keep whole (grep SETTLE_CENSUS)."
  );
  for (name, src) in SOURCES {
    if *name == "scan.rs" {
      continue;
    }
    assert!(
      count(src, ".adopt(") == 0,
      "SETTLE_CENSUS drift: `{name}` calls `adopt` outside the scanner's skip settle \
       (grep SETTLE_CENSUS)."
    );
  }
  assert!(
    count(source("mod.rs"), "fn adopt(") == 1,
    "SETTLE_CENSUS drift: `AtFrontier::adopt` must be defined exactly once, in `mod.rs`"
  );
}
