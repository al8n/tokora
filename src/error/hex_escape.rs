//! Hexadecimal escape sequence error types for lexer error reporting.
//!
//! This module provides error types for handling failures in hexadecimal
//! escape sequences during lexical analysis. It supports the `\xXX` format,
//! which requires exactly 2 hexadecimal digits.
//!
//! # Design Philosophy
//!
//! Hexadecimal escape sequences can fail in two ways:
//! - **Syntax errors**: Invalid hex digits, wrong format
//! - **Incompleteness**: Reaching end-of-input mid-escape (fewer than 2 digits)
//!
//! This module distinguishes between:
//! - **Malformed** syntax (invalid hex digits)
//! - **Incomplete** sequences (unexpected EOF or non-hex character)
//!
//! # Hex Escape Format: `\xXX`
//!
//! Hex escapes require exactly 2 hexadecimal digits and can encode:
//! - Any byte value: `\x00` to `\xFF`
//! - Common examples: `\x0A` (newline), `\x09` (tab), `\x20` (space)
//!
//! Common errors:
//! - `\x` - incomplete (no digits)
//! - `\xA` - incomplete (only 1 digit)
//! - `\xGG` - malformed (invalid hex)
//! - `\xZ9` - malformed (first digit invalid)
//!
//! # Error Type Hierarchy
//!
//! ```text
//! HexEscapeError
//! ├─ Incomplete (IncompleteHexEscape)
//! └─ Malformed (MalformedHexEscape)
//! ```
//!
//! # Examples
//!
//! ## Detecting Incomplete Hex Escapes
//!
//! ```
//! use logosky::error::HexEscapeError;
//! use logosky::utils::{Lexeme, Span};
//!
//! // Incomplete: \xA (only 1 digit)
//! let error = HexEscapeError::<char>::incomplete(
//!     Span::new(10, 13) // \xA
//! );
//! assert!(error.is_incomplete());
//! ```
//!
//! ## Detecting Malformed Hex Escapes
//!
//! ```
//! use logosky::error::{HexEscapeError, InvalidHexDigits};
//! use logosky::utils::{Span, PositionedChar};
//!
//! // Invalid hex digit 'G' at position 12
//! let mut digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
//! digits.push(PositionedChar::with_position('G', 13));
//!
//! let error = HexEscapeError::malformed(
//!     digits,
//!     Span::new(10, 14) // \xGG
//! );
//! assert!(error.is_malformed());
//! ```

use crate::{
  error::InvalidHexDigits,
  utils::{Span, human_display::DisplayHuman},
};
use derive_more::{From, IsVariant, TryUnwrap, Unwrap};

/// A type alias for invalid hex digits in hex escape sequences.
pub type InvalidHexEscapeDigits<Char> = InvalidHexDigits<Char, 2>;

/// An incomplete hex escape sequence error.
///
/// This error occurs when a hex escape (`\xXX`) has fewer than 2 hex digits,
/// typically due to unexpected end-of-input or a non-hex character.
///
/// # Examples
///
/// ```
/// use logosky::error::IncompleteHexEscape;
/// use logosky::utils::Span;
///
/// // Incomplete: \xA (only 1 hex digit)
/// let error = IncompleteHexEscape::new(
///     Span::new(10, 13)
/// );
/// assert_eq!(error.span(), Span::new(10, 13));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IncompleteHexEscape(Span);

impl core::fmt::Display for IncompleteHexEscape {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "incomplete hexadecimal escape sequence at {}, hexadecimal escape must contains exactly two hexadecimal digits",
      self.0
    )
  }
}

impl core::error::Error for IncompleteHexEscape {}

impl IncompleteHexEscape {
  /// Creates a new incomplete hex escape error.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::IncompleteHexEscape;
  /// use logosky::utils::Span;
  ///
  /// let error = IncompleteHexEscape::new(Span::new(10, 12));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: Span) -> Self {
    Self(span)
  }

  /// Returns the span of the incomplete hex escape.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::IncompleteHexEscape;
  /// use logosky::utils::Span;
  ///
  /// let error = IncompleteHexEscape::new(Span::new(10, 13));
  /// assert_eq!(error.span(), Span::new(10, 13));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    self.0
  }

  /// Bumps the span or position by `n`.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining spans from different contexts.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::IncompleteHexEscape;
  /// use logosky::utils::Span;
  ///
  /// let mut error = IncompleteHexEscape::new(Span::new(10, 12));
  /// error.bump(5);
  /// assert_eq!(error.span(), Span::new(15, 17));
  /// ```
  #[inline]
  pub const fn bump(&mut self, n: usize) -> &mut Self {
    self.0.bump(n);
    self
  }
}

