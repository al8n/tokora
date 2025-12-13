use core::marker::PhantomData;

/// A parser that collects results into a container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Collect<P, Container, Ctx, Lang: ?Sized = ()> {
  pub(crate) parser: P,
  pub(crate) container: Container,
  pub(crate) _ctx: PhantomData<Ctx>,
  pub(crate) _lang: PhantomData<Lang>,
}

impl<P, Container, Ctx, Lang: ?Sized> Collect<P, Container, Ctx, Lang> {
  /// Creates a new `Collect` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(parser: P, container: Container) -> Self {
    Self {
      parser,
      container,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Creates a mutable reference version of this `Collect` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> Collect<&mut P, &mut Container, Ctx, Lang> {
    Collect {
      parser: &mut self.parser,
      container: &mut self.container,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Maps the inner parser to a new parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map_parser<F, P2>(self, f: F) -> Collect<P2, Container, Ctx, Lang>
  where
    F: FnOnce(P) -> P2,
  {
    Collect {
      parser: f(self.parser),
      container: self.container,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Maps the inner container to a new container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map_container<F, C2>(self, f: F) -> Collect<P, C2, Ctx, Lang>
  where
    F: FnOnce(Container) -> C2,
  {
    Collect {
      parser: self.parser,
      container: f(self.container),
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}
