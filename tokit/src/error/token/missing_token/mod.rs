//! Missing token error type for parser error reporting.
//!
//! This module provides the [`MissingToken`] type, which represents parser errors
//! when a missing token is encountered. It captures both the location of the error,
//! what tokens were expected, and an optional message.
//!
//! # Design Philosophy
//!
//! `MissingToken` is designed to provide rich, actionable error messages:
//! - **Location tracking**: The `offset` field pinpoints exactly where the error occurred
//! - **Flexible expectations**: Can express single or multiple alternative expected tokens
//! - **Position adjustment**: The `bump()` method allows adjusting error positions when
//!   combining errors from different parsing contexts
//!
//! # Common Patterns
//!
//! ## End of Input Errors
//!
//! When the parser reaches the end of input with a missing token, use constructors without a found token:
//!
//! ```
//! use tokit::{SimpleSpan, error::token::MissingToken};
//!
//! // Simple missing token error
//! let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one(
//!     SimpleSpan::new(100, 100),
//!     "}"
//! );
//! assert_eq!(error.offset(), SimpleSpan::new(100, 100));
//! ```
//!
//! ## Unexpected Token Errors
//!
//! When a specific token was found but something else was expected:
//!
//! ```
//! use tokit::{SimpleSpan, utils::Expected, error::token::MissingToken};
//!
//! let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one(
//!     SimpleSpan::new(10, 15),
//!     "else"
//! );
//! assert!(matches!(error.expected(), Some(Expected::One(value)) if *value == "else"));
//! ```

use core::{marker::PhantomData, ops::AddAssign};

use crate::{
  Lexer, Token,
  error::token::{Leading, Trailing},
  utils::{CowStr, Expected},
};

/// A type alias for a `MissingToken` error for a given lexer and separator.
pub type MissingTokenOf<'inp, L, Lang = ()> = MissingToken<
  'inp,
  <<L as Lexer<'inp>>::Token as Token<'inp>>::Kind,
  <L as Lexer<'inp>>::Offset,
  Lang,
>;

/// An error representing a missing token encountered during parsing.
///
/// This error type captures the location (offset) and what token(s) were expected.
/// It's commonly used in parsers to provide
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
/// use tokit::{SimpleSpan, utils::Expected, error::token::MissingToken};
///
/// // Error when expecting a specific token
/// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one(
///     SimpleSpan::new(10, 15),
///     "}"
/// );
/// assert_eq!(error.offset(), SimpleSpan::new(10, 15));
///
/// // Error when expecting one of multiple tokens
/// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one_of(
///     SimpleSpan::new(0, 10),
///     &["if", "while", "for"]
/// );
/// if let Some(Expected::OneOf(values)) = error.expected() {
///     assert_eq!(values.as_slice(), &["if", "while", "for"]);
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MissingToken<'a, Kind: Clone, O = usize, Lang: ?Sized = ()> {
  offset: O,
  expected: Option<Expected<'a, Kind>>,
  message: Option<CowStr>,
  _lang: PhantomData<Lang>,
}

impl<Kind: Clone, O, Data> MissingToken<'_, Kind, O, Trailing<Data>> {
  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing(offset: O) -> Self {
    Self::trailing_of(offset)
  }

  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing_with_message(offset: O, message: CowStr) -> Self {
    Self::trailing_with_message_of(offset, message)
  }
}

impl<Kind: Clone, O, Data, Lang> MissingToken<'_, Kind, O, Leading<Data, Lang>> {
  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading(offset: O) -> Self {
    Self::leading_of(offset)
  }

  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading_with_message(offset: O, message: CowStr) -> Self {
    Self::leading_with_message_of(offset, message)
  }
}

impl<Kind: Clone, O, Data, Lang: ?Sized> MissingToken<'_, Kind, O, Trailing<Data, Lang>> {
  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing_of(offset: O) -> Self {
    Self::of(offset)
  }

  /// Creates a new `MissingToken` error indicating a trailing token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing_with_message_of(offset: O, message: CowStr) -> Self {
    Self::with_message_of(offset, message)
  }
}

impl<Kind: Clone, O, Data, Lang: ?Sized> MissingToken<'_, Kind, O, Leading<Data, Lang>> {
  /// Creates a new `MissingToken` error indicating a leading token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading_of(offset: O) -> Self {
    Self::of(offset)
  }

  /// Creates a new `MissingToken` error indicating a leading token was found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading_with_message_of(offset: O, message: CowStr) -> Self {
    Self::with_message_of(offset, message)
  }
}

impl<Kind: Clone, O> MissingToken<'_, Kind, O> {
  /// Creates a new missing token error.
  ///
  /// This error indicates that a missing token was encountered,
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
  pub const fn with_message(offset: O, message: CowStr) -> Self {
    Self::with_message_of(offset, message)
  }
}

