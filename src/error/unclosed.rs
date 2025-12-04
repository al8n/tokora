//! Unclosed delimiter error type for tracking missing closing delimiters.
//!
//! This module provides the [`Unclosed`] type for representing errors where an opening
//! delimiter was found but never closed before reaching end-of-input or another syntactic
//! boundary.
//!
//! # Design Philosophy
//!
//! When parsing structured text with paired delimiters (parentheses, brackets, braces,
//! quotes, etc.), it's common to encounter situations where an opening delimiter is found
//! but the corresponding closing delimiter is missing. This error type captures both:
//!
//! - **Where** the opening delimiter was found (via [`Span`])
//! - **What** delimiter was left unclosed (via the generic `Delimiter` parameter)
//!
//! # Unclosed vs Unterminated
//!
//! - **`Unclosed`**: For **paired delimiters** that have distinct opening and closing forms
//!   - Examples: `(...)`, `[...]`, `{...}`, `"..."`, `/*...*/`
//!   - The span points to the **opening delimiter** position
//!   - Used when you expect a matching closing delimiter
//!
//! - **`Unterminated`**: For **sequences or operators** that need completion
//!   - Examples: GraphQL's `...` spread operator (where `.` or `..` is incomplete)
//!   - The span points to the **incomplete sequence** position
//!   - Used when you expect more characters to complete a construct
//!
//! # Type Parameter
//!
//! - `Delimiter`: The type representing the delimiter (typically `char`, `&'static str`, or a custom enum)
//!
//! # Examples
//!
//! ## Basic Usage with Character Delimiters
//!
//! ```rust
//! use logosky::{error::Unclosed, utils::Span};
//!
//! // Opening parenthesis at position 10, never closed
//! let error = Unclosed::new(Span::new(10, 11), '(');
//!
//! assert_eq!(error.span(), Span::new(10, 11));
//! assert_eq!(error.delimiter(), '(');
//! assert_eq!(error.to_string(), "unclosed delimiter '('");
//! ```
//!
//! ## Custom Delimiter Enum
//!
//! ```rust
//! use logosky::{error::Unclosed, utils::Span};
//! use core::fmt;
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum Delimiter {
//!     Paren,      // ()
//!     Bracket,    // []
//!     Brace,      // {}
//!     Quote,      // ""
//!     BlockComment, // /**/
//! }
//!
//! impl fmt::Display for Delimiter {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         match self {
//!             Self::Paren => write!(f, "("),
//!             Self::Bracket => write!(f, "["),
//!             Self::Brace => write!(f, "{{"),
//!             Self::Quote => write!(f, "\""),
//!             Self::BlockComment => write!(f, "/*"),
//!         }
//!     }
//! }
//!
//! let error = Unclosed::new(Span::new(5, 6), Delimiter::BlockComment);
//! assert_eq!(error.to_string(), "unclosed delimiter '/*'");
//! ```
//!
//! ## Tracking Nested Delimiters
//!
//! ```rust
//! use logosky::{error::Unclosed, utils::Span};
//!
//! // When parsing: "{ foo: [ bar, baz }"
//! // The '[' at position 7 is never closed
//! let error = Unclosed::new(Span::new(7, 8), '[');
//!
//! // Error reporting can show:
//! // "error at 7..8: unclosed delimiter '['"
//! ```
//!
//! ## Position Adjustment
//!
//! ```rust
//! use logosky::{error::Unclosed, utils::Span};
//!
//! // Error from a nested parsing context
//! let mut error = Unclosed::new(Span::new(5, 6), '{');
//!
//! // Adjust to absolute position in the larger document
//! error.bump(100);
//! assert_eq!(error.span(), Span::new(105, 106));
//! ```

use core::marker::PhantomData;

use crate::{
  punct::{Angle, Brace, Bracket, Paren},
  utils::Span,
};

/// A unclosed bracket error
pub type UnclosedBracket<S = Span, Lang = ()> = Unclosed<Bracket, S, Lang>;
/// A unclosed parenthesis error
pub type UnclosedParen<S = Span, Lang = ()> = Unclosed<Paren, S, Lang>;
/// A unclosed brace error
pub type UnclosedBrace<S = Span, Lang = ()> = Unclosed<Brace, S, Lang>;
/// A unclosed angle bracket error
pub type UnclosedAngle<S = Span, Lang = ()> = Unclosed<Angle, S, Lang>;

/// A zero-copy error type representing an unclosed delimiter.
///
/// This type tracks the position of an opening delimiter that was never closed,
/// enabling precise error reporting for missing closing delimiters in structured text.
///
/// # Type Parameter
///
/// - `Delimiter`: The type representing the delimiter (typically `char`, `&'static str`,
///   or a custom enum). Must implement `Display` for error messages.
///
/// # Common Use Cases
///
/// - **Unmatched parentheses** in expressions: `(a + b * c`
/// - **Unclosed strings** in source code: `"hello world`
/// - **Missing closing braces** in JSON: `{"key": "value"`
/// - **Incomplete block comments**: `/* This comment never ends`
/// - **Unmatched brackets** in arrays: `[1, 2, 3`
///
/// # Design
///
/// The span points to the **opening delimiter** position, not where the closing
/// delimiter was expected. This allows error messages to point users to where
/// the delimiter was opened, making it easier to find and fix the issue.
///
/// # Examples
///
/// ## Detecting Unclosed Parentheses
///
/// ```rust
/// use logosky::{error::Unclosed, utils::Span};
///
/// // Parse error: (1 + 2
/// //              ^--- unclosed
/// let error = Unclosed::new(Span::new(0, 1), '(');
///
/// println!("Error: {} at position {}", error, error.span().start());
/// // Output: "Error: unclosed delimiter '(' at position 0"
/// ```
///
/// ## Tracking Multiple Unclosed Delimiters
///
/// ```rust
/// use logosky::{error::Unclosed, utils::Span};
///
/// let errors = vec![
///     Unclosed::new(Span::new(5, 6), '{'),
///     Unclosed::new(Span::new(10, 11), '['),
///     Unclosed::new(Span::new(15, 16), '('),
/// ];
///
/// for error in errors {
///     eprintln!("Unclosed {} at {}", error.delimiter(), error.span());
/// }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Unclosed<Delimiter, S = Span, Lang: ?Sized = ()> {
  span: S,
  delimiter: Delimiter,
  _lang: PhantomData<Lang>,
}

