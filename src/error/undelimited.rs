//! Undelimited error type for tracking missing delimiter pairs.
//!
//! This module provides the [`Undelimited`] type for representing errors where content
//! was expected to be delimited but **both** opening and closing delimiters are missing.
//!
//! # Design Philosophy
//!
//! When parsing structured text with paired delimiters (parentheses, brackets, braces,
//! quotes, etc.), it's common to encounter situations where content is expected to be
//! delimited but neither the opening nor closing delimiter is present. This error type captures:
//!
//! - **Where** the undelimited content was found (via [`Span`])
//! - **What** delimiter type was expected (via the generic `Delimiter` parameter)
//!
//! # Undelimited vs Unclosed vs Unopened
//!
//! - **`Undelimited`**: For content missing **both** opening and closing delimiters
//!   - Examples: Expected `"hello"` but got just `hello`, expected `{...}` but got just `...`
//!   - The span points to the **undelimited content** position
//!   - Used when you expect delimiters around content but find neither
//!
//! - **`Unclosed`**: For opening delimiters found **without** a matching closing delimiter
//!   - Examples: `(a + b`, `[foo`, `{bar`
//!   - The span points to the **opening delimiter** position
//!   - Used when you find an opening delimiter that was never closed
//!
//! - **`Unopened`**: For closing delimiters found **without** a matching opening delimiter
//!   - Examples: `a + b)`, `foo]`, `bar}`
//!   - The span points to the **closing delimiter** position
//!   - Used when you find a closing delimiter that was never opened
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
//! use logosky::{error::Undelimited, utils::Span};
//!
//! // Expected string literal with quotes, but found undelimited content
//! // Example: hello instead of "hello"
//! let error = Undelimited::new(Span::new(10, 15), '"');
//!
//! assert_eq!(error.span(), Span::new(10, 15));
//! assert_eq!(error.delimiter(), '"');
//! assert_eq!(error.to_string(), "undelimited content, expected '\"'");
//! ```
//!
//! ## Custom Delimiter Enum
//!
//! ```rust
//! use logosky::{error::Undelimited, utils::Span};
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
//! let error = Undelimited::new(Span::new(5, 10), Delimiter::BlockComment);
//! assert_eq!(error.to_string(), "undelimited content, expected '/*'");
//! ```
//!
//! ## Undelimited Content in Parsing
//!
//! ```rust
//! use logosky::{error::Undelimited, utils::Span};
//!
//! // When parsing a configuration file expecting bracketed arrays
//! // but finding: items = foo, bar, baz (missing the brackets)
//! let error = Undelimited::new(Span::new(9, 20), '[');
//!
//! // Error reporting can show:
//! // "error at 9..20: undelimited content, expected '['"
//! ```
//!
//! ## Position Adjustment
//!
//! ```rust
//! use logosky::{error::Undelimited, utils::Span};
//!
//! // Error from a nested parsing context
//! let mut error = Undelimited::new(Span::new(5, 6), '}');
//!
//! // Adjust to absolute position in the larger document
//! error.bump(100);
//! assert_eq!(error.span(), Span::new(105, 106));
//! ```

use crate::{
  punct::{Angle, Brace, Bracket, Paren},
  utils::Span,
};
use core::marker::PhantomData;

/// Content missing both opening `[` and closing `]`
pub type UndelimitedBracket<S = Span, Lang = ()> = Undelimited<Bracket, S, Lang>;
/// Content missing both opening `(` and closing `)`
pub type UndelimitedParen<S = Span, Lang = ()> = Undelimited<Paren, S, Lang>;
/// Content missing both opening `{` and closing `}`
pub type UndelimitedBrace<S = Span, Lang = ()> = Undelimited<Brace, S, Lang>;
/// Content missing both opening `<` and closing `>`
pub type UndelimitedAngle<S = Span, Lang = ()> = Undelimited<Angle, S, Lang>;

