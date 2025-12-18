use super::*;

/// A parser that matches its inner parser at most `maximum` times.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AllowSurrounded<P> {
  pub(in crate::parser) parser: P,
}

impl<P> AllowSurrounded<P> {
  /// Creates a new `AllowSurrounded` parser that matches its inner parser at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(in crate::parser) const fn new(parser: P) -> Self {
    Self { parser }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, maximum: usize) -> AllowSurrounded<AtMost<P>>
  where
    Self: Apply<AllowSurrounded<AtMost<P>>, Options = Maximum>,
  {
    self.apply(Maximum::new(maximum))
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, minimum: usize) -> AllowSurrounded<AtLeast<P>>
  where
    Self: Apply<AllowSurrounded<AtLeast<P>>, Options = Minimum>,
  {
    self.apply(Minimum::new(minimum))
  }

  /// Sets both the minimum and maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bounded(self, minimum: usize, maximum: usize) -> AllowSurrounded<Bounded<P>>
  where
    Self: Apply<AllowSurrounded<Bounded<P>>, Options = With<Minimum, Maximum>>,
  {
    self.apply(With::new(Minimum::new(minimum), Maximum::new(maximum)))
  }

  /// Returns a mutable reference to the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn parser_mut(&mut self) -> &mut P {
    &mut self.parser
  }

  /// Returns a mutable reference to the `AllowSurrounded` parser wrapping the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> AllowSurrounded<&mut P> {
    AllowSurrounded {
      parser: &mut self.parser,
    }
  }

  /// Maps the inner parser to a new parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn map_parser_mut<'a, F, NP>(&'a mut self, f: F) -> AllowSurrounded<NP>
  where
    F: FnOnce(&'a mut P) -> NP,
    NP: 'a,
  {
    AllowSurrounded {
      parser: f(&mut self.parser),
    }
  }
}

impl<F, Condition, Sep, O, W, L, Ctx, Lang: ?Sized>
  AllowSurrounded<SeparatedBy<F, Sep, Condition, O, W, L, Ctx, Lang>>
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

