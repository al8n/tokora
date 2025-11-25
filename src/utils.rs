use core::ops::{AddAssign, Range};

pub use escaped::*;
pub use expected::*;
pub use generic_arraydeque::GenericArrayDeque;
pub use lexeme::*;
pub use message::Message;
pub use positioned_char::*;
pub use to_equivalent::*;

/// Re-export of generic-arraydeque for direct access.
pub use generic_arraydeque;

/// Trackers for preventing infinite recursion in parsers.
pub mod recursion_tracker;
/// A token tracker for tracking tokens in a lexer.
pub mod token_tracker;
/// A tracker for tracking recursion depth and tokens.
pub mod tracker;

/// A module for custom comparing traits.
pub mod cmp;
/// A module for displaying in a human-friendly way.
pub mod human_display;
/// A module for displaying in SDL.
pub mod sdl_display;
/// A module for displaying in syntax trees.
pub mod syntax_tree_display;

/// Common delimiters used in lexing and parsing.
pub mod delimiter;

/// Common knowledge types for lexing and parsing.
pub mod knowledge;

/// Re-export typenum for type-level numbers.
pub use generic_arraydeque::typenum;

/// A module for container types with small size optimizations.
#[cfg(feature = "smallvec")]
#[cfg_attr(docsrs, doc(cfg(feature = "smallvec")))]
pub mod container;

mod escaped;
mod expected;
mod lexeme;
mod message;
mod positioned_char;
mod to_equivalent;

/// A lightweight span representing a range of positions in source input.
///
/// `Span` is a simple but powerful type that tracks where in the source code a particular
/// element came from. It stores just two byte offsets: the start and end positions.
/// While similar to [`Range<usize>`], `Span` provides additional methods tailored for
/// working with source locations in parsers and compilers.
///
/// # Use Cases
///
/// - **Error Reporting**: Show users exactly where errors occurred in their code
/// - **Source Mapping**: Track how parsed elements relate to original source
/// - **IDE Integration**: Enable features like go-to-definition and hover tooltips
/// - **Code Formatting**: Preserve the original location of code elements
/// - **Debugging**: Understand which part of input produced which AST node
///
/// # Design
///
/// `Span` is designed to be:
/// - **Copy**: Can be freely copied without allocation (just two `usize` values)
/// - **Comparable**: Supports equality and ordering for span-based algorithms
/// - **Hashable**: Can be used as map/set keys for span-indexed data structures
/// - **Chumsky-compatible**: Implements `chumsky::span::Span` for parser integration
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use logosky::utils::Span;
///
/// // Create a span covering characters 10-20
/// let span = Span::new(10, 20);
///
/// assert_eq!(span.start(), 10);
/// assert_eq!(span.end(), 20);
/// assert_eq!(span.len(), 10);
/// assert!(!span.is_empty());
/// ```
///
/// ## Safe Creation
///
/// ```rust
/// use logosky::utils::Span;
///
/// // try_new returns None for invalid spans
/// assert!(Span::try_new(10, 5).is_none());  // end < start
/// assert!(Span::try_new(10, 10).is_some()); // empty span is valid
/// assert!(Span::try_new(10, 20).is_some()); // normal span
/// ```
///
/// ## Span Manipulation
///
/// ```rust
/// use logosky::utils::Span;
///
/// let mut span = Span::new(10, 20);
///
/// // Move the start forward
/// span.bump_start(5);
/// assert_eq!(span.start(), 15);
///
/// // Extend the end
/// span.bump_end(10);
/// assert_eq!(span.end(), 30);
///
/// // Shift the entire span
/// span.bump(5);
/// assert_eq!(span.start(), 20);
/// assert_eq!(span.end(), 35);
/// ```
///
/// ## Builder-Style Methods
///
/// ```rust
/// use logosky::utils::Span;
///
/// let span = Span::new(0, 10)
///     .with_start(5)
///     .with_end(15);
///
/// assert_eq!(span.start(), 5);
/// assert_eq!(span.end(), 15);
/// ```
///
/// ## Error Reporting Example
///
/// ```rust,ignore
/// use logosky::utils::Span;
///
/// fn report_error(message: &str, span: Span, source: &str) {
///     let line_start = source[..span.start()].rfind('\n')
///         .map(|pos| pos + 1)
///         .unwrap_or(0);
///     let line_end = source[span.end()..]
///         .find('\n')
///         .map(|pos| span.end() + pos)
///         .unwrap_or(source.len());
///
///     let line = &source[line_start..line_end];
///     let column = span.start() - line_start;
///
///     eprintln!("Error: {}", message);
///     eprintln!("{}", line);
///     eprintln!("{}^", " ".repeat(column));
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Span<Offset = usize> {
  pub(crate) start: Offset,
  pub(crate) end: Offset,
}