/// A zero-copy error type representing undelimited content.
///
/// This type tracks the position of content that should have been delimited by a pair
/// of delimiters (opening and closing) but neither delimiter was present.
///
/// # Type Parameter
///
/// - `Delimiter`: The type representing the delimiter (typically `char`, `&'static str`,
///   or a custom enum). Must implement `Display` for error messages.
///
/// # Common Use Cases
///
/// - **Unquoted strings**: Expected `"hello"` but found just `hello`
/// - **Missing array brackets**: Expected `[1, 2, 3]` but found just `1, 2, 3`
/// - **Missing braces**: Expected `{key: value}` but found just `key: value`
/// - **Undelimited expressions**: Expected `(a + b)` but found just `a + b`
/// - **Missing block delimiters**: Expected `/* comment */` but found just `comment`
///
/// # Design
///
/// The span points to the **undelimited content** position, indicating where delimiters
/// were expected but not found. This allows error messages to highlight the content
/// that should have been wrapped in delimiters.
///
/// # Examples
///
/// ## Detecting Undelimited Content
///
/// ```rust
/// use logosky::{error::Undelimited, utils::Span};
///
/// // Parse error: expected string literal "hello" but found hello
/// //              ^^^^^--- undelimited content
/// let error = Undelimited::new(Span::new(0, 5), '"');
///
/// println!("Error: {} at position {}", error, error.span().start());
/// // Output: "Error: undelimited content, expected '\"' at position 0"
/// ```
///
/// ## Tracking Multiple Undelimited Regions
///
/// ```rust
/// use logosky::{error::Undelimited, utils::Span};
///
/// let errors = vec![
///     Undelimited::new(Span::new(5, 10), '{'),   // Missing braces
///     Undelimited::new(Span::new(15, 20), '['),  // Missing brackets
///     Undelimited::new(Span::new(25, 30), '"'),  // Missing quotes
/// ];
///
/// for error in errors {
///     eprintln!("Undelimited content at {}, expected {}", error.span(), error.delimiter());
/// }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Undelimited<Delimiter, S = Span, Lang: ?Sized = ()> {
  span: S,
  delimiter: Delimiter,
  _lang: PhantomData<Lang>,
}

impl<Delimiter, S, Lang: ?Sized> core::fmt::Display for Undelimited<Delimiter, S, Lang>
where
  Delimiter: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "undelimited content, expected '{}'", self.delimiter)
  }
}

impl<Delimiter, S, Lang: ?Sized> core::error::Error for Undelimited<Delimiter, S, Lang>
where
  Delimiter: core::fmt::Display + core::fmt::Debug,
  S: core::fmt::Debug,
  Lang: core::fmt::Debug,
{
}

impl<S> Undelimited<Paren, S> {
  /// Creates a new undelimited content error for missing parentheses.
  ///
  /// The span should point to the position of the content that should have been
  /// wrapped in parentheses but wasn't.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::{Span, delimiter::Paren}};
  ///
  /// // Undelimited content from position 3 to 7
  /// let error = Undelimited::paren(Span::new(3, 7));
  /// assert_eq!(error.span(), Span::new(3, 7));
  /// assert_eq!(error.delimiter(), Paren);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn paren(span: S) -> Self {
    Self::paren_of(span)
  }
}

impl<S, Lang: ?Sized> Undelimited<Paren, S, Lang> {
  /// Creates a new undelimited content error for missing parentheses.
  ///
  /// The span should point to the position of the content that should have been
  /// wrapped in parentheses but wasn't.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::{Span, delimiter::Paren}};
  ///
  /// // Undelimited content from position 3 to 7
  /// let error = Undelimited::paren_of(Span::new(3, 7));
  /// assert_eq!(error.span(), Span::new(3, 7));
  /// assert_eq!(error.delimiter(), Paren);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn paren_of(span: S) -> Self {
    Self::of(span, Paren::PHANTOM)
  }
}

impl<S> Undelimited<Bracket, S> {
  /// Creates a new undelimited content error for missing brackets.
  ///
  /// The span should point to the position of the content that should have been
  /// wrapped in brackets but wasn't.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::{Span, delimiter::Bracket}};
  ///
  /// // Undelimited content from position 8 to 15
  /// let error = Undelimited::bracket(Span::new(8, 15));
  /// assert_eq!(error.span(), Span::new(8, 15));
  /// assert_eq!(error.delimiter(), Bracket);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bracket(span: S) -> Self {
    Self::bracket_of(span)
  }
}

impl<S, Lang: ?Sized> Undelimited<Bracket, S, Lang> {
  /// Creates a new undelimited content error for missing brackets.
  ///
  /// The span should point to the position of the content that should have been
  /// wrapped in brackets but wasn't.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::{Span, delimiter::Bracket}};
  ///
  /// // Undelimited content from position 8 to 15
  /// let error = Undelimited::bracket_of(Span::new(8, 15));
  /// assert_eq!(error.span(), Span::new(8, 15));
  /// assert_eq!(error.delimiter(), Bracket);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bracket_of(span: S) -> Self {
    Self::of(span, Bracket::PHANTOM)
  }
}

impl<S> Undelimited<Brace, S> {
  /// Creates a new undelimited content error for missing braces.
  ///
  /// The span should point to the position of the content that should have been
  /// wrapped in braces but wasn't.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::{Span, delimiter::Brace}};
  ///
  /// // Undelimited content from position 12 to 20
  /// let error = Undelimited::brace(Span::new(12, 20));
  /// assert_eq!(error.span(), Span::new(12, 20));
  /// assert_eq!(error.delimiter(), Brace);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn brace(span: S) -> Self {
    Self::brace_of(span)
  }
}