/// A malformed hex escape sequence error.
///
/// This error occurs when a hex escape (`\xXX`) contains invalid hexadecimal
/// digits. The error captures both the invalid characters encountered and the
/// span of the malformed escape sequence.
///
/// # Type Parameters
///
/// * `Char` - The character type (typically `char` for UTF-8 or `u8` for bytes)
///
/// # Examples
///
/// ```
/// use logosky::error::{MalformedHexEscape, InvalidHexDigits};
/// use logosky::utils::{Span, PositionedChar};
///
/// // Create error for malformed escape like \xGH
/// let mut digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
/// digits.push(PositionedChar::with_position('H', 13));
///
/// let error = MalformedHexEscape::new(
///     digits,
///     Span::new(10, 14) // \xGH
/// );
///
/// assert_eq!(error.span(), Span::new(10, 14));
/// assert!(!error.is_incomplete()); // Only 4 chars total, expected 4 (\xXX)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MalformedHexEscape<Char = char, S = Span> {
  digits: InvalidHexEscapeDigits<Char>,
  span: S,
}

impl<Char> core::fmt::Display for MalformedHexEscape<Char>
where
  Char: DisplayHuman,
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "malformed hexadecimal escape sequence with invalid digits at {}, {}",
      self.span,
      self.digits_ref()
    )
  }
}

impl<Char> core::error::Error for MalformedHexEscape<Char> where
  Char: DisplayHuman + core::fmt::Debug
{
}

impl<Char, S> MalformedHexEscape<Char, S> {
  /// Creates a new malformed hex escape error.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::{MalformedHexEscape, InvalidHexDigits};
  /// use logosky::utils::{Span, PositionedChar};
  ///
  /// let mut digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_positioned_char(PositionedChar::with_position('Z', 12));
  /// let error = MalformedHexEscape::new(digits, Span::new(10, 13));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(digits: InvalidHexEscapeDigits<Char>, span: Span) -> Self {
    Self { digits, span }
  }

  /// Returns `true` if the sequence is also incomplete.
  ///
  /// A hex escape `\xXX` is 4 characters long total.
  /// If the span is shorter, it means the escape was cut off mid-sequence.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::{MalformedHexEscape, InvalidHexDigits};
  /// use logosky::utils::Span;
  ///
  /// let digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(12, 'G');
  /// let error = MalformedHexEscape::new(digits, Span::new(10, 13));
  /// assert!(error.is_incomplete()); // Only 3 chars, not 4
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_incomplete(&self) -> bool {
    self.span.len() < 4 // \x[0-9a-fA-F]{2} is 4 characters long
  }

  /// Returns the invalid hex digits.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::{MalformedHexEscape, InvalidHexDigits};
  /// use logosky::utils::{Span, PositionedChar};
  ///
  /// let mut digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  /// let error = MalformedHexEscape::new(digits, Span::new(10, 13));
  /// assert_eq!(error.digits().len(), 1);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn digits(&self) -> InvalidHexEscapeDigits<Char>
  where
    Char: Clone,
  {
    self.digits.clone()
  }

  /// Returns a reference to the invalid hex digits.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::{MalformedHexEscape, InvalidHexDigits};
  /// use logosky::utils::{Span, PositionedChar};
  ///
  /// let mut digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(12, 'G');
  /// let error = MalformedHexEscape::new(digits, Span::new(10, 13));
  /// assert_eq!(error.digits_ref().len(), 1);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn digits_ref(&self) -> &InvalidHexEscapeDigits<Char> {
    &self.digits
  }

  /// Returns a mutable reference to the invalid hex digits.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn digits_mut(&mut self) -> &mut InvalidHexEscapeDigits<Char> {
    &mut self.digits
  }

  /// Returns the span of the malformed hex escape.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::{MalformedHexEscape, InvalidHexDigits};
  /// use logosky::utils::{Span, PositionedChar};
  ///
  /// let error = MalformedHexEscape::new(
  ///     InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12)),
  ///     Span::new(10, 14)
  /// );
  /// assert_eq!(error.span(), Span::new(10, 14));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    self.span
  }

  /// Returns a reference to the span of the malformed hex escape.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &S {
    &self.span
  }

  /// Returns a mutable reference to the span of the malformed hex escape.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut S {
    &mut self.span
  }

  /// Bumps the span and all digit positions by `n`.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining spans from different contexts.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::{MalformedHexEscape, InvalidHexDigits};
  /// use logosky::utils::{Span, PositionedChar};
  ///
  /// let mut digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  /// let mut error = MalformedHexEscape::new(digits, Span::new(10, 14));
  /// error.bump(5);
  /// assert_eq!(error.span(), Span::new(15, 19));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bump(&mut self, n: usize) -> &mut Self {
    self.span.bump(n);
    self.digits.bump(n);
    self
  }
}

