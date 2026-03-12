use super::*;

/// A parser that requires trailing separators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequireTrailing<P> {
  pub(in crate::parser) parser: P,
}

impl<P> RequireTrailing<P> {
  /// Creates a new `RequireTrailing` parser that requires trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(parser: P) -> Self {
    Self { parser }
  }

  /// Sets the parser to require leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_leading(self) -> RequireLeading<RequireTrailing<P>> {
    RequireLeading { parser: self }
  }

  /// Sets the parser to allow leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_leading(self) -> AllowLeading<RequireTrailing<P>> {
    AllowLeading { parser: self }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, maximum: usize) -> RequireTrailing<AtMost<P>> {
    RequireTrailing {
      parser: AtMost::new(self.parser, maximum),
    }
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, minimum: usize) -> RequireTrailing<AtLeast<P>> {
    RequireTrailing {
      parser: AtLeast::new(self.parser, minimum),
    }
  }

  /// Sets both the minimum and maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bounded(self, minimum: usize, maximum: usize) -> RequireTrailing<Bounded<P>> {
    RequireTrailing {
      parser: Bounded::new(self.parser, maximum, minimum),
    }
  }

  /// Returns a mutable reference to the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn parser_mut(&mut self) -> &mut P {
    &mut self.parser
  }

  /// Returns a mutable reference to the `RequireTrailing` parser wrapping the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> RequireTrailing<&mut P> {
    RequireTrailing {
      parser: &mut self.parser,
    }
  }

  /// Delimits the parser with the given open and close classifiers and delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimited<Delim>(self) -> DelimitedBy<Self, Delim> {
    DelimitedBy::<_, Delim>::new(self)
  }

  /// Maps the inner parser to a new parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map_parser_mut<'a, F, NP>(&'a mut self, f: F) -> RequireTrailing<NP>
  where
    F: FnOnce(&'a mut P) -> NP,
    NP: 'a,
  {
    RequireTrailing {
      parser: f(&mut self.parser),
    }
  }
}
