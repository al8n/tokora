use core::marker::PhantomData;

use super::*;

mod parse_input;

/// A type-safe alias for configuring `Repeated` parsers.
///
/// Canonical configuration layout: `With<With<Trailing, Leading>, With<Maximum, Minimum>>`.
pub type RepeatedOptions<Max = (), Min = ()> = With<PhantomData<()>, With<Max, Min>>;

impl<MAX, MIN> MaxSpec for RepeatedOptions<MAX, MIN>
where
  MAX: MaxSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    self.secondary.primary.maximum()
  }
}

impl<MAX, MIN> MinSpec for RepeatedOptions<MAX, MIN>
where
  MIN: MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    self.secondary.secondary.minimum()
  }
}

/// A parser that parses a sequence of elements separated by a specific separator.
pub struct Repeated<F, Condition, O, DecisionWindow, Config = RepeatedOptions> {
  f: F,
  condition: Condition,
  config: Config,
  _m: PhantomData<O>,
  _cap: PhantomData<DecisionWindow>,
}

impl<F, Condition, O, DecisionWindow: Capacity> Repeated<F, Condition, O, DecisionWindow> {
  /// Creates a new `Repeated` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(f: F, condition: Condition) -> Self {
    Self::new_in(f, condition)
  }

  /// Creates a new `Repeated` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new_in(f: F, condition: Condition) -> Self {
    Self {
      f,
      condition,
      config: RepeatedOptions::new(PhantomData, With::new((), ())),
      _m: PhantomData,
      _cap: PhantomData,
    }
  }
}

impl<F, Condition, O, Options, DecisionWindow: Capacity>
  Repeated<F, Condition, O, DecisionWindow, Options>
{
  /// Collects the parsed elements into the specified container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn collect<Container>(self) -> Collect<Self, Container>
  where
    Container: Default,
  {
    Collect::new(self, Container::default())
  }

  /// Collects the parsed elements with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn collect_with<Container>(self, container: Container) -> Collect<Self, Container> {
    Collect::new(self, container)
  }
}

impl<F, Condition, O, Max, Min, DecisionWindow: Capacity>
  Repeated<F, Condition, O, DecisionWindow, RepeatedOptions<Max, Min>>
{
  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(
    self,
    n: Min::Options,
  ) -> Repeated<F, Condition, O, DecisionWindow, RepeatedOptions<Max, Minimum>>
  where
    Min: Apply<Minimum>,
  {
    Repeated {
      f: self.f,
      condition: self.condition,
      config: RepeatedOptions::new(
        self.config.primary,
        With::new(
          self.config.secondary.primary,
          Min::apply(self.config.secondary.secondary, n),
        ),
      ),
      _m: PhantomData,
      _cap: PhantomData,
    }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(
    self,
    n: Max::Options,
  ) -> Repeated<F, Condition, O, DecisionWindow, RepeatedOptions<Maximum, Min>>
  where
    Max: Apply<Maximum>,
  {
    Repeated {
      f: self.f,
      condition: self.condition,
      config: RepeatedOptions::new(
        self.config.primary,
        With::new(
          Max::apply(self.config.secondary.primary, n),
          self.config.secondary.secondary,
        ),
      ),
      _m: PhantomData,
      _cap: PhantomData,
    }
  }

  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn minimum(&self) -> usize
  where
    Min: MinSpec,
  {
    Min::minimum(&self.config.secondary.secondary)
  }

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn maximum(&self) -> usize
  where
    Max: MaxSpec,
  {
    Max::maximum(&self.config.secondary.primary)
  }
}
