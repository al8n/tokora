//! Blazing fast parser combinators with deterministic parsing and zero-copy streaming.
//!
//! This module provides a unique parser combinator framework that combines:
//!
//! 1. **Parse-While-Lexing Architecture**: Zero-copy streaming - tokens consumed directly from
//!    the lexer without buffering, eliminating allocation overhead
//! 2. **Deterministic LALR-Style Parsing**: Explicit lookahead with compile-time buffer capacity, no hidden backtracking
//! 3. **Flexible Error Handling**: Same parser adapts for fail-fast runtime ([`Fatal`](crate::emitter::Fatal))
//!    or greedy compiler diagnostics (via custom [`Emitter`](crate::Emitter))
//!
//! # Architecture
//!
//! Unlike traditional parser combinators that buffer all tokens and rely on implicit backtracking:
//!
//! **Traditional (Two-Phase)**:
//! ```text
//! Source → Lexer → [Vec<Token>] → Parser
//!                   ↑ Extra allocation!
//! ```
//!
//! **Tokit (Streaming)**:
//! ```text
//! Source → Lexer ←→ Parser
//!          ↑________↓
//!     Zero-copy, on-demand
//! ```
//!
//! Parsers pull tokens on-demand from the lexer. Only a small lookahead window (1-32 tokens)
//! is buffered on the stack for deterministic decisions.
//!
//! # Core Concepts
//!
//! ## Parse-While-Lexing
//!
//! Tokens flow directly from lexer to parser without intermediate buffering:
//! - **Zero extra allocations**: No `Vec<Token>` buffer
//! - **Lower memory**: Only lookahead window buffered on stack
//! - **Better cache locality**: Tokens processed immediately after lexing
//!
//! ## Deterministic Parsing (No Hidden Backtracking)
//!
//! Unlike traditional parser combinators with implicit backtracking, Tokit uses
//! **explicit lookahead-based decisions**:
//!
//! ```ignore
//! // Traditional: Hidden backtracking
//! let parser = try_parser1.or(try_parser2).or(try_parser3);
//!
//! // Tokit: Explicit lookahead, deterministic
//! let parser = any().peek_then::<_, typenum::U2>(|peeked, _| {
//!     match peeked.front() {
//!         Some(Token::If) => Ok(Action::Continue),  // Deterministic!
//!         _ => Ok(Action::Stop),
//!     }
//! });
//! ```
//!
//! The [`Window`] trait provides compile-time fixed lookahead capacity (`typenum::U1` to `typenum::U32`),
//! enabling LALR-style deterministic table parsing.
//!
//! ## Flexible Error Handling via Emitter
//!
//! The [`Emitter`](crate::Emitter) trait decouples parsing logic from error handling strategy:
//!
//! ```ignore
//! // Fail-fast for runtime/REPL (stop on first error)
//! let parser = Parser::with_context(FatalContext::new());
//! let result = parser.parse(source);  // Uses Fatal emitter
//!
//! // Custom greedy emitter for compiler diagnostics (collect all errors)
//! struct DiagnosticEmitter { errors: Vec<Error> }
//! impl Emitter for DiagnosticEmitter { /* collect errors */ }
//! ```
//!
//! **Same parser code, different behavior** - just swap the `Emitter` type.
//!
//! # Quick Start
//!
//! ```ignore
//! use tokit::{Any, Parse, Parser, parser::FatalContext};
//!
//! // 1. Parse any token
//! let parser = Any::parser::<'_, MyLexer<'_>, ()>();
//! let result = parser.parse(source);
//!
//! // 2. Chain combinators
//! let parser = Any::parser::<'_, MyLexer<'_>, ()>()
//!     .map(|tok| tok.kind())
//!     .filter(|kind| matches!(kind, TokenKind::Number));
//!
//! // 3. Explicit lookahead (deterministic choice)
//! let parser = Any::parser::<'_, MyLexer<'_>, ()>()
//!     .peek_then::<_, typenum::U1>(|peeked, _| {
//!         match peeked.get(0) {
//!             Some(tok) if tok.is_keyword("if") => Ok(Action::Continue),
//!             _ => Ok(Action::Stop),
//!         }
//!     });
//! ```
//!
//! # Available Combinators
//!
//! ## Basic Parsers
//!
//! - [`any`] - Accept any single token
//! - [`expect`] - Expect specific token, emit error if not found
//! - [`empty`] - No-op parser
//! - [`todo`] - Placeholder for incomplete implementations
//!
//! ## Sequencing
//!
//! - [`then`] - Sequential composition: parse `p1` then `p2`
//! - [`then_ignore`] - Parse both, keep only first result
//! - [`ignore_then`] - Parse both, keep only second result
//!
//! ## Repetition & Collections
//!
//! - [`repeated`] - Repeat until condition returns `Action::Stop`
//! - [`separated_by`](SeparatedBy) - Parse elements separated by delimiter
//! - [`delim`] - Parse delimited content (e.g., parentheses)
//! - [`delim_seq`] - Parse delimited, separated sequences
//!
//! ## Lookahead & Conditional (Deterministic)
//!
//! - [`peek_then`](PeekThen) - Peek ahead with fixed window, make deterministic decision
//! - [`peek_then_choice`](PeekThenChoice) - Choose between alternatives based on lookahead
//! - [`or_not`](OrNot) - Optional parsing
//!
//! ## Transformation
//!
//! - [`map`](Map) - Transform output
//! - [`filter`](Filter) - Filter with validation
//! - [`filter_map`](FilterMap) - Filter and transform
//! - [`validate`](Validate) - Validate with full location context
//!
//! ## Error Recovery
//!
//! - [`recover`](Recover) - Try parser, use recovery on error
//! - [`padded`](Padded) - Skip trivia (whitespace/comments) before and after
//!
//! # Performance Characteristics
//!
//! - **Memory**: O(1) - only small lookahead window on stack, no token buffering
//! - **Parsing**: O(n) - single-pass, deterministic, no backtracking
//! - **Lookahead**: O(1) - fixed compile-time capacity (1-32 tokens)
//!
//! # Design Priorities
//!
//! 1. **Performance**: Parse-while-lexing (zero-copy), no hidden allocations
//! 2. **Predictability**: No hidden backtracking, deterministic decisions
//! 3. **Composability**: Small parsers combine into complex grammars
//! 4. **Versatility**: Same parser for runtime (fail-fast) or compiler (greedy) via `Emitter`

