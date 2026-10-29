//! Blazing fast parser combinators with deterministic parsing and zero-copy streaming.
//!
//! This module provides a unique parser combinator framework that combines:
//!
//! 1. **Parse-While-Lexing Architecture**: Zero-copy streaming - tokens consumed directly from
//!    the lexer without buffering, eliminating allocation overhead
//! 2. **Deterministic LALR-Style Parsing**: Explicit lookahead with compile-time buffer capacity, no hidden backtracking
//! 3. **Flexible Error Handling**: Same parser adapts for fail-fast runtime ([`Fatal`](crate::emitter::Fatal))
//!    or greedy compiler diagnostics (via custom [`Emitter`])
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
//! **Tokora (Streaming)**:
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
//! Unlike traditional parser combinators with implicit backtracking, Tokora uses
//! **explicit lookahead-based decisions**:
//!
//! ```ignore
//! // Traditional: Hidden backtracking
//! let parser = try_parser1.or(try_parser2).or(try_parser3);
//!
//! // Tokora: Explicit lookahead, deterministic
//! let parser = any().peek_then::<_, typenum::U2>(|peeked, _| {
//!     match peeked.front() {
//!         Some(Token::If) => Ok(Action::Continue),  // Deterministic!
//!         _ => Ok(Action::Stop),
//!     }
//! });
//! ```
//!
//! The [`Window`](crate::parse::Window) trait provides compile-time fixed lookahead capacity
//! (`typenum::U1` to `typenum::U32`), enabling LALR-style deterministic table parsing.
//!
//! ## Flexible Error Handling via Emitter
//!
//! The [`Emitter`] trait decouples parsing logic from error handling strategy:
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
//! use tokora::{Any, Parse, Parser, parse::FatalContext};
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
//! - `any` - Accept any single token
//! - `expect` - Expect specific token, emit error if not found
//! - `empty` - No-op parser
//! - `todo` - Placeholder for incomplete implementations
//!
//! ## Sequencing
//!
//! - `then` - Sequential composition: parse `p1` then `p2`
//! - `then_ignore` - Parse both, keep only first result
//! - `ignore_then` - Parse both, keep only second result
//!
//! ## Repetition & Collections
//!
//! - `repeated` - Repeat until condition returns `Action::Stop`
//! - `separated_by` - Parse elements separated by delimiter
//! - `delim` - Parse delimited content (e.g., parentheses)
//! - `delim_seq` - Parse delimited, separated sequences
//! - `delimited`/`parens`/`braces`/`brackets`/`angles` (+ `try_` attempt twins) - one delimited region as a span-carrying `Delimited`
//!
//! ## Lookahead & Conditional (Deterministic)
//!
//! - `peek_then` - Peek ahead with fixed window, make deterministic decision
//! - `peek_then_choice` - Choose between alternatives based on lookahead
//!
//! ## Transformation
//!
//! - `map` - Transform output
//! - `filter` - Filter with validation
//! - `filter_map` - Filter and transform
//! - `validate` - Validate with full location context
//!
//! ## Error Recovery
//!
//! - `recover` - Try parser, use recovery on error with backtracking
//! - `inplace_recover` - Try parser, use recovery on error without backtracking
//! - `padded` - Skip trivia (whitespace/comments) before and after
//!
//! # Feature gating
//!
//! Each combinator **family** is its own feature, so a no-std or no-alloc build compiles only
//! the ones its grammar calls: `any`, `fail`, `filter` (which covers `filter_map`), `fold`
//! (implies `many`), `ident`, `keyword`, `many`, `map`, `peek`, `pratt`, `punct`, `then`,
//! `validate`. The `combinators` umbrella turns all of them on and is a **default** feature,
//! so only a `default-features = false` dependency has to name any of this. Every gated item
//! carries its feature in the rustdoc label.
//!
//! The substrate is unconditional: [`Parser`], [`Parse`], the [`parse()`] family, the
//! [`ParseInput`] / [`TryParseInput`] / [`ParseChoice`] traits, and the combinators the
//! families are written against — `expect`, `delimited`, `recover`, `select`, `opt`,
//! `padded`, `node`, `skip_then_retry`, `ignore`, `labelled`, `empty`, `todo`, `with`,
//! `by_ref`, `collect`, `accepted`, `unwrapped`.
//!
//! Three edges are not guessable from the names. `fold` routes its "no more elements" exits
//! through `many`'s absence gate, so it pulls `many` in. `list` / `separated1` and
//! `try_ident_list` are named shapes of `many`'s drivers, so they need `many` on top of their
//! own gate. And `punct` gates the punctuator *parsers* — including `delimited`'s
//! `parens`/`braces`/`brackets`/`angles` shapes, which run them — but **not** the
//! [`Punctuator`](crate::punct::Punctuator) impls for the built-in markers, which are
//! vocabulary the whole crate reads and stay unconditional.
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

use core::marker::PhantomData;

use crate::{
  Emitter, Lexer, Source, Token,
  input::{Complete, Completeness, InputRef, SurfaceIncomplete},
  located::Located,
  parse::{context::ParseContext, input::*},
  slice::Sliced,
  span::Spanned,
  utils::{Expected, PhantomLocated, PhantomSliced, PhantomSpan},
};