impl<O> core::fmt::Display for Span<O>
where
  O: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}..{}", self.start, self.end)
  }
}

impl<O> Span<&O>
where
  O: Clone,
{
  /// Clone the span into owned offsets.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn cloned(self) -> Span<O> {
    Span {
      start: self.start.clone(),
      end: self.end.clone(),
    }
  }
}

#[cfg(feature = "chumsky")]
const _: () = {
  use chumsky::{error::Cheap, span::SimpleSpan};

  impl From<Span> for Cheap {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(span: Span) -> Self {
      Cheap::new(span.into())
    }
  }

  impl From<Cheap> for Span {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(cheap: Cheap) -> Self {
      Self::from(*cheap.span())
    }
  }

  impl From<SimpleSpan> for Span {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(span: SimpleSpan) -> Self {
      Self::new(span.start, span.end)
    }
  }

  impl From<Span> for SimpleSpan {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(span: Span) -> Self {
      SimpleSpan::from(span.start..span.end)
    }
  }

  impl chumsky::span::Span for Span {
    type Context = ();

    type Offset = usize;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn new(_: Self::Context, range: Range<Self::Offset>) -> Self {
      Self::new(range.start, range.end)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn context(&self) -> Self::Context {}

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn start(&self) -> Self::Offset {
      self.start
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn end(&self) -> Self::Offset {
      self.end
    }
  }
};

impl Span {
  /// Create a new span.
  ///
  /// ## Panics
  ///
  /// Panics if `end < start`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn const_new(start: usize, end: usize) -> Self {
    assert!(end >= start, "end must be greater than or equal to start");
    Self { start, end }
  }

  /// Try to create a new span.
  ///
  /// Returns `None` if `end < start`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_const_new(start: usize, end: usize) -> Option<Self> {
    if end >= start {
      Some(Self { start, end })
    } else {
      None
    }
  }

  /// Bump the start of the span by `n`.
  ///
  /// ## Panics
  ///
  /// Panics if `self.start + n > self.end`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.bump_start(3);
  /// assert_eq!(span, Span::new(8, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bump_start_const(&mut self, n: usize) -> &mut Self {
    self.start += n;
    assert!(
      self.start <= self.end,
      "start must be less than or equal to end"
    );
    self
  }

  /// Bump the end of the span by `n`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.bump_end(5);
  /// assert_eq!(span, Span::new(5, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bump_end_const(&mut self, n: usize) -> &mut Self {
    self.end += n;
    self
  }

  /// Bump the start and the end of the span by `n`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.bump(10);
  /// assert_eq!(span, Span::new(15, 25));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bump_const(&mut self, n: usize) -> &mut Self {
    self.start += n;
    self.end += n;
    self
  }

  /// Set the start of the span, returning a mutable reference to self.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.set_start(10);
  /// assert_eq!(span, Span::new(10, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_start_const(&mut self, start: usize) -> &mut Self {
    self.start = start;
    self
  }

  /// Set the end of the span, returning a mutable reference to self.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.set_end(20);
  /// assert_eq!(span, Span::new(5, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_end_const(&mut self, end: usize) -> &mut Self {
    self.end = end;
    self
  }

  /// Set the start of the span, returning self.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15).with_start(10);
  /// assert_eq!(span, Span::new(10, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_start_const(mut self, start: usize) -> Self {
    self.start = start;
    self
  }

  /// Set the end of the span, returning self.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15).with_end(20);
  /// assert_eq!(span, Span::new(5, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_end_const(mut self, end: usize) -> Self {
    self.end = end;
    self
  }
}