#![allow(clippy::type_complexity)]

use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{
  Check, Emitter, Lexed, Lexer, Source, Token,
  emitter::{Fatal, FromEmitterError},
  error::{UnexpectedEot, token::UnexpectedToken},
  lexer::{Cursor, Input, InputRef, Peeked, PunctuatorToken},
  punct::Comma,
  utils::{
    Expected, Located, Sliced, Spanned,
    marker::{PhantomLocated, PhantomSliced, PhantomSpan},
  },
};

use derive_more::{IsVariant, TryUnwrap, Unwrap};
use generic_arraydeque::{ArrayLength, GenericArrayDeque, array::GenericArray, typenum};

pub use any::*;
pub use choice::*;
pub use collect::Collect;
pub use ctx::{FatalContext, ParseContext, ParserContext};
pub use delim::*;
pub use delim_seq::*;
pub use empty::*;
pub use expect::*;
pub use filter::*;
pub use filter_map::*;
pub use ignore::*;
pub use map::*;
pub use or_not::*;
pub use padded::*;
pub use peek_then::*;
pub use peek_then_choice::*;
pub use recover::*;
pub use repeated::*;
pub use sep::{SepFixSpec, SeparatedBy, SeparatedByOptions};
pub use then::*;
pub use todo::*;
pub use unwrapped::*;
pub use validate::*;

// #[cfg(any(feature = "std", feature = "alloc"))]
// #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
// pub use recursive::*;

