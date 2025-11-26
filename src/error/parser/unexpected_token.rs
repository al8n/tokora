//! Unexpected token error type for parser error reporting.
//!
//! This module provides the [`UnexpectedToken`] type, which represents parser errors
//! when an unexpected token is encountered. It captures both the location of the error,
//! what token was found (if any), and what tokens were expected.
//!
//! # Design Philosophy
//!
//! `UnexpectedToken` is designed to provide rich, actionable error messages:
//! - **Location tracking**: The `span` field pinpoints exactly where the error occurred
//! - **Optional found token**: Distinguishes between unexpected tokens and end-of-input
//! - **Flexible expectations**: Can express single or multiple alternative expected tokens
//! - **Position adjustment**: The `bump()` method allows adjusting error positions when
//!   combining errors from different parsing contexts
//!
//! # Common Patterns
//!
//! ## End of Input Errors
//!
//! When the parser reaches the end of input unexpectedly, use constructors without a found token:
//!
//! ```
//! use logosky::{utils::Span, error::UnexpectedToken};
//!
//! // Simple end-of-input error
//! let error: UnexpectedToken<&str, &str> = UnexpectedToken::expected_one(
//!     Span::new(100, 100),
//!     "}"
//! );
//! assert_eq!(format!("{}", error), "unexpected end of input, expected '}'");
//! ```
//!
//! ## Unexpected Token Errors
//!
//! When a specific token was found but something else was expected:
//!
//! ```
//! use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
//!
//! let error = UnexpectedToken::expected_one_with_found(
//!     Span::new(10, 15),
//!     "else",
//!     "if"
//! );
//! assert_eq!(format!("{}", error), "unexpected token 'else', expected 'if'");
//! ```

use crate::{error::parser::{Trailing, Leading}, utils::{Expected, Span}};

pub use unexpected_trailing::*;
pub use unexpected_leading::*;

mod unexpected_leading;
mod unexpected_trailing;

/// An error representing an unexpected token encountered during parsing.
///
/// This error type captures the location (span), what token was found (if any),
/// and what token(s) were expected. It's commonly used in parsers to provide
/// detailed error messages when the input doesn't match the expected syntax.
///
/// # Type Parameters
///
/// * `T` - The type of the actual token that was found
/// * `Kind` - The type of the expected token (often an enum of token kinds)
///
/// # Examples
///
/// ```
/// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
///
/// // Error when expecting a specific token but got something else
/// let error = UnexpectedToken::expected_one_with_found(
///     Span::new(10, 15),
///     "}",
///     "{"
/// );
/// assert_eq!(error.span(), Span::new(10, 15));
/// assert_eq!(format!("{}", error), "unexpected token '}', expected '{'");
///
/// // Error when expecting one of multiple tokens
/// let error = UnexpectedToken::expected_one_of_with_found(
///     Span::new(0, 10),
///     "identifier",
///     &["if", "while", "for"]
/// );
/// assert_eq!(
///     format!("{}", error),
///     "unexpected token 'identifier', expected one of: 'if', 'while', 'for'"
/// );
///
/// // Error when reaching end of input unexpectedly
/// let error: UnexpectedToken<&str, &str> = UnexpectedToken::expected_one(
///     Span::new(100, 100),
///     "}"
/// );
/// assert_eq!(format!("{}", error), "unexpected end of input, expected '}'");
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct UnexpectedToken<'a, T, Kind, S = Span, Knowledge = ()> {
  span: S,
  found: Option<T>,
  expected: Option<Expected<'a, Kind>>,
  knowledge: Option<Knowledge>,
}

impl<T, Kind, S, Knowledge> UnexpectedToken<'_, T, Kind, S, Trailing<Knowledge>> {
  /// Creates a new `UnexpectedToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing(span: S, found: T) -> Self {
    Self::new_in(span, Some(found), None, Some(Trailing::new()))
  }
}

impl<T, Kind, S, Knowledge> UnexpectedToken<'_, T, Kind, S, Leading<Knowledge>> {
  /// Creates a new `UnexpectedToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading(span: S, found: T) -> Self {
    Self::new_in(span, Some(found), None, Some(Leading::new()))
  }
}

