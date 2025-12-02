use super::{Span, AsSpan, IntoComponents, IntoSpan};

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
}