mod any;
mod choice;
mod collect;
mod ctx;
mod delim;
mod delim_seq;
mod empty;
mod expect;
mod filter;
mod filter_map;
mod ignore;
mod map;
mod or_not;
mod padded;
mod peek_then;
mod peek_then_choice;
mod recover;
mod repeated;
mod sep;
mod then;
mod todo;
mod unwrapped;
mod validate;

#[cfg(any(feature = "std", feature = "alloc"))]
mod recursive;

mod sealed {
  pub trait Sealed {}
}

/// A trait for parsers that specify the capacity of their peek buffer.
pub trait Window: sealed::Sealed {
  /// The capacity of the peek buffer.
  type CAPACITY: ArrayLength;

  /// Create an uninitialized array of the specified capacity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn array<T>() -> GenericArray<MaybeUninit<T>, Self::CAPACITY> {
    GenericArray::uninit()
  }

  /// Create a deque of the specified capacity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deque<T>() -> GenericArrayDeque<MaybeUninit<T>, Self::CAPACITY> {
    GenericArrayDeque::new()
  }
}

macro_rules! peek_buf_capacity_impl_for_typenum {
  ($($size:literal), + $(,)?) => {
    paste::paste! {
      $(
        impl sealed::Sealed for typenum::[< U $size >] {}

        impl Window for typenum::[< U $size >] {
          type CAPACITY = typenum::[< U $size >];
        }
      )*
    }
  };
}

seq_macro::seq!(N in 1..=32 {
  peek_buf_capacity_impl_for_typenum! {
    #(N,)*
  }
});

/// Decision action for conditional parsing.
pub trait Decision<'inp, L, E, W, Lang: ?Sized = ()> {
  /// Decide the next action based on the peeked tokens.
  fn decide(&mut self, toks: Peeked<'_, 'inp, L, W>, emitter: &mut E) -> Result<Action, E::Error>
  where
    L: Lexer<'inp>,
    E: Emitter<'inp, L, Lang>,
    W: Window;
}