impl<'a, T, Kind, S, Knowledge> UnexpectedToken<'a, T, Kind, S, Knowledge> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(span: S, found: Option<T>, expected: Option<Expected<'a, Kind>>, knowledge: Option<Knowledge>) -> Self {
    Self {
      span,
      found,
      expected,
      knowledge,
    }
  }

  /// Creates a new unexpected token error.
  /// 
  /// This error indicates that an unexpected token was encountered,
  /// without specifying what token was found or expected.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    span: S,
  ) -> Self {
    Self::new_in(span, None, None, None)
  }

  /// Adds knowledge to the `UnexpectedToken` error.
  /// 
  /// This method allows attaching additional context or information
  /// to the error, which can be useful for debugging or reporting.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_knowledge_const(mut self, knowledge: Knowledge) -> Self
  where
    Knowledge: Copy,
  {
    self.knowledge = Some(knowledge);
    self
  }

  /// Adds knowledge to the `UnexpectedToken` error.
  /// 
  /// This method allows attaching additional context or information
  /// to the error, which can be useful for debugging or reporting.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_knowledge(mut self, knowledge: Knowledge) -> Self {
    self.knowledge = Some(knowledge);
    self
  }

  /// Creates an unexpected token error without a found token.
  ///
  /// This is useful when the parser reaches the end of input unexpectedly.
  /// The error will indicate "unexpected end of input" in its display message.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// let error: UnexpectedToken<&str, &str> = UnexpectedToken::new(
  ///     Span::new(100, 101),
  ///     Expected::one("}")
  /// );
  /// assert!(error.found().is_none());
  /// assert_eq!(error.span(), Span::new(100, 101));
  /// assert_eq!(format!("{}", error), "unexpected end of input, expected '}'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_expected(span: S, expected: Expected<'a, Kind>) -> Self {
    Self::new_in(span, None, Some(expected), None)
  }

  /// Creates a new unexpected token error with a single expected token.
  ///
  /// This is a convenience method that combines `new` with `Expected::one`.
  /// The error has no found token, indicating the end of input was reached.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::Span, error::UnexpectedToken};
  ///
  /// let error: UnexpectedToken<&str, &str> = UnexpectedToken::expected_one(
  ///     Span::new(50, 51),
  ///     ";"
  /// );
  /// assert!(error.found().is_none());
  /// assert_eq!(format!("{}", error), "unexpected end of input, expected ';'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected_one(span: S, expected: Kind) -> Self {
    Self::with_expected(span, Expected::one(expected))
  }

  /// Creates a new unexpected token error with a single expected token.
  ///
  /// This is a convenience method that combines `new` with `Expected::one`.
  /// The error has no found token, indicating the end of input was reached.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::Span, error::UnexpectedToken};
  ///
  /// let error: UnexpectedToken<&str, &str> = UnexpectedToken::expected_one_with_found(
  ///     Span::new(50, 51),
  ///     ":",
  ///     ";"
  /// );
  /// assert!(error.found().is_some());
  /// assert_eq!(format!("{}", error), "unexpected token ':', expected ';'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected_one_with_found(span: S, found: T, expected: Kind) -> Self {
    Self::new_in(span, Some(found), Some(Expected::one(expected)), None)
  }

  /// Creates a new unexpected token error with multiple expected tokens.
  ///
  /// This is a convenience method that combines `new` with `Expected::one_of`.
  /// The error has no found token, indicating the end of input was reached.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::Span, error::UnexpectedToken};
  ///
  /// let error: UnexpectedToken<&str, &str> = UnexpectedToken::expected_one_of(
  ///     Span::new(25, 26),
  ///     &["+", "-", "*", "/"]
  /// );
  /// assert!(error.found().is_none());
  /// assert_eq!(
  ///     format!("{}", error),
  ///     "unexpected end of input, expected one of: '+', '-', '*', '/'"
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected_one_of(span: S, expected: &'static [Kind]) -> Self {
    Self::with_expected(span, Expected::one_of(expected))
  }

  /// Creates a new unexpected token error with multiple expected tokens.
  ///
  /// This is a convenience method that combines `new` with `Expected::one_of`.
  /// The error has no found token, indicating the end of input was reached.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::Span, error::UnexpectedToken};
  ///
  /// let error: UnexpectedToken<&str, &str> = UnexpectedToken::expected_one_of_with_found(
  ///     Span::new(25, 26),
  ///     ":",
  ///     &["+", "-", "*", "/"]
  /// );
  /// assert!(!error.found().is_none());
  /// assert_eq!(
  ///     format!("{}", error),
  ///     "unexpected token ':', expected one of: '+', '-', '*', '/'"
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected_one_of_with_found(span: S, found: T, expected: &'static [Kind]) -> Self {
    Self::new_in(span, Some(found), Some(Expected::one_of(expected)), None)
  }

  /// Creates a new unexpected token error with an optional found token.
  ///
  /// This is the most general constructor. When `found` is `None`, the error
  /// indicates the end of input was reached. When `found` is `Some`, it indicates
  /// an unexpected token was encountered.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// // With a found token
  /// let error = UnexpectedToken::expected_one(
  ///     Span::new(10, 14),
  ///     "if"
  /// ).maybe_found(Some("else"));
  /// assert_eq!(error.found(), Some(&"else"));
  /// assert_eq!(format!("{}", error), "unexpected token 'else', expected 'if'");
  ///
  /// // Without a found token (end of input)
  /// let error: UnexpectedToken<&str, &str> = UnexpectedToken::expected_one(
  ///     Span::new(50, 50),
  ///     "if"
  /// ).maybe_found(None);
  /// assert_eq!(error.found(), None);
  /// assert_eq!(format!("{}", error), "unexpected end of input, expected 'if'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn maybe_found(mut self, found: Option<T>) -> Self {
    self.found = found;
    self
  }

  /// Creates a new unexpected token error with an optional found token.
  ///
  /// This is the most general constructor. When `found` is `None`, the error
  /// indicates the end of input was reached. When `found` is `Some`, it indicates
  /// an unexpected token was encountered.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// // With a found token
  /// let error = UnexpectedToken::expected_one(
  ///     Span::new(10, 14),
  ///     "if"
  /// ).maybe_found_const(Some("else"));
  /// assert_eq!(error.found(), Some(&"else"));
  /// assert_eq!(format!("{}", error), "unexpected token 'else', expected 'if'");
  ///
  /// // Without a found token (end of input)
  /// let error: UnexpectedToken<&str, &str> = UnexpectedToken::expected_one(
  ///     Span::new(50, 50),
  ///     "if"
  /// ).maybe_found_const(None);
  /// assert_eq!(error.found(), None);
  /// assert_eq!(format!("{}", error), "unexpected end of input, expected 'if'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn maybe_found_const(mut self, found: Option<T>) -> Self
  where
    T: Copy,
  {
    self.found = found;
    self
  }

  /// Creates a new unexpected token error with a found token.
  ///
  /// This indicates that a specific token was encountered when a different
  /// token (or one of several alternative tokens) was expected.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// let error = UnexpectedToken::expected_one(
  ///     Span::new(5, 10),
  ///     "fn"
  /// ).with_found("class");
  /// assert_eq!(error.found(), Some(&"class"));
  /// assert_eq!(format!("{}", error), "unexpected token 'class', expected 'fn'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_found(mut self, found: T) -> Self {
    self.found = Some(found);
    self
  }

  /// Creates a new unexpected token error with a found token.
  ///
  /// This indicates that a specific token was encountered when a different
  /// token (or one of several alternative tokens) was expected.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// let error = UnexpectedToken::expected_one(
  ///     Span::new(5, 10),
  ///     "fn"
  /// ).with_found_const("class");
  /// assert_eq!(error.found(), Some(&"class"));
  /// assert_eq!(format!("{}", error), "unexpected token 'class', expected 'fn'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_found_const(mut self, found: T) -> Self
  where
    T: Copy,
  {
    self.found = Some(found);
    self
  }

  /// Returns the span of the unexpected token.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// let error = UnexpectedToken::expected_one_with_found(
  ///     Span::new(10, 15),
  ///     "identifier",
  ///     "number"
  /// );
  /// assert_eq!(error.span(), Span::new(10, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    self.span
  }

  /// Returns a reference to the found token, if any.
  ///
  /// Returns `None` if the error represents an unexpected end of input.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// let error = UnexpectedToken::expected_one_with_found(
  ///     Span::new(0, 10),
  ///     "identifier",
  ///     "number"
  /// );
  /// assert_eq!(error.found(), Some(&"identifier"));
  ///
  /// let eof_error: UnexpectedToken<&str, &str> = UnexpectedToken::expected_one(
  ///     Span::new(100, 100),
  ///     "}"
  /// );
  /// assert_eq!(eof_error.found(), None);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn found(&self) -> Option<&T> {
    self.found.as_ref()
  }

  /// Returns a reference to the expected token(s).
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// let error = UnexpectedToken::expected_one_with_found(
  ///     Span::new(5, 6),
  ///     "}",
  ///     "{"
  /// );
  /// assert_eq!(*error.expected(), Expected::one("{"));
  /// if let Expected::One(value) = error.expected() {
  ///     assert_eq!(*value, "{");
  /// }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected(&self) -> Option<&Expected<'a, Kind>> {
    self.expected.as_ref()
  }

  /// Bumps both the start and end positions of the span by the given offset.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining spans from different contexts.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// let mut error = UnexpectedToken::expected_one_with_found(
  ///     Span::new(10, 15),
  ///     "}",
  ///     "{"
  /// );
  /// error.bump(5);
  /// assert_eq!(error.span(), Span::new(15, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &S::Offset)
  where
    S: crate::lexer::Span,
  {
    self.span.bump(offset);
  }

  /// Maps the expected token(s) using the provided function.
  ///
  /// This is useful for transforming the expected token type while preserving
  /// the rest of the error information.
  ///
  /// ## Examples
  ///
  /// ```
  /// # #[cfg(feature = "std")] {
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// let error = UnexpectedToken::expected_one_with_found(
  ///    Span::new(0, 5),
  ///   "identifier",
  ///   "number"
  /// );
  /// let mapped_error = error.map_expected(|expected| {
  ///     // Transform the expected token type here
  ///     Expected::one(expected.unwrap_one().to_string())
  /// });
  /// # }
  /// ```
  pub fn map_expected<F, Kind2>(self, f: F) -> UnexpectedToken<'a, T, Kind2, S, Knowledge>
  where
    F: FnOnce(Expected<'a, Kind>) -> Expected<'a, Kind2>,
  {
    UnexpectedToken {
      span: self.span,
      found: self.found,
      expected: self.expected.map(f),
      knowledge: self.knowledge,
    }
  }

  /// Consumes the error and returns its components.
  ///
  /// This method deconstructs the error into its constituent parts:
  /// the span, the found token (if any), and the expected token(s).
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::UnexpectedToken};
  ///
  /// let error = UnexpectedToken::expected_one_with_found(
  ///     Span::new(5, 6),
  ///     "}",
  ///     "{"
  /// );
  /// let (span, found, expected) = error.into_components();
  /// assert_eq!(span, Span::new(5, 6));
  /// assert_eq!(found, Some("}"));
  /// assert_eq!(expected, Expected::one("{"));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (S, Option<T>, Option<Expected<'a, Kind>>, Option<Knowledge>) {
    (self.span, self.found, self.expected, self.knowledge)
  }
}

// impl<T: core::fmt::Display, Kind: core::fmt::Display + 'static, S> core::fmt::Display
//   for UnexpectedToken<'_, T, Kind, S>
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//     match &self.found {
//       Some(found) => write!(f, "unexpected token '{found}', {}", self.expected),
//       None => write!(f, "unexpected end of input, {}", self.expected),
//     }
//   }
// }

// impl<
//   T: core::fmt::Debug + core::fmt::Display,
//   Kind: core::fmt::Display + core::fmt::Debug + 'static,
//   S: core::fmt::Debug,
// > core::error::Error for UnexpectedToken<'_, T, Kind, S>
// {
// }