impl<O> Span<O> {
  /// Convert to a span of references.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15);
  /// let span_ref = span.as_ref();
  /// assert_eq!(*span_ref.start, 5);
  /// assert_eq!(*span_ref.end, 15);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_ref(&self) -> Span<&O> {
    Span {
      start: &self.start,
      end: &self.end,
    }
  }

  /// Convert to a span of mutable references.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// let span_mut = span.as_mut();
  /// *span_mut.start = 10;
  /// *span_mut.end = 20;
  /// assert_eq!(span, Span::new(10, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> Span<&mut O> {
    Span {
      start: &mut self.start,
      end: &mut self.end,
    }
  }

  /// Create a new span.
  ///
  /// ## Panics
  ///
  /// Panics if `end < start`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn new(start: O, end: O) -> Self
  where
    O: Ord,
  {
    assert!(end >= start, "end must be greater than or equal to start");
    Self { start, end }
  }

  /// Try to create a new span.
  ///
  /// Returns `None` if `end < start`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_new(start: O, end: O) -> Option<Self>
  where
    O: Ord,
  {
    if end >= start {
      Some(Self { start, end })
    } else {
      None
    }
  }

  /// Bump the start of the span by `n`.
  ///
  /// ## Panics
  ///
  /// Panics if `self.start + n > self.end`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.bump_start(3);
  /// assert_eq!(span, Span::new(8, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump_start(&mut self, n: O) -> &mut Self
  where
    O: AddAssign<O> + Ord,
  {
    self.start += n;
    assert!(
      self.start <= self.end,
      "start must be less than or equal to end"
    );
    self
  }

  /// Bump the end of the span by `n`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.bump_end(5);
  /// assert_eq!(span, Span::new(5, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump_end(&mut self, n: O) -> &mut Self
  where
    O: AddAssign<O>,
  {
    self.end += n;
    self
  }

  /// Bump the start and the end of the span by `n`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.bump(10);
  /// assert_eq!(span, Span::new(15, 25));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bump(&mut self, n: &O) -> &mut Self
  where
    O: for<'a> AddAssign<&'a O> + Clone,
  {
    self.start += n;
    self.end += n;
    self
  }

  /// Set the start of the span, returning a mutable reference to self.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.set_start(10);
  /// assert_eq!(span, Span::new(10, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_start(&mut self, start: O) -> &mut Self {
    self.start = start;
    self
  }

  /// Set the end of the span, returning a mutable reference to self.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// span.set_end(20);
  /// assert_eq!(span, Span::new(5, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_end(&mut self, end: O) -> &mut Self {
    self.end = end;
    self
  }

  /// Set the start of the span, returning self.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15).with_start(10);
  /// assert_eq!(span, Span::new(10, 15));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_start(mut self, start: O) -> Self {
    self.start = start;
    self
  }

  /// Set the end of the span, returning self.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15).with_end(20);
  /// assert_eq!(span, Span::new(5, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_end(mut self, end: O) -> Self {
    self.end = end;
    self
  }

  /// Get the start of the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15);
  /// assert_eq!(span.start(), 5);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn start(&self) -> O
  where
    O: Copy,
  {
    self.start
  }

  /// Get the reference to the start of the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15);
  ///
  /// assert_eq!(*span.start_ref(), 5);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn start_ref(&self) -> &O {
    &self.start
  }

  /// Get the mutable reference to the start of the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// *span.start_mut() = 10;
  /// assert_eq!(span.start(), 10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn start_mut(&mut self) -> &mut O {
    &mut self.start
  }

  /// Get the end of the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15);
  /// assert_eq!(span.end(), 15);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn end(&self) -> O
  where
    O: Copy,
  {
    self.end
  }

  /// Get the reference to the end of the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15);
  ///
  /// assert_eq!(*span.end_ref(), 15);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn end_ref(&self) -> &O {
    &self.end
  }

  /// Get the mutable reference to the end of the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let mut span = Span::new(5, 15);
  /// *span.end_mut() = 20;
  /// assert_eq!(span.end(), 20);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn end_mut(&mut self) -> &mut O {
    &mut self.end
  }

  /// Get the length of the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15);
  /// assert_eq!(span.len(), 10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn len(&self) -> O
  where
    O: for<'a> core::ops::Sub<&'a O, Output = O> + Clone,
  {
    self.end.clone().sub(&self.start)
  }

  /// Check if the span is empty.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let empty = Span::new(5, 5);
  /// assert!(empty.is_empty());
  ///
  /// let not_empty = Span::new(5, 15);
  /// assert!(!not_empty.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_empty(&self) -> bool
  where
    O: PartialEq,
  {
    self.start == self.end
  }

  /// Returns a range covering the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::Span;
  ///
  /// let span = Span::new(5, 15);
  /// assert_eq!(span.range(), 5..15);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn range(&self) -> Range<&O> {
    &self.start..&self.end
  }
}

