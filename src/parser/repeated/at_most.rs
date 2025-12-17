use super::*;

/// A parser that matches its inner parser at most `maximum` times.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AtMost<P> {
  pub(in crate::parser) maximum: usize,
  pub(in crate::parser) parser: P,
}

impl<P> AtMost<P> {
  /// Creates a new `AtMost` parser that matches its inner parser at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(in crate::parser) const fn new(parser: P, maximum: usize) -> Self {
    Self { maximum, parser }
  }

  /// Creates a `Bounded` parser that matches its inner parser at least `minimum` and at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, minimum: usize) -> Bounded<P>
  where
    Self: Apply<Bounded<P>, Options = Minimum>,
  {
    self.apply(Minimum::new(minimum))
  }

  /// Returns the maximum number of times the inner parser should match.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maximum(&self) -> usize {
    self.maximum
  }

  /// Returns a mutable reference to the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn parser_mut(&mut self) -> &mut P {
    &mut self.parser
  }

  /// Maps the inner parser to a new parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn map_parser_mut<'a, F, NP>(&'a mut self, f: F) -> AtMost<NP>
  where
    F: FnOnce(&'a mut P) -> NP,
    NP: 'a,
  {
    AtMost {
      maximum: self.maximum,
      parser: f(&mut self.parser),
    }
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized> AtMost<Repeated<F, Condition, O, W, L, Ctx, Lang>> {
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
