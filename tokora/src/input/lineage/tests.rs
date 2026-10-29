use super::Lineage;

/// The three witness counters are load-bearing for a **shipped optimization**, not just
/// bookkeeping: the `contains` / `pop_through` fast paths are binary searches licensed by
/// "ids monotone, `live_ckpts` sorted ascending", and a wrapped id violates that in silence.
/// So each increment panics at the wrap point rather than rolling over.
///
/// Falsifying output: no panic — the counter wrapped, and the search's precondition is now
/// false with nothing to say so.
#[test]
#[cfg(any(feature = "std", feature = "alloc"))]
#[should_panic(expected = "checkpoint id space exhausted")]
fn checkpoint_id_exhaustion_panics() {
  let mut lineage = Lineage::new();
  lineage.seed_counters_at_max_for_tests();
  let _ = lineage.open();
}

#[test]
#[cfg(any(feature = "std", feature = "alloc"))]
#[should_panic(expected = "savepoint sequence exhausted")]
fn savepoint_seq_exhaustion_panics() {
  let mut lineage = Lineage::new();
  lineage.seed_counters_at_max_for_tests();
  let _ = lineage.next_savepoint_seq();
}

#[test]
#[should_panic(expected = "cache-push counter exhausted")]
fn cache_push_exhaustion_panics() {
  let mut lineage = Lineage::new();
  lineage.seed_counters_at_max_for_tests();
  lineage.record_cache_push();
}

/// The keep-green direction: none of the three fires on an ordinary history.
#[test]
fn ordinary_use_never_trips_a_witness() {
  let mut lineage = Lineage::new();
  for _ in 0..1_000 {
    lineage.record_cache_push();
    #[cfg(any(feature = "std", feature = "alloc"))]
    let _ = lineage.next_savepoint_seq();
  }
  assert_eq!(lineage.cache_pushes(), 1_000);
}