impl<Delimiter, O, Lang: ?Sized> core::fmt::Display for Unclosed<Delimiter, O, Lang>
where
  Delimiter: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "unclosed delimiter '{}'", self.delimiter)
  }
}

impl<Delimiter, O, Lang: ?Sized> core::error::Error for Unclosed<Delimiter, O, Lang>
where
  Delimiter: core::fmt::Display + core::fmt::Debug,
  O: core::fmt::Debug,
  Lang: core::fmt::Debug,
{
}

impl<S> Unclosed<Paren, S> {
  /// Creates a new unclosed parenthesis error.
  ///
  /// The span should cover the opening parenthesis to the end of the syntax element.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::{Span, delimiter::Paren}};
  ///
  /// // Opening parenthesis at position 3
  /// let error = Unclosed::paren(Span::new(3, 4));
  /// assert_eq!(error.span(), Span::new(3, 4));
  /// assert_eq!(error.delimiter(), Paren);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn paren(span: S) -> Self {
    Self::paren_of(span)
  }
}

impl<S, Lang: ?Sized> Unclosed<Paren, S, Lang> {
  /// Creates a new unclosed parenthesis error.
  ///
  /// The span should cover the opening parenthesis to the end of the syntax element.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::{Span, delimiter::Paren}};
  ///
  /// // Opening parenthesis at position 3
  /// let error = Unclosed::paren(Span::new(3, 4));
  /// assert_eq!(error.span(), Span::new(3, 4));
  /// assert_eq!(error.delimiter(), Paren);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn paren_of(span: S) -> Self {
    Self::of(span, Paren::PHANTOM)
  }
}

impl<S> Unclosed<Bracket, S> {
  /// Creates a new unclosed bracket error.
  ///
  /// The span should point to the position of the opening bracket.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::{Span, delimiter::Bracket}};
  ///
  /// // Opening bracket at position 8
  /// let error = Unclosed::bracket(Span::new(8, 9));
  /// assert_eq!(error.span(), Span::new(8, 9));
  /// assert_eq!(error.delimiter(), Bracket);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bracket(span: S) -> Self {
    Self::bracket_of(span)
  }
}

impl<S, Lang: ?Sized> Unclosed<Bracket, S, Lang> {
  /// Creates a new unclosed bracket error.
  ///
  /// The span should point to the position of the opening bracket.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::{Span, delimiter::Bracket}};
  ///
  /// // Opening bracket at position 8
  /// let error = Unclosed::bracket(Span::new(8, 9));
  /// assert_eq!(error.span(), Span::new(8, 9));
  /// assert_eq!(error.delimiter(), Bracket);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bracket_of(span: S) -> Self {
    Self::of(span, Bracket::PHANTOM)
  }
}

impl<S> Unclosed<Brace, S> {
  /// Creates a new unclosed brace error.
  ///
  /// The span should point to the position of the opening brace.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::{Span, delimiter::Brace}};
  ///
  /// // Opening brace at position 12
  /// let error = Unclosed::brace(Span::new(12, 13));
  /// assert_eq!(error.span(), Span::new(12, 13));
  /// assert_eq!(error.delimiter(), Brace);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn brace(span: S) -> Self {
    Self::brace_of(span)
  }
}

