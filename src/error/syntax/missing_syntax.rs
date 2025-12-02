//! Unexpected token error type for parser error reporting.
//!
//! This module provides the [`MissingSyntax`] type, which represents parser errors
//! when an missing token is encountered. It captures both the location of the error,
//! what token was found (if any), and what tokens were expected.
//!
//! # Design Philosophy
//!
//! `MissingSyntax` is designed to provide rich, actionable error messages:
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
//! use logosky::{utils::Span, error::MissingSyntax};
//!
//! // Simple end-of-input error
//! let error: MissingSyntax<&str, &str> = MissingSyntax::expected_one(
//!     Span::new(100, 100),
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
//! use logosky::{utils::{Expected, Span}, error::MissingSyntax};
//!
//! let error = MissingSyntax::expected_one_with_found(
//!     Span::new(10, 15),
//!     "else",
//!     "if"
//! );
//! assert_eq!(format!("{}", error), "missing token 'else', expected 'if'");
//! ```

use core::{marker::PhantomData, ops::AddAssign};

use crate::{Lexer, utils::Message};

/// A type alias for a `MissingSyntax` error for a given lexer and separator.
pub type MissingSyntaxOf<'inp, Syntax, L, Lang = ()> =
  MissingSyntax<Syntax, <L as Lexer<'inp>>::Offset, Lang>;

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
/// use logosky::{utils::{Expected, Span}, error::MissingSyntax};
///
/// // Error when expecting a specific token but got something else
/// let error = MissingSyntax::expected_one_with_found(
///     Span::new(10, 15),
///     "}",
///     "{"
/// );
/// assert_eq!(error.span(), Span::new(10, 15));
/// assert_eq!(format!("{}", error), "missing token '}', expected '{'");
///
/// // Error when expecting one of multiple tokens
/// let error = MissingSyntax::expected_one_of_with_found(
///     Span::new(0, 10),
///     "identifier",
///     &["if", "while", "for"]
/// );
/// assert_eq!(
///     format!("{}", error),
///     "missing token 'identifier', expected one of: 'if', 'while', 'for'"
/// );
///
/// // Error when reaching end of input missingly
/// let error: MissingSyntax<&str, &str> = MissingSyntax::expected_one(
///     Span::new(100, 100),
///     "}"
/// );
/// assert_eq!(format!("{}", error), "missing end of input, expected '}'");
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MissingSyntax<Syntax, O = usize, Lang: ?Sized = ()> {
  offset: O,
  msg: Option<Message>,
  _syntax: PhantomData<Syntax>,
  _lang: PhantomData<Lang>,
}

impl<Syntax, O> MissingSyntax<Syntax, O> {
  /// Creates a new missing token error.
  ///
  /// This error indicates that an missing token was encountered,
  /// without specifying what token was found or expected.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(offset: O) -> Self {
    Self::of(offset)
  }

  /// Sets a custom message for the error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_message(offset: O, message: Message) -> Self {
    Self::with_message_of(offset, message)
  }
}

impl<Syntax, O, Lang: ?Sized> MissingSyntax<Syntax, O, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(offset: O, message: Option<Message>) -> Self {
    Self {
      offset,
      msg: message,
      _syntax: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Creates a new missing token error for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(offset: O) -> Self {
    Self::new_in(offset, None)
  }

  /// Sets a custom message for the error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_message_of(offset: O, msg: Message) -> Self {
    Self::new_in(offset, Some(msg))
  }

  /// Returns the span of the missing token.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::MissingSyntax};
  ///
  /// let error = MissingSyntax::new(
  ///     10,
  /// );
  /// assert_eq!(error.offset(), 10);
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
  /// use logosky::{utils::{Expected, Span}, error::MissingSyntax};
  ///
  /// let error = MissingSyntax::new(
  ///     10,
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
  /// use logosky::{utils::{Expected, Span}, error::MissingSyntax};
  ///
  /// let error = MissingSyntax::new(
  ///     10,
  /// );
  /// assert_eq!(error.offset_mut(), &mut 10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn offset_mut(&mut self) -> &mut O {
    &mut self.offset
  }

  /// Returns the custom message associated with the error, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn message(&self) -> Option<&Message> {
    self.msg.as_ref()
  }

  /// Returns the custom message associated with the error, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn message_mut(&mut self) -> Option<&mut Message> {
    self.msg.as_mut()
  }

  /// Bumps both the start and end positions of the span by the given offset.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining spans from different contexts.
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::MissingSyntax};
  ///
  /// let mut error = MissingSyntax::expected_one_with_found(
  ///     Span::new(10, 15),
  ///     "}",
  ///     "{"
  /// );
  /// error.bump(5);
  /// assert_eq!(error.span(), Span::new(15, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &O)
  where
    O: for<'b> AddAssign<&'b O>,
  {
    self.offset += offset;
  }

  /// Consumes the error and returns its components.
  ///
  /// This method deconstructs the error into its constituent parts:
  /// the span, the found token (if any), and the expected token(s).
  ///
  /// # Examples
  ///
  /// ```
  /// use logosky::{utils::{Expected, Span}, error::MissingSyntax};
  ///
  /// let error = MissingSyntax::expected_one_with_found(
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
  pub fn into_components(self) -> (O, Option<Message>) {
    (self.offset, self.msg)
  }
}

impl<Syntax, O, Lang> From<MissingSyntax<Syntax, O, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: MissingSyntax<Syntax, O, Lang>) -> Self {}
}
