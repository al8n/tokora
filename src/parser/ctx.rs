//! Parse context trait and implementations.
//!
//! A parse context bundles the error emitter and cache configuration
//! used during parsing. This provides better type inference and a
//! simpler API compared to configuring emitter and cache separately.

use core::marker::PhantomData;

use crate::{BlackHole, Cache, DefaultCache, Emitter, InputContext, Lexer, emitter::Fatal};

/// A context that provides emitter and cache configuration for parsing.
pub trait ParseContext<'inp, L, Lang: ?Sized = ()> {
  /// The emitter type used for error handling.
  type Emitter: Emitter<'inp, L, Lang>
  where
    L: Lexer<'inp>;

  /// The cache type used for lookahead.
  type Cache: Cache<'inp, L>
  where
    L: Lexer<'inp>;

  /// Provides the emitter and cache instances for parsing.
  fn provide(self) -> InputContext<Self::Emitter, Self::Cache>
  where
    L: Lexer<'inp>;
}

impl<'inp, L, Lang: ?Sized> ParseContext<'inp, L, Lang> for ()
where
  L: Lexer<'inp>,
  Fatal<(), Lang>: Emitter<'inp, L, Lang>,
{
  type Emitter = Fatal<(), Lang>;
  type Cache = DefaultCache<'inp, L>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn provide(self) -> InputContext<Self::Emitter, Self::Cache>
  where
    L: Lexer<'inp>,
  {
    InputContext::new(Fatal::of(), DefaultCache::<'inp, L>::new())
  }
}

impl<'inp, L, Lang: ?Sized> ParseContext<'inp, L, Lang> for BlackHole
where
  L: Lexer<'inp>,
  Fatal<(), Lang>: Emitter<'inp, L, Lang>,
{
  type Emitter = Fatal<(), Lang>;
  type Cache = Self;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn provide(self) -> InputContext<Self::Emitter, Self::Cache>
  where
    L: Lexer<'inp>,
  {
    InputContext::new(Fatal::of(), Self)
  }
}

/// Custom context: use a custom emitter and cache pair.
impl<'inp, L, E, C, Lang: ?Sized> ParseContext<'inp, L, Lang> for (E, C)
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  C: Cache<'inp, L>,
{
  type Emitter = E;
  type Cache = C;

  fn provide(self) -> InputContext<Self::Emitter, Self::Cache>
  where
    L: Lexer<'inp>,
  {
    InputContext::new(self.0, self.1)
  }
}

/// Convenient type alias for the default parse context.
pub type FatalContext<'inp, L, Error, Lang = ()> =
  ParserContext<'inp, L, Fatal<Error, Lang>, DefaultCache<'inp, L>, Lang>;

/// A concrete [`ParseContext`] implementation that holds an emitter and optional cache options.
pub struct ParserContext<'inp, L, E, C, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  C: Cache<'inp, L>,
{
  emitter: E,
  cache: Option<C::Options>,
  _marker: PhantomData<&'inp L>,
  _lang: PhantomData<Lang>,
}

impl<'inp, L, E, C> ParserContext<'inp, L, E, C>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  /// Creates a new parser context with the given emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(emitter: E) -> Self {
    Self::of(emitter)
  }

  /// Creates a new parser context with the given emitter and cache options.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_cache_options(emitter: E, options: C::Options) -> Self {
    Self::with_cache_options_of(emitter, options)
  }
}

impl<'inp, L, E, C, Lang: ?Sized> ParserContext<'inp, L, E, C, Lang>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  C: Cache<'inp, L>,
{
  const fn new_in(emitter: E, opts: Option<C::Options>) -> Self {
    Self {
      emitter,
      cache: opts,
      _marker: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Creates a new parser context with the given emitter for a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(emitter: E) -> Self {
    Self::new_in(emitter, None)
  }

  /// Creates a new parser context with the given emitter and cache options for a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_cache_options_of(emitter: E, options: C::Options) -> Self {
    Self::new_in(emitter, Some(options))
  }
}

impl<'inp, L, E, C, Lang: ?Sized> ParseContext<'inp, L, Lang> for ParserContext<'inp, L, E, C, Lang>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  C: Cache<'inp, L>,
{
  type Emitter = E;
  type Cache = C;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn provide(self) -> InputContext<Self::Emitter, Self::Cache>
  where
    L: Lexer<'inp>,
  {
    match self.cache {
      Some(options) => InputContext::new(self.emitter, C::with_options(options)),
      None => InputContext::new(self.emitter, C::new()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::DummyLexer;

  #[test]
  fn test_default_context() {
    fn assert_context<'inp, Ctx>()
    where
      Ctx: ParseContext<'inp, DummyLexer>,
    {
    }

    assert_context::<()>();
    assert_context::<FatalContext<'_, DummyLexer, ()>>();
  }

  #[test]
  fn test_custom_context() {
    fn assert_context<'inp, Ctx>()
    where
      Ctx: ParseContext<'inp, DummyLexer>,
    {
    }

    assert_context::<(Fatal<()>, DefaultCache<'_, DummyLexer>)>();
  }
}