impl<'inp, F, L, E, W, Lang: ?Sized> Decision<'inp, L, E, W, Lang> for F
where
  F: FnMut(Peeked<'_, 'inp, L, W>, &mut E) -> Result<Action, E::Error>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  W: Window,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn decide(&mut self, toks: Peeked<'_, 'inp, L, W>, emitter: &mut E) -> Result<Action, E::Error>
  where
    W: Window,
  {
    (self)(toks, emitter)
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
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
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
    With::new(PhantomSpan::phantom(), self)
  }

  /// Wraps the output of this parser in a `Sliced` with the source slice of the parsed input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn sourced(self) -> With<PhantomSliced, Self>
  where
    Self: Sized,
  {
    With::new(PhantomSliced::phantom(), self)
  }

  /// Wraps the output of this parser in a `Located` with the span and source slice of the parsed input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn located(self) -> With<PhantomLocated, Self>
  where
    Self: Sized,
  {
    With::new(PhantomLocated::phantom(), self)
  }

  /// Ignores the output of this parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn ignored(self) -> Ignore<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
  {
    Ignore::new(self)
  }

  /// Creates a `Repeated` combinator that applies this parser repeatedly
  /// until the condition handler `Condition` returns [`RepeatedAction::End`] or an fatal error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn repeated<Condition, W>(
    self,
    condition: Condition,
  ) -> Repeated<Self, Condition, O, W, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W::CAPACITY>,
    W: Window,
  {
    Repeated::new(self, condition)
  }

  /// Creates a `SeparatedBy` combinator that applies this parser repeatedly,
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn separated_by<SepClassifier, Condition, W>(
    self,
    sep_classifier: SepClassifier,
    condition: Condition,
  ) -> SeparatedBy<Self, SepClassifier, Condition, O, W, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    SepClassifier: Check<L::Token>,
    W: Window,
  {
    SeparatedBy::new(self, sep_classifier, condition)
  }

  /// Creates a `SeparatedBy` combinator that applies this parser repeatedly,
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn separated_by_comma<Condition, W>(
    self,
    condition: Condition,
  ) -> SeparatedBy<Self, Comma, Condition, O, W, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    L::Token: PunctuatorToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    W: Window,
  {
    SeparatedBy::new(self, Comma::PHANTOM, condition)
  }

  /// Creates a `PeekThen` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `C` returns `Ok(())`, the inner parser is applied, otherwise,
  /// parsing is stopped and return the error from the handler.
  fn peek_then<C, W>(self, condition: C) -> PeekThen<Self, C, L::Token, W>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    C: FnMut(
      Peeked<'_, 'inp, L, W>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    W: Window,
    PeekThen<Self, C, L::Token, W>: ParseInput<'inp, L, O, Ctx, Lang>,
  {
    PeekThen::of(self, condition)
  }

  /// Creates a `PeekThen` combinator that peeks at most `N` tokens first from the input before parsing.
  ///
  /// If the condition handler `C` returns `Ok(Action::Continue)`, the inner parser is applied,
  /// otherwise returns `None`.
  #[doc(alias = "or_not")]
  fn peek_then_or_not<C, W>(self, condition: C) -> OrNot<PeekThen<Self, C, L::Token, W>>
  where
    Self: Sized,
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    C: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    W: Window,
    OrNot<PeekThen<Self, C, L::Token, W>>: ParseInput<'inp, L, Option<O>, Ctx, Lang>,
  {
    PeekThen::or_not_of(self, condition)
  }

  /// Map the output of this parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn map<U, F>(self, f: F) -> Map<Self, F, L, Ctx, O, U, Lang>
  where
    Self: Sized,
    F: FnMut(O) -> U,
  {
    Map::new(self, f)
  }

  /// Map the output of this parser using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn map_with<U, F>(self, f: F) -> MapWith<Self, F, L, Ctx, O, U, Lang>
  where
    Self: Sized,
    F: FnMut(O, ParseState<'_, 'inp, '_, L, Ctx, Lang>) -> U,
  {
    MapWith::new(self, f)
  }

  /// Filter the output of this parser using a validation function.
  ///
  /// The parser must produce a `Spanned<O>` value. The validator receives
  /// the data and span, and returns `Ok(())` if valid or an error otherwise.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn filter<F>(self, validator: F) -> Filter<Self, F, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(&O) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Filter::of(self, validator)
  }

  /// Filter the output of this parser using a validation function.
  ///
  /// The parser must produce a `Spanned<O>` value. The validator receives
  /// the data and span, and returns `Ok(())` if valid or an error otherwise.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn filter_with<F>(self, validator: F) -> FilterWith<Self, F, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(
      &O,
      ParseState<'_, 'inp, '_, L, Ctx, Lang>,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    FilterWith::of(self, validator)
  }

  /// Filter and map the output of this parser using a validation/transformation function.
  ///
  /// The parser must produce a `Spanned<O>` value. The mapper receives
  /// the data and span, and returns `Ok(new_value)` or an error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn filter_map<U, F>(self, mapper: F) -> FilterMap<Self, F, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(O) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    FilterMap::of(self, mapper)
  }

  /// Filter and map the output of this parser using a validation/transformation function.
  ///
  /// The parser must produce a `Spanned<O>` value. The mapper receives
  /// the data and span, and returns `Ok(new_value)` or an error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn filter_map_with<U, F>(self, mapper: F) -> FilterMapWith<Self, F, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(
      O,
      ParseState<'_, 'inp, '_, L, Ctx, Lang>,
    ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    FilterMapWith::of(self, mapper)
  }

  /// Validate the output of this parser with full location context.
  ///
  /// The parser must produce a `Located<O>` value. The validator receives
  /// the data, span, and slice, and returns `Ok(())` if valid or an error otherwise.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn validate<F>(self, validator: F) -> Validate<Self, F, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(&O) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Validate::of(self, validator)
  }

  /// Validate the output of this parser with full location context.
  ///
  /// The parser must produce a `Located<O>` value. The validator receives
  /// the data, span, and slice, and returns `Ok(())` if valid or an error otherwise.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn validate_with<F>(self, validator: F) -> ValidateWith<Self, F, O, L, Ctx, Lang>
  where
    Self: Sized,
    L: Lexer<'inp>,
    F: FnMut(
      &O,
      ParseState<'_, 'inp, '_, L, Ctx, Lang>,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    ValidateWith::of(self, validator)
  }

  /// Sequence this parser with another, ignoring the output of the second.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn then_ignore<G, U>(self, second: G) -> ThenIgnore<Self, G, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    G: ParseInput<'inp, L, U, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    ThenIgnore::new(self, second)
  }

  /// Sequence this parser with another, using the first result to determine the second parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn and_then<T, U>(self, then: T) -> AndThen<Self, T, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    T: ParseInput<'inp, L, U, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    AndThen::new(self, then)
  }

  /// Sequence this parser with another, using the first result to determine the second parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn and_then_with<T, U>(self, then: T) -> AndThenWith<Self, T, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    T: ParseInput<'inp, L, U, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    AndThenWith::new(self, then)
  }

  /// Sequence this parser with another, using the first result to determine the second parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn then<T, U>(self, then: T) -> Then<Self, T, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    T: ParseInput<'inp, L, U, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Then::new(self, then)
  }

  /// Sequence this parser with another, ignoring the output of the first.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn ignore_then<G, U>(self, second: G) -> IgnoreThen<Self, G, O, U, L, Ctx, Lang>
  where
    Self: Sized,
    G: ParseInput<'inp, L, U, Ctx, Lang>,
  {
    IgnoreThen::new(self, second)
  }

  /// Recover from errors produced by this parser using the given recovery parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn recover<R>(self, recovery: R) -> Recover<Self, R, O, L, Ctx, Lang>
  where
    Self: Sized,
    R: ParseInput<'inp, L, O, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Recover::new(self, recovery)
  }

  /// Recover in-place from errors produced by this parser using the given recovery parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn inplace_recover<R>(self, recovery: R) -> InplaceRecover<Self, R, O, L, Ctx, Lang>
  where
    Self: Sized,
    R: ParseInput<'inp, L, O, Ctx, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    InplaceRecover::new(self, recovery)
  }

  /// Creates a parser that accepts any token with optional padding.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn padded(self) -> Padded<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
  {
    Padded::new(self)
  }

  /// Creates a parser that accepts any token with optional padding.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn padded_left(self) -> PaddedLeft<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
  {
    PaddedLeft::new(self)
  }

  /// Creates a parser that accepts any token with optional padding.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn padded_right(self) -> PaddedRight<Self, O, L, Ctx, Lang>
  where
    Self: Sized,
  {
    PaddedRight::new(self)
  }
}

