use super::{
  slice::Sliced,
  span::{SimpleSpan, Spanned},
  utils::IntoComponents,
};

/// A value with complete location information: both which source and where in that source.
///
/// `Located<D, Sp, Sl>` combines a value of type `D` with:
/// - A slice identifier `Sl` indicating *which* source the data came from
/// - A span `Sp` indicating *where* within that source the data is located
///
/// This provides the most complete location tracking possible, combining the benefits
/// of both [`Sliced`] (source tracking) and [`Spanned`] (position tracking).
///
/// # Design
///
/// `Located` uses public fields for direct access, but also provides accessor methods
/// for consistency. It implements `Deref` and `DerefMut` to allow transparent access
/// to the inner data while keeping full location information available when needed.
///
/// # Common Patterns
///
/// ## Transparent Access via Deref
///
/// Thanks to `Deref`, you can call methods on the wrapped value directly:
///
/// ```rust
/// use tokit::{Located, SimpleSpan};
///
/// let located = Located::new("main.rs", SimpleSpan::new(10, 15), "hello");
///
/// // Can call str methods directly
/// assert_eq!(located.len(), 5);
/// assert_eq!(located.to_uppercase(), "HELLO");
///
/// // But can still access location info
/// assert_eq!(located.slice(), "main.rs");
/// assert_eq!(located.span().start(), 10);
/// ```
///
/// ## Multi-File Error Reporting
///
/// ```rust,ignore
/// use tokit::{Located, SimpleSpan};
/// use std::path::PathBuf;
///
/// fn report_error<T>(loc: &Located<T, SimpleSpan, PathBuf>, message: &str)
/// where
///     T: core::fmt::Debug
/// {
///     eprintln!(
///         "Error in {}:{}:{}: {}\n  {:?}",
///         loc.slice().display(),
///         get_line_number(loc.span().start()),
///         get_column_number(loc.span().start()),
///         message,
///         loc.data()
///     );
/// }
/// ```
///
/// ## Building Complete AST Nodes
///
/// ```rust,ignore
/// use tokit::{Located, SimpleSpan};
/// use std::path::PathBuf;
///
/// type Loc<T> = Located<T, SimpleSpan, PathBuf>;
///
/// enum Expr {
///     Number(i64),
///     BinOp {
///         op: String,
///         left: Box<Loc<Expr>>,
///         right: Box<Loc<Expr>>,
///     },
/// }
///
/// // Each expression knows exactly where it came from
/// let expr = Loc::new(
///     PathBuf::from("src/calc.rs"),
///     SimpleSpan::new(45, 52),
///     Expr::Number(42)
/// );
///
/// // Can report: "Error in src/calc.rs:3:12-19"
/// ```
///
/// ## Cross-File Reference Checking
///
/// ```rust,ignore
/// use tokit::{Located, SimpleSpan};
///
/// fn check_reference(
///     reference: &Located<String, SimpleSpan, String>,
///     definition: &Located<String, SimpleSpan, String>
/// ) -> Result<(), String> {
///     if reference.slice() != definition.slice() {
///         Err(format!(
///             "Cross-file reference: {} (in {}) references {} (in {})",
///             reference.data(),
///             reference.slice(),
///             definition.data(),
///             definition.slice()
///         ))
///     } else {
///         Ok(())
///     }
/// }
/// ```
///
/// ## Mapping Values While Preserving Full Location
///
/// ```rust
/// use tokit::{Located, SimpleSpan};
///
/// let located_str = Located::new("input.txt", SimpleSpan::new(5, 7), "42");
///
/// // Parse the string, keeping both source and position
/// let parsed: Located<i32, SimpleSpan, &str> = located_str.map_data(|s| s.parse().unwrap());
///
/// assert_eq!(*parsed, 42);
/// assert_eq!(parsed.slice(), "input.txt");
/// assert_eq!(parsed.span().start(), 5);
/// ```
///
/// ## IDE Integration
///
/// ```rust,ignore
/// use tokit::{Located, SimpleSpan};
/// use std::path::PathBuf;
///
/// struct Diagnostic {
///     severity: Severity,
///     message: String,
///     location: Located<(), SimpleSpan, PathBuf>,
/// }
///
/// // Generate diagnostics with complete location info
/// fn undefined_variable(name: &Located<String, SimpleSpan, PathBuf>) -> Diagnostic {
///     Diagnostic {
///         severity: Severity::Error,
///         message: format!("Undefined variable '{}'", name.data()),
///         location: name.as_ref().map_data(|_| ()),
///     }
/// }
///
/// // IDE can jump to exact location:
/// // - Open file: diagnostic.location.slice()
/// // - Navigate to: diagnostic.location.span()
/// ```
///
/// ## Incremental Compilation with Position Tracking
///
/// ```rust,ignore
/// use tokit::{Located, SimpleSpan};
/// use std::collections::HashMap;
///
/// struct Definition {
///     name: String,
///     location: Located<(), SimpleSpan, String>,
/// }
///
/// // Track where each definition is located
/// let mut definitions: HashMap<String, Definition> = HashMap::new();
///
/// // When a file changes, only recheck definitions in that file
/// fn recheck_file(file: &str, definitions: &mut HashMap<String, Definition>) {
///     definitions.retain(|_, def| def.location.slice() != file);
///     // Re-parse and add new definitions from the changed file
/// }
/// ```
///
/// # Trait Implementations
///
/// - **`Deref` / `DerefMut`**: Access the inner data transparently
/// - **`Display`**: Delegates to the inner data's `Display` implementation
/// - **`IntoComponents`**: Destructure into `(Sl, Sp, D)` tuple
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use tokit::{Located, SimpleSpan};
///
/// let located = Located::new("file.rs", SimpleSpan::new(0, 5), "hello");
///
/// assert_eq!(located.slice(), "file.rs");
/// assert_eq!(located.span(), SimpleSpan::new(0, 5));
/// assert_eq!(located.data(), &"hello");
/// assert_eq!(*located, "hello"); // Via Deref
/// ```
///
/// ## Destructuring
///
/// ```rust
/// use tokit::{Located, SimpleSpan};
///
/// let located = Located::new("main.rs", SimpleSpan::new(10, 20), 42);
///
/// let (slice, span, value) = located.into_components();
/// assert_eq!(slice, "main.rs");
/// assert_eq!(span, SimpleSpan::new(10, 20));
/// assert_eq!(value, 42);
/// ```
///
/// ## Mutable Access
///
/// ```rust
/// use tokit::{Located, SimpleSpan};
///
/// let mut located = Located::new("input", SimpleSpan::new(0, 2), 10);
///
/// // Modify the data
/// *located += 5;
/// assert_eq!(*located, 15);
///
/// // Modify the slice
/// *located.slice_mut() = "output";
/// assert_eq!(located.slice(), "output");
///
/// // Modify the span
/// located.span_mut().set_end(5);
/// assert_eq!(located.span().end(), 5);
/// ```
///
/// ## Conversion from Spanned or Sliced
///
/// ```rust
/// use tokit::{Located, SimpleSpan};
/// use tokit::slice::Sliced;
/// use tokit::span::Spanned;
///
/// // From Spanned by adding slice info
/// let spanned = Spanned::new(SimpleSpan::new(5, 10), "data");
/// let located = Located::new("file.rs", spanned.span(), spanned.into_data());
///
/// // From Sliced by adding span info
/// let sliced = Sliced::new("config.toml", "value");
/// let located = Located::new(sliced.into_slice(), SimpleSpan::new(0, 5), "value");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Located<D, Sp = SimpleSpan, Sl = ()> {
  /// The slice identifier indicating which source this data came from.
  pub(crate) slice: Sl,
  /// The span indicating where in the source this data is located.
  pub(crate) span: Sp,
  /// The wrapped data value.
  pub(crate) data: D,
}

