use super::*;

/// A parser that matches its inner parser at most `maximum` times.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bounded<P> {
  pub(super) maximum: usize,
  pub(super) minimum: usize,
  pub(super) parser: P,
}

impl<P> Bounded<P> {
  /// Creates a new `Bounded` parser that matches its inner parser at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(parser: P, maximum: usize, minimum: usize) -> Self {
    Self {
      maximum,
      minimum,
      parser,
    }
  }

  /// Returns the maximum number of times the inner parser should match.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maximum(&self) -> usize {
    self.maximum
  }

  /// Returns the minimum number of times the inner parser should match.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn minimum(&self) -> usize {
    self.minimum
  }

  /// Returns a mutable reference to the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn parser_mut(&mut self) -> &mut P {
    &mut self.parser
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized> Bounded<Repeated<F, Condition, O, W, L, Ctx, Lang>> {
  /// Collects the parsed elements into the specified container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn collect<Container>(self) -> Collect<Self, Container, (), ()>
  where
    Container: Default,
  {
    Collect::new(self, Container::default())
  }

  /// Collects the parsed elements with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn collect_with<Container>(
    self,
    container: Container,
  ) -> Collect<Self, Container, (), ()> {
    Collect::new(self, container)
  }

  // /// Creates a new `Delimited` parser with the given delimiters and separator.
  // #[cfg_attr(not(tarpaulin), inline(always))]
  // pub const fn delimited_by<Open, Close, Delim>(
  //   self,
  //   left: Open,
  //   right: Close,
  //   delim: Delim,
  // ) -> DelimitedBy<Self, Open, Close, Delim, O, W, L, Ctx, Lang> {
  //   DelimitedBy::new_in(self, left, right, delim)
  // }
}
