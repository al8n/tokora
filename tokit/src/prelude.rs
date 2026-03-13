//! Convenience re-exports for common tokit usage.
//!
//! ```
//! use tokit::prelude::*;
//! ```
//!
//! This module re-exports the most commonly needed traits, types, and macros
//! for writing parsers with tokit.

// Core traits
pub use crate::{
  Emitter,
  Lexer,
  Parse,
  ParseContext,
  ParseInput,
  Token,
  TryParseInput,
};

// Core types
pub use crate::{
  FatalContext,
  InputRef,
  Parser,
  ParserContext,
  SimpleSpan,
  Span,
};
pub use crate::span::Spanned;

// Error
pub use crate::error::UnexpectedEot;
pub use crate::error::token::UnexpectedTokenOf;
