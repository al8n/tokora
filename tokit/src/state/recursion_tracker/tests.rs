use super::*;

#[test]
fn recursion_increase_saturates_at_max() {
  // At the ceiling `increase` must saturate rather than overflow-panic,
  // keeping symmetry with the saturating `decrease`.
  let mut r = RecursionLimiter {
    max: usize::MAX,
    current: usize::MAX,
  };
  r.increase();
  assert_eq!(r.depth(), usize::MAX);
}
