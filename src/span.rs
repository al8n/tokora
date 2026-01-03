use core::{
  hash::Hash,
  ops::{AddAssign, Range},
};

use crate::utils::{IntoComponents, marker::Ignored};

/// A trait representing a span in the source code.
pub trait Span {
  /// The offset type of the span.
  type Offset: Ord + Clone + Hash;

  /// Creates a new span from the given start and end offsets.
  fn new(start: Self::Offset, end: Self::Offset) -> Self;

  /// Consumes the span and returns it.
  fn into_range(self) -> core::ops::Range<Self::Offset>
  where
    Self: Sized;

  /// Returns the start offset of the span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start(&self) -> Self::Offset {
    self.start_ref().clone()
  }

  /// Returns the start offset of the span.
  fn start_ref(&self) -> &Self::Offset;

  /// Returns the mutable reference to the start offset of the span.
  fn start_mut(&mut self) -> &mut Self::Offset;

  /// Consumes the span and returns the start offset.
  fn into_start(self) -> Self::Offset
  where
    Self: Sized;

  /// Returns the end offset of the span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end(&self) -> Self::Offset {
    self.end_ref().clone()
  }

  /// Returns the end offset of the span.
  fn end_ref(&self) -> &Self::Offset;

  /// Returns the mutable reference to the end offset of the span.
  fn end_mut(&mut self) -> &mut Self::Offset;

  /// Consumes the span and returns the end offset.
  fn into_end(self) -> Self::Offset
  where
    Self: Sized;

  /// Bumps the span by `n` offsets.
  fn bump(&mut self, n: &Self::Offset);
}

