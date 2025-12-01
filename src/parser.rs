//! Parser combinators with flexible emitter and cache configuration.
//!
//! This module provides a type-safe parser combinator framework with:
//!
//! - **Flexible configuration**: Configure error emitters and caches independently
//! - **Type-level state tracking**: The type system ensures correct configurations
//! - **Zero-cost abstractions**: All configuration resolved at compile time
//!
//! # Quick Start
//!
//! ```ignore
//! use logosky::parser::any;
//!
//! // Parse with defaults
//! let result = any::<MyLexer, ()>().parse(source);
//!
//! // Configure emitter
//! let result = any::<MyLexer, ()>()
//!     .with_emitter(MyEmitter::new())
//!     .parse(source);
//!
//! // Full configuration
//! let result = any::<MyLexer, ()>()
//!     .with_emitter(MyEmitter::new())
//!     .with_cache::<MyCache>(cache_opts)
//!     .parse(source);
//! ```

#![allow(clippy::type_complexity)]

use core::{hash, marker::PhantomData};

use crate::{
  Cache, DefaultCache, Emitter, Lexed, Lexer, Token,
  emitter::Fatal,
  error::UnexpectedEot,
  lexer::{Input, InputRef},
  utils::{Expected, Spanned},
};

use derive_more::{From, IsVariant, TryUnwrap, Unwrap};

pub use any::*;
pub use collect::Collect;
pub use expect::*;
pub use map::*;
pub use sep::{SepFixSpec, SeqSep, SeqSepOptions};
pub use then::*;

mod any;
mod collect;
mod expect;
mod map;
mod sep;
mod then;

/// The result type returned by parsers.
pub type ParseResult<'inp, O, L, E> = Result<O, ParseError<'inp, L, E>>;

/// An error type returned by parsers.
#[derive(Debug, Clone, From, IsVariant, Unwrap, TryUnwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum ParseError<'inp, L, E>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
{
  /// Parser error encountered during parsing.
  #[from(skip)]
  Parser(E::Error),
  /// Lexer error encountered during lexing.
  #[from(skip)]
  Lexer(<L::Token as Token<'inp>>::Error),
  /// End of input reached unexpectedly.
  End(UnexpectedEot<L::Span>),
}

impl<'inp, L, E> PartialEq for ParseError<'inp, L, E>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  E::Error: PartialEq,
  <L::Token as Token<'inp>>::Error: PartialEq,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Parser(a), Self::Parser(b)) => a == b,
      (Self::Lexer(a), Self::Lexer(b)) => a == b,
      (Self::End(a), Self::End(b)) => a == b,
      _ => false,
    }
  }
}

impl<'inp, L, E> Eq for ParseError<'inp, L, E>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  E::Error: Eq,
  <L::Token as Token<'inp>>::Error: Eq,
{
}

impl<'inp, L, E> hash::Hash for ParseError<'inp, L, E>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  E::Error: hash::Hash,
  <L::Token as Token<'inp>>::Error: hash::Hash,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    match self {
      Self::Parser(v) => {
        0u8.hash(state);
        v.hash(state);
      }
      Self::Lexer(v) => {
        1u8.hash(state);
        v.hash(state);
      }
      Self::End(v) => {
        2u8.hash(state);
        v.hash(state);
      }
    }
  }
}

/// Core trait implemented by every parser combinator.
///
/// This mirrors the ergonomics of libraries like `winnow`: a parser is
/// simply something that can mutate an [`InputRef`] and either produce
/// a value or a spanned error using the configured `Emitter`.
pub trait ParseInput<'inp, L, O, E, C> {
  /// Try to parse from the given input.
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> Result<O, E::Error>
  where
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>;

  /// Map the output of this parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn map<U, F>(self, f: F) -> Map<Self, O, F>
  where
    Self: Sized,
    F: FnMut(O) -> U,
  {
    Map::new(self, f)
  }

  /// Sequence this parser with another, ignoring the output of the second.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn then_ignore<G, U>(self, second: G) -> ThenIgnore<Self, G, U>
  where
    Self: Sized,
    G: ParseInput<'inp, L, U, E, C>,
  {
    ThenIgnore::new(self, second)
  }

  /// Sequence this parser with another, using the first result to determine the second parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn then<T, U>(self, then: T) -> Then<Self, T>
  where
    Self: Sized,
    T: ParseInput<'inp, L, U, E, C>,
  {
    Then::new(self, then)
  }

  /// Sequence this parser with another, ignoring the output of the first.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn ignore_then<G, U>(self, second: G) -> IgnoreThen<Self, G, O>
  where
    Self: Sized,
    G: ParseInput<'inp, L, U, E, C>,
  {
    IgnoreThen::new(self, second)
  }
}

