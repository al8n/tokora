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
  CachedToken, Emitter, Lexed, Lexer, Source, Token,
  emitter::Fatal,
  error::{UnexpectedEot, token::UnexpectedToken},
  lexer::{Input, InputRef},
  utils::{
    Expected, Located, Sliced, Spanned,
    marker::{PhantomLocated, PhantomSliced, PhantomSpan},
  },
};

use derive_more::{From, IsVariant, TryUnwrap, Unwrap};

pub use any::*;
pub use choice::*;
pub use collect::Collect;
pub use ctx::{FatalContext, ParseContext, ParserContext};
pub use expect::*;
pub use map::*;
use mayber::MaybeRef;
pub use or_not::*;
pub use peek_then::*;
pub use peek_then_choice::*;
pub use sep::{SepFixSpec, SeqSep, SeqSepOptions};
pub use then::*;

mod any;
mod choice;
mod collect;
mod ctx;
mod expect;
mod map;
mod or_not;
mod peek_then;
mod peek_then_choice;
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
pub trait ParseInput<'inp, L, O, Ctx, Lang: ?Sized = ()> {
  /// Try to parse from the given input.
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>;

  /// Wraps the output of this parser in a `Spanned` with the span of the parsed input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn spanned(self) -> With<PhantomSpan, Self>
  where
    Self: Sized,
  {
    With::new(PhantomSpan::PHANTOM, self)
  }

  /// Wraps the output of this parser in a `Sliced` with the source slice of the parsed input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn sourced(self) -> With<PhantomSliced, Self>
  where
    Self: Sized,
  {
    With::new(PhantomSliced::PHANTOM, self)
  }

  /// Wraps the output of this parser in a `Located` with the span and source slice of the parsed input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn located(self) -> With<PhantomLocated, Self>
  where
    Self: Sized,
  {
    With::new(PhantomLocated::PHANTOM, self)
  }

  /// Creates a `PeekThen` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `C` returns `Ok(())`, the inner parser is applied, otherwise,
  /// parsing is stopped and return the error from the handler.
  fn peek_then<C, const N: usize>(self, condition: C) -> PeekThen<Self, C, L::Token, N>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    C: FnMut(
      &mut [MaybeRef<'_, CachedToken<'_, L>>],
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    PeekThen::of(self, condition)
  }

  /// Creates a `PeekThen` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `C` returns `Ok(())`, the inner parser is applied, otherwise,
  /// parsing is stopped and return the error from the handler.
  fn peek_then_or_not<C, const N: usize>(
    self,
    condition: C,
  ) -> OrNot<PeekThen<Self, C, L::Token, N>>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    C: FnMut(
      &mut [MaybeRef<'_, CachedToken<'_, L>>],
      &mut Ctx::Emitter,
    ) -> Result<bool, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    PeekThen::or_not_of(self, condition)
  }

  /// Creates a `PeekThenChoice` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `H` returns `Ok(id)`, the inner choice parser is applied with the given id, otherwise,
  /// parsing is stopped and return the error from the handler.
  fn peek_then_choice<H, const N: usize>(self, condition: H) -> PeekThenChoice<Self, H, L::Token, N>
  where
    Self: Sized + ParseChoice<'inp, L, O, Ctx, Lang>,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    H: FnMut(
      &mut [MaybeRef<'_, CachedToken<'_, L>>],
      &mut Ctx::Emitter,
    ) -> Result<Self::Id, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    PeekThenChoice::of(self, condition)
  }

  /// Creates a `PeekThenChoice` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `H` returns `Ok(id)`, the inner choice parser is applied with the given id, otherwise,
  /// parsing is stopped and return the error from the handler.
  fn peek_then_choice_or_not<H, const N: usize>(
    self,
    condition: H,
  ) -> OrNot<PeekThenChoice<Self, H, L::Token, N>>
  where
    Self: Sized + ParseChoice<'inp, L, O, Ctx, Lang>,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    H: FnMut(
      &mut [MaybeRef<'_, CachedToken<'_, L>>],
      &mut Ctx::Emitter,
    ) -> Result<Option<Self::Id>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    PeekThenChoice::or_not_of(self, condition)
  }

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
    G: ParseInput<'inp, L, U, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    ThenIgnore::new(self, second)
  }

  /// Sequence this parser with another, using the first result to determine the second parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn then<T, U>(self, then: T) -> Then<Self, T>
  where
    Self: Sized,
    T: ParseInput<'inp, L, U, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Then::new(self, then)
  }

  /// Sequence this parser with another, ignoring the output of the first.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn ignore_then<G, U>(self, second: G) -> IgnoreThen<Self, G, O>
  where
    Self: Sized,
    G: ParseInput<'inp, L, U, Ctx, Lang>,
  {
    IgnoreThen::new(self, second)
  }
}

