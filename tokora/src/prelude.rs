//! Convenience re-exports for common tokora usage.
//!
//! ```
//! use tokora::prelude::*;
//! ```
//!
//! This module re-exports the most commonly needed traits, types, and macros
//! for writing parsers with tokora.

// Core traits
pub use crate::{Emitter, Lexer, Parse, ParseContext, ParseInput, Token, TryParseInput};

// Core types
pub use crate::span::Spanned;
pub use crate::{FatalContext, InputRef, Parser, ParserContext, SimpleSpan, Span};

// Error
pub use crate::error::UnexpectedEot;
pub use crate::error::token::UnexpectedTokenOf;