// Combinator constructors whose declarations sit with the parsing traits they serve: `pinned`
// commits a `ParseInput`, and `while_head` / `while_kind` build `Decision`s. They are combinators
// all the same, so this is their one public path.
pub use crate::parse::input::{Pinned, WhileHead, WhileKind, pinned, while_head, while_kind};

#[cfg(feature = "trace")]
#[cfg_attr(docsrs, doc(cfg(feature = "trace")))]
pub use crate::trace::Traced;
/// Parser tracing DX: wrap any parser in [`traced`] to print an indented event tree of its
/// run — enter, exit-ok with the consumed span, exit-err — interleaved with the crate's own
/// instrumented combinators (`try_expect`, `peek`, the `sync` family, the transaction guards,
/// `attempt`/`try_attempt`, and the separated/repeated drivers) as they fire. Gated on the
/// `trace` feature; with it off, [`traced`] is the identity and every hook compiles away.
pub use crate::trace::traced;

pub use accepted::*;
pub use by_ref::*;
pub use collect::Collect;
pub use delimited::*;
pub use empty::*;
pub use expect::*;
pub use ignore::*;
pub use labelled::*;
// `separated1` and `list` are two named shapes of `many`'s while-drivers with a `Vec` sink;
// they need the allocator for the sink and the family for the driver.
#[cfg(all(feature = "many", any(feature = "alloc", feature = "std")))]
#[cfg_attr(
  docsrs,
  doc(cfg(all(feature = "many", any(feature = "alloc", feature = "std"))))
)]
pub use list::*;
pub use node::*;
pub use opt::*;
pub use padded::*;
// `RecoverInput` and `InplaceRecoverInput` are declared in `recover` too; they are traits, so
// `crate::parse` re-exports them instead.
pub use recover::{InplaceRecover, Recover};
pub use select::*;
pub use skip_then_retry::*;
pub use todo::*;
pub use unwrapped::*;
pub use with::*;

#[cfg(feature = "any")]
#[cfg_attr(docsrs, doc(cfg(feature = "any")))]
pub use any::*;
#[cfg(feature = "fail")]
#[cfg_attr(docsrs, doc(cfg(feature = "fail")))]
pub use fail::*;
#[cfg(feature = "filter")]
#[cfg_attr(docsrs, doc(cfg(feature = "filter")))]
pub use filter::*;
#[cfg(feature = "filter")]
#[cfg_attr(docsrs, doc(cfg(feature = "filter")))]
pub use filter_map::*;
#[cfg(feature = "fold")]
#[cfg_attr(docsrs, doc(cfg(feature = "fold")))]
pub use fold::*;
#[cfg(all(feature = "ident", feature = "many"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "ident", feature = "many"))))]
pub use ident_list::*;
#[cfg(feature = "many")]
#[cfg_attr(docsrs, doc(cfg(feature = "many")))]
pub use many::*;
#[cfg(feature = "map")]
#[cfg_attr(docsrs, doc(cfg(feature = "map")))]
pub use map::*;
#[cfg(feature = "peek")]
#[cfg_attr(docsrs, doc(cfg(feature = "peek")))]
pub use peek::*;
#[cfg(feature = "then")]
#[cfg_attr(docsrs, doc(cfg(feature = "then")))]
pub use then::*;
#[cfg(feature = "validate")]
#[cfg_attr(docsrs, doc(cfg(feature = "validate")))]
pub use validate::*;

mod accepted;
mod by_ref;
mod collect;
mod delimited;
mod empty;
mod expect;
mod ignore;
mod labelled;
#[cfg(all(feature = "many", any(feature = "alloc", feature = "std")))]
mod list;
mod node;
mod opt;
mod padded;
pub(crate) mod recover;
mod recovery_gate;
mod select;
mod skip_then_retry;
mod todo;
mod unwrapped;
mod with;

#[cfg(feature = "any")]
mod any;
#[cfg(feature = "fail")]
mod fail;
#[cfg(feature = "filter")]
mod filter;
#[cfg(feature = "filter")]
mod filter_map;
#[cfg(feature = "fold")]
mod fold;
#[cfg(feature = "ident")]
mod ident;
// `try_ident_list` is `Ident::try_parse` driven through `many`'s `separated` — one item, both
// families, so it is gated on both rather than duplicating either.
#[cfg(all(feature = "ident", feature = "many"))]
mod ident_list;
#[cfg(feature = "keyword")]
mod keyword;
#[cfg(feature = "many")]
pub(crate) mod many;
#[cfg(feature = "map")]
mod map;
#[cfg(feature = "peek")]
mod peek;
// Always compiled: it also carries the `Punctuator` impls for the built-in markers, which are
// crate-wide vocabulary. The `punct` feature gates the inherent `parse`/`try_parse` parsers
// inside it.
mod punct;
#[cfg(feature = "then")]
mod then;
#[cfg(feature = "validate")]
mod validate;

#[cfg(test)]
mod tests;
