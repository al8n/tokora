//! The budget arithmetic: the gate the driver asks before it works, and the charge it takes after.

use super::{TokenBudget, TokenBudgetTally};

/// A tally the way an `Input` gets one: from a configuration and from nothing else.
fn tally(max: usize) -> TokenBudgetTally {
  TokenBudgetTally::new(TokenBudget::with_limitation(max))
}

#[test]
fn an_unlimited_budget_never_refuses() {
  let mut budget = TokenBudgetTally::new(TokenBudget::unlimited());
  for _ in 0..1_000 {
    assert!(!budget.is_exhausted());
    budget.spend();
  }
  assert_eq!(budget.spent(), 1_000);
  assert!(!budget.is_exhausted());
}

#[test]
fn a_bounded_budget_charges_exactly_its_limitation() {
  let mut budget = tally(3);
  for _ in 0..3 {
    assert!(!budget.is_exhausted());
    budget.spend();
  }
  assert!(budget.is_exhausted());
  // The gate is a pure read, so asking it again reads the same — a refusal is not a charge, and
  // the driver re-asks it on every entry precisely because that is free and idempotent.
  assert!(budget.is_exhausted());
  assert_eq!(budget.spent(), 3);
  assert_eq!(budget.limitation(), 3);
}

#[test]
fn a_zero_budget_refuses_the_first_item() {
  let budget = tally(0);
  assert!(budget.is_exhausted());
  assert_eq!(budget.spent(), 0);
}

/// **Plant.** The **unlimited sentinel is not a ceiling**, and the gate has to say so out of the
/// `max` value rather than out of the comparison alone.
///
/// `unlimited()` is `max == usize::MAX`, and the gate was `spent >= max`. One charge from
/// `usize::MAX - 1` therefore lands `spent == max`, and from that point the budget that promised to
/// refuse nothing refuses **everything**, terminally.
///
/// # Why the seed and not a drain
///
/// The distance is `usize::MAX` produce-events, and the budget charges a **re-lex** as a
/// produce-event, so it is a distance one long-lived `Input` over a speculating grammar can cover —
/// not a four-billion-token document. At a 64-bit `usize` the seed is unreachable and the
/// arithmetic is the whole claim; at a **32-bit** `usize` the seeded value is `4_294_967_294`,
/// which is why this cell is also run under `--target wasm32-wasip1` (32-bit `usize`, executed by
/// wasmtime). [`the_sentinel_distance_is_a_reachable_count_at_thirty_two_bit_width`] pins the width
/// the run actually had.
#[test]
fn the_unlimited_sentinel_never_refuses_at_the_top_of_the_counter() {
  let mut budget = TokenBudgetTally::new(TokenBudget::unlimited());
  // The last state before the sentinel, reached by charging rather than by writing `spent`
  // directly, so `spend`'s own precondition is exercised on the way in.
  budget.spent = usize::MAX - 1;

  assert!(!budget.is_exhausted());
  budget.spend();

  assert_eq!(budget.spent(), usize::MAX);
  assert!(
    !budget.is_exhausted(),
    "`unlimited()` promised to refuse nothing; `spent >= usize::MAX` made it refuse everything",
  );
}

/// **Plant.** And the charge at the sentinel must not wrap. With the gate corrected, `spend` is
/// now reached *with* `spent == usize::MAX` — the one state the old precondition ruled out — so the
/// increment has to saturate. A wrap would put `spent` back to `0` and hand an unbounded budget a
/// fresh, silently-restarted counter.
///
/// Falsifying output with a wrapping charge: a `debug_assert` panic in a debug build
/// (`spent < max` is false at the sentinel) and a wrap to `0` in a release one — the same defect,
/// reported in one build and silent in the other.
#[test]
fn the_charge_at_the_sentinel_saturates_rather_than_wrapping() {
  let mut budget = TokenBudgetTally::new(TokenBudget::unlimited());
  budget.spent = usize::MAX;

  budget.spend();
  assert_eq!(budget.spent(), usize::MAX, "saturated, not wrapped to zero");
  assert!(!budget.is_exhausted());
}

/// The width the two cells above were actually run at, asserted rather than assumed.
///
/// The sentinel defect is arithmetic at any width and *reachable* only at 32 bits, and a defect
/// that exists only there is easy to ship. There is no 32-bit test leg, so the evidence is this
/// cell plus the target it ran under: at `wasm32-wasip1` the branch below is the live one and the
/// distance is a nine-digit number.
#[test]
fn the_sentinel_distance_is_a_reachable_count_at_thirty_two_bit_width() {
  #[cfg(target_pointer_width = "32")]
  assert_eq!(
    usize::MAX,
    4_294_967_295,
    "the 32-bit leg: the distance to the sentinel is 4.29e9 produce-events, re-lexes included",
  );
  #[cfg(target_pointer_width = "64")]
  assert_eq!(usize::MAX, 18_446_744_073_709_551_615);
  // `with_limitation(usize::MAX)` is the sentinel by value, so it must be the sentinel by
  // behaviour: nothing else would explain two budgets that compare equal and refuse differently.
  assert_eq!(
    TokenBudget::with_limitation(usize::MAX),
    TokenBudget::unlimited()
  );
}

