use super::*;

/// A parser that matches its inner parser at most `maximum` times.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequireTrailing<P> {
  pub(in crate::parser) parser: P,
}

impl<P> RequireTrailing<P> {
  /// Creates a new `RequireTrailing` parser that matches its inner parser at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(in crate::parser) const fn new(parser: P) -> Self {
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
  pub const fn delimited_by<Open, Close, Delim>(
    self,
    left: Open,
    right: Close,
    delim: Delim,
  ) -> DelimitedBy<Self, Open, Close, Delim> {
    DelimitedBy::new_in(self, left, right, delim)
  }

  /// Maps the inner parser to a new parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn map_parser_mut<'a, F, NP>(&'a mut self, f: F) -> RequireTrailing<NP>
  where
    F: FnOnce(&'a mut P) -> NP,
    NP: 'a,
  {
    RequireTrailing {
      parser: f(&mut self.parser),
    }
  }
}

impl<F, Condition, Sep, O, W, L, Ctx, Lang: ?Sized>
  RequireTrailing<SeparatedBy<F, Sep, Condition, O, W, L, Ctx, Lang>>
{
  /// Collects the parsed elements into the specified container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn collect<Container>(self) -> Collect<Self, Container, Ctx, Lang>
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
  ) -> Collect<Self, Container, Ctx, Lang> {
    Collect::new(self, container)
  }
}

impl<F, Condition, Sep, Open, Close, Delim, O, W, L, Ctx, Lang: ?Sized>
  Apply<DelimitedBy<Self, Open, Close, Delim>>
  for RequireTrailing<SeparatedBy<F, Sep, Condition, O, W, L, Ctx, Lang>>
{
  type Options = (Open, Close, Delim);

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(
    self,
    (open, close, delim): (Open, Close, Delim),
  ) -> DelimitedBy<Self, Open, Close, Delim> {
    DelimitedBy::new_in(self, open, close, delim)
  }
}
