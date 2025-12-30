//! Unexpected token error type for parser error reporting.
//!
//! This module provides the [`MissingToken`] type, which represents parser errors
//! when an missing token is encountered. It captures both the location of the error,
//! what token was found (if any), and what tokens were expected.
//!
//! # Design Philosophy
//!
//! `MissingToken` is designed to provide rich, actionable error messages:
//! - **Location tracking**: The `span` field pinpoints exactly where the error occurred
//! - **Optional found token**: Distinguishes between missing tokens and end-of-input
//! - **Flexible expectations**: Can express single or multiple alternative expected tokens
//! - **Position adjustment**: The `bump()` method allows adjusting error positions when
//!   combining errors from different parsing contexts
//!
//! # Common Patterns
//!
//! ## End of Input Errors
//!
//! When the parser reaches the end of input missingly, use constructors without a found token:
//!
//! ```
//! use tokit::{utils::SimpleSpan, error::token::MissingToken};
//!
//! // Simple end-of-input error
//! let error: MissingToken<&str, &str> = MissingToken::expected_one(
//!     SimpleSpan::new(100, 100),
//!     "}"
//! );
//! assert_eq!(format!("{}", error), "missing end of input, expected '}'");
//! ```
//!
//! ## Unexpected Token Errors
//!
//! When a specific token was found but something else was expected:
//!
//! ```
//! use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
//!
//! let error = MissingToken::expected_one_with_found(
//!     SimpleSpan::new(10, 15),
//!     "else",
//!     "if"
//! );
//! assert_eq!(format!("{}", error), "missing token 'else', expected 'if'");
//! ```

use core::{marker::PhantomData, ops::AddAssign};

use crate::{
  Lexer, Token,
  error::token::{Leading, Separator, Trailing},
  utils::{Expected, Message, Ownable},
};

pub use missing_leading::*;
pub use missing_trailing::*;

mod missing_leading;
mod missing_trailing;

/// A type alias for a `MissingToken` error for a given lexer and separator.
pub type MissingSeparatorOf<'inp, Sep, L, Lang = ()> = MissingToken<
  'inp,
  <<L as Lexer<'inp>>::Token as Token<'inp>>::Kind,
  <L as Lexer<'inp>>::Offset,
  Separator<Sep, Lang>,
>;

/// An error representing an missing token encountered during parsing.
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
/// use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
///
/// // Error when expecting a specific token but got something else
/// let error = MissingToken::expected_one_with_found(
///     SimpleSpan::new(10, 15),
///     "}",
///     "{"
/// );
/// assert_eq!(error.span(), SimpleSpan::new(10, 15));
/// assert_eq!(format!("{}", error), "missing token '}', expected '{'");
///
/// // Error when expecting one of multiple tokens
/// let error = MissingToken::expected_one_of_with_found(
///     SimpleSpan::new(0, 10),
///     "identifier",
///     &["if", "while", "for"]
/// );
/// assert_eq!(
///     format!("{}", error),
///     "missing token 'identifier', expected one of: 'if', 'while', 'for'"
/// );
///
/// // Error when reaching end of input missingly
/// let error: MissingToken<&str, &str> = MissingToken::expected_one(
///     SimpleSpan::new(100, 100),
///     "}"
/// );
/// assert_eq!(format!("{}", error), "missing end of input, expected '}'");
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MissingToken<'a, Kind: Ownable, O = usize, Lang: ?Sized = ()> {
  offset: O,
  expected: Option<Expected<'a, Kind>>,
  message: Option<Message>,
  _lang: PhantomData<Lang>,
}

impl<Kind: Ownable, O, Data> MissingToken<'_, Kind, O, Trailing<Data>> {
  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing(offset: O) -> Self {
    Self::trailing_of(offset)
  }

  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing_with_message(offset: O, message: Message) -> Self {
    Self::trailing_with_message_of(offset, message)
  }
}

impl<Kind: Ownable, O, Data, Lang> MissingToken<'_, Kind, O, Leading<Data, Lang>> {
  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading(offset: O) -> Self {
    Self::leading_of(offset)
  }

  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading_with_message(offset: O, message: Message) -> Self {
    Self::leading_with_message_of(offset, message)
  }
}

impl<Kind: Ownable, O, Data, Lang: ?Sized> MissingToken<'_, Kind, O, Trailing<Data, Lang>> {
  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing_of(offset: O) -> Self {
    Self::of(offset)
  }

  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing_with_message_of(offset: O, message: Message) -> Self {
    Self::with_message_of(offset, message)
  }
}

impl<Kind: Ownable, O, Data, Lang: ?Sized> MissingToken<'_, Kind, O, Leading<Data, Lang>> {
  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading_of(offset: O) -> Self {
    Self::of(offset)
  }

  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading_with_message_of(offset: O, message: Message) -> Self {
    Self::with_message_of(offset, message)
  }
}

