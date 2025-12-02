/// A parser that collects results into a container.
pub struct Collect<P, Container> {
  pub(crate) parser: P,
  pub(crate) container: Container,
}

impl<P, Container> Collect<P, Container> {
  /// Creates a new `Collect` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(parser: P, container: Container) -> Self {
    Self { parser, container }
  }
}
