//! Missing syntax error type for parser error reporting.
//!
//! This module provides the [`MissingSyntax`] type, which represents parser errors
//! when a missing token is encountered. It captures the offset of the error and
//! an optional custom message.
//!
//! # Design Philosophy
//!
//! `MissingSyntax` is designed to provide rich, actionable error messages:
//! - **Location tracking**: The `offset` field pinpoints exactly where the error occurred
//! - **Optional message**: Attach custom context to the missing syntax
//! - **Position adjustment**: The `bump()` method allows adjusting error positions when
//!   combining errors from different parsing contexts

use core::{marker::PhantomData, ops::AddAssign};

use crate::{Lexer, utils::CowStr};

/// A type alias for a `MissingSyntax` error for a given lexer and separator.
pub type MissingSyntaxOf<'inp, L, Lang = ()> = MissingSyntax<<L as Lexer<'inp>>::Offset, Lang>;

/// An error representing a missing token encountered during parsing.
///
/// This error type captures the location (offset) and an optional message.
/// It's commonly used in parsers to provide
/// detailed error messages when the input doesn't match the expected syntax.
///
/// # Type Parameters
///
/// # Examples
///
/// ```ignore
/// use tokit::error::syntax::MissingSyntax;
///
/// // Basic missing syntax error with no message
/// let error = MissingSyntax::new(10);
/// assert_eq!(error.offset(), 10);
///
/// // Attach a custom message
/// let error = MissingSyntax::with_message(20, "expected expression".into());
/// assert_eq!(error.message().unwrap(), "expected expression");
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
  /// This error indicates that a missing token was encountered,
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

  /// Returns the offset of the missing token.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// let error = MissingSyntax::new(10);
  /// assert_eq!(error.offset(), 10);
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
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// let error = MissingSyntax::new(10);
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
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// let mut error = MissingSyntax::new(10);
  /// *error.offset_mut() = 12;
  /// assert_eq!(error.offset(), 12);
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

  /// Bumps the offset by the given amount.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining offsets from different contexts.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// let mut error = MissingSyntax::new(10);
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
  /// the offset and the optional message.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokit::error::syntax::MissingSyntax;
  ///
  /// let error = MissingSyntax::new(10);
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

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;

  use std::format;

  #[test]
  fn missing_syntax_new() {
    let err = MissingSyntax::new(10usize);
    assert_eq!(err.offset(), 10);
    assert!(err.message().is_none());
  }

  #[test]
  fn missing_syntax_with_message() {
    let err = MissingSyntax::with_message(20, CowStr::from_static("expected expression"));
    assert_eq!(err.offset(), 20);
    assert_eq!(err.message().unwrap().as_str(), "expected expression");
  }

  #[test]
  fn missing_syntax_offset_ref() {
    let err = MissingSyntax::new(15usize);
    assert_eq!(err.offset_ref(), &15);
  }

  #[test]
  fn missing_syntax_offset_mut() {
    let mut err = MissingSyntax::new(10usize);
    *err.offset_mut() = 20;
    assert_eq!(err.offset(), 20);
  }

  #[test]
  fn missing_syntax_message_mut() {
    let mut err = MissingSyntax::with_message(10, CowStr::from_static("original"));
    if let Some(msg) = err.message_mut() {
      *msg = CowStr::from_static("updated");
    }
    assert_eq!(err.message().unwrap().as_str(), "updated");
  }

  #[test]
  fn missing_syntax_bump() {
    let mut err = MissingSyntax::new(10usize);
    err.bump(&5);
    assert_eq!(err.offset(), 15);
  }

  #[test]
  fn missing_syntax_into_components() {
    let err = MissingSyntax::new(10usize);
    let (offset, msg) = err.into_components();
    assert_eq!(offset, 10);
    assert!(msg.is_none());
  }

  #[test]
  fn missing_syntax_into_components_with_message() {
    let err = MissingSyntax::with_message(20, CowStr::from_static("test"));
    let (offset, msg) = err.into_components();
    assert_eq!(offset, 20);
    assert_eq!(msg.unwrap().as_str(), "test");
  }

  #[test]
  fn missing_syntax_into_unit() {
    let err = MissingSyntax::new(10usize);
    let _: () = err.into();
  }

  #[test]
  fn missing_syntax_of_with_lang() {
    struct MyLang;
    let err = MissingSyntax::<usize, MyLang>::of(10);
    assert_eq!(err.offset(), 10);
  }

  #[test]
  fn missing_syntax_display_fmt_no_message() {
    let err = MissingSyntax::new(10usize);
    let msg = format!("{}", DisplayWrapper(&err));
    assert_eq!(msg, "missing syntax at offset 10");
  }

  #[test]
  fn missing_syntax_display_fmt_with_message() {
    let err = MissingSyntax::with_message(20usize, CowStr::from_static("expected ident"));
    let msg = format!("{}", DisplayWrapper(&err));
    assert_eq!(msg, "missing syntax at offset 20: expected ident");
  }

  #[test]
  fn missing_syntax_debug_fmt() {
    let err = MissingSyntax::new(10usize);
    let msg = format!("{}", DebugWrapper(&err));
    assert!(msg.contains("MissingSyntax"));
    assert!(msg.contains("10"));
  }

  struct DisplayWrapper<'a>(&'a MissingSyntax<usize>);
  impl core::fmt::Display for DisplayWrapper<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      self.0.display_fmt(f)
    }
  }

  struct DebugWrapper<'a>(&'a MissingSyntax<usize>);
  impl core::fmt::Display for DebugWrapper<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      self.0.debug_fmt(f)
    }
  }
}
