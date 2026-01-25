use super::*;

/// A parser that matches its inner parser at most `maximum` times.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequireLeading<P> {
  pub(in crate::parser) parser: P,
}

impl<P> RequireLeading<P> {
  /// Creates a new `RequireLeading` parser that matches its inner parser at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(in crate::parser) const fn new(parser: P) -> Self {
    Self { parser }
  }

  /// Sets the parser to require trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_trailing(self) -> RequireLeading<RequireTrailing<P>> {
    RequireLeading {
      parser: RequireTrailing {
        parser: self.parser,
      },
    }
  }

  /// Sets the parser to allow trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(self) -> RequireLeading<AllowTrailing<P>> {
    RequireLeading {
      parser: AllowTrailing {
        parser: self.parser,
      },
    }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, maximum: usize) -> RequireLeading<AtMost<P>> {
    RequireLeading {
      parser: AtMost::new(self.parser, maximum),
    }
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, minimum: usize) -> RequireLeading<AtLeast<P>> {
    RequireLeading {
      parser: AtLeast::new(self.parser, minimum),
    }
  }

  /// Sets both the minimum and maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bounded(self, minimum: usize, maximum: usize) -> RequireLeading<Bounded<P>> {
    RequireLeading {
      parser: Bounded::new(self.parser, maximum, minimum),
    }
  }

  /// Returns a mutable reference to the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn parser_mut(&mut self) -> &mut P {
    &mut self.parser
  }

  /// Returns a mutable reference to the `RequireLeading` parser wrapping the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> RequireLeading<&mut P> {
    RequireLeading {
      parser: &mut self.parser,
    }
  }

  /// Delimits the parser with the given open and close classifiers and delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimited<Delim>(self) -> DelimitedBy<Self, Delim> {
    DelimitedBy::<_, Delim>::new_in(self)
  }

  /// Maps the inner parser to a new parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn map_parser_mut<'a, F, NP>(&'a mut self, f: F) -> RequireLeading<NP>
  where
    F: FnOnce(&'a mut P) -> NP,
    NP: 'a,
  {
    RequireLeading {
      parser: f(&mut self.parser),
    }
  }
}