impl<D, Sp, Sl> core::ops::Deref for Located<D, Sp, Sl> {
  type Target = D;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

impl<D, Sp, Sl> core::ops::DerefMut for Located<D, Sp, Sl> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.data
  }
}

impl<D, Sp, Sl> core::fmt::Display for Located<D, Sp, Sl>
where
  D: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.data.fmt(f)
  }
}

impl<D, Sp, Sl> core::error::Error for Located<D, Sp, Sl>
where
  D: core::error::Error,
  Sp: core::fmt::Debug,
  Sl: core::fmt::Debug,
{
}

impl<D, Sp, Sl> IntoComponents for Located<D, Sp, Sl> {
  type Components = (Sl, Sp, D);

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_components(self) -> Self::Components {
    (self.slice, self.span, self.data)
  }
}

impl<D, Sp, Sl> Located<D, Sp, Sl> {
  /// Create a new located value.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let located = Located::new("file.rs", SimpleSpan::new(10, 15), "hello");
  /// assert_eq!(located.slice(), "file.rs");
  /// assert_eq!(located.span(), SimpleSpan::new(10, 15));
  /// assert_eq!(located.data(), &"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(slice: Sl, span: Sp, data: D) -> Self {
    Self { slice, span, data }
  }

  /// Get a copy of the slice.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let located = Located::new("main.rs", SimpleSpan::new(0, 5), "data");
  /// assert_eq!(located.slice(), "main.rs");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn slice(&self) -> Sl
  where
    Sl: Copy,
  {
    self.slice
  }

  /// Get a reference to the slice.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let located = Located::new("config.toml", SimpleSpan::new(5, 10), "data");
  /// assert_eq!(located.slice_ref(), &"config.toml");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn slice_ref(&self) -> &Sl {
    &self.slice
  }

  /// Get a mutable reference to the slice.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let mut located = Located::new("old.txt", SimpleSpan::new(0, 3), "data");
  /// *located.slice_mut() = "new.txt";
  /// assert_eq!(located.slice(), "new.txt");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn slice_mut(&mut self) -> &mut Sl {
    &mut self.slice
  }

  /// Get a copy of the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let located = Located::new("file.rs", SimpleSpan::new(5, 10), "data");
  /// assert_eq!(located.span(), SimpleSpan::new(5, 10));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> Sp
  where
    Sp: Copy,
  {
    self.span
  }

  /// Get a reference to the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let located = Located::new("file.rs", SimpleSpan::new(5, 10), "data");
  /// assert_eq!(located.span_ref(), &SimpleSpan::new(5, 10));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &Sp {
    &self.span
  }

  /// Get a mutable reference to the span.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let mut located = Located::new("file.rs", SimpleSpan::new(0, 5), "data");
  /// located.span_mut().set_end(10);
  /// assert_eq!(located.span().end(), 10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut Sp {
    &mut self.span
  }

  /// Get a reference to the data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let located = Located::new("file.txt", SimpleSpan::new(0, 2), 42);
  /// assert_eq!(*located.data(), 42);
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
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let mut located = Located::new("file.txt", SimpleSpan::new(0, 2), 42);
  /// *located.data_mut() = 100;
  /// assert_eq!(*located.data(), 100);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data_mut(&mut self) -> &mut D {
    &mut self.data
  }

  /// Returns a reference to the slice, span, and data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let located = Located::new(
  ///     String::from("file.txt"),
  ///     SimpleSpan::new(0, 5),
  ///     String::from("hello")
  /// );
  /// let borrowed: Located<&String, &SimpleSpan, &String> = located.as_ref();
  /// assert_eq!(borrowed.data(), &"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_ref(&self) -> Located<&D, &Sp, &Sl> {
    Located {
      slice: &self.slice,
      span: &self.span,
      data: &self.data,
    }
  }

  /// Returns a mutable reference to the slice, span, and data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::{Located, SimpleSpan};
  ///
  /// let mut located = Located::new(
  ///     String::from("file.txt"),
  ///     SimpleSpan::new(0, 5),
  ///     String::from("hello")
  /// );
  /// let mut borrowed: Located<&mut String, &mut SimpleSpan, &mut String> = located.as_mut();
  /// borrowed.data_mut().push_str(" world");
  /// assert_eq!(located.data(), &"hello world");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> Located<&mut D, &mut Sp, &mut Sl> {
    Located {
      slice: &mut self.slice,
      span: &mut self.span,
      data: &mut self.data,
    }
  }

  /// Consume the located value and return the slice.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_slice(self) -> Sl {
    self.slice
  }

  /// Consume the located value and return the span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_span(self) -> Sp {
    self.span
  }

  /// Consume the located value and return the data.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_data(self) -> D {
    self.data
  }

  /// Decompose the located value into its slice, span, and data.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (Sl, Sp, D) {
    (self.slice, self.span, self.data)
  }

  /// Convert into a `Spanned` value, discarding the slice information.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_spanned(self) -> Spanned<D, Sp> {
    Spanned::new(self.span, self.data)
  }

  /// Convert into a `Sliced` value, discarding the span information.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_sliced(self) -> Sliced<D, Sl> {
    Sliced::new(self.slice, self.data)
  }

  /// Map the data to a new value, preserving the slice and span.
  #[inline]
  pub fn map_data<F, U>(self, f: F) -> Located<U, Sp, Sl>
  where
    F: FnOnce(D) -> U,
  {
    Located {
      slice: self.slice,
      span: self.span,
      data: f(self.data),
    }
  }
}