impl<O> From<Range<O>> for Span<O>
where
  O: Ord,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(range: Range<O>) -> Self {
    Self::new(range.start, range.end)
  }
}

impl<O> From<Span<O>> for Range<O> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(span: Span<O>) -> Self {
    span.start..span.end
  }
}

impl<O> From<(O, O)> for Span<O>
where
  O: Ord,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from((start, end): (O, O)) -> Self {
    Self::new(start, end)
  }
}

impl<O> From<Span<O>> for (O, O) {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(span: Span<O>) -> Self {
    (span.start, span.end)
  }
}

/// A value paired with its source location span.
///
/// `Spanned<D>` combines a value of type `D` with a [`Span`] that indicates where in
/// the source input the value came from. This is fundamental for building parsers and
/// compilers that need to track source locations for error reporting, debugging, and
/// IDE integration.
///
/// # Design
///
/// `Spanned` uses public fields for direct access, but also provides accessor methods
/// for consistency. It implements `Deref` and `DerefMut` to allow transparent access
/// to the inner data while keeping span information available when needed.
///
/// # Common Patterns
///
/// ## Transparent Access via Deref
///
/// Thanks to `Deref`, you can call methods on the wrapped value directly:
///
/// ```rust
/// use logosky::utils::{Span, Spanned};
///
/// let spanned_str = Spanned::new(Span::new(0, 5), "hello");
///
/// // Can call str methods directly
/// assert_eq!(spanned_str.len(), 5);
/// assert_eq!(spanned_str.to_uppercase(), "HELLO");
///
/// // But can still access the span
/// assert_eq!(spanned_str.span().start(), 0);
/// ```
///
/// ## Mapping Values While Preserving Spans
///
/// ```rust,ignore
/// use logosky::utils::{Span, Spanned};
///
/// let spanned_num = Spanned::new(Span::new(10, 12), "42");
///
/// // Parse the string, keeping the same span
/// let parsed: Spanned<i32> = Spanned::new(
///     spanned_num.span,
///     spanned_num.data.parse().unwrap()
/// );
///
/// assert_eq!(*parsed, 42);
/// assert_eq!(parsed.span().start(), 10);
/// ```
///
/// ## Building AST Nodes with Locations
///
/// ```rust,ignore
/// use logosky::utils::{Span, Spanned};
///
/// enum Expr {
///     Number(i64),
///     Add(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
/// }
///
/// // Each AST node knows its source location
/// let left = Spanned::new(Span::new(0, 2), Expr::Number(1));
/// let right = Spanned::new(Span::new(5, 7), Expr::Number(2));
///
/// let add = Spanned::new(
///     Span::new(0, 7), // Covers the whole expression
///     Expr::Add(Box::new(left), Box::new(right))
/// );
/// ```
///
/// ## Error Reporting with Context
///
/// ```rust,ignore
/// fn type_error<T>(expected: &str, got: &Spanned<T>) -> Error
/// where
///     T: core::fmt::Debug
/// {
///     Error {
///         message: format!("Expected {}, got {:?}", expected, got.data),
///         span: *got.span(),
///         help: Some("Try using a different type".to_string()),
///     }
/// }
/// ```
///
/// # Trait Implementations
///
/// - **`Deref` / `DerefMut`**: Access the inner data transparently
/// - **`Display`**: Delegates to the inner data's `Display` implementation
/// - **`AsSpan` / `IntoSpan`**: Extract just the span information
/// - **`IntoComponents`**: Destructure into `(Span, D)` tuple
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use logosky::utils::{Span, Spanned};
///
/// let span = Span::new(10, 15);
/// let spanned = Spanned::new(span, "hello");
///
/// assert_eq!(spanned.span(), &span);
/// assert_eq!(spanned.data(), &"hello");
/// assert_eq!(*spanned, "hello"); // Via Deref
/// ```
///
/// ## Destructuring
///
/// ```rust
/// use logosky::utils::{Span, Spanned};
///
/// let spanned = Spanned::new(Span::new(0, 5), 42);
///
/// let (span, value) = spanned.into_components();
/// assert_eq!(span.start(), 0);
/// assert_eq!(value, 42);
/// ```
///
/// ## Mutable Access
///
/// ```rust
/// use logosky::utils::{Span, Spanned};
///
/// let mut spanned = Spanned::new(Span::new(0, 1), 10);
///
/// // Modify the data
/// *spanned += 5;
/// assert_eq!(*spanned, 15);
///
/// // Modify the span
/// spanned.span_mut().bump_end(4);
/// assert_eq!(spanned.span().end(), 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Spanned<D, S = Span> {
  /// The source location span of the data.
  ///
  /// This indicates where in the source input this value came from,
  /// expressed as byte offsets.
  pub span: S,

  /// The wrapped data value.
  ///
  /// This is the actual parsed or processed value, paired with its
  /// source location for error reporting and debugging.
  pub data: D,
}

