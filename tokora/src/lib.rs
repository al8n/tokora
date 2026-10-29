#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(clippy::double_parens, clippy::type_complexity)]
#![deny(missing_docs, warnings)]
// With `unstable-raw` off, `InputRef::{save, restore, commit}` are `pub(crate)`, so the many
// public items documenting the raw checkpoint contract (the `Checkpoint` type, the transaction
// guards, `ParseState`, `attempt`) link to crate-private methods. Those links are intentionally
// inert in that build and fully live under the feature (and on docs.rs, which builds all
// features); relax the lint only when the feature is off so the on-feature docs stay strict.
#![cfg_attr(not(feature = "unstable-raw"), allow(rustdoc::private_intra_doc_links))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

// Declared before the combinator modules so the internal `trace_event!` hook macro is in
// scope for them. With the `trace` feature off the macro expands to nothing, so every hook
// site compiles away entirely.
#[macro_use]
mod trace;

// `tokora::logos` resolves to the one supported logos version, 0.16, so a derive written
// against this alias compiles under it. The plain `logos` feature enables `logos_0_16`, so
// `--features logos` keeps meaning exactly what it always meant.
//
// Same re-export as the adapter's in `lexer/logos/mod.rs`; keep the two in step, since a derive
// resolving to a different logos than the `LogosLexer` it is driven through will not compile.
#[cfg(feature = "logos_0_16")]
#[cfg_attr(docsrs, doc(cfg(feature = "logos_0_16")))]
pub use logos_0_16 as logos;

// The versioned name of the same crate (#350, rule 6): a consumer pinned to logos 0.16 keeps
// writing `#[logos(crate = tokora::logos_0_16)]` after a newer integration lands and the
// unversioned `tokora::logos` moves on to it.
#[cfg(feature = "logos_0_16")]
#[cfg_attr(docsrs, doc(cfg(feature = "logos_0_16")))]
pub use logos_0_16;

// The crate root is an explicit list, by one criterion: a name a production must have in scope —
// the traits whose methods it calls, the types its signature spells, and the drivers. Every item
// here also lives in the module of its concept, and nothing else is re-exported into the root;
// in particular no glob is, so adding a root name is a deliberate edit to this list (and to the
// census's frozen `ci/layout/root.txt`).
pub use emitter::{Emitter, EmitterView};
pub use input::InputRef;
pub use lexer::{Lexer, ReadFrontier, ScanLookahead};
pub use parse::{
  Accumulator, Action, ComposableParseContext, ErrorOf, FatalContext, Parse, ParseAttempt,
  ParseChoice, ParseContext, ParseInput, ParseTokenChoice, Parser, ParserContext, TryParseInput,
  parse, parse_partial, parse_with, parse_with_state,
};
pub use slice::Slice;
pub use source::Source;
pub use span::{SimpleSpan, Span};
pub use token::Token;

// Each module is documented in exactly one place. A module whose file opens with an inner `//!`
// header carries no outer `///` comment here, because rustdoc resolves the links of a module
// documented from both sides differently per toolchain: stable 1.99 resolves the merged fragments
// from the crate root when the first fragment is outer, while nightly resolves the inner header
// in the module itself, so a header that links correctly on one breaks on the other. With the
// header alone, every link in it resolves in the module's own scope on both: a name the module
// defines or imports is written bare, anything else crate-absolute. (Spelling a header's links
// crate-absolute to suit the root scope instead trades an unresolved link for a redundant one, and
// ties the correct spelling to the root's re-export list.) The modules below that do carry an
// outer comment have no inner header; their links resolve from the crate root on both toolchains.
pub mod cst;

/// Lexical analysis and token extraction.
///
/// Contains the [`Lexer`] trait and [`Lexed`](lexer::Lexed) type for converting source text into
/// tokens. Tokens flow on demand to parsers and are cached only when explicit lookahead or
/// backtracking needs them.
pub mod lexer;

pub mod combinator;

pub mod parse;

