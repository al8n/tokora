use super::{Apply, DelimitedBy, With};

pub use allow_leading::AllowLeading;
pub use allow_trailing::AllowTrailing;
pub use at_least::*;
pub use at_most::*;
pub use bounded::*;
pub use require_leading::RequireLeading;
pub use require_trailing::RequireTrailing;

mod allow_leading;
mod allow_trailing;
mod at_least;
mod at_most;
mod bounded;
mod require_leading;
mod require_trailing;

/// A marker type representing the maximum number of elements allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Maximum(usize);

impl Maximum {
  /// The maximum possible value for `Maximum`.
  pub const MAX: Self = Self::new(usize::MAX);

  /// Creates a new `Maximum`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(n: usize) -> Self {
    Self(n)
  }

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

/// A marker type representing the minimum number of elements required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Minimum(usize);

impl Minimum {
  /// The minimum possible value for `Minimum`.
  pub const MIN: Self = Self::new(0);

  /// Creates a new `Minimum`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(n: usize) -> Self {
    Self(n)
  }

  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

pub(super) struct Unbounded;

#[allow(warnings)]
#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;
  use std::format;

  // --- Maximum tests ---

  #[test]
  fn maximum_new_and_get() {
    let m = Maximum::new(42);
    assert_eq!(m.get(), 42);
  }

  #[test]
  fn maximum_max_constant() {
    assert_eq!(Maximum::MAX.get(), usize::MAX);
  }

  #[test]
  fn maximum_eq() {
    assert_eq!(Maximum::new(10), Maximum::new(10));
    assert_ne!(Maximum::new(10), Maximum::new(20));
  }

  #[test]
  fn maximum_clone_copy() {
    let m = Maximum::new(5);
    let m2 = m;
    let m3 = m.clone();
    assert_eq!(m, m2);
    assert_eq!(m, m3);
  }

  #[test]
  fn maximum_debug() {
    let m = Maximum::new(7);
    let dbg = format!("{:?}", m);
    assert!(dbg.contains("7"));
  }

  #[test]
  #[cfg(feature = "std")]
  fn maximum_hash() {
    use std::collections::HashSet;
    let mut s = HashSet::new();
    s.insert(Maximum::new(1));
    s.insert(Maximum::new(1));
    assert_eq!(s.len(), 1);
    s.insert(Maximum::new(2));
    assert_eq!(s.len(), 2);
  }

  // --- Minimum tests ---

  #[test]
  fn minimum_new_and_get() {
    let m = Minimum::new(3);
    assert_eq!(m.get(), 3);
  }

  #[test]
  fn minimum_min_constant() {
    assert_eq!(Minimum::MIN.get(), 0);
  }

  #[test]
  fn minimum_eq() {
    assert_eq!(Minimum::new(10), Minimum::new(10));
    assert_ne!(Minimum::new(10), Minimum::new(20));
  }

  #[test]
  fn minimum_clone_copy() {
    let m = Minimum::new(5);
    let m2 = m;
    let m3 = m.clone();
    assert_eq!(m, m2);
    assert_eq!(m, m3);
  }

  #[test]
  fn minimum_debug() {
    let m = Minimum::new(9);
    let dbg = format!("{:?}", m);
    assert!(dbg.contains("9"));
  }

  #[test]
  #[cfg(feature = "std")]
  fn minimum_hash() {
    use std::collections::HashSet;
    let mut s = HashSet::new();
    s.insert(Minimum::new(1));
    s.insert(Minimum::new(1));
    assert_eq!(s.len(), 1);
    s.insert(Minimum::new(2));
    assert_eq!(s.len(), 2);
  }

  #[test]
  fn minimum_zero() {
    let m = Minimum::new(0);
    assert_eq!(m.get(), 0);
    assert_eq!(m, Minimum::MIN);
  }

  // --- AtLeast tests ---

  #[test]
  fn at_least_new_and_minimum() {
    let al = AtLeast::new("parser", 3);
    assert_eq!(al.minimum().get(), 3);
  }

  #[test]
  fn at_least_into_parser() {
    let al = AtLeast::new("my_parser", 1);
    assert_eq!(al.into_parser(), "my_parser");
  }

  #[test]
  fn at_least_parser_mut() {
    let mut al = AtLeast::new(10u32, 1);
    *al.parser_mut() = 20;
    assert_eq!(al.into_parser(), 20);
  }

  #[test]
  fn at_least_map_parser_mut() {
    let mut al = AtLeast::new(10u32, 3);
    let al2 = al.map_parser_mut(|p| *p * 2);
    assert_eq!(al2.minimum().get(), 3);
    assert_eq!(al2.into_parser(), 20);
  }

  // --- AtMost tests ---

  #[test]
  fn at_most_new_and_maximum() {
    let am = AtMost::new("parser", 10);
    assert_eq!(am.maximum().get(), 10);
  }

  #[test]
  fn at_most_parser_mut() {
    let mut am = AtMost::new(5u32, 10);
    *am.parser_mut() = 15;
    assert_eq!(am.maximum().get(), 10);
  }

  #[test]
  fn at_most_map_parser_mut() {
    let mut am = AtMost::new(10u32, 7);
    let am2 = am.map_parser_mut(|p| *p + 1);
    assert_eq!(am2.maximum().get(), 7);
  }

  // --- Bounded tests ---

  #[test]
  fn bounded_new_and_accessors() {
    let b = Bounded::new("p", 10, 2);
    assert_eq!(b.minimum().get(), 2);
    assert_eq!(b.maximum().get(), 10);
  }

  #[test]
  fn bounded_parser_mut() {
    let mut b = Bounded::new(5u32, 10, 2);
    *b.parser_mut() = 99;
    assert_eq!(b.minimum().get(), 2);
    assert_eq!(b.maximum().get(), 10);
  }

  #[test]
  fn bounded_as_mut() {
    let mut b = Bounded::new(5u32, 10, 2);
    let bm = b.as_mut();
    assert_eq!(bm.minimum().get(), 2);
    assert_eq!(bm.maximum().get(), 10);
  }

  #[test]
  fn bounded_to_with() {
    let b = Bounded::new("p", 10, 2);
    let w = b.to_with();
    assert_eq!(w.primary().get(), 2);
    assert_eq!(w.secondary().get(), 10);
  }

  #[test]
  fn bounded_map_parser_mut() {
    let mut b = Bounded::new(3u32, 10, 1);
    let b2 = b.map_parser_mut(|p| *p * 3);
    assert_eq!(b2.minimum().get(), 1);
    assert_eq!(b2.maximum().get(), 10);
  }

  // --- AllowLeading tests ---

  #[test]
  fn allow_leading_parser_mut() {
    let mut al = AllowLeading::new(42u32);
    *al.parser_mut() = 99;
    assert_eq!(*al.parser_mut(), 99);
  }

  #[test]
  fn allow_leading_allow_trailing() {
    let al = AllowLeading::new("p");
    let _alt = al.allow_trailing();
  }

  #[test]
  fn allow_leading_require_trailing() {
    let al = AllowLeading::new("p");
    let _rt = al.require_trailing();
  }

  #[test]
  fn allow_leading_at_most() {
    let al = AllowLeading::new("p");
    let am = al.at_most(5);
    assert_eq!(am.parser.maximum().get(), 5);
  }

  #[test]
  fn allow_leading_at_least() {
    let al = AllowLeading::new("p");
    let aleast = al.at_least(2);
    assert_eq!(aleast.parser.minimum().get(), 2);
  }

  #[test]
  fn allow_leading_bounded() {
    let al = AllowLeading::new("p");
    let b = al.bounded(1, 10);
    assert_eq!(b.parser.minimum().get(), 1);
    assert_eq!(b.parser.maximum().get(), 10);
  }

  #[test]
  fn allow_leading_as_mut() {
    let mut al = AllowLeading::new(5u32);
    let mut am = al.as_mut();
    assert_eq!(**am.parser_mut(), 5);
  }

  #[test]
  fn allow_leading_map_parser_mut() {
    let mut al = AllowLeading::new(10u32);
    let mut al2 = al.map_parser_mut(|p| *p + 5);
    assert_eq!(*al2.parser_mut(), 15);
  }

  // --- AllowTrailing tests ---

  #[test]
  fn allow_trailing_parser_mut() {
    let mut at = AllowTrailing::new(42u32);
    *at.parser_mut() = 99;
    assert_eq!(*at.parser_mut(), 99);
  }

  #[test]
  fn allow_trailing_allow_leading() {
    let at = AllowTrailing::new("p");
    let _al = at.allow_leading();
  }

  #[test]
  fn allow_trailing_require_leading() {
    let at = AllowTrailing::new("p");
    let _rl = at.require_leading();
  }

  #[test]
  fn allow_trailing_at_most() {
    let at = AllowTrailing::new("p");
    let am = at.at_most(8);
    assert_eq!(am.parser.maximum().get(), 8);
  }

  #[test]
  fn allow_trailing_at_least() {
    let at = AllowTrailing::new("p");
    let al = at.at_least(3);
    assert_eq!(al.parser.minimum().get(), 3);
  }

  #[test]
  fn allow_trailing_bounded() {
    let at = AllowTrailing::new("p");
    let b = at.bounded(2, 7);
    assert_eq!(b.parser.minimum().get(), 2);
    assert_eq!(b.parser.maximum().get(), 7);
  }

  #[test]
  fn allow_trailing_as_mut() {
    let mut at = AllowTrailing::new(5u32);
    let mut am = at.as_mut();
    assert_eq!(**am.parser_mut(), 5);
  }

  #[test]
  fn allow_trailing_map_parser_mut() {
    let mut at = AllowTrailing::new(10u32);
    let mut at2 = at.map_parser_mut(|p| *p * 2);
    assert_eq!(*at2.parser_mut(), 20);
  }

  // --- RequireLeading tests ---

  #[test]
  fn require_leading_parser_mut() {
    let mut rl = RequireLeading::new(42u32);
    *rl.parser_mut() = 99;
    assert_eq!(*rl.parser_mut(), 99);
  }

  #[test]
  fn require_leading_require_trailing() {
    let rl = RequireLeading::new("p");
    let _rt = rl.require_trailing();
  }

  #[test]
  fn require_leading_allow_trailing() {
    let rl = RequireLeading::new("p");
    let _at = rl.allow_trailing();
  }

  #[test]
  fn require_leading_at_most() {
    let rl = RequireLeading::new("p");
    let am = rl.at_most(5);
    assert_eq!(am.parser.maximum().get(), 5);
  }

  #[test]
  fn require_leading_at_least() {
    let rl = RequireLeading::new("p");
    let al = rl.at_least(2);
    assert_eq!(al.parser.minimum().get(), 2);
  }

  #[test]
  fn require_leading_bounded() {
    let rl = RequireLeading::new("p");
    let b = rl.bounded(1, 10);
    assert_eq!(b.parser.minimum().get(), 1);
    assert_eq!(b.parser.maximum().get(), 10);
  }

  #[test]
  fn require_leading_as_mut() {
    let mut rl = RequireLeading::new(5u32);
    let mut rm = rl.as_mut();
    assert_eq!(**rm.parser_mut(), 5);
  }

  #[test]
  fn require_leading_map_parser_mut() {
    let mut rl = RequireLeading::new(10u32);
    let mut rl2 = rl.map_parser_mut(|p| *p + 3);
    assert_eq!(*rl2.parser_mut(), 13);
  }

  // --- RequireTrailing tests ---

  #[test]
  fn require_trailing_parser_mut() {
    let mut rt = RequireTrailing::new(42u32);
    *rt.parser_mut() = 99;
    assert_eq!(*rt.parser_mut(), 99);
  }

  #[test]
  fn require_trailing_require_leading() {
    let rt = RequireTrailing::new("p");
    let _rl = rt.require_leading();
  }

  #[test]
  fn require_trailing_allow_leading() {
    let rt = RequireTrailing::new("p");
    let _al = rt.allow_leading();
  }

  #[test]
  fn require_trailing_at_most() {
    let rt = RequireTrailing::new("p");
    let am = rt.at_most(5);
    assert_eq!(am.parser.maximum().get(), 5);
  }

  #[test]
  fn require_trailing_at_least() {
    let rt = RequireTrailing::new("p");
    let al = rt.at_least(2);
    assert_eq!(al.parser.minimum().get(), 2);
  }

  #[test]
  fn require_trailing_bounded() {
    let rt = RequireTrailing::new("p");
    let b = rt.bounded(1, 10);
    assert_eq!(b.parser.minimum().get(), 1);
    assert_eq!(b.parser.maximum().get(), 10);
  }

  #[test]
  fn require_trailing_as_mut() {
    let mut rt = RequireTrailing::new(5u32);
    let mut rm = rt.as_mut();
    assert_eq!(**rm.parser_mut(), 5);
  }

  #[test]
  fn require_trailing_map_parser_mut() {
    let mut rt = RequireTrailing::new(10u32);
    let mut rt2 = rt.map_parser_mut(|p| *p + 7);
    assert_eq!(*rt2.parser_mut(), 17);
  }
}