impl<D, S> AsRef<S> for Spanned<D, S> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &S {
    self.span_ref()
  }
}

impl<D, S> AsSpan<S> for Spanned<D, S> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_span(&self) -> &S {
    AsRef::as_ref(self)
  }
}

impl<D, S> IntoSpan<S> for Spanned<D, S> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_span(self) -> S {
    self.span
  }
}

impl<D, S> core::ops::Deref for Spanned<D, S> {
  type Target = D;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

impl<D, S> core::ops::DerefMut for Spanned<D, S> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.data
  }
}

impl<D, S> core::fmt::Display for Spanned<D, S>
where
  D: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.data.fmt(f)
  }
}

impl<D, S> core::error::Error for Spanned<D, S>
where
  D: core::error::Error,
  S: core::fmt::Debug,
{
}

impl<D, S> IntoComponents for Spanned<D, S> {
  type Components = (S, D);

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_components(self) -> Self::Components {
    (self.span, self.data)
  }
}

impl<D, S> Spanned<D, S> {
  /// Create a new spanned value.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, data: D) -> Self {
    Self { span, data }
  }

  /// Get a reference to the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::{Span, Spanned};
  ///
  /// let spanned = Spanned::new(Span::new(5, 10), "data");
  /// assert_eq!(spanned.span(), Span::new(5, 10));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    self.span
  }

  /// Get a reference to the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::{Span, Spanned};
  ///
  /// let spanned = Spanned::new(Span::new(5, 10), "data");
  /// assert_eq!(spanned.span_ref(), &Span::new(5, 10));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &S {
    &self.span
  }

  /// Get a mutable reference to the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::{Span, Spanned};
  ///
  /// let mut spanned = Spanned::new(Span::new(5, 10), "data");
  /// spanned.span_mut().set_end(15);
  /// assert_eq!(spanned.span().end(), 15);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut S {
    &mut self.span
  }

  /// Get a reference to the data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::{Span, Spanned};
  ///
  /// let spanned = Spanned::new(Span::new(5, 10), 42);
  /// assert_eq!(*spanned.data(), 42);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data(&self) -> &D {
    &self.data
  }

  /// Get a mutable reference to the data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::{Span, Spanned};
  ///
  /// let mut spanned = Spanned::new(Span::new(5, 10), 42);
  /// *spanned.data_mut() = 100;
  /// assert_eq!(*spanned.data(), 100);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data_mut(&mut self) -> &mut D {
    &mut self.data
  }

  /// Returns a reference to the span and data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::{Span, Spanned};
  ///
  /// let spanned = Spanned::new(Span::new(5, 10), String::from("hello"));
  /// let borrowed: Spanned<&String> = spanned.as_ref();
  /// assert_eq!(borrowed.data(), &"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_ref(&self) -> Spanned<&D, &S> {
    Spanned {
      span: &self.span,
      data: &self.data,
    }
  }

  /// Returns a mutable reference to the span and data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use logosky::utils::{Span, Spanned};
  ///
  /// let mut spanned = Spanned::new(Span::new(5, 10), String::from("hello"));
  /// let borrowed: Spanned<&mut String> = spanned.as_mut();
  /// borrowed.data.push_str(" world");
  /// assert_eq!(spanned.data(), &"hello world");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> Spanned<&mut D, &mut S> {
    Spanned {
      span: &mut self.span,
      data: &mut self.data,
    }
  }

  /// Consume the spanned value and return the data.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_data(self) -> D {
    self.data
  }

  /// Decompose the spanned value into its span and data.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (S, D) {
    (self.span, self.data)
  }

  /// Map the data to a new value, preserving the span.
  #[inline]
  pub fn map_data<F, U>(self, f: F) -> Spanned<U, S>
  where
    F: FnOnce(D) -> U,
  {
    Spanned {
      span: self.span,
      data: f(self.data),
    }
  }
}

