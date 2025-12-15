use super::*;

/// A parser that matches its inner parser at most `maximum` times.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AtLeast<P> {
  pub(super) minimum: usize,
  pub(super) parser: P,
}

impl<P> AtLeast<P> {
  /// Creates a new `AtLeast` parser that matches its inner parser at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(parser: P, minimum: usize) -> Self {
    Self { minimum, parser }
  }

  /// Returns the minimum number of times the inner parser should match.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn minimum(&self) -> usize {
    self.minimum
  }

  /// Creates a `Bounded` parser that matches its inner parser at least `minimum` and at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, maximum: usize) -> Bounded<P>
  where
    Self: Apply<Bounded<P>, Options = Maximum>,
  {
    self.apply(Maximum::new(maximum))
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized> AtLeast<Repeated<F, Condition, O, W, L, Ctx, Lang>> {
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
  // ) -> DelimitedBy<F, Condition, Open, Close, Delim, O, W, L, Ctx, Lang> {
  //   DelimitedBy::new_in(self, left, right, delim)
  // }
}
