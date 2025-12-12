use core::ops::{AddAssign, Range};

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
/// span.bump(5);
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
  /// span.bump(10);
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
  /// span.bump(10);
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