/// Enables accessing the source span of a parsed element.
///
/// This trait provides a way to retrieve the span information associated with
/// a parsed element without taking ownership of the element itself. This is
/// useful for scenarios where you need to reference the location of the element
/// in the source input, such as for error reporting or diagnostics.
///
/// ## Usage Patterns
/// Common scenarios for using this trait:
/// - **Error reporting**: Attaching span information to error messages
/// - **Diagnostics**: Highlighting source locations in IDEs or tools
/// - **Logging**: Recording where certain elements were parsed from
/// - **Analysis**: Performing source-based analysis or transformations
///
/// ## Implementation Notes
///
/// Implementing types should ensure that:
///   - The returned span is accurate and corresponds to the element's location in the source
///   - The method is efficient and does not involve unnecessary allocations or computations
///   - The trait is implemented for all relevant types
///   - The span information is preserved during parsing and transformations
///   - The implementation is consistent with other span-related traits
///   - The method is efficient (ideally zero-cost)
///   - The returned reference is valid for the lifetime of the element
pub trait AsSpan<Span> {
  /// Consumes this element and returns the owned source span.
  ///
  /// This method takes ownership of the element and extracts its span information
  /// as an owned value. This is useful when you need to transfer ownership of
  /// the span data to another data structure or when the element itself is no
  /// longer needed but the location information should be preserved.
  fn as_span(&self) -> &Span;
}

/// Enables consuming a parsed element to extract its source span.
///
/// This trait provides a way to take ownership of the span information from
/// a parsed element, which is useful when the element itself is no longer
/// needed but the span data should be preserved or transferred to another
/// data structure.
///
/// ## Usage Patterns
///
/// Common scenarios for using this trait:
/// - **AST construction**: Building higher-level AST nodes that need owned spans
/// - **Error collection**: Gathering span information for batch error reporting
/// - **Transformation**: Converting between different representations while preserving location
/// - **Optimization**: Avoiding clones when transferring ownership is acceptable
///
/// ## Implementation Notes
///
/// Implementing types should ensure that:
/// - The returned span is equivalent to what `AsSpan::spanned()` would return
/// - All span information is preserved during the conversion
/// - The conversion is efficient (ideally zero-cost)
pub trait IntoSpan<Span>: AsSpan<Span> {
  /// Consumes this element and returns the owned source span.
  ///
  /// This method takes ownership of the element and extracts its span information
  /// as an owned value. This is useful when you need to transfer ownership of
  /// the span data to another data structure or when the element itself is no
  /// longer needed but the location information should be preserved.
  fn into_span(self) -> Span;
}