impl<S, Lang: ?Sized> Undelimited<Brace, S, Lang> {
  /// Creates a new undelimited content error for missing braces.
  ///
  /// The span should point to the position of the content that should have been
  /// wrapped in braces but wasn't.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::{Span, delimiter::Brace}};
  ///
  /// // Undelimited content from position 12 to 20
  /// let error = Undelimited::brace_of(Span::new(12, 20));
  /// assert_eq!(error.span(), Span::new(12, 20));
  /// assert_eq!(error.delimiter(), Brace);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn brace_of(span: S) -> Self {
    Self::of(span, Brace::PHANTOM)
  }
}

impl<S> Undelimited<Angle, S> {
  /// Creates a new undelimited content error for missing angle brackets.
  ///
  /// The span should point to the position of the content that should have been
  /// wrapped in angle brackets but wasn't.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::{Span, delimiter::Angle}};
  ///
  /// // Undelimited content from position 20 to 25
  /// let error = Undelimited::angle(Span::new(20, 25));
  /// assert_eq!(error.span(), Span::new(20, 25));
  /// assert_eq!(error.delimiter(), Angle);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn angle(span: S) -> Self {
    Self::angle_of(span)
  }
}

impl<S, Lang: ?Sized> Undelimited<Angle, S, Lang> {
  /// Creates a new undelimited content error for missing angle brackets.
  ///
  /// The span should point to the position of the content that should have been
  /// wrapped in angle brackets but wasn't.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::{Span, delimiter::Angle}};
  ///
  /// // Undelimited content from position 20 to 25
  /// let error = Undelimited::angle_of(Span::new(20, 25));
  /// assert_eq!(error.span(), Span::new(20, 25));
  /// assert_eq!(error.delimiter(), Angle);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn angle_of(span: S) -> Self {
    Self::of(span, Angle::PHANTOM)
  }
}

impl<Delimiter, S> Undelimited<Delimiter, S> {
  /// Creates a new undelimited content error.
  ///
  /// The span should point to the position of the content that should have been
  /// delimited but wasn't.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::Span};
  ///
  /// // Undelimited content from position 5 to 10
  /// let error = Undelimited::new(Span::new(5, 10), '{');
  /// assert_eq!(error.span(), Span::new(5, 10));
  /// assert_eq!(error.delimiter(), '{');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, delimiter: Delimiter) -> Self {
    Self::of(span, delimiter)
  }
}

impl<Delimiter, S, Lang: ?Sized> Undelimited<Delimiter, S, Lang> {
  /// Creates a new undelimited content error.
  ///
  /// The span should point to the position of the content that should have been
  /// delimited but wasn't.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::Span};
  ///
  /// // Undelimited content from position 5 to 10
  /// let error = Undelimited::new(Span::new(5, 10), '{');
  /// assert_eq!(error.span(), Span::new(5, 10));
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

  /// Returns the span of the undelimited content.
  ///
  /// This is the position of the content that should have been delimited but wasn't.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::Span};
  ///
  /// let error = Undelimited::new(Span::new(10, 15), '"');
  /// assert_eq!(error.span(), Span::new(10, 15));
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

  /// Returns a reference to the expected delimiter.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::Span};
  ///
  /// let error = Undelimited::new(Span::new(5, 10), '{');
  /// assert_eq!(error.delimiter_ref(), &'{');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimiter_ref(&self) -> &Delimiter {
    &self.delimiter
  }

  /// Returns the expected delimiter.
  ///
  /// This method is only available when the delimiter type implements `Copy`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::Span};
  ///
  /// let error = Undelimited::new(Span::new(5, 10), '[');
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
  /// use logosky::{error::Undelimited, utils::Span};
  ///
  /// let mut error = Undelimited::new(Span::new(5, 10), '(');
  /// error.bump(100);
  /// assert_eq!(error.span(), Span::new(105, 110));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, offset: &S::Offset) -> &mut Self
  where
    S: crate::lexer::Span,
  {
    self.span.bump(offset);
    self
  }

  /// Consumes the error and returns its components.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Undelimited, utils::Span};
  ///
  /// let error = Undelimited::new(Span::new(10, 15), '"');
  /// let (span, delimiter) = error.into_components();
  /// assert_eq!(span, Span::new(10, 15));
  /// assert_eq!(delimiter, '"');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (S, Delimiter) {
    (self.span, self.delimiter)
  }
}