/// Extension trait for unwrapping `Option` outputs.
pub trait ParseInputUnwrapExt<'inp, L, O, Ctx, Lang: ?Sized> {
  /// Creates an `Unwrapped` parser that unwraps the `Option` result of this parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  fn unwrap(self) -> Unwrapped<Self, O, Ctx, Lang>
  where
    Self: Sized + ParseInput<'inp, L, Option<O>, Ctx, Lang>,
  {
    Unwrapped::new(self)
  }
}

impl<'inp, F, L, O, Ctx, Lang: ?Sized> ParseInputUnwrapExt<'inp, L, O, Ctx, Lang> for F
where
  F: ParseInput<'inp, L, Option<O>, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
}

impl<'inp, F, L, O, Ctx, Lang: ?Sized> ParseInput<'inp, L, O, Ctx, Lang> for F
where
  F: FnMut(
    &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
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
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
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
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
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
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
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
  Error: FromEmitterError<'inp, L>,
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
    Error: FromEmitterError<'inp, L>,
  {
    Self::of()
  }

  /// Creates a parser with the given context.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_context<'inp, L, O, Error, Ctx>(ctx: Ctx) -> Parser<(), L, O, Error, Ctx>
  where
    L: Lexer<'inp>,
    Error: FromEmitterError<'inp, L>,
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
    Error: FromEmitterError<'inp, L, Lang>,
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
    Error: FromEmitterError<'inp, L, Lang>,
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
    Error: FromEmitterError<'inp, L>,
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
    Error: FromEmitterError<'inp, L, Lang>,
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
    Error: FromEmitterError<'inp, L>,
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
    Error: FromEmitterError<'inp, L, Lang>,
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

/// A parsing state passed to parser functions.
pub struct ParseState<'a, 'inp, 'closure, L, Ctx, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  inp: &'a mut InputRef<'inp, 'closure, L, Ctx, Lang>,
  start: Cursor<'inp, 'closure, L>,
}

