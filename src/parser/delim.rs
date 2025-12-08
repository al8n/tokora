use super::*;

mod parse_input;

/// A parser that parses a construct delimited by left and right tokens.
///
/// See also: [`DelimSepSeq`]
pub struct DelimitedBy<P, Condition, Open, Close, Delim, O, W, Config = RepeatedOptions> {
  parser: Repeated<P, Condition, O, W, Config>,
  left_classifier: Open,
  right_classifier: Close,
  delimiter: Delim,
  _m: PhantomData<O>,
  _window: PhantomData<W>,
}

impl<P, Condition, Open, Close, Delim, O, W, Options>
  DelimitedBy<P, Condition, Open, Close, Delim, O, W, Options>
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

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(
    parser: Repeated<P, Condition, O, W, Options>,
    left: Open,
    right: Close,
    delim: Delim,
  ) -> Self {
    Self {
      parser,
      left_classifier: left,
      right_classifier: right,
      delimiter: delim,
      _m: PhantomData,
      _window: PhantomData,
    }
  }
}

impl<F, Condition, Open, Close, Delim, O, Max, Min, W>
  DelimitedBy<F, Condition, Open, Close, Delim, O, W, RepeatedOptions<Max, Min>>
{
  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn minimum(&self) -> usize
  where
    Min: MinSpec,
  {
    Min::minimum(&self.parser.config.secondary.secondary)
  }

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn maximum(&self) -> usize
  where
    Max: MaxSpec,
  {
    Max::maximum(&self.parser.config.secondary.primary)
  }
}