/// An error encountered during lexing for `\xXX` hex escape sequences.
///
/// Hex escapes require exactly 2 hexadecimal digits after `\x`.
/// They can encode any byte value from 0x00 to 0xFF.
///
/// # Variants
///
/// - **Incomplete**: The escape has fewer than 2 hex digits, e.g., `\x`, `\xA`
/// - **Malformed**: The 2 characters are not valid hexadecimal, e.g., `\xGG`, `\xZ9`
///
/// # Examples
///
/// ## Incomplete Escape
///
/// ```
/// use logosky::error::HexEscapeError;
/// use logosky::utils::Span;
///
/// // Incomplete: \xA (only 1 digit)
/// let error = HexEscapeError::<char>::incomplete(
///     Span::new(10, 13)
/// );
/// assert!(error.is_incomplete());
/// ```
///
/// ## Malformed Escape
///
/// ```
/// use logosky::error::{HexEscapeError, InvalidHexDigits};
/// use logosky::utils::{Span, PositionedChar};
///
/// // Invalid hex: \xGG
/// let mut digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
/// digits.push(PositionedChar::with_position('G', 13));
///
/// let error = HexEscapeError::malformed(digits, Span::new(10, 14));
/// assert!(error.is_malformed());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, IsVariant, TryUnwrap, Unwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum HexEscapeError<Char = char> {
  /// An incomplete hex escape sequence.
  ///
  /// This occurs when the escape has fewer than 2 hex digits, typically
  /// due to unexpected end-of-input or a non-hex character.
  ///
  /// Examples: `\x`, `\xA`
  Incomplete(IncompleteHexEscape),

  /// A malformed hex escape sequence.
  ///
  /// This occurs when 2 characters follow `\x` but they are not both
  /// valid hexadecimal digits.
  ///
  /// Examples: `\xGG`, `\xZ9`, `\xAZ`
  Malformed(MalformedHexEscape<Char>),
}

impl<Char> core::fmt::Display for HexEscapeError<Char>
where
  Char: DisplayHuman,
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Incomplete(err) => err.fmt(f),
      Self::Malformed(malformed) => malformed.fmt(f),
    }
  }
}

impl<Char> core::error::Error for HexEscapeError<Char>
where
  Char: DisplayHuman + core::fmt::Debug + 'static,
{
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    match self {
      Self::Incomplete(err) => Some(err),
      Self::Malformed(err) => Some(err),
    }
  }
}

impl<Char, S> HexEscapeError<Char, S> {
  /// Creates an incomplete hex escape error.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::HexEscapeError;
  /// use logosky::utils::{Lexeme, Span};
  ///
  /// let error = HexEscapeError::<char>::incomplete(
  ///     Span::new(10, 12)
  /// );
  /// assert!(error.is_incomplete());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn incomplete(span: Span) -> Self {
    Self::Incomplete(IncompleteHexEscape::new(span))
  }

  /// Creates a malformed hex escape error.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::{HexEscapeError, InvalidHexDigits};
  /// use logosky::utils::Span;
  ///
  /// let mut digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(12, 'G');
  /// let error = HexEscapeError::malformed(digits, Span::new(10, 13));
  /// assert!(error.is_malformed());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn malformed(digits: InvalidHexEscapeDigits<Char>, span: Span) -> Self {
    Self::Malformed(MalformedHexEscape::new(digits, span))
  }

  /// Returns the span of the hex escape error.
  ///
  /// This returns the span where the error occurred, which could be either
  /// the incomplete sequence or the malformed escape sequence.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::HexEscapeError;
  /// use logosky::utils::Span;
  ///
  /// let error = HexEscapeError::<char>::incomplete(Span::new(10, 12));
  /// assert_eq!(error.span(), Span::new(10, 12));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    match self {
      Self::Incomplete(incomplete) => incomplete.span(),
      Self::Malformed(malformed) => malformed.span(),
    }
  }

  /// Bumps the span or position of the error by `n`.
  ///
  /// This is useful when adjusting error positions after processing or
  /// when combining spans from different contexts.
  ///
  /// ## Examples
  ///
  /// ```
  /// use logosky::error::HexEscapeError;
  /// use logosky::utils::{Lexeme, Span};
  ///
  /// let mut error = HexEscapeError::<char>::incomplete(
  ///     Span::new(10, 12)
  /// );
  /// error.bump(5);
  /// // The span is now adjusted
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bump(&mut self, n: usize) -> &mut Self {
    match self {
      Self::Incomplete(incomplete) => {
        incomplete.bump(n);
      }
      Self::Malformed(malformed) => {
        malformed.bump(n);
      }
    }
    self
  }
}