impl<'inp, F, L, O, E, C> ParseInput<'inp, L, O, E, C> for F
where
  F: FnMut(&mut InputRef<'inp, '_, L, E, C>) -> Result<O, E::Error>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> Result<O, E::Error> {
    (self)(input)
  }
}

// impl<'inp, F, L, O, Error, E, C, Em, Ca> ParseInput<'inp, L, O, Em, Ca>
//   for With<F, Parser<(), L, O, Error, ParserOptions<L, E, C>>>
// where
//   F: ParseInput<'inp, L, O, Em, Ca>,
//   L: Lexer<'inp>,
//   Em: Emitter<'inp, L>,
//   Ca: Cache<'inp, L>,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, Em, Ca>) -> O {
//     self.primary.parse_input(input)
//   }
// }

// impl<'inp, F, L, O, Error, E, C, Em, Ca> ParseInput<'inp, L, O, Em, Ca>
//   for Parser<F, L, O, Error, ParserOptions<L, E, C>>
// where
//   F: ParseInput<'inp, L, O, Em, Ca>,
//   L: Lexer<'inp>,
//   Em: Emitter<'inp, L>,
//   Ca: Cache<'inp, L>,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, Em, Ca>) -> O {
//     self.f.parse_input(input)
//   }
// }

/// Type alias for parser configuration.
///
/// Normalizes emitter and cache configuration into a canonical form:
/// `With<PhantomData<L>, With<E, C>>` where:
/// - `L` is the lexer type
/// - `E` is the emitter (default `()` for [`Fatal`] emitter)
/// - `C` is the cache (default `()` for [`DefaultCache`])
pub type ParserOptions<L, E = (), C = ()> = With<PhantomData<L>, With<E, C>>;

/// Wrapper for cache configuration in parsers.
///
/// Wraps a cache type `C` to distinguish it from bare `()` in type parameters,
/// preventing trait overlap in Parse implementations.
#[repr(transparent)]
pub struct WithCache<'inp, L, C> {
  cache: C,
  _marker: PhantomData<&'inp L>,
}

/// Wrapper for emitter configuration in parsers.
///
/// Wraps an emitter type `E` to distinguish it from bare `()` in type parameters,
/// preventing trait overlap in Parse implementations.
#[repr(transparent)]
pub struct WithEmitter<E: ?Sized>(E);

/// A parser with configurable emitter and cache.
///
/// # Type Parameters
///
/// - `F`: The parsing function
/// - `L`: The lexer type
/// - `O`: The output type
/// - `Error`: The error type
/// - `Options`: Configuration for emitter and cache (defaults to `ParserOptions<L>`)
///
/// # Examples
///
/// ```ignore
/// // Create parser with defaults
/// let p = Parser::with(|inp| inp.next());
///
/// // Configure emitter
/// let p = Parser::with(|inp| inp.next())
///     .with_emitter(MyEmitter::new());
/// ```
pub struct Parser<F, L, O, Error, Options = ParserOptions<L>> {
  f: F,
  opts: Options,
  _marker: PhantomData<(L, O, Error)>,
}

impl<F, L, O, Error> core::ops::Deref for Parser<F, L, O, Error> {
  type Target = F;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    &self.f
  }
}

impl<F, L, O, Error> core::ops::DerefMut for Parser<F, L, O, Error> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.f
  }
}

impl<L> Default for Parser<(), L, (), ()> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, O, Error> Parser<(), L, O, Error> {
  /// A parser without any behavior.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      f: (),
      opts: With::new(PhantomData, With::new((), ())),
      _marker: PhantomData,
    }
  }
}

impl<L, O, Error> Parser<(), L, O, Error> {
  /// A parser with the given parser
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with<F>(f: F) -> With<F, Self> {
    With::new(f, Self::new())
  }
}

impl<L, O, Error, E, C> Parser<(), L, O, Error, ParserOptions<L, E, C>> {
  // pub fn apply<F>(self, f: F) -> Parser<F, L, O, Error, ParserOptions<L, E, C>>
  // where
  //   F: ParseInput<'inp, L, O, E, C>,
  // {
  //   Parser {
  //     f,
  //     opts: self.opts,
  //     _marker: PhantomData,
  //   }
  // }