/// Enables destructuring a parsed element into its constituent components.
///
/// This trait provides a way to break down complex parsed elements into their
/// individual parts, taking ownership of each component. This is particularly
/// useful for transformation, analysis, or when building different representations
/// of the parsed data.
///
/// ## Design Philosophy
///
/// The trait uses an associated type rather than generic parameters to ensure
/// that each implementing type has exactly one way to be decomposed. This provides
/// type safety and makes the interface predictable for consumers.
///
/// ## Usage Patterns
///
/// Common scenarios for using this trait:
/// - **AST transformation**: Converting parsed elements into different AST representations
/// - **Analysis**: Extracting specific components for validation or processing
/// - **Serialization**: Breaking down elements for custom serialization formats
/// - **Testing**: Accessing individual components for detailed assertions
///
/// ## Examples
///
/// ```rust,ignore
/// // Extracting components for transformation
/// let float_value: FloatValue<&str, SimpleSpan> = parse_float("3.14e-2")?;
/// let (span, int_part, frac_part, exp_part) = float_value.into_components();
///
/// // Building a custom representation
/// let custom_float = CustomFloat {
///     location: span,
///     integer: int_part,
///     fractional: frac_part,
///     exponent: exp_part,
/// };
///
/// // Component analysis
/// let int_literal: IntValue<&str, SimpleSpan> = parse_int("-42")?;
/// let (span, sign, digits) = int_literal.into_components();
///
/// if sign.is_some() {
///     println!("Found negative integer at {:?}", span);
/// }
/// ```
///
/// ## Implementation Guidelines
///
/// When implementing this trait:
/// - Include all meaningful components of the parsed element
/// - Order components logically (typically: span first, then sub-components in source order)
/// - Use tuples for simple decomposition, custom structs for complex cases
/// - Ensure the decomposition is complete (no information loss)
/// - Document the component structure clearly
///
/// ## Component Ordering Convention
///
/// To maintain consistency across implementations, follow this ordering:
/// 1. **Overall span**: The span covering the entire element
/// 2. **Required components**: Core parts that are always present
/// 3. **Optional components**: Parts that may or may not be present
/// 4. **Sub-elements**: Nested parsed elements in source order
pub trait IntoComponents {
  /// The tuple or struct type containing the decomposed components.
  ///
  /// This associated type defines the structure returned by `into_components()`.
  /// It should include all meaningful parts of the parsed element in a logical
  /// order that makes sense for the specific element type.
  type Components;

  /// Consumes this element and returns its constituent components.
  ///
  /// This method breaks down the parsed element into its individual parts,
  /// providing owned access to each component. The exact structure of the
  /// returned components is defined by the `Components` associated type.
  fn into_components(self) -> Self::Components;
}

/// A trait for checking if a token is an ASCII character.
pub trait IsAsciiChar {
  /// Returns `true` if self is equal to the given ASCII character.
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool;

  /// Checks if the value is an ASCII decimal digit:
  /// U+0030 '0' ..= U+0039 '9'.
  fn is_ascii_digit(&self) -> bool;

  /// Returns `true` if self is one of the given ASCII characters.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn one_of(&self, choices: &[ascii::AsciiChar]) -> bool {
    choices.iter().any(|&ch| self.is_ascii_char(ch))
  }
}

impl<T> IsAsciiChar for &T
where
  T: IsAsciiChar + ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <T as IsAsciiChar>::is_ascii_char(*self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <T as IsAsciiChar>::is_ascii_digit(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn one_of(&self, choices: &[ascii::AsciiChar]) -> bool {
    <T as IsAsciiChar>::one_of(*self, choices)
  }
}

impl<T> IsAsciiChar for &mut T
where
  T: IsAsciiChar + ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <T as IsAsciiChar>::is_ascii_char(*self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <T as IsAsciiChar>::is_ascii_digit(*self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn one_of(&self, choices: &[ascii::AsciiChar]) -> bool {
    <T as IsAsciiChar>::one_of(*self, choices)
  }
}

impl IsAsciiChar for char {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    if self.is_ascii() {
      *self as u8 == ch as u8
    } else {
      false
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    char::is_ascii_digit(self)
  }
}

impl IsAsciiChar for u8 {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    *self == ch as u8
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    u8::is_ascii_digit(self)
  }
}

impl IsAsciiChar for str {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    self.len() == 1 && self.as_bytes()[0] == ch as u8
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    self.len() == 1 && self.as_bytes()[0].is_ascii_digit()
  }
}

impl IsAsciiChar for [u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    self.len() == 1 && self[0] == ch as u8
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    self.len() == 1 && self[0].is_ascii_digit()
  }
}

#[cfg(feature = "bstr")]
impl IsAsciiChar for bstr::BStr {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_digit(self)
  }
}