impl Span for core::ops::Range<usize> {
  type Offset = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn new(start: Self::Offset, end: Self::Offset) -> Self {
    start..end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_range(self) -> core::ops::Range<Self::Offset> {
    self.start..self.end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start_ref(&self) -> &Self::Offset {
    &self.start
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start_mut(&mut self) -> &mut Self::Offset {
    &mut self.start
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_start(self) -> Self::Offset {
    self.start
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end_ref(&self) -> &Self::Offset {
    &self.end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end_mut(&mut self) -> &mut Self::Offset {
    &mut self.end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_end(self) -> Self::Offset {
    self.end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn bump(&mut self, n: &Self::Offset) {
    self.end += *n;
  }
}

impl<O> Span for SimpleSpan<O>
where
  O: Ord + Clone + Hash + for<'a> AddAssign<&'a O>,
{
  type Offset = O;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn new(start: Self::Offset, end: Self::Offset) -> Self {
    SimpleSpan::new(start, end)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_range(self) -> core::ops::Range<Self::Offset> {
    self.start..self.end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start_ref(&self) -> &Self::Offset {
    self.start_ref()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start_mut(&mut self) -> &mut Self::Offset {
    self.start_mut()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_start(self) -> Self::Offset {
    self.start
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end_ref(&self) -> &Self::Offset {
    self.end_ref()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end_mut(&mut self) -> &mut Self::Offset {
    self.end_mut()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_end(self) -> Self::Offset {
    self.end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn bump(&mut self, n: &Self::Offset) {
    self.bump(n);
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

/// A lightweight span representing a range of positions in source input.
///
/// `SimpleSpan` is a simple but powerful type that tracks where in the source code a particular
/// element came from. It stores just two byte offsets: the start and end positions.
/// While similar to [`Range<usize>`], `SimpleSpan` provides additional methods tailored for
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
/// `SimpleSpan` is designed to be:
/// - **Copy**: Can be freely copied without allocation (just two `usize` values)
/// - **Comparable**: Supports equality and ordering for span-based algorithms
/// - **Hashable**: Can be used as map/set keys for span-indexed data structures
/// - **Chumsky-compatible**: Implements `chumsky::span::SimpleSpan` for parser integration
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use tokit::utils::SimpleSpan;
///
/// // Create a span covering characters 10-20
/// let span = SimpleSpan::new(10, 20);
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
/// use tokit::utils::SimpleSpan;
///
/// // try_new returns None for invalid spans
/// assert!(SimpleSpan::try_new(10, 5).is_none());  // end < start
/// assert!(SimpleSpan::try_new(10, 10).is_some()); // empty span is valid
/// assert!(SimpleSpan::try_new(10, 20).is_some()); // normal span
/// ```
///
/// ## SimpleSpan Manipulation
///
/// ```rust
/// use tokit::utils::SimpleSpan;
///
/// let mut span = SimpleSpan::new(10, 20);
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
/// span.bump(&5);
/// assert_eq!(span.start(), 20);
/// assert_eq!(span.end(), 35);
/// ```
///
/// ## Builder-Style Methods
///
/// ```rust
/// use tokit::utils::SimpleSpan;
///
/// let span = SimpleSpan::new(0, 10)
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
/// use tokit::utils::SimpleSpan;
///
/// fn report_error(message: &str, span: SimpleSpan, source: &str) {
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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct SimpleSpan<Offset = usize> {
  pub(crate) start: Offset,
  pub(crate) end: Offset,
}

impl<O> core::fmt::Display for SimpleSpan<O>
where
  O: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}..{}", self.start, self.end)
  }
}

impl<O> SimpleSpan<&O>
where
  O: Clone,
{
  /// Clone the span into owned offsets.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn cloned(self) -> SimpleSpan<O> {
    SimpleSpan {
      start: self.start.clone(),
      end: self.end.clone(),
    }
  }
}

impl SimpleSpan {
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.bump_start(3);
  /// assert_eq!(span, SimpleSpan::new(8, 15));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.bump_end(5);
  /// assert_eq!(span, SimpleSpan::new(5, 20));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.bump(&10);
  /// assert_eq!(span, SimpleSpan::new(15, 25));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.set_start(10);
  /// assert_eq!(span, SimpleSpan::new(10, 15));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.set_end(20);
  /// assert_eq!(span, SimpleSpan::new(5, 20));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15).with_start(10);
  /// assert_eq!(span, SimpleSpan::new(10, 15));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15).with_end(20);
  /// assert_eq!(span, SimpleSpan::new(5, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_end_const(mut self, end: usize) -> Self {
    self.end = end;
    self
  }
}

impl<O> SimpleSpan<O> {
  /// Convert to a span of references.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15);
  /// let span_ref = span.as_ref();
  /// assert_eq!(*span_ref.start, 5);
  /// assert_eq!(*span_ref.end, 15);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_ref(&self) -> SimpleSpan<&O> {
    SimpleSpan {
      start: &self.start,
      end: &self.end,
    }
  }

  /// Convert to a span of mutable references.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// let span_mut = span.as_mut();
  /// *span_mut.start = 10;
  /// *span_mut.end = 20;
  /// assert_eq!(span, SimpleSpan::new(10, 20));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> SimpleSpan<&mut O> {
    SimpleSpan {
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.bump_start(3);
  /// assert_eq!(span, SimpleSpan::new(8, 15));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.bump_end(5);
  /// assert_eq!(span, SimpleSpan::new(5, 20));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.bump(&10);
  /// assert_eq!(span, SimpleSpan::new(15, 25));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.set_start(10);
  /// assert_eq!(span, SimpleSpan::new(10, 15));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
  /// span.set_end(20);
  /// assert_eq!(span, SimpleSpan::new(5, 20));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15).with_start(10);
  /// assert_eq!(span, SimpleSpan::new(10, 15));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15).with_end(20);
  /// assert_eq!(span, SimpleSpan::new(5, 20));
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15);
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15);
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15);
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15);
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let mut span = SimpleSpan::new(5, 15);
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15);
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let empty = SimpleSpan::new(5, 5);
  /// assert!(empty.is_empty());
  ///
  /// let not_empty = SimpleSpan::new(5, 15);
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
  /// use tokit::utils::SimpleSpan;
  ///
  /// let span = SimpleSpan::new(5, 15);
  /// assert_eq!(span.range(), 5..15);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn range(&self) -> Range<&O> {
    &self.start..&self.end
  }
}

impl<O> From<Range<O>> for SimpleSpan<O>
where
  O: Ord,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(range: Range<O>) -> Self {
    Self::new(range.start, range.end)
  }
}

impl<O> From<SimpleSpan<O>> for Range<O> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(span: SimpleSpan<O>) -> Self {
    span.start..span.end
  }
}

impl<O> From<(O, O)> for SimpleSpan<O>
where
  O: Ord,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from((start, end): (O, O)) -> Self {
    Self::new(start, end)
  }
}

impl<O> From<SimpleSpan<O>> for (O, O) {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(span: SimpleSpan<O>) -> Self {
    (span.start, span.end)
  }
}

/// A value paired with its source location span.
///
/// `Spanned<D>` combines a value of type `D` with a span `S` that indicates where in
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
/// use tokit::utils::{Span, Spanned};
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
/// use tokit::utils::{Span, Spanned};
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
/// use tokit::utils::{Span, Spanned};
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
/// use tokit::utils::{Span, Spanned};
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
/// use tokit::utils::{Span, Spanned};
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
/// use tokit::utils::{Span, Spanned};
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
pub struct Spanned<D, S = SimpleSpan> {
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

impl<D, S> Spanned<&D, &S> {
  /// Returns a copied version of the spanned value.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn copied(&self) -> Spanned<D, S>
  where
    D: Copy,
    S: Copy,
  {
    Spanned {
      span: *self.span,
      data: *self.data,
    }
  }

  /// Returns a cloned version of the spanned value.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn cloned(&self) -> Spanned<D, S>
  where
    D: Clone,
    S: Clone,
  {
    self.map(Clone::clone, Clone::clone)
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
  /// use tokit::utils::{Span, Spanned};
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
  /// use tokit::utils::{Span, Spanned};
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
  /// use tokit::utils::{Span, Spanned};
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
  /// use tokit::utils::{Span, Spanned};
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
  /// use tokit::utils::{Span, Spanned};
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
  /// use tokit::utils::{Span, Spanned};
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
  /// use tokit::utils::{Span, Spanned};
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

  /// Consume the spanned value and return the span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_span(self) -> S {
    self.span
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

  /// Map the span to a new value, preserving the data.
  #[inline]
  pub fn map_span<F, T>(self, f: F) -> Spanned<D, T>
  where
    F: FnOnce(S) -> T,
  {
    Spanned {
      span: f(self.span),
      data: self.data,
    }
  }

  /// Map both the span and data to new values.
  #[inline]
  pub fn map<F, G, U, T>(self, f: F, g: G) -> Spanned<U, T>
  where
    F: FnOnce(S) -> T,
    G: FnOnce(D) -> U,
  {
    Spanned {
      span: f(self.span),
      data: g(self.data),
    }
  }
}

impl<D, S> From<Spanned<D, S>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: Spanned<D, S>) -> Self {}
}

impl<D, S> From<Spanned<D, S>> for Ignored<()> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: Spanned<D, S>) -> Self {
    Ignored::default()
  }
}