impl<'a, Kind: Clone, O, Lang: ?Sized> MissingToken<'a, Kind, O, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(
    offset: O,
    expected: Option<Expected<'a, Kind>>,
    message: Option<CowStr>,
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
  /// This error indicates that a missing token was encountered,
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
  pub const fn with_message_of(offset: O, message: CowStr) -> Self {
    Self::new_in(offset, None, Some(message))
  }

  /// Creates a missing token error without a found token.
  ///
  /// This is useful when the parser reaches the end of input with a missing token.
  /// The error will indicate "missing end of input" in its display message.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{SimpleSpan, utils::Expected, error::token::MissingToken};
  ///
  /// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::with_expected(
  ///     SimpleSpan::new(100, 101),
  ///     Expected::one("}")
  /// );
  /// assert_eq!(error.offset(), SimpleSpan::new(100, 101));
  /// if let Some(Expected::One(value)) = error.expected() {
  ///     assert_eq!(*value, "}");
  /// }
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
  /// use tokit::{SimpleSpan, error::token::MissingToken};
  ///
  /// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one(
  ///     SimpleSpan::new(50, 51),
  ///     ";"
  /// );
  /// assert_eq!(error.offset(), SimpleSpan::new(50, 51));
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
  /// use tokit::{SimpleSpan, error::token::MissingToken};
  ///
  /// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one_with_found(
  ///     SimpleSpan::new(50, 51),
  ///     ";"
  /// );
  /// assert_eq!(error.offset(), SimpleSpan::new(50, 51));
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
  /// use tokit::{SimpleSpan, error::token::MissingToken};
  ///
  /// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one_of(
  ///     SimpleSpan::new(25, 26),
  ///     &["+", "-", "*", "/"]
  /// );
  /// assert_eq!(error.offset(), SimpleSpan::new(25, 26));
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
  /// use tokit::{SimpleSpan, error::token::MissingToken};
  ///
  /// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one_of_with_found(
  ///     SimpleSpan::new(25, 26),
  ///     &["+", "-", "*", "/"]
  /// );
  /// assert_eq!(error.offset(), SimpleSpan::new(25, 26));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected_one_of_with_found(offset: O, expected: &'static [Kind]) -> Self {
    Self::new_in(offset, Some(Expected::one_of(expected)), None)
  }

  /// Returns the offset of the missing token.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{SimpleSpan, error::token::MissingToken};
  ///
  /// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one(
  ///     SimpleSpan::new(10, 15),
  ///     "identifier"
  /// );
  /// assert_eq!(error.offset(), SimpleSpan::new(10, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset(&self) -> O
  where
    O: Copy,
  {
    self.offset
  }

  /// Returns the offset of the missing token.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::error::token::MissingToken;
  ///
  /// let error: MissingToken<'_, &str> = MissingToken::expected_one(10, "identifier");
  /// assert_eq!(error.offset_ref(), &10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset_ref(&self) -> &O {
    &self.offset
  }

  /// Returns the offset of the missing token.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::error::token::MissingToken;
  ///
  /// let mut error: MissingToken<'_, &str> = MissingToken::expected_one(10, "identifier");
  /// *error.offset_mut() = 12;
  /// assert_eq!(error.offset(), 12);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset_mut(&mut self) -> &mut O {
    &mut self.offset
  }

  /// Returns a reference to the custom message, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn message(&self) -> Option<&CowStr> {
    self.message.as_ref()
  }

  /// Returns a mutable reference to the custom message, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn message_mut(&mut self) -> Option<&mut CowStr> {
    self.message.as_mut()
  }

  /// Returns a reference to the expected token(s).
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{SimpleSpan, utils::Expected, error::token::MissingToken};
  ///
  /// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one(SimpleSpan::new(5, 6), "}");
  /// assert!(matches!(error.expected(), Some(Expected::One(value)) if *value == "}"));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn expected(&self) -> Option<&Expected<'a, Kind>> {
    self.expected.as_ref()
  }

  /// Bumps the offset by the given amount.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining offsets from different contexts.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::error::token::MissingToken;
  ///
  /// let mut error: MissingToken<'_, &str> = MissingToken::expected_one(10, "}");
  /// error.bump(&5);
  /// assert_eq!(error.offset(), 15);
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
  /// use tokit::{utils::Expected, error::token::MissingToken};
  ///
  /// let error: MissingToken<'_, &str> = MissingToken::expected_one(0, "identifier");
  /// let mapped_error = error.map_expected(|expected| {
  ///     // Transform the expected token type here
  ///     Expected::one(expected.unwrap_one().to_string())
  /// });
  /// # }
  /// ```
  pub fn map_expected<F, Kind2>(self, f: F) -> MissingToken<'a, Kind2, O, Lang>
  where
    F: FnOnce(Expected<'a, Kind>) -> Expected<'a, Kind2>,
    Kind2: Clone,
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
  /// the offset, expected token(s), and optional message.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::{SimpleSpan, utils::Expected, error::token::MissingToken};
  ///
  /// let error: MissingToken<'_, &str, SimpleSpan> = MissingToken::expected_one(SimpleSpan::new(5, 6), "}");
  /// let (offset, expected, message) = error.into_components();
  /// assert_eq!(offset, SimpleSpan::new(5, 6));
  /// assert_eq!(expected, Some(Expected::one("}")));
  /// assert_eq!(message, None);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (O, Option<Expected<'a, Kind>>, Option<CowStr>) {
    (self.offset, self.expected, self.message)
  }
}

impl<'a, Kind: Clone, O, Lang: ?Sized> From<MissingToken<'a, Kind, O, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: MissingToken<'a, Kind, O, Lang>) -> Self {}
}

impl<Kind: Clone, O, Lang: ?Sized> MissingToken<'_, Kind, O, Lang> {
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
