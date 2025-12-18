use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

/// A parser that matches its inner parser at most `maximum` times.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AllowLeading<P> {
  pub(in crate::parser) parser: P,
}

impl<P> AllowLeading<P> {
  /// Creates a new `AllowLeading` parser that matches its inner parser at most `maximum` times.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(in crate::parser) const fn new(parser: P) -> Self {
    Self { parser }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, maximum: usize) -> AllowLeading<AtMost<P>>
  where
    Self: Apply<AllowLeading<AtMost<P>>, Options = Maximum>,
  {
    self.apply(Maximum::new(maximum))
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, minimum: usize) -> AllowLeading<AtLeast<P>>
  where
    Self: Apply<AllowLeading<AtLeast<P>>, Options = Minimum>,
  {
    self.apply(Minimum::new(minimum))
  }

  /// Sets both the minimum and maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bounded(self, minimum: usize, maximum: usize) -> AllowLeading<Bounded<P>>
  where
    Self: Apply<AllowLeading<Bounded<P>>, Options = With<Minimum, Maximum>>,
  {
    self.apply(With::new(Minimum::new(minimum), Maximum::new(maximum)))
  }

  /// Returns a mutable reference to the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn parser_mut(&mut self) -> &mut P {
    &mut self.parser
  }

  /// Returns a mutable reference to the `AllowLeading` parser wrapping the inner parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> AllowLeading<&mut P> {
    AllowLeading {
      parser: &mut self.parser,
    }
  }

  /// Maps the inner parser to a new parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn map_parser_mut<'a, F, NP>(&'a mut self, f: F) -> AllowLeading<NP>
  where
    F: FnOnce(&'a mut P) -> NP,
    NP: 'a,
  {
    AllowLeading {
      parser: f(&mut self.parser),
    }
  }
}

impl<F, Condition, Sep, O, W, L, Ctx, Lang: ?Sized>
  AllowLeading<SeparatedBy<F, Sep, Condition, O, W, L, Ctx, Lang>>
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