// The Pratt subsystem: one module behind one feature (#350, rule 4).
#[cfg(feature = "pratt")]
#[cfg_attr(docsrs, doc(cfg(feature = "pratt")))]
pub mod pratt;

pub mod types;

pub mod syntax;

/// Utility types and helpers for lexing and parsing.
///
/// Contains common utilities including:
/// - Generic array deque and type-level numbers (re-exported from `hybrid-arraydeque`)
/// - Delimited and escaped sequence helpers
/// - Display traits for human-readable, SDL, and syntax tree output
/// - Positioned character iterators
/// - Message and knowledge types for error reporting
pub mod utils;

/// Container trait for accumulating parsed results.
///
/// Defines the [`Container`](container::Container) trait for types that can accumulate
/// parsing results. Implemented for standard collections like `Vec`, arrays, and
/// `ArrayDeque`, enabling parsers to collect multiple elements into containers.
pub mod container;

/// Atomically composable error handling and reporting.
///
/// Provides the [`Emitter`] trait and related traits for flexible error handling during
/// parsing. The atomic design allows implementing only needed traits for specific use
/// cases. Includes pre-built emitters:
/// - [`Fatal`](emitter::Fatal): Fail-fast on first error (for runtime/REPL)
#[cfg_attr(
  any(feature = "std", feature = "alloc"),
  doc = " - [`Verbose`](emitter::Verbose): Collect all errors (for compiler diagnostics)"
)]
#[cfg_attr(
  not(any(feature = "std", feature = "alloc")),
  doc = " - `Verbose`: Collect all errors (for compiler diagnostics)"
)]
/// - [`Silent`](emitter::Silent): Suppress errors (for speculative parsing)
pub mod emitter;

/// Comprehensive error types for lexer and parser diagnostics.
///
/// Contains detailed error types organized by category:
/// - **Token errors**: Unexpected tokens, missing/extra separators
/// - **Lexer errors**: Unknown lexemes, malformed literals, invalid escape sequences
/// - **Syntax errors**: Incomplete syntax, too few/many elements, container overflow
/// - **Delimiter errors**: Unclosed/unopened/undelimited constructs
///
/// All errors carry span information for precise diagnostic reporting.
pub mod error;

pub mod diagnostic;

/// Macro for defining punctuator types.
///
/// Provides the [`punctuator!`] macro for generating zero-sized punctuator types with
/// span tracking. Punctuators are generic over span and source types, enabling both
/// phantom (zero-size) and concrete instances for use in ASTs.
pub mod punct;

/// Delimiter types and utilities.
///
/// Defines common delimiter types (brackets, braces, parentheses) and utilities
/// for working with delimited constructs in parsing.
pub mod delimiter;

/// The dialect anchor: one type per language, naming its lexer and its brand.
///
/// A grammar's productions all need the same block of lexer equalities — source, slice, token,
/// span, offset, token capabilities — and Rust cannot name that block once. The
/// [`Dialect`](dialect::Dialect) trait is the anchor it hangs off: a consumer writes one impl per
/// language, pins the projections in a one-line subtrait, and each production then carries two
/// `where`-clause lines. The [`LexerOf`](dialect::LexerOf) / [`LangOf`](dialect::LangOf) /
/// [`DialectSlice`](dialect::DialectSlice) / [`DialectInput`](dialect::DialectInput) /
/// [`DialectErrorOf`](dialect::DialectErrorOf) aliases spell the projections a signature actually
/// mentions.
pub mod dialect;

/// Source text abstraction for lexers.
///
/// Defines the [`Source`] trait for accessing source text during lexing.
/// Supports both string (`&str`) and byte (`&[u8]`) sources with proper boundary
/// checking. Handles UTF-8 character boundaries for string sources and byte boundaries
/// for binary sources.
pub mod source;

/// Source location tracking and span types.
///
/// Defines the [`span::Span`] trait for representing source code ranges with
/// start and end offsets. Provides operations for creating, manipulating, and querying
/// spans. Implemented for `Range<usize>` and custom span types.
pub mod span;

