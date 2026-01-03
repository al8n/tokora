#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(clippy::double_parens, clippy::type_complexity)]
#![deny(missing_docs, warnings)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "logos")]
#[cfg_attr(docsrs, doc(cfg(feature = "logos")))]
pub use logos;

pub use cache::{Cache, DefaultCache};
pub use check::Check;
pub use emitter::Emitter;
pub use input::InputRef;
pub use lexer::{Lexed, Lexer, State};
pub use located::*;
pub use parse_choice::*;
pub use parse_input::*;
pub use parser::{Parse, ParseContext, Parser};
pub use require::Require;
pub use separator_handler::SeparatorHandler;
pub use slice::Slice;
pub use source::Source;
pub use token::Token;
pub use try_parse_input::TryParseInput;

/// Concrete Syntax Tree (CST) representations and utilities.
#[cfg(feature = "rowan")]
#[cfg_attr(docsrs, doc(cfg(feature = "rowan")))]
pub mod cst;

/// Lexers and token definitions.
pub mod lexer;

/// Parsers and combinators.
pub mod parser;

/// Common types for any programming language.
pub mod types;

/// Syntax definitions and traits.
pub mod syntax;

/// Common utilities for working with tokens and lexers.
pub mod utils;

/// Trait for container types.
pub mod container;

/// The emitter related structures and traits
pub mod emitter;

/// Common error types for lexers and parsers.
pub mod error;

/// Common punctuation tokens.
pub mod punct;

/// The source related structures and traits
pub mod source;

/// The span related structures and traits
pub mod span;

/// Slice related structures and traits
pub mod slice;

/// The cache related structures and traits
pub mod cache;

/// The token related structures and traits
pub mod token;

/// The input related structures and traits
pub mod input;

mod check;
mod keyword;
mod located;
mod parse_choice;
mod parse_input;
mod require;
mod separator_handler;
mod try_parse_input;

#[doc(hidden)]
pub mod __private {
  pub use super::{check::Check, error, lexer::*, require::Require, span, syntax, token, utils};
  #[cfg(feature = "logos")]
  pub use logos;
  pub use paste;

  #[cfg(any(feature = "std", feature = "alloc"))]
  pub use std::{boxed::Box, string::String, vec::Vec};
}