  /// Configure a custom error emitter for this parser.
  ///
  /// Replaces the current emitter configuration with a new one. The emitter
  /// controls how parsing errors are collected and reported.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let parser = Parser::with(|inp| inp.next())
  ///     .with_emitter(MyEmitter::new());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_emitter<'inp, NE>(
    self,
    emitter: NE,
  ) -> Parser<(), L, O, Error, ParserOptions<L, WithEmitter<NE>, C>>
  where
    E: Apply<WithEmitter<NE>, Options = NE>,
    L: Lexer<'inp>,
    NE: Emitter<'inp, L, Error = Error>,
  {
    Parser {
      f: (),
      opts: With::new(
        PhantomData,
        With::new(
          self.opts.secondary.primary.apply(emitter),
          self.opts.secondary.secondary,
        ),
      ),
      _marker: PhantomData,
    }
  }

  /// Configure a custom token cache for this parser.
  ///
  /// Replaces the current cache configuration with a new one. The cache
  /// controls how parsed tokens are stored for backtracking and lookahead.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let parser = Parser::with(|inp| inp.next())
  ///     .with_cache::<MyCache>(cache_options);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_cache<'inp, NC>(
    self,
    options: NC::Options,
  ) -> Parser<(), L, O, Error, ParserOptions<L, E, WithCache<'inp, L, NC>>>
  where
    C: Apply<WithCache<'inp, L, NC>, Options = NC::Options>,
    L: Lexer<'inp>,
    NC: Cache<'inp, L>,
  {
    Parser {
      f: (),
      opts: With::new(
        PhantomData,
        With::new(
          self.opts.secondary.primary,
          self.opts.secondary.secondary.apply(options),
        ),
      ),
      _marker: PhantomData,
    }
  }
}

impl<F, L, O, Error, E, C> With<F, Parser<(), L, O, Error, ParserOptions<L, E, C>>> {
  /// Convert a `With<F, Parser<()>>` back into a `Parser<F>`.
  ///
  /// This flattens the parser function into the parser, creating a fully
  /// configured parser ready to use.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let parser = Expect::parser(classifier).into_parser();
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_parser(self) -> Parser<F, L, O, Error, ParserOptions<L, E, C>> {
    Parser {
      f: self.primary,
      opts: self.secondary.opts,
      _marker: PhantomData,
    }
  }

  /// Apply a new emitter to the parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_emitter<'inp, NE>(
    self,
    emitter: NE,
  ) -> With<F, Parser<(), L, O, Error, ParserOptions<L, WithEmitter<NE>, C>>>
  where
    E: Apply<WithEmitter<NE>, Options = NE>,
    L: Lexer<'inp>,
    NE: Emitter<'inp, L, Error = Error>,
  {
    With::new(
      self.primary,
      Parser {
        f: self.secondary.f,
        opts: With::new(
          PhantomData,
          With::new(
            self.secondary.opts.secondary.primary.apply(emitter),
            self.secondary.opts.secondary.secondary,
          ),
        ),
        _marker: PhantomData,
      },
    )
  }

  /// Apply a new cache to the parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_cache<'inp, NC>(
    self,
    options: NC::Options,
  ) -> With<F, Parser<(), L, O, Error, ParserOptions<L, E, WithCache<'inp, L, NC>>>>
  where
    C: Apply<WithCache<'inp, L, NC>, Options = NC::Options>,
    L: Lexer<'inp>,
    NC: Cache<'inp, L>,
  {
    With::new(
      self.primary,
      Parser {
        f: self.secondary.f,
        opts: With::new(
          PhantomData,
          With::new(
            self.secondary.opts.secondary.primary,
            self.secondary.opts.secondary.secondary.apply(options),
          ),
        ),
        _marker: PhantomData,
      },
    )
  }
}

impl<'inp, L, C> Apply<WithCache<'inp, L, C>> for ()
where
  L: Lexer<'inp>,
  C: Cache<'inp, L>,
{
  type Options = C::Options;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> WithCache<'inp, L, C> {
    WithCache {
      cache: C::with_options(options),
      _marker: PhantomData,
    }
  }
}

impl<E> Apply<WithEmitter<E>> for () {
  type Options = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> WithEmitter<E> {
    WithEmitter(options)
  }
}

