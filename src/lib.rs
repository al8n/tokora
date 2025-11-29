#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(clippy::double_parens)]
#![deny(missing_docs, warnings)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

pub use logos;

pub use check::*;
pub use lexer::*;
pub use parser::*;
pub use require::*;

mod check;
mod lexer;
mod parser;
mod require;

/// Concrete Syntax Tree (CST) representations and utilities.
#[cfg(feature = "rowan")]
#[cfg_attr(docsrs, doc(cfg(feature = "rowan")))]
pub mod cst;

/// Common types for any programming language.
pub mod types;

/// Syntax definitions and traits.
pub mod syntax;

/// Common utilities for working with tokens and lexers.
pub mod utils;

/// Common error types for lexers and parsers.
pub mod error;

mod keyword;
mod punct;

#[doc(hidden)]
pub mod __private {
  pub use super::{check::Check, error, lexer::*, require::Require, syntax, utils};
  pub use paste;
  pub use logos;

  #[cfg(any(feature = "std", feature = "alloc"))]
  pub use std::{boxed::Box, string::String, vec::Vec};
}
