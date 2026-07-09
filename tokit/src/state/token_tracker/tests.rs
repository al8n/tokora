use super::*;

#[test]
fn token_increase_saturates_at_max() {
  // At the ceiling `increase` must saturate rather than overflow-panic.
  let mut t = TokenLimiter {
    max: usize::MAX,
    current: usize::MAX,
  };
  t.increase();
  assert_eq!(t.tokens(), usize::MAX);
}
