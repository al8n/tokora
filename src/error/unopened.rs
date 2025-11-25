//! Unopened delimiter error type for tracking closing delimiters without opening delimiters.
//!
//! This module provides the [`Unopened`] type for representing errors where a closing
//! delimiter was found but never opened (i.e., the corresponding opening delimiter is missing).
//!
//! # Design Philosophy
//!
//! When parsing structured text with paired delimiters (parentheses, brackets, braces,
//! quotes, etc.), it's common to encounter situations where a closing delimiter is found
//! but the corresponding opening delimiter is missing. This error type captures both:
//!
//! - **Where** the closing delimiter was found (via [`Span`])
//! - **What** closing delimiter was left unopened (via the generic `Delimiter` parameter)
//!
//! # Unopened vs Unclosed vs Unterminated
//!
//! - **`Unopened`**: For closing delimiters found **without** a matching opening delimiter
//!   - Examples: `a + b)`, `foo]`, `bar}`
//!   - The span points to the **closing delimiter** position
//!   - Used when you find a closing delimiter that was never opened
//!
//! - **`Unclosed`**: For opening delimiters found **without** a matching closing delimiter
//!   - Examples: `(a + b`, `[foo`, `{bar`
//!   - The span points to the **opening delimiter** position
//!   - Used when you find an opening delimiter that was never closed
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
//! use logosky::{error::Unopened, utils::Span};
//!
//! // Closing parenthesis at position 10, never opened
//! // Example: "a + b * c)" where the ')' has no matching '('
//! let error = Unopened::new(Span::new(10, 11), ')');
//!
//! assert_eq!(error.span(), Span::new(10, 11));
//! assert_eq!(error.delimiter(), ')');
//! assert_eq!(error.to_string(), "unopened delimiter ')'");
//! ```
//!
//! ## Custom Delimiter Enum
//!
//! ```rust
//! use logosky::{error::Unopened, utils::Span};
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
//! let error = Unopened::new(Span::new(5, 6), Delimiter::BlockComment);
//! assert_eq!(error.to_string(), "unopened delimiter '/*'");
//! ```
//!
//! ## Tracking Nested Delimiters
//!
//! ```rust
//! use logosky::{error::Unopened, utils::Span};
//!
//! // When parsing: "{ foo: bar, baz ] }"
//! // The ']' at position 16 was never opened
//! let error = Unopened::new(Span::new(16, 17), ']');
//!
//! // Error reporting can show:
//! // "error at 16..17: unopened delimiter ']'"
//! ```
//!
//! ## Position Adjustment
//!
//! ```rust
//! use logosky::{error::Unopened, utils::Span};
//!
//! // Error from a nested parsing context
//! let mut error = Unopened::new(Span::new(5, 6), '}');
//!
//! // Adjust to absolute position in the larger document
//! error.bump(100);
//! assert_eq!(error.span(), Span::new(105, 106));
//! ```

use crate::{
  punct::{Angle, Brace, Bracket, Paren},
  utils::Span,
};

/// An unopened bracket error (closing `]` without opening `[`)
pub type UnopenedBracket = Unopened<Bracket>;
/// An unopened parenthesis error (closing `)` without opening `(`)
pub type UnopenedParen = Unopened<Paren>;
/// An unopened brace error (closing `}` without opening `{`)
pub type UnopenedBrace = Unopened<Brace>;
/// An unopened angle bracket error (closing `>` without opening `<`)
pub type UnopenedAngle = Unopened<Angle>;

/// A zero-copy error type representing an unopened delimiter.
///
/// This type tracks the position of a closing delimiter that was never opened,
/// enabling precise error reporting for missing opening delimiters in structured text.
///
/// # Type Parameter
///
/// - `Delimiter`: The type representing the delimiter (typically `char`, `&'static str`,
///   or a custom enum). Must implement `Display` for error messages.
///
/// # Common Use Cases
///
/// - **Unmatched parentheses** in expressions: `a + b * c)`
/// - **Unopened strings** in source code: `hello world"`
/// - **Missing opening braces** in JSON: `"key": "value"}`
/// - **Incomplete block comments**: `/* This comment never ends`
/// - **Unmatched brackets** in arrays: `1, 2, 3]`
///
/// # Design
///
/// The span points to the **closing delimiter** position. This allows error messages
/// to point users to the exact location of the unexpected closing delimiter, making
/// it easier to find and fix the issue.
///
/// # Examples
///
/// ## Detecting Unopened Parentheses
///
/// ```rust
/// use logosky::{error::Unopened, utils::Span};
///
/// // Parse error: 1 + 2)
/// //                   ^--- unopened closing delimiter
/// let error = Unopened::new(Span::new(5, 6), ')');
///
/// println!("Error: {} at position {}", error, error.span().start());
/// // Output: "Error: unopened delimiter ')' at position 5"
/// ```
///
/// ## Tracking Multiple Unopened Delimiters
///
/// ```rust
/// use logosky::{error::Unopened, utils::Span};
///
/// let errors = vec![
///     Unopened::new(Span::new(5, 6), '}'),
///     Unopened::new(Span::new(10, 11), ']'),
///     Unopened::new(Span::new(15, 16), ')'),
/// ];
///
/// for error in errors {
///     eprintln!("Unopened {} at {}", error.delimiter(), error.span());
/// }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Unopened<Delimiter, S = Span> {
  span: S,
  delimiter: Delimiter,
}

