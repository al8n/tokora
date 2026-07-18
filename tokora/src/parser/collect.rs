use core::marker::PhantomData;

use crate::input::Complete;

/// A parser that collects results into a container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Collect<P, Container, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  pub(crate) parser: P,
  pub(crate) container: Container,
  pub(crate) _ctx: PhantomData<Ctx>,
  pub(crate) _lang: PhantomData<Lang>,
  pub(crate) _cmpl: PhantomData<Cmpl>,
}

impl<P, Container, Ctx, Lang: ?Sized, Cmpl> Collect<P, Container, Ctx, Lang, Cmpl> {
  /// Creates a new `Collect` combinator.
  #[inline(always)]
  pub const fn new(parser: P, container: Container) -> Self {
    Self {
      parser,
      container,
      _ctx: PhantomData,
      _lang: PhantomData,
      _cmpl: PhantomData,
    }
  }

  /// Creates a mutable reference version of this `Collect` combinator.
  #[inline(always)]
  pub const fn as_mut(&mut self) -> Collect<&mut P, &mut Container, Ctx, Lang, Cmpl> {
    Collect {
      parser: &mut self.parser,
      container: &mut self.container,
      _ctx: PhantomData,
      _lang: PhantomData,
      _cmpl: PhantomData,
    }
  }

  /// Maps the inner parser to a new parser.
  #[inline(always)]
  pub fn map_parser<F, P2>(self, f: F) -> Collect<P2, Container, Ctx, Lang, Cmpl>
  where
    F: FnOnce(P) -> P2,
  {
    Collect {
      parser: f(self.parser),
      container: self.container,
      _ctx: PhantomData,
      _lang: PhantomData,
      _cmpl: PhantomData,
    }
  }

  /// Returns mutable references to the inner parser and container.
  #[inline(always)]
  pub fn parts_mut(&mut self) -> (&mut P, &mut Container) {
    (&mut self.parser, &mut self.container)
  }

  /// Maps the inner container to a new container.
  #[inline(always)]
  pub fn map_container<F, C2>(self, f: F) -> Collect<P, C2, Ctx, Lang, Cmpl>
  where
    F: FnOnce(Container) -> C2,
  {
    Collect {
      parser: self.parser,
      container: f(self.container),
      _ctx: PhantomData,
      _lang: PhantomData,
      _cmpl: PhantomData,
    }
  }
}
