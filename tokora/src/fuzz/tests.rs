use super::*;

/// The alphabet census is intact: exactly [`EXPECTED_OP_COUNT`] operations, all labels distinct.
/// (The size is also a compile-time `const _` assert in `ops.rs`; this gives a runtime message.)
#[test]
fn op_alphabet_census() {
  assert_eq!(
    Op::COUNT,
    EXPECTED_OP_COUNT,
    "OP_SURFACE_CENSUS drift: the fuzz Op alphabet changed size (grep OP_SURFACE_CENSUS)"
  );
  let mut labels = all_labels();
  labels.sort_unstable();
  labels.dedup();
  assert_eq!(labels.len(), Op::COUNT, "two Op variants share a label");
}

/// The default corpus passes every oracle AND exercises **every** operation in the alphabet. The
/// coverage half is the runtime anti-lag prod: a declared-but-unfuzzed op fails here, naming
/// itself, so a new alphabet entry cannot sit un-exercised.
#[test]
fn corpus_passes_and_covers_every_op() {
  let cov = run_seeds(0..DEFAULT_CORPUS);
  let missing = cov.missing();
  assert!(
    missing.is_empty(),
    "the default corpus (0..{DEFAULT_CORPUS}) did not exercise every Op: missing {:?}. \
     Widen the corpus or teach a generator to emit these (grep OP_SURFACE_CENSUS).",
    missing.iter().map(|op| op.label()).collect::<Vec<_>>()
  );

  // The corpus-margin cell's aggregate half for THIS population: the recorded figure must be
  // ATTAINED, not merely not-exceeded, so a corpus that quietly got shallower cannot leave a
  // stale number behind it in the margin the green-tree depth ceiling is argued from.
  //
  // A floor rather than an equality, and the counter's process-wide monotonicity is why: in CI
  // this test and `fuzz_deep_run` never share a process, but `--include-ignored` puts them in
  // one and the maximum could then be the sweep's. The upper side is the per-case assertion in
  // `cst::run`, which is exact in every run and pinned at the deep sweep's figure.
  #[cfg(feature = "rowan_0_17")]
  assert!(
    cst::deepest_tree_seen() >= cst::CORPUS_DEEPEST_TREE,
    "the default corpus no longer materializes a tree as deep as the recorded {}. Re-record \
     it in `fuzz/cst.rs`; it is the floor witness that the depth cell measures a non-empty \
     population at all.",
    cst::CORPUS_DEEPEST_TREE
  );
}

/// A deeper manual sweep. Ignored by default (part of neither CI nor the normal suite); run with
/// `cargo test --features fuzz -- --ignored fuzz_deep_run`.
#[test]
#[ignore = "long-running manual deep fuzz sweep"]
fn fuzz_deep_run() {
  let cov = run_seeds(0..50_000);
  assert!(
    cov.is_complete(),
    "deep run failed to cover the alphabet: {:?}",
    cov.missing()
  );

  // The deep sweep owns the figure the per-case depth bound is pinned at, so it is the sweep
  // that has to reach it. This is the only cell that runs this population, and `test (release)`
  // is the only CI job that runs this cell — which is why a bound pinned at the SMALLER
  // corpus's maximum was green everywhere else and red only there.
  #[cfg(feature = "rowan_0_17")]
  assert_eq!(
    cst::deepest_tree_seen(),
    cst::DEEP_SWEEP_DEEPEST_TREE,
    "the deepest green tree the 50 000-seed sweep materializes is no longer the recorded \
     figure. Re-record it in `fuzz/cst.rs`; it is the bound `cst::run` enforces per case and \
     the figure the margin against `cst::MAX_TREE_DEPTH` is argued from."
  );
}