impl<Kind: Ownable, O> MissingToken<'_, Kind, O> {
  /// Creates a new missing token error.
  ///
  /// This error indicates that an missing token was encountered,
  /// without specifying what token was found or expected.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(offset: O) -> Self {
    Self::of(offset)
  }

  /// Adds knowledge to the `MissingToken` error.
  ///
  /// This method allows attaching additional context or information
  /// to the error, which can be useful for debugging or reporting.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_message(offset: O, message: Message) -> Self {
    Self::with_message_of(offset, message)
  }
}

impl<'a, Kind: Ownable, O, Lang: ?Sized> MissingToken<'a, Kind, O, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(
    offset: O,
    expected: Option<Expected<'a, Kind>>,
    message: Option<Message>,
  ) -> Self {
    Self {
      offset,
      expected,
      message,
      _lang: PhantomData,
    }
  }

  /// Creates a new missing token error.
  ///
  /// This error indicates that an missing token was encountered,
  /// without specifying what token was found or expected.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(offset: O) -> Self {
    Self::new_in(offset, None, None)
  }

  /// Adds knowledge to the `MissingToken` error.
  ///
  /// This method allows attaching additional context or information
  /// to the error, which can be useful for debugging or reporting.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_message_of(offset: O, message: Message) -> Self {
    Self::new_in(offset, None, Some(message))
  }

  /// Creates an missing token error without a found token.
  ///
  /// This is useful when the parser reaches the end of input missingly.
  /// The error will indicate "missing end of input" in its display message.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
  ///
  /// let error: MissingToken<&str, &str> = MissingToken::new(
  ///     SimpleSpan::new(100, 101),
  ///     Expected::one("}")
  /// );
  /// assert!(error.found().is_none());
  /// assert_eq!(error.span(), SimpleSpan::new(100, 101));
  /// assert_eq!(format!("{}", error), "missing end of input, expected '}'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_expected(offset: O, expected: Expected<'a, Kind>) -> Self {
    Self::new_in(offset, Some(expected), None)
  }

  /// Creates a new missing token error with a single expected token.
  ///
  /// This is a convenience method that combines `new` with `Expected::one`.
  /// The error has no found token, indicating the end of input was reached.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{utils::SimpleSpan, error::token::MissingToken};
  ///
  /// let error: MissingToken<&str, &str> = MissingToken::expected_one(
  ///     SimpleSpan::new(50, 51),
  ///     ";"
  /// );
  /// assert!(error.found().is_none());
  /// assert_eq!(format!("{}", error), "missing end of input, expected ';'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected_one(offset: O, expected: Kind) -> Self {
    Self::with_expected(offset, Expected::one(expected))
  }

  /// Creates a new missing token error with a single expected token.
  ///
  /// This is a convenience method that combines `new` with `Expected::one`.
  /// The error has no found token, indicating the end of input was reached.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{utils::SimpleSpan, error::token::MissingToken};
  ///
  /// let error: MissingToken<&str, &str> = MissingToken::expected_one_with_found(
  ///     SimpleSpan::new(50, 51),
  ///     ":",
  ///     ";"
  /// );
  /// assert!(error.found().is_some());
  /// assert_eq!(format!("{}", error), "missing token ':', expected ';'");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected_one_with_found(offset: O, expected: Kind) -> Self {
    Self::new_in(offset, Some(Expected::one(expected)), None)
  }

  /// Creates a new missing token error with multiple expected tokens.
  ///
  /// This is a convenience method that combines `new` with `Expected::one_of`.
  /// The error has no found token, indicating the end of input was reached.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// use tokit::{utils::SimpleSpan, error::token::MissingToken};
  ///
  /// let error: MissingToken<&str, &str> = MissingToken::expected_one_of(
  ///     SimpleSpan::new(25, 26),
  ///     &["+", "-", "*", "/"]
  /// );
  /// assert!(error.found().is_none());
  /// assert_eq!(
  ///     format!("{}", error),
  ///     "missing end of input, expected one of: '+', '-', '*', '/'"
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected_one_of(offset: O, expected: &'static [Kind]) -> Self {
    Self::with_expected(offset, Expected::one_of(expected))
  }

  /// Creates a new missing token error with multiple expected tokens.
  ///
  /// This is a convenience method that combines `new` with `Expected::one_of`.
  /// The error has no found token, indicating the end of input was reached.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// use tokit::{utils::SimpleSpan, error::token::MissingToken};
  ///
  /// let error: MissingToken<&str, &str> = MissingToken::expected_one_of_with_found(
  ///     SimpleSpan::new(25, 26),
  ///     ":",
  ///     &["+", "-", "*", "/"]
  /// );
  /// assert!(!error.found().is_none());
  /// assert_eq!(
  ///     format!("{}", error),
  ///     "missing token ':', expected one of: '+', '-', '*', '/'"
  /// );
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected_one_of_with_found(offset: O, expected: &'static [Kind]) -> Self {
    Self::new_in(offset, Some(Expected::one_of(expected)), None)
  }

  /// Returns the span of the missing token.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
  ///
  /// let error = MissingToken::expected_one_with_found(
  ///     SimpleSpan::new(10, 15),
  ///     "identifier",
  ///     "number"
  /// );
  /// assert_eq!(error.span(), SimpleSpan::new(10, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset(&self) -> O
  where
    O: Copy,
  {
    self.offset
  }

  /// Returns the span of the missing token.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
  ///
  /// let error = MissingToken::expected_one_with_found(
  ///     10,
  ///     "identifier",
  ///     "number"
  /// );
  /// assert_eq!(error.offset_ref(), &10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset_ref(&self) -> &O {
    &self.offset
  }

  /// Returns the span of the missing token.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
  ///
  /// let error = MissingToken::expected_one_with_found(
  ///     10,
  ///     "identifier",
  ///     "number"
  /// );
  /// assert_eq!(error.offset_mut(), &mut 10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset_mut(&mut self) -> &mut O {
    &mut self.offset
  }

  /// Returns a reference to the custom message, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn message(&self) -> Option<&Message> {
    self.message.as_ref()
  }

  /// Returns a mutable reference to the custom message, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn message_mut(&mut self) -> Option<&mut Message> {
    self.message.as_mut()
  }

  /// Returns a reference to the expected token(s).
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
  ///
  /// let error = MissingToken::expected_one_with_found(
  ///     SimpleSpan::new(5, 6),
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
  /// use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
  ///
  /// let mut error = MissingToken::expected_one_with_found(
  ///     SimpleSpan::new(10, 15),
  ///     "}",
  ///     "{"
  /// );
  /// error.bump(&5);
  /// assert_eq!(error.span(), SimpleSpan::new(15, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &O)
  where
    O: for<'b> AddAssign<&'b O>,
  {
    self.offset += offset;
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
  /// use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
  ///
  /// let error = MissingToken::expected_one_with_found(
  ///    SimpleSpan::new(0, 5),
  ///   "identifier",
  ///   "number"
  /// );
  /// let mapped_error = error.map_expected(|expected| {
  ///     // Transform the expected token type here
  ///     Expected::one(expected.unwrap_one().to_string())
  /// });
  /// # }
  /// ```
  pub fn map_expected<F, Kind2>(self, f: F) -> MissingToken<'a, Kind2, O, Lang>
  where
    F: FnOnce(Expected<'a, Kind>) -> Expected<'a, Kind2>,
    Kind2: Ownable,
  {
    MissingToken {
      offset: self.offset,
      expected: self.expected.map(f),
      message: self.message,
      _lang: PhantomData,
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
  /// use tokit::{utils::{Expected, SimpleSpan}, error::token::MissingToken};
  ///
  /// let error = MissingToken::expected_one_with_found(
  ///     SimpleSpan::new(5, 6),
  ///     "}",
  ///     "{"
  /// );
  /// let (offset, found, expected) = error.into_components();
  /// assert_eq!(offset, 5);
  /// assert_eq!(found, Some("}"));
  /// assert_eq!(expected, Expected::one("{"));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (O, Option<Expected<'a, Kind>>, Option<Message>) {
    (self.offset, self.expected, self.message)
  }
}

impl<'a, Kind: Ownable, O, Lang: ?Sized> From<MissingToken<'a, Kind, O, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: MissingToken<'a, Kind, O, Lang>) -> Self {}
}

impl<Kind: Ownable, O, Lang: ?Sized> MissingToken<'_, Kind, O, Lang> {
  /// Formats the error using the provided formatter in debug style.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn debug_fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
  where
    O: core::fmt::Debug,
    Kind: core::fmt::Debug,
  {
    f.debug_struct("MissingToken")
      .field("offset", &self.offset)
      .field("expected", &self.expected)
      .field("message", &self.message)
      .finish()
  }

  /// Formats the error using the provided formatter in display style.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn display_fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
  where
    O: core::fmt::Display,
    Kind: core::fmt::Display,
  {
    match &self.expected {
      Some(expected) => match &self.message {
        Some(message) => write!(
          f,
          "missing token at {}, expected {}, message: {}",
          self.offset, expected, message
        ),
        None => write!(f, "missing token at {}, expected {}", self.offset, expected),
      },
      None => match &self.message {
        Some(message) => write!(f, "missing token at {}, message: {}", self.offset, message),
        None => write!(f, "missing token at {}", self.offset),
      },
    }
  }
}