impl<'inp, L, O, E, Error, C> Parser<(), L, O, Error, ParserOptions<L, E, C>>
where
  L: Lexer<'inp>,
  E: EmitterProvider<'inp, L, Error>,
  E::Emitter: Emitter<'inp, L, Error = Error>,
  C: CacheProvider<'inp, L>,
  C::Cache: Cache<'inp, L>,
{
  /// Apply a new parsing function to the parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn apply<F>(self, f: F) -> Parser<F, L, O, Error, ParserOptions<L, E, C>> {
    Parser {
      f,
      opts: self.opts,
      _marker: PhantomData,
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
  fn parse(self, src: &'inp L::Source) -> Result<O, Error>
  where
    L: Lexer<'inp>,
    L::State: Default,
  {
    self.parse_with_state(src, L::State::default())
  }

  /// Parse using an explicit lexer state.
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> Result<O, Error>
  where
    L: Lexer<'inp>;
}

impl<'inp, F, L, O, Error, E, C> Parse<'inp, L, O, Error>
  for Parser<F, L, O, Error, ParserOptions<L, E, C>>
where
  F: ParseInput<'inp, L, O, E::Emitter, C::Cache>,
  L: Lexer<'inp>,
  E::Emitter: Emitter<'inp, L, Error = Error>,
  E: EmitterProvider<'inp, L, Error>,
  C::Cache: Cache<'inp, L>,
  C: CacheProvider<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> Result<O, Error> {
    let Parser {
      mut f,
      opts:
        With {
          secondary: With {
            primary: emitter,
            secondary: cache,
          },
          ..
        },
      ..
    } = self;

    let cache = cache.provide();
    let mut emitter = emitter.provide();
    let mut input = Input::with_state_and_cache(src, state, cache);
    let mut input_ref = input.as_ref(&mut emitter);
    f.parse_input(&mut input_ref)
  }
}

impl<'inp, F, L, O, Error, E, C> Parse<'inp, L, O, Error>
  for With<F, Parser<(), L, O, Error, ParserOptions<L, E, C>>>
where
  F: ParseInput<'inp, L, O, E::Emitter, C::Cache>,
  L: Lexer<'inp>,
  E::Emitter: Emitter<'inp, L, Error = Error>,
  E: EmitterProvider<'inp, L, Error>,
  C::Cache: Cache<'inp, L>,
  C: CacheProvider<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> Result<O, Error> {
    self.into_parser().parse_with_state(src, state)
  }
}

mod sealed_provider {
  use super::*;

  pub trait Sealed {}

  impl Sealed for () {}

  impl<L, C> Sealed for WithCache<'_, L, C> {}

  impl<E: ?Sized> Sealed for WithEmitter<E> {}
}

/// A provider for cache instances.
#[doc(hidden)]
pub trait CacheProvider<'inp, L>: sealed_provider::Sealed {
  /// The cache type provided.
  type Cache;

  /// Provide a cache instance.
  fn provide(self) -> Self::Cache
  where
    L: Lexer<'inp>,
    Self::Cache: Cache<'inp, L>;
}

impl<'inp, L> CacheProvider<'inp, L> for ()
where
  L: Lexer<'inp>,
{
  type Cache = DefaultCache<'inp, L>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn provide(self) -> Self::Cache
  where
    L: Lexer<'inp>,
  {
    DefaultCache::new()
  }
}

impl<'inp, L, C> CacheProvider<'inp, L> for WithCache<'inp, L, C> {
  type Cache = C;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn provide(self) -> Self::Cache
  where
    L: Lexer<'inp>,
    C: Cache<'inp, L>,
  {
    self.cache
  }
}

/// A provider for emitter instances.
#[doc(hidden)]
pub trait EmitterProvider<'inp, L, Error>: sealed_provider::Sealed {
  /// The emitter type provided.
  type Emitter;

  /// Provide an emitter instance.
  fn provide(self) -> Self::Emitter
  where
    L: Lexer<'inp>,
    Self::Emitter: Emitter<'inp, L, Error = Error>;
}

impl<'inp, L, Error> EmitterProvider<'inp, L, Error> for ()
where
  L: Lexer<'inp>,
{
  type Emitter = Fatal<Error>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn provide(self) -> Self::Emitter
  where
    L: Lexer<'inp>,
    Self::Emitter: Emitter<'inp, L, Error = Error>,
  {
    Fatal::new()
  }
}