/// Slice abstractions for different string types.
///
/// Defines the [`Slice`] trait for working with string slices in a
/// generic way. Supports multiple string types through feature flags:
/// - `bytes`: `&[u8]` (byte slices)
/// - `bstr`: `bstr::BStr` (byte strings)
/// - `hipstr`: `hipstr::HipStr` (inline/heap strings)
pub mod slice;

/// State management for lexers.
///
/// Provides state tracking types for lexers:
/// - [`State`](state::State): Base trait for lexer state
/// - [`RecursionLimiter`](state::RecursionLimiter) / [`RecursionTracker`](state::RecursionTracker):
///   Prevent infinite recursion
/// - [`TokenLimiter`](state::TokenLimiter) / [`TokenTracker`](state::TokenTracker): Track token
///   occurrences
/// - [`Limiter`](state::Limiter) / [`Tracker`](state::Tracker): Combined recursion and token
///   tracking
pub mod state;

/// Token caching for lookahead and backtracking.
///
/// Defines the [`Cache`](cache::Cache) trait for buffering tokens to enable lookahead
/// and backtracking operations. The complete set of implementations is three, all
/// bounded at compile time:
/// - `ArrayDeque<_, N>`: a fixed-capacity ring (`DefaultCache` is this at `U3`)
/// - `Option<CachedToken<..>>`: capacity 1
/// - `()`: capacity 0 — no caching, for streaming-only scenarios
///
/// There is no dynamic, allocator-backed cache, and `alloc` does not add one: it enables
/// allocator-backed containers and drivers and forwards the sub-crates' `alloc` features.
/// Nor would one buy unbounded lookahead — [`Window`](parse::Window) is sealed at U1–U32, so no
/// cache can serve a public peek past 32 tokens. Lookahead beyond the window is what
/// transactions are for: they speculate arbitrarily far and re-lex on rollback.
pub mod cache;

/// Token trait and related types.
///
/// Defines the [`Token`] trait that bridges lexical analysis (Logos) and
/// structured token representation for parsing. Separates raw lexer output from the
/// token type used in parsing, allowing custom data and behavior beyond what Logos
/// provides.
pub mod token;

pub mod input;

#[cfg(feature = "conformance")]
#[cfg_attr(docsrs, doc(cfg(feature = "conformance")))]
pub mod conformance;

#[cfg(feature = "fuzz")]
#[cfg_attr(docsrs, doc(cfg(feature = "fuzz")))]
pub mod fuzz;

#[cfg(all(feature = "std", feature = "logos_0_16", feature = "combinators"))]
#[cfg_attr(
  docsrs,
  doc(cfg(all(feature = "std", feature = "logos_0_16", feature = "combinators")))
)]
pub mod guide;

/// API_DIGEST_CENSUS — the guide's re-declarations of public items, read against the items.
///
/// Gated on `std` rather than on the guide's own `all(std, logos_0_16, combinators)`: the census
/// walks `src/` with `std::fs` and needs no other feature, so matching the guide's cfg would run
/// it under `--all-features` alone instead of every leg that has `std`.
#[cfg(all(test, feature = "std"))]
mod api_digest_census;
mod check;
mod integration;
mod keyword;
mod located;
#[cfg(feature = "pratt")]
mod native_stack;

mod require;

#[doc(hidden)]
pub mod __private {
  pub use super::{check::Check, error, lexer::*, require::Require, span, syntax, token, utils};

  // Same re-export as the crate-level `tokora::logos` alias above; the macro expansions that
  // name `$crate::__private::logos` must resolve to the same version a user derive does.
  #[cfg(feature = "logos_0_16")]
  pub use ::logos_0_16 as logos;
  pub use paste;

  #[cfg(any(feature = "std", feature = "alloc"))]
  pub use std::{boxed::Box, string::String, vec::Vec};
}