impl<Delimiter, S> core::fmt::Display for Unopened<Delimiter, S>
where
  Delimiter: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "unopened delimiter '{}'", self.delimiter)
  }
}

impl<Delimiter, S> core::error::Error for Unopened<Delimiter, S>
where
  Delimiter: core::fmt::Display + core::fmt::Debug,
  S: core::fmt::Debug,
{
}

impl<S> Unopened<Paren, S> {
  /// Creates a new unopened parenthesis error.
  ///
  /// The span should point to the position of the closing parenthesis that was never opened.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unopened, utils::{Span, delimiter::Paren}};
  ///
  /// // Closing parenthesis at position 3, never opened
  /// let error = Unopened::paren(Span::new(3, 4));
  /// assert_eq!(error.span(), Span::new(3, 4));
  /// assert_eq!(error.delimiter(), Paren);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn paren(span: S) -> Self {
    Self {
      span,
      delimiter: Paren::PHANTOM,
    }
  }
}

impl<S> Unopened<Bracket, S> {
  /// Creates a new unopened bracket error.
  ///
  /// The span should point to the position of the closing bracket that was never opened.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unopened, utils::{Span, delimiter::Bracket}};
  ///
  /// // Closing bracket at position 8, never opened
  /// let error = Unopened::bracket(Span::new(8, 9));
  /// assert_eq!(error.span(), Span::new(8, 9));
  /// assert_eq!(error.delimiter(), Bracket);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bracket(span: S) -> Self {
    Self {
      span,
      delimiter: Bracket::PHANTOM,
    }
  }
}

impl<S> Unopened<Brace, S> {
  /// Creates a new unopened brace error.
  ///
  /// The span should point to the position of the closing brace that was never opened.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unopened, utils::{Span, delimiter::Brace}};
  ///
  /// // Closing brace at position 12, never opened
  /// let error = Unopened::brace(Span::new(12, 13));
  /// assert_eq!(error.span(), Span::new(12, 13));
  /// assert_eq!(error.delimiter(), Brace);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn brace(span: S) -> Self {
    Self {
      span,
      delimiter: Brace::PHANTOM,
    }
  }
}

impl<S> Unopened<Angle, S> {
  /// Creates a new unopened angle bracket error.
  ///
  /// The span should point to the position of the closing angle bracket that was never opened.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use logosky::{error::Unopened, utils::{Span, delimiter::Angle}};
  ///
  /// // Closing angle bracket at position 20, never opened
  /// let error = Unopened::angle(Span::new(20, 21));
  /// assert_eq!(error.span(), Span::new(20, 21));
  /// assert_eq!(error.delimiter(), Angle);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn angle(span: S) -> Self {
    Self {
      span,
      delimiter: Angle::PHANTOM,
    }
  }
}

impl<Delimiter, S> Unopened<Delimiter, S> {
  /// Creates a new unopened delimiter error.
  ///
  /// The span should point to the position of the closing delimiter that was never opened.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unopened, utils::Span};
  ///
  /// // Closing brace at position 5, never opened
  /// let error = Unopened::new(Span::new(5, 6), '}');
  /// assert_eq!(error.span(), Span::new(5, 6));
  /// assert_eq!(error.delimiter(), '}');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, delimiter: Delimiter) -> Self {
    Self { span, delimiter }
  }

  /// Returns the span of the closing delimiter.
  ///
  /// This is the position where the closing delimiter was found without a matching opening.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unopened, utils::Span};
  ///
  /// let error = Unopened::new(Span::new(10, 11), ')');
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

  /// Returns a reference to the unopened delimiter.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unopened, utils::Span};
  ///
  /// let error = Unopened::new(Span::new(5, 6), '}');
  /// assert_eq!(error.delimiter_ref(), &'}');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimiter_ref(&self) -> &Delimiter {
    &self.delimiter
  }

  /// Returns the unopened delimiter.
  ///
  /// This method is only available when the delimiter type implements `Copy`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::{error::Unopened, utils::Span};
  ///
  /// let error = Unopened::new(Span::new(5, 6), ']');
  /// assert_eq!(error.delimiter(), ']');
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
  /// use logosky::{error::Unopened, utils::Span};
  ///
  /// let mut error = Unopened::new(Span::new(5, 6), ')');
  /// error.bump(100);
  /// assert_eq!(error.span(), Span::new(105, 106));
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
  /// use logosky::{error::Unopened, utils::Span};
  ///
  /// let error = Unopened::new(Span::new(10, 11), '"');
  /// let (span, delimiter) = error.into_components();
  /// assert_eq!(span, Span::new(10, 11));
  /// assert_eq!(delimiter, '"');
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (S, Delimiter) {
    (self.span, self.delimiter)
  }
}