impl<'inp, L, Error, E> EmitterProvider<'inp, L, Error> for WithEmitter<E>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Error = Error>,
{
  type Emitter = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn provide(self) -> Self::Emitter
  where
    L: Lexer<'inp>,
    Self::Emitter: Emitter<'inp, L, Error = Error>,
  {
    self.0
  }
}

/// Type-level function for configuration transformations.
///
/// This trait enables progressive parser configuration by transforming
/// one configuration type into another. For example:
///
/// - `()` → `WithEmitter<E>` (add emitter configuration)
/// - `()` → `WithCache<C>` (add cache configuration)
///
/// Used internally by `.with_emitter()` and `.with_cache()` methods.
pub trait Apply<State> {
  /// The input required to perform the transformation
  type Options;

  /// Transform `self` into `State` using the provided `options`.
  fn apply(self, options: Self::Options) -> State;
}

/// Shorthand for building a [`Parser`] from a closure.
pub const fn parser<'inp, L, O, E, C, F>(f: F) -> With<F, Parser<(), L, O, E::Error>>
where
  F: FnMut(&mut InputRef<'inp, '_, L, E, C>) -> O,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  Parser::with(f)
}

/// Combines two values in a type-safe way.
///
/// This type is used throughout the parser system for:
///
/// - Wrapping parser functions with base parsers: `With<F, Parser<()>>`
/// - Building configuration structures: `With<E, C>` for emitter + cache
/// - Nested configurations: `With<PhantomData<L>, With<E, C>>` for ParserOptions
///
/// # Type Parameters
///
/// - `P`: The primary value (typically a parser function or marker)
/// - `S`: The secondary value (typically configuration or a base parser)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct With<P, S> {
  primary: P,
  secondary: S,
}

impl<P, S> With<P, S> {
  /// Create a new `With` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(primary: P, secondary: S) -> Self {
    Self { primary, secondary }
  }

  /// Returns a reference to the primary.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn primary(&self) -> &P {
    &self.primary
  }

  /// Returns a reference to the secondary.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn secondary(&self) -> &S {
    &self.secondary
  }

  /// Returns a mutable reference to the primary.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn primary_mut(&mut self) -> &mut P {
    &mut self.primary
  }

  /// Returns a mutable reference to the secondary.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn secondary_mut(&mut self) -> &mut S {
    &mut self.secondary
  }
}

/// A hint used during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Unwrap, TryUnwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Action<'a, Kind> {
  /// Indicates the token belongs to another syntactic element, hint to end parsing.
  #[unwrap(ignore)]
  #[try_unwrap(ignore)]
  End,
  /// Indicates a token belongs to an element was found, hint to continue parsing.
  #[unwrap(ignore)]
  #[try_unwrap(ignore)]
  Continue,
  /// Indicates that we should skip the token, useful for trivial tokens like whitespace, comments, etc.
  #[unwrap(ignore)]
  #[try_unwrap(ignore)]
  Skip,
  /// Indicates this is an unexpected token, but this token should not terminate the parsing,
  /// the unexpected token will be emitted to the emitter.
  Unexpected(Option<Expected<'a, Kind>>),
}

#[cfg(test)]
mod tests {
  #![allow(warnings)]

  use super::{Token as TokenT, *};
  use crate::{BlackHole, emitter::Fatal, punct::Comma, utils::marker::Ignored};
  use derive_more::Display;

  // fn assert_any_parse_impl<'inp>() -> impl Parse<'inp, JsonLexer<'inp>, Result<Token, ()>, ()> {
  //   any()
  // }

  // fn assert_comma_seq_parse_impl<'inp>()
  // -> impl Parse<'inp, JsonLexer<'inp>, Result<(), ()>, ()> {
  //   Parser::new()
  //     .with_cache::<()>(())
  //     .with_emitter(Fatal::new())
  //     .apply(
  //       comma_seq::<_, _, JsonLexer<'inp>, Token, (), Fatal<()>, ()>(any(), |t: &Token| {
  //         if let TokenKind::Comma = t.kind() {
  //           SeqSepAction::Separator
  //         } else {
  //           SeqSepAction::Continue
  //         }
  //       }),
  //     )
  // }

  // #[test]
  // fn t() {
  //   let src = "{}";

  //   let tok = Parser::any::<JsonLexer<'_>, ()>().parse(src);
  //   let a = Parse::parse(Parser::comma_seq::<'_, _, JsonLexer<'_>, Option<Spanned<Lexed<'_, Token>>>, (), ()>(Parser::any()), src);
  // }
}
