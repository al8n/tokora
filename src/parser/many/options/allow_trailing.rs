use super::*;

/// A parser that matches its inner parser at most `maximum` times.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AllowTrailing<P> {
  pub(in crate::parser) parser: P,
}

impl<P> AllowTrailing<P> {
  /// Creates a new `AllowTrailing` parser that matches its inner parser at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(in crate::parser) const fn new(parser: P) -> Self {
    Self { parser }
  }

  /// Sets the parser to allow trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn allow_leading(self) -> AllowLeading<AllowTrailing<P>> {
    AllowLeading::new(self)
  }

  /// Sets the parser to require trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_leading(self) -> RequireLeading<AllowTrailing<P>> {
    RequireLeading::new(self)
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, maximum: usize) -> AllowTrailing<AtMost<P>> {
    AllowTrailing::new(AtMost::new(self.parser, maximum))
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, minimum: usize) -> AllowTrailing<AtLeast<P>> {
    AllowTrailing::new(AtLeast::new(self.parser, minimum))
  }

  /// Sets both the minimum and maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bounded(self, minimum: usize, maximum: usize) -> AllowTrailing<Bounded<P>> {
    AllowTrailing::new(Bounded::new(self.parser, maximum, minimum))
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

  /// Returns a mutable reference to the `AllowTrailing` parser wrapping the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> AllowTrailing<&mut P> {
    AllowTrailing {
      parser: &mut self.parser,
    }
  }

  /// Maps the inner parser to a new parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn map_parser_mut<'a, F, NP>(&'a mut self, f: F) -> AllowTrailing<NP>
  where
    F: FnOnce(&'a mut P) -> NP,
    NP: 'a,
  {
    AllowTrailing {
      parser: f(&mut self.parser),
    }
  }
}