#[cfg(feature = "bytes")]
impl IsAsciiChar for bytes::Bytes {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_digit(self)
  }
}

#[cfg(feature = "hipstr")]
impl IsAsciiChar for hipstr::HipByt<'_> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <[u8] as IsAsciiChar>::is_ascii_digit(self)
  }
}

#[cfg(feature = "hipstr")]
impl IsAsciiChar for hipstr::HipStr<'_> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_char(&self, ch: ascii::AsciiChar) -> bool {
    <str as IsAsciiChar>::is_ascii_char(self, ch)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ascii_digit(&self) -> bool {
    <str as IsAsciiChar>::is_ascii_digit(self)
  }
}

/// A trait for character-like types that can report their encoded length in bytes.
///
/// `CharLen` provides a uniform way to query the byte length of different character
/// types, which is essential for converting positioned characters into byte spans.
///
/// # Implementations
///
/// LogoSky provides implementations for:
/// - **`u8`**: Always returns `1` (single byte)
/// - **`char`**: Returns `len_utf8()` (1-4 bytes depending on the character)
/// - **`&T`**: Delegates to `T::len()` for any `T: CharLen`
///
/// # Design Note
///
/// This trait is **sealed** and cannot be implemented outside of LogoSky. If you need
/// to work with a custom character type, use [`Lexeme::span_with`] or
/// [`UnknownLexeme::from_range`](crate::error::UnknownLexeme::from_range) and provide your own length function.
///
/// # Use Cases
///
/// - **Span calculation**: Convert positioned characters to byte spans automatically
/// - **UTF-8 handling**: Properly account for multi-byte characters
/// - **Error reporting**: Determine the exact byte range of an unexpected character
///
/// # Examples
///
/// ## Automatic Length Detection
///
/// ```rust
/// use logosky::utils::{Lexeme, PositionedChar};
///
/// // ASCII character (1 byte)
/// let ascii = Lexeme::from(PositionedChar::with_position('a', 10));
/// let span = ascii.span();
/// assert_eq!(span.len(), 1);
///
/// // Multi-byte UTF-8 character (3 bytes)
/// let emoji = Lexeme::from(PositionedChar::with_position('€', 20));
/// let span = emoji.span();
/// assert_eq!(span.len(), 3);
/// ```
///
/// ## With Custom Length Function
///
/// ```rust
/// use logosky::utils::{Lexeme, PositionedChar};
///
/// // For types that don't implement CharLen, use span_with
/// struct CustomChar(char);
///
/// let lexeme = Lexeme::from(PositionedChar::with_position(CustomChar('€'), 5));
/// let span = lexeme.span_with(|c| c.0.len_utf8());
///
/// assert_eq!(span.start(), 5);
/// assert_eq!(span.end(), 8);
/// ```
#[allow(clippy::len_without_is_empty)]
pub trait CharLen: sealed::Sealed {
  /// Returns the length of this character in bytes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use logosky::utils::{Lexeme, PositionedChar};
  ///
  /// // The trait is used internally by span()
  /// let ascii = Lexeme::from(PositionedChar::with_position('A', 0));
  /// assert_eq!(ascii.span().len(), 1);
  ///
  /// let euro = Lexeme::from(PositionedChar::with_position('€', 0));
  /// assert_eq!(euro.span().len(), 3);
  ///
  /// let crab = Lexeme::from(PositionedChar::with_position('🦀', 0));
  /// assert_eq!(crab.span().len(), 4);
  /// ```
  fn char_len(&self) -> usize;
}

mod sealed {
  use super::{CharLen, PositionedChar};

  pub trait Sealed {}

  impl Sealed for u8 {}
  impl Sealed for char {}
  impl<T: Sealed> Sealed for PositionedChar<T> {}

  impl<T: Sealed> Sealed for &T {}

  impl CharLen for u8 {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn char_len(&self) -> usize {
      1
    }
  }

  impl CharLen for char {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn char_len(&self) -> usize {
      self.len_utf8()
    }
  }

  impl<T: CharLen> CharLen for PositionedChar<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn char_len(&self) -> usize {
      self.char_ref().char_len()
    }
  }

  impl<T: CharLen> CharLen for &T {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn char_len(&self) -> usize {
      (*self).char_len()
    }
  }
}
