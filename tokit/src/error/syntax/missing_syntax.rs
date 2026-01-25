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

use core::{marker::PhantomData, ops::AddAssign};

use crate::{Lexer, utils::CowStr};

/// A type alias for a `MissingSyntax` error for a given lexer and separator.
pub type MissingSyntaxOf<'inp, L, Lang = ()> = MissingSyntax<<L as Lexer<'inp>>::Offset, Lang>;

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
/// ```ignore
/// use tokit::{utils::{Expected, SimpleSpan}, error::syntax::MissingSyntax};
///
/// // Error when expecting a specific token but got something else
/// let error = MissingSyntax::expected_one_with_found(
///     SimpleSpan::new(10, 15),
///     "}",
///     "{"
/// );
/// assert_eq!(error.span(), SimpleSpan::new(10, 15));
/// assert_eq!(format!("{}", error), "missing token '}', expected '{'");
///
/// // Error when expecting one of multiple tokens
/// let error = MissingSyntax::expected_one_of_with_found(
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
/// let error: MissingSyntax<&str, &str> = MissingSyntax::expected_one(
///     SimpleSpan::new(100, 100),
///     "}"
/// );
/// assert_eq!(format!("{}", error), "missing end of input, expected '}'");
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MissingSyntax<Offset = usize, Lang: ?Sized = ()> {
  offset: Offset,
  msg: Option<CowStr>,
  _lang: PhantomData<Lang>,
}

impl<O> MissingSyntax<O> {
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
  pub const fn with_message(offset: O, message: CowStr) -> Self {
    Self::with_message_of(offset, message)
  }
}

impl<O, Lang: ?Sized> MissingSyntax<O, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(offset: O, message: Option<CowStr>) -> Self {
    Self {
      offset,
      msg: message,
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
  pub const fn with_message_of(offset: O, msg: CowStr) -> Self {
    Self::new_in(offset, Some(msg))
  }

  /// Returns the span of the missing token.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// struct Lit;
  ///
  /// let error: MissingSyntax<Lit, usize> = MissingSyntax::new(10);
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
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// struct Lit;
  ///
  /// let error: MissingSyntax<Lit, usize> = MissingSyntax::new(
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
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// struct Lit;
  ///
  /// let mut error: MissingSyntax<Lit, usize> = MissingSyntax::new(
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
  pub const fn message(&self) -> Option<&CowStr> {
    self.msg.as_ref()
  }

  /// Returns the custom message associated with the error, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn message_mut(&mut self) -> Option<&mut CowStr> {
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
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// struct Lit;
  ///
  /// let mut error: MissingSyntax<Lit, usize> = MissingSyntax::new(
  ///    10
  /// );
  /// error.bump(&5);
  /// assert_eq!(error.offset(), 15);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &O) -> &mut Self
  where
    O: for<'b> AddAssign<&'b O>,
  {
    self.offset += offset;
    self
  }

  /// Consumes the error and returns its components.
  ///
  /// This method deconstructs the error into its constituent parts:
  /// the span, the found token (if any), and the expected token(s).
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// struct Lit;
  ///
  /// let error: MissingSyntax<Lit, usize> = MissingSyntax::new(
  ///     10,
  /// );
  /// let (offset, msg) = error.into_components();
  /// assert_eq!(offset, 10);
  /// assert_eq!(msg, None);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (O, Option<CowStr>) {
    (self.offset, self.msg)
  }
}

impl<O, Lang> From<MissingSyntax<O, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: MissingSyntax<O, Lang>) -> Self {}
}

impl<O, Lang> MissingSyntax<O, Lang>
where
  O: core::fmt::Debug,
  Lang: ?Sized,
{
  /// Formats the error for debugging purposes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn debug_fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("MissingSyntax")
      .field("offset", &self.offset)
      .field("message", &self.msg)
      .finish()
  }

  /// Formats the error for display purposes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn display_fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
  where
    O: core::fmt::Display,
  {
    match &self.msg {
      Some(msg) => write!(f, "missing syntax at offset {}: {}", self.offset, msg),
      None => write!(f, "missing syntax at offset {}", self.offset),
    }
  }
}