impl<S, Lang: ?Sized> Unclosed<Brace, S, Lang> {
  /// Creates a new unclosed brace error.
  ///
  /// The span should point to the position of the opening brace.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::{Span, delimiter::Brace}};
  ///
  /// // Opening brace at position 12
  /// let error = Unclosed::brace(Span::new(12, 13));
  /// assert_eq!(error.span(), Span::new(12, 13));
  /// assert_eq!(error.delimiter(), Brace);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn brace_of(span: S) -> Self {
    Self::of(span, Brace::PHANTOM)
  }
}

impl<S> Unclosed<Angle, S> {
  /// Creates a new unclosed angle bracket error.
  ///
  /// The span should point to the position of the opening angle bracket.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::{Span, delimiter::Angle}};
  ///
  /// // Opening angle bracket at position 20
  /// let error = Unclosed::angle(Span::new(20, 21));
  /// assert_eq!(error.span(), Span::new(20, 21));
  /// assert_eq!(error.delimiter(), Angle);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn angle(span: S) -> Self {
    Self::angle_of(span)
  }
}

impl<S, Lang: ?Sized> Unclosed<Angle, S, Lang> {
  /// Creates a new unclosed angle bracket error.
  ///
  /// The span should point to the position of the opening angle bracket.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::{Span, delimiter::Angle}};
  ///
  /// // Opening angle bracket at position 20
  /// let error = Unclosed::angle(Span::new(20, 21));
  /// assert_eq!(error.span(), Span::new(20, 21));
  /// assert_eq!(error.delimiter(), Angle);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn angle_of(span: S) -> Self {
    Self::of(span, Angle::PHANTOM)
  }
}

impl<Delimiter, S> Unclosed<Delimiter, S> {
  /// Creates a new unclosed delimiter error.
  ///
  /// The span should point to the position of the opening delimiter.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::Span};
  ///
  /// // Opening brace at position 5
  /// let error = Unclosed::new(Span::new(5, 6), '{');
  /// assert_eq!(error.span(), Span::new(5, 6));
  /// assert_eq!(error.delimiter(), '{');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, delimiter: Delimiter) -> Self {
    Self::of(span, delimiter)
  }
}

impl<Delimiter, S, Lang: ?Sized> Unclosed<Delimiter, S, Lang> {
  /// Creates a new unclosed delimiter error.
  ///
  /// The span should point to the position of the opening delimiter.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::Span};
  ///
  /// // Opening brace at position 5
  /// let error = Unclosed::of(Span::new(5, 6), '{');
  /// assert_eq!(error.span(), Span::new(5, 6));
  /// assert_eq!(error.delimiter(), '{');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(span: S, delimiter: Delimiter) -> Self {
    Self {
      span,
      delimiter,
      _lang: PhantomData,
    }
  }

  /// Returns the span of the opening delimiter.
  ///
  /// This is the position where the delimiter was opened but never closed.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::Span};
  ///
  /// let error = Unclosed::new(S::new(10, 11), '(');
  /// assert_eq!(error.span(), Span::new(10, 11));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    self.span
  }

  /// Returns a reference to the span of the opening delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &S {
    &self.span
  }

  /// Returns a mutable reference to the span of the opening delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut S {
    &mut self.span
  }

  /// Returns a reference to the unclosed delimiter.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::Span};
  ///
  /// let error = Unclosed::new(Span::new(5, 6), '{');
  /// assert_eq!(error.delimiter_ref(), &'{');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimiter_ref(&self) -> &Delimiter {
    &self.delimiter
  }

  /// Returns the unclosed delimiter.
  ///
  /// This method is only available when the delimiter type implements `Copy`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::Span};
  ///
  /// let error = Unclosed::new(Span::new(5, 6), '[');
  /// assert_eq!(error.delimiter(), '[');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimiter(&self) -> Delimiter
  where
    Delimiter: Copy,
  {
    self.delimiter
  }

  /// Bumps the span by the given offset.
  ///
  /// This adjusts both the start and end positions of the span, which is useful
  /// when adjusting error positions after processing or when combining errors
  /// from different parsing contexts.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::Span};
  ///
  /// let mut error = Unclosed::new(Span::new(5, 6), '(');
  /// error.bump(100);
  /// assert_eq!(error.span(), Span::new(105, 106));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &S::Offset) -> &mut Self
  where
    S: crate::Span,
  {
    self.span.bump(offset);
    self
  }

  /// Consumes the error and returns its components.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unclosed, utils::Span};
  ///
  /// let error = Unclosed::new(Span::new(10, 11), '"');
  /// let (span, delimiter) = error.into_components();
  /// assert_eq!(span, Span::new(10, 11));
  /// assert_eq!(delimiter, '"');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (S, Delimiter) {
    (self.span, self.delimiter)
  }
}