impl<'inp, F, L, O, Ctx, Lang: ?Sized> ParseInput<'inp, L, O, Ctx, Lang> for F
where
  F: FnMut(
    &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    (self)(input)
  }
}

impl<'inp, L, O, Ctx, P, Lang: ?Sized> ParseInput<'inp, L, Spanned<O, L::Span>, Ctx, Lang>
  for With<PhantomSpan, P>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<Spanned<O, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let cursor = inp.cursor().clone();
    self
      .secondary
      .parse_input(inp)
      .map(|output| Spanned::new(inp.span_since(&cursor), output))
  }
}

impl<'inp, L, O, Ctx, P, Lang: ?Sized>
  ParseInput<'inp, L, Sliced<O, <L::Source as Source<L::Offset>>::Slice<'inp>>, Ctx, Lang>
  for With<PhantomSliced, P>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<
    Sliced<O, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    let cursor = inp.cursor().clone();
    self.secondary.parse_input(inp).map(|output| {
      Sliced::new(
        inp
          .slice_since(&cursor)
          .expect("parser should guarantee slice"),
        output,
      )
    })
  }
}

impl<'inp, L, O, Ctx, P, Lang: ?Sized>
  ParseInput<'inp, L, Located<O, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>, Ctx, Lang>
  for With<PhantomLocated, P>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<
    Located<O, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    let cursor = inp.cursor().clone();
    self.secondary.parse_input(inp).map(|output| {
      Located::new(
        inp
          .slice_since(&cursor)
          .expect("parser should guarantee slice"),
        inp.span_since(&cursor),
        output,
      )
    })
  }
}

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
pub struct Parser<F, L, O, Error, Context> {
  f: F,
  ctx: Context,
  _marker: PhantomData<(L, O, Error)>,
}

impl<F, L, O, Error, Context> core::ops::Deref for Parser<F, L, O, Error, Context> {
  type Target = F;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    &self.f
  }
}

impl<F, L, O, Error, Context> core::ops::DerefMut for Parser<F, L, O, Error, Context> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.f
  }
}

impl<'inp, L, O, Error> Default for Parser<(), L, O, Error, FatalContext<'inp, L, Error>>
where
  L: Lexer<'inp>,
  Error: From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>
    + From<UnexpectedEot<L::Span>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Parser::new()
  }
}

impl Parser<(), (), (), (), ()> {
  /// A parser without any behavior.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new<'inp, L, O, Error>() -> Parser<(), L, O, Error, FatalContext<'inp, L, Error>>
  where
    L: Lexer<'inp>,
    Error: From<<L::Token as Token<'inp>>::Error>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>
      + From<UnexpectedEot<L::Span>>,
  {
    Self::of()
  }

  /// Creates a parser with the given context.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_context<'inp, L, O, Error, Ctx>(ctx: Ctx) -> Parser<(), L, O, Error, Ctx>
  where
    L: Lexer<'inp>,
    Error: From<<L::Token as Token<'inp>>::Error>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>
      + From<UnexpectedEot<L::Span>>,
    Ctx: ParseContext<'inp, L>,
    Ctx::Emitter: Emitter<'inp, L, Error = Error>,
  {
    Self::with_context_of(ctx)
  }

  /// A parser without any behavior.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of<'inp, L, O, Error, Lang>()
  -> Parser<(), L, O, Error, FatalContext<'inp, L, Error, Lang>>
  where
    L: Lexer<'inp>,
    Error: From<<L::Token as Token<'inp>>::Error>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
      + From<UnexpectedEot<L::Span, Lang>>,
    Lang: ?Sized,
  {
    Self::with_context_of(FatalContext::of(Fatal::of()))
  }

