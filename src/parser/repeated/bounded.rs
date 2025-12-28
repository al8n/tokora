use super::*;

/// A parser that matches its inner parser at most `maximum` times.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bounded<P> {
  pub(in crate::parser) maximum: Maximum,
  pub(in crate::parser) minimum: Minimum,
  pub(in crate::parser) parser: P,
}

impl<P> Bounded<P> {
  /// Creates a new `Bounded` parser that matches its inner parser at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(in crate::parser) const fn new(parser: P, maximum: usize, minimum: usize) -> Self {
    Self {
      maximum: Maximum::new(maximum),
      minimum: Minimum::new(minimum),
      parser,
    }
  }

  /// Returns the maximum number of times the inner parser should match.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maximum(&self) -> Maximum {
    self.maximum
  }

  /// Returns the minimum number of times the inner parser should match.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn minimum(&self) -> Minimum {
    self.minimum
  }

  /// Delimits the parser with the given open and close classifiers and delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimited_by<Open, Close, Delim>(
    self,
    left: Open,
    right: Close,
    delim: Delim,
  ) -> DelimitedBy<Self, Open, Close, Delim> {
    DelimitedBy::new_in(self, left, right, delim)
  }

  /// Returns a mutable reference to the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn parser_mut(&mut self) -> &mut P {
    &mut self.parser
  }

  /// Returns a mutable `Bounded` parser with a mutable reference to the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> Bounded<&mut P> {
    Bounded {
      maximum: self.maximum,
      minimum: self.minimum,
      parser: &mut self.parser,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn to_with(&self) -> With<Minimum, Maximum> {
    With::new(self.minimum(), self.maximum())
  }

  /// Maps the inner parser to a new parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn map_parser_mut<'a, F, NP>(&'a mut self, f: F) -> Bounded<NP>
  where
    F: FnOnce(&'a mut P) -> NP,
    NP: 'a,
  {
    Bounded {
      maximum: self.maximum,
      minimum: self.minimum,
      parser: f(&mut self.parser),
    }
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
}
