#![allow(clippy::type_complexity)]

use core::marker::PhantomData;

use crate::{
  Cache, DefaultCache, Emitter, Lexer, Noop, Token,
  lexer::{Input, InputRef},
  utils::Spanned,
};

mod sealed {
  use super::*;

  pub trait Sealed<'inp, L, O, E, C> {}

  impl<'inp, F, L, O, E, C> sealed::Sealed<'inp, L, O, E, C> for F
  where
    F: FnMut(&mut InputRef<'inp, '_, L, E, C>) -> ParseResult<O, E::Error>,
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {
  }


  impl<'inp, F, L, O, E, C> Sealed<'inp, L, O, E, C> for Parser<F, L, O, E::Error>
  where
    F: ParseInput<'inp, L, O, E, C>,
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {
  }
}

/// Convenience result type returned by parser combinators.
pub type ParseResult<O, Err> = Result<O, Err>;

/// Core trait implemented by every parser combinator.
///
/// This mirrors the ergonomics of libraries like `winnow`: a parser is
/// simply something that can mutate an [`InputRef`] and either produce
/// a value or a spanned error using the configured `Emitter`.
pub trait ParseInput<'inp, L, O, E, C = DefaultCache<'inp, L>>:
  sealed::Sealed<'inp, L, O, E, C>
{
  /// Error type produced when the parser fails.
  type Error;

  /// Try to parse from the given input.
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, E, C>,
  ) -> ParseResult<O, Self::Error>
  where
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>;
}

impl<'inp, F, L, O, E, C> ParseInput<'inp, L, O, E, C> for F
where
  F: FnMut(&mut InputRef<'inp, '_, L, E, C>) -> ParseResult<O, E::Error>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  type Error = E::Error;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, E, C>,
  ) -> ParseResult<O, Self::Error> {
    (self)(input)
  }
}

/// Lightweight wrapper around a parsing function.
#[repr(transparent)]
pub struct Parser<F, L, O, Error> {
  f: F,
  _marker: PhantomData<(L, O, Error)>,
}

impl<F, L, O, Error> Parser<F, L, O, Error> {
  /// Wrap a parsing function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(f: F) -> Self {
    Self {
      f,
      _marker: PhantomData,
    }
  }

  /// Attach a custom emitter to the parser.
  pub fn with_emitter<E>(self, emitter: E) -> WithEmitter<Self, E> {
    WithEmitter {
      inner: self,
      emitter,
    }
  }

  /// Attach custom cache options to the parser.
  pub fn with_cache<'inp, C>(self, options: C::Options) -> WithCache<'inp, Self, L, C>
  where
    L: Lexer<'inp>,
    C: Cache<'inp, L>,
  {
    WithCache {
      inner: self,
      cache_opts: options,
      _marker: PhantomData,
    }
  }
}

impl<'inp, F, L, O, E, C> ParseInput<'inp, L, O, E, C>
  for Parser<F, L, O, E::Error>
where
  F: ParseInput<'inp, L, O, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  type Error = F::Error;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, E, C>,
  ) -> ParseResult<O, Self::Error> {
    self.f.parse_input(input)
  }
}

/// Parser configured with a concrete emitter.
pub struct WithEmitter<P, E> {
  inner: P,
  emitter: E,
}

impl<P, E> WithEmitter<P, E> {
  /// Attach cache options after an emitter has been selected.
  pub fn with_cache<'inp, L, C>(self, options: C::Options) -> WithEmitter<WithCache<'inp, P, L, C>, E>
  where
    L: Lexer<'inp>,
    C: Cache<'inp, L>,
  {
    WithEmitter {
      inner: WithCache {
        inner: self.inner,
        cache_opts: options,
        _marker: PhantomData,
      },
      emitter: self.emitter,
    }
  }
}

/// Parser configured with a concrete cache.
pub struct WithCache<'inp, P, L: Lexer<'inp>, C: Cache<'inp, L>> {
  inner: P,
  cache_opts: C::Options,
  _marker: PhantomData<fn() -> (&'inp L::Source, L, C)>,
}

impl<'inp, P, L, C> WithCache<'inp, P, L, C>
where
  L: Lexer<'inp>,
  C: Cache<'inp, L>,
{
  /// Attach an emitter after cache options have been selected.
  pub fn with_emitter<E>(self, emitter: E) -> WithEmitter<Self, E> {
    WithEmitter {
      inner: self,
      emitter,
    }
  }
}

/// Entry-point trait: run a parser against a source.
///
/// This provides the ergonomic `.parse()` API similar to Chumsky and
/// Winnow. Implementations wire up `Input`, `Emitter`, and `Cache`
/// before delegating to [`ParseInput`].
pub trait Parse<'inp, L, O, Error>: Sized {
  /// Parse using the lexer's default state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse(self, src: &'inp L::Source) -> ParseResult<O, Error>
  where
    L: Lexer<'inp>,
    L::State: Default,
  {
    self.parse_with_state(src, L::State::default())
  }

  /// Parse using an explicit lexer state.
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> ParseResult<O, Error>
  where
    L: Lexer<'inp>;
}