/// **The split, at the only place an input's tally can come from.** A tally is built from a
/// [`TokenBudget`] and from nothing else, so the ceiling crosses the seam and the spend cannot.
///
/// This is what makes the transplant unrepresentable rather than merely unlikely. With `spent`
/// and the probe latch riding in the `Copy` configuration a caller hands to `with_token_budget`,
/// `*input.token_budget()` out of a refused parse and into the next context would make the next
/// `Input` start exhausted with a refusal on record — through both public doors over an
/// **empty** source: `Err(terminal)`, `refused_an_item() == true`, and **zero** `Lexer::lex`
/// calls.
///
/// Falsifying output if a spend ever finds its way back into the configuration and through here:
/// the second block reads a nonzero `spent` or a latched probe.
#[test]
fn a_tally_is_built_from_the_ceiling_alone() {
  let config = TokenBudget::with_limitation(4);

  let fresh = TokenBudgetTally::new(config);
  assert_eq!(fresh.limitation(), 4, "the ceiling crosses");
  assert_eq!(fresh.spent(), 0);
  assert!(!fresh.refused_an_item());
  assert!(!fresh.limit_probe_spent());

  // Spend the first tally to its ceiling and latch its probe — the exact state a transplant
  // would carry — then build again from the same configuration value.
  let mut refused = TokenBudgetTally::new(config);
  for _ in 0..4 {
    refused.spend();
  }
  refused.latch_limit_probe();
  assert!(refused.is_exhausted() && refused.refused_an_item());

  let again = TokenBudgetTally::new(config);
  assert_eq!(again.spent(), 0, "the configuration carried no spend");
  assert!(!again.refused_an_item(), "and no refusal");
  assert!(
    !again.is_exhausted(),
    "so the new input is not born refusing"
  );
  assert_eq!(again.limitation(), 4, "while the ceiling still crossed");
}

/// A look-ahead's settle puts the difference back to its entry value, and a nested pair counts the
/// inner's items once: the outer excludes its whole spend, the inner's included.
#[test]
fn a_look_ahead_settle_restores_the_difference_and_counts_a_nested_one_once() {
  let mut budget = TokenBudgetTally::new(TokenBudget::unlimited());
  budget.spend();
  budget.spend();
  let outer = budget.spent() - budget.look_ahead_spent();
  budget.spend();
  let inner = budget.spent() - budget.look_ahead_spent();
  for _ in 0..3 {
    budget.spend();
  }
  budget.settle_look_ahead(inner);
  assert_eq!(
    budget.look_ahead_spent(),
    3,
    "the inner excludes its own three"
  );
  budget.spend();
  budget.settle_look_ahead(outer);
  assert_eq!(
    budget.look_ahead_spent(),
    5,
    "the outer excludes its whole spend once: its own two and the inner's three"
  );
  assert_eq!(budget.spent() - budget.look_ahead_spent(), outer);
}

/// The settle never lowers the count, whatever `kept` it is handed: the `max` makes that a
/// property of the body.
#[test]
fn a_look_ahead_settle_never_lowers_the_count() {
  let mut budget = TokenBudgetTally::new(TokenBudget::unlimited());
  for _ in 0..4 {
    budget.spend();
  }
  budget.settle_look_ahead(0);
  assert_eq!(budget.look_ahead_spent(), 4);
  budget.settle_look_ahead(4);
  assert_eq!(
    budget.look_ahead_spent(),
    4,
    "a `kept` that would lower the count leaves it where it is"
  );
  budget.settle_look_ahead(usize::MAX);
  assert_eq!(
    budget.look_ahead_spent(),
    4,
    "and one past `spent` saturates to no change"
  );
}

/// At a saturated `spent` the settle still puts the difference back exactly: a look-ahead that
/// spends past `usize::MAX` is excluded by what it moved the counter, which is all it could move.
#[test]
fn a_look_ahead_settle_at_a_saturated_spend_keeps_the_difference() {
  let mut budget = TokenBudgetTally::new(TokenBudget::unlimited());
  budget.spent = usize::MAX - 1;
  let kept = budget.spent() - budget.look_ahead_spent();
  budget.spend();
  budget.spend();
  assert_eq!(budget.spent(), usize::MAX, "the second charge saturated");
  budget.settle_look_ahead(kept);
  assert_eq!(budget.spent() - budget.look_ahead_spent(), kept);
  assert!(budget.look_ahead_spent() <= budget.spent());
}

/// The configuration type is a number, and equality says so.
///
/// Two budgets are equal exactly when they name the same ceiling. That is only true because the
/// spend lives elsewhere: while it rode here, two budgets with the same ceiling and different
/// spends compared unequal, and — far worse — the unequal half was installable.
#[test]
fn the_configuration_is_the_ceiling_and_nothing_else() {
  assert_eq!(
    TokenBudget::with_limitation(7),
    TokenBudget::with_limitation(7)
  );
  assert_eq!(TokenBudget::default(), TokenBudget::unlimited());
  assert_eq!(
    core::mem::size_of::<TokenBudget>(),
    core::mem::size_of::<usize>(),
    "a ceiling, and no room for anything else",
  );
}
