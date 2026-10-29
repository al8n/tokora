//! BUFFERED_FOLD_CENSUS — a buffering fold leaves its loop by `break`, never by `return`.
//!
//! Two of the four folds here buffer their input and drain it in reverse on a single shared
//! tail; that drain is their epilogue, and every `Ok` exit must run it exactly once. The
//! other two accumulate in place and have no epilogue, so their loops end with
//! `return Ok(output)`. Transplanting the epilogue-free arm into a buffering fold is what
//! consumed an accepted run into the buffer and then dropped it, returning the untouched
//! initializer on the `Ok` path. This census pins the two shapes apart by count so the
//! transplant cannot recur.
//!
//! It lives here rather than in `tests/census.rs` so the fold family's rail travels with the
//! file it guards.

/// Counts occurrences of `needle` on lines that are not whole-line comments, so prose
/// mentions of a counted form do not skew the tally.
fn code_matches(src: &str, needle: &str) -> usize {
  src
    .lines()
    .filter(|line| !line.trim_start().starts_with("//"))
    .map(|line| line.matches(needle).count())
    .sum()
}

/// The four fold methods, i.e. `mod.rs` with the census itself cut off the end — the
/// needles below appear verbatim in the census's own source and would otherwise be counted.
fn fold_methods() -> &'static str {
  let src = include_str!("mod.rs");
  let (methods, census) = src
    .split_once("#[cfg(test)]")
    .expect("the census marker must be present in its own source");
  assert!(
    census.contains("mod fold_census"),
    "the split must cut at the census module, not at some other cfg(test) item"
  );
  methods
}

#[test]
fn buffering_folds_break_and_unbuffered_folds_return() {
  let src = fold_methods();

  // `fold`'s exhaustion arm, and `foldn`'s two (capacity reached, input exhausted). These
  // three accumulate in place, so returning straight out of the loop is correct.
  assert_eq!(
    code_matches(src, "return Ok(output)"),
    3,
    "only the two unbuffered folds may return out of their loop: `fold` once and `foldn` \
     twice. A fourth means a buffering fold acquired an exit that jumps over its drain"
  );

  // `foldr_within` and `foldrn` each leave their loop two ways — capacity reached (`break;`)
  // and input exhausted (`None => break,`) — and both converge on the one shared drain.
  assert_eq!(
    code_matches(src, "=> break,"),
    2,
    "each buffering fold ends its input-exhausted arm with `break`, so its drain runs"
  );
  assert_eq!(
    code_matches(src, "break;"),
    2,
    "each buffering fold ends its capacity arm with `break`, so its drain runs"
  );
}