impl<'a, 'inp, 'closure, L, Ctx, Lang: ?Sized> ParseState<'a, 'inp, 'closure, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Create a new `ParseState`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new(
    inp: &'a mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    start: Cursor<'inp, 'closure, L>,
  ) -> Self {
    Self { inp, start }
  }

  /// Returns the span covering the output being parsed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span(&self) -> L::Span {
    self.inp.span_since(&self.start)
  }

  /// Returns a mutable reference to an emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn emitter(&mut self) -> &mut Ctx::Emitter {
    self.inp.emitter()
  }

  /// Returns the state of the lexer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn state(&self) -> &L::State {
    self.inp.state()
  }

  /// Returns the state of the lexer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn state_mut(&mut self) -> &mut L::State {
    self.inp.state_mut()
  }

  /// Returns the source slice covering the output being parsed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice(&self) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    self.inp.slice_since(&self.start)
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

  /// Maps the primary value using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map_primary<U, F>(self, f: F) -> With<U, S>
  where
    F: FnOnce(P) -> U,
  {
    With {
      primary: f(self.primary),
      secondary: self.secondary,
    }
  }

  /// Maps the secondary value using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map_secondary<U, F>(self, f: F) -> With<P, U>
  where
    F: FnOnce(S) -> U,
  {
    With {
      primary: self.primary,
      secondary: f(self.secondary),
    }
  }
}

/// A hint used during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Unwrap, TryUnwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Action {
  /// Indicates the token belongs to another syntactic element, hint to stop parsing.
  #[unwrap(ignore)]
  #[try_unwrap(ignore)]
  Stop,
  /// Indicates a token belongs to an element was found, hint to continue parsing.
  #[unwrap(ignore)]
  #[try_unwrap(ignore)]
  Continue,
}

impl Apply<Maximum> for () {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Maximum {
    Maximum(options)
  }
}

impl Apply<Minimum> for () {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Minimum {
    Minimum(options)
  }
}

impl Apply<Maximum> for Maximum {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Maximum {
    Maximum(options)
  }
}

impl Apply<Minimum> for Minimum {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Minimum {
    Minimum(options)
  }
}

/// A marker type representing the maximum number of elements allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Maximum(pub usize);

impl Maximum {
  /// The maximum possible value for `Maximum`.
  pub const MAX: Self = Self::new(usize::MAX);

  /// Creates a new `Maximum`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(n: usize) -> Self {
    Self(n)
  }

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

/// A marker type representing the minimum number of elements required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Minimum(usize);

impl Minimum {
  /// The minimum possible value for `Minimum`.
  pub const MIN: Self = Self::new(0);

  /// Creates a new `Minimum`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(n: usize) -> Self {
    Self(n)
  }

  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

trait MinSpec {
  fn minimum(&self) -> usize;
}

impl<T: MinSpec> MinSpec for &mut T {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    (**self).minimum()
  }
}

impl MinSpec for Minimum {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    self.0
  }
}

impl MinSpec for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    0
  }
}

trait MaxSpec {
  fn maximum(&self) -> usize;
}

impl<T: MaxSpec> MaxSpec for &mut T {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    (**self).maximum()
  }
}

impl MaxSpec for Maximum {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    self.0
  }
}

impl MaxSpec for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    usize::MAX
  }
}

/// The result of a parsing attempt.
pub enum ParseResult<O, E> {
  /// No output, no error, no consumption; the input was rewound to its original state.
  Rewind,
  /// Successful parse with output `O` and no emitted errors.
  Ok(O),
  /// Fatal parse failure with error `E`; caller should stop or propagate.
  Err(E),
}