  /// Creates a parser with the given context for a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_context_of<'inp, L, O, Error, Ctx, Lang>(
    ctx: Ctx,
  ) -> Parser<(), L, O, Error, Ctx>
  where
    L: Lexer<'inp>,
    Error: From<<L::Token as Token<'inp>>::Error>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
      + From<UnexpectedEot<L::Span, Lang>>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: Emitter<'inp, L, Lang, Error = Error>,
    Lang: ?Sized,
  {
    Parser {
      f: (),
      ctx,
      _marker: PhantomData,
    }
  }

  /// Creates a parser with a parser function and the fatal context.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_parser<'inp, L, O, Error, F>(
    f: F,
  ) -> Parser<F, L, O, Error, FatalContext<'inp, L, Error>>
  where
    L: Lexer<'inp>,
    F: ParseInput<'inp, L, O, FatalContext<'inp, L, Error>>,
    Error: From<<L::Token as Token<'inp>>::Error>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>
      + From<UnexpectedEot<L::Span>>,
  {
    Self::with_parser_of(f)
  }

  /// Creates a parser with a parser function and the fatal context for a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_parser_of<'inp, L, O, Error, F, Lang>(
    f: F,
  ) -> Parser<F, L, O, Error, FatalContext<'inp, L, Error, Lang>>
  where
    L: Lexer<'inp>,
    F: ParseInput<'inp, L, O, FatalContext<'inp, L, Error, Lang>>,
    Error: From<<L::Token as Token<'inp>>::Error>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
      + From<UnexpectedEot<L::Span, Lang>>,
    Lang: ?Sized,
  {
    Self::with_parser_and_context_of(f, FatalContext::of(Fatal::of()))
  }

  /// Creates a parser with a parser function and the fatal context.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_parser_and_context<'inp, L, O, Error, Ctx, F>(
    f: F,
    ctx: Ctx,
  ) -> Parser<F, L, O, Error, Ctx>
  where
    L: Lexer<'inp>,
    F: ParseInput<'inp, L, O, Ctx>,
    Ctx: ParseContext<'inp, L>,
    Error: From<<L::Token as Token<'inp>>::Error>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>
      + From<UnexpectedEot<L::Span>>,
  {
    Self::with_parser_and_context_of(f, ctx)
  }

  /// Creates a parser with a parser function and the fatal context for a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_parser_and_context_of<'inp, L, O, Error, Ctx, F, Lang>(
    f: F,
    ctx: Ctx,
  ) -> Parser<F, L, O, Error, Ctx>
  where
    L: Lexer<'inp>,
    F: ParseInput<'inp, L, O, Ctx>,
    Ctx: ParseContext<'inp, L, Lang>,
    Error: From<<L::Token as Token<'inp>>::Error>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
      + From<UnexpectedEot<L::Span, Lang>>,
    Lang: ?Sized,
  {
    Parser {
      f,
      ctx,
      _marker: PhantomData,
    }
  }
}

impl<'inp, L, O, Error, Ctx> Parser<(), L, O, Error, Ctx>
where
  L: Lexer<'inp>,
{
  /// Apply a new parsing function to the parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn apply<F>(self, f: F) -> Parser<F, L, O, Error, Ctx>
  where
    Ctx: ParseContext<'inp, L>,
    F: ParseInput<'inp, L, O, Ctx>,
  {
    self.apply_of(f)
  }

  /// Apply a new parsing function to the parser for a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn apply_of<F, Lang>(self, f: F) -> Parser<F, L, O, Error, Ctx>
  where
    Ctx: ParseContext<'inp, L, Lang>,
    F: ParseInput<'inp, L, O, Ctx>,
  {
    Parser {
      f,
      ctx: self.ctx,
      _marker: PhantomData,
    }
  }
}

/// Entry-point trait: run a parser against a source.
///
/// This provides the ergonomic `.parse()` API similar to Chumsky and
/// Winnow. Implementations wire up `Input`, `Emitter`, and `Cache`
/// before delegating to [`ParseInput`].
pub trait Parse<'inp, L, O, Error, Lang: ?Sized = ()>: Sized {
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

impl<'inp, F, L, O, Error, Ctx, Lang: ?Sized> Parse<'inp, L, O, Error, Lang>
  for Parser<F, L, O, Error, Ctx>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: Emitter<'inp, L, Lang, Error = Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> Result<O, Error> {
    let Parser { mut f, ctx, .. } = self;

    let (mut emitter, cache) = ctx.provide().into_components();
    let mut input = Input::with_state_and_cache(src, state, cache);
    let mut input_ref = input.as_ref(&mut emitter);
    f.parse_input(&mut input_ref)
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
