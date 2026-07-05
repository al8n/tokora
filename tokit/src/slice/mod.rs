use super::utils::IntoComponents;

#[cfg(feature = "bytes_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
mod bytes_1;

#[cfg(feature = "bstr_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bstr_1")))]
mod bstr_1;

#[cfg(feature = "hipstr_0_8")]
#[cfg_attr(docsrs, doc(cfg(feature = "hipstr_0_8")))]
mod hipstr_0_8;

/// The slice type returned by lexers' sources.
pub trait Slice<'source>: PartialEq + Eq + core::fmt::Debug {
  /// The character type used by the lexer.
  ///
  /// - Use `char` for text-based lexers processing UTF-8 strings
  /// - Use `u8` for byte-based lexers processing binary data or non-UTF-8 input
  ///
  /// This type must match the character type used by the Logos lexer's source.
  type Char: Copy + core::fmt::Debug + PartialEq + Eq + core::hash::Hash;

  /// An iterator over the characters in the slice.
  type Iter<'a>: Iterator<Item = Self::Char>
  where
    Self: 'a;

  /// An iterator over the characters in the slice with their offsets to the start of the slice.
  type PositionedIter<'a>: Iterator<Item = (usize, Self::Char)>
  where
    Self: 'a;

  /// Returns an iterator over the characters in the slice.
  fn iter<'a>(&'a self) -> Self::Iter<'a>
  where
    Self: 'a;

  /// Returns an iterator over the characters in the slice with their offsets to the start of the slice.
  fn positioned_iter<'a>(&'a self) -> Self::PositionedIter<'a>
  where
    Self: 'a;

  /// Returns the length of the slice.
  fn len(&self) -> usize;

  /// Returns `true` if the slice is empty.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

impl<'source> Slice<'source> for &'source [u8] {
  type Char = u8;

  type Iter<'a>
    = core::iter::Copied<core::slice::Iter<'a, u8>>
  where
    Self: 'a;

  type PositionedIter<'a>
    = core::iter::Enumerate<core::iter::Copied<core::slice::Iter<'a, u8>>>
  where
    Self: 'a;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn iter<'a>(&'a self) -> Self::Iter<'a>
  where
    Self: 'a,
  {
    <[u8]>::iter(self).copied()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn positioned_iter<'a>(&'a self) -> Self::PositionedIter<'a>
  where
    Self: 'a,
  {
    <[u8]>::iter(self).copied().enumerate()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    <[u8]>::len(self)
  }
}

impl<'source> Slice<'source> for &'source str {
  type Char = char;

  type Iter<'a>
    = core::str::Chars<'a>
  where
    Self: 'a;

  type PositionedIter<'a>
    = core::str::CharIndices<'a>
  where
    Self: 'a;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn iter<'a>(&'a self) -> Self::Iter<'a>
  where
    Self: 'a,
  {
    self.chars()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn positioned_iter<'a>(&'a self) -> Self::PositionedIter<'a>
  where
    Self: 'a,
  {
    self.char_indices()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    <str>::len(self)
  }
}

/// A value paired with its slice metadata.
///
/// `Sliced<D, Src>` combines a value of type `D` with slice metadata of type `Src`.
/// This is fundamental for tracking the origin of data, such as file names, URLs,
/// module paths, or any other contextual information about where the data came from.
/// Unlike [`Spanned`](crate::span::Spanned) which tracks *location within* a slice,
/// `Sliced` tracks *which* slice the data came from.
///
/// # Design
///
/// `Sliced` uses public fields for direct access, but also provides accessor methods
/// for consistency. It implements `Deref` and `DerefMut` to allow transparent access
/// to the inner data while keeping slice information available when needed.
///
/// # Common Patterns
///
/// ## Transparent Access via Deref
///
/// Thanks to `Deref`, you can call methods on the wrapped value directly:
///
/// ```rust
/// use tokit::slice::Sliced;
///
/// let sliced_str = Sliced::new("main.rs", "hello world");
///
/// // Can call str methods directly
/// assert_eq!(sliced_str.len(), 11);
/// assert_eq!(sliced_str.to_uppercase(), "HELLO WORLD");
///
/// // But can still access the slice
/// assert_eq!(sliced_str.slice(), "main.rs");
/// ```
///
/// ## Tracking File Origins
///
/// ```rust,ignore
/// use tokit::slice::Sliced;
/// use std::path::PathBuf;
///
/// // Parse configuration from different slices
/// let config_from_file = Sliced::new(
///     PathBuf::from("/etc/app/config.toml"),
///     parse_config(file_contents)
/// );
///
/// let config_from_env = Sliced::new(
///     PathBuf::from("<environment>"),
///     parse_env_config()
/// );
///
/// // Later, when reporting errors, you know where the config came from
/// if let Err(e) = validate(&config_from_file) {
///     eprintln!("Invalid config in {}: {}", config_from_file.slice().display(), e);
/// }
/// ```
///
/// ## Multi-File Compilation
///
/// ```rust,ignore
/// use tokit::slice::Sliced;
///
/// struct Module {
///     name: String,
///     items: Vec<Item>,
/// }
///
/// // Each module knows which file it came from
/// let modules: Vec<Sliced<Module, String>> = vec![
///     Sliced::new("src/main.rs".to_string(), parse_file("main.rs")),
///     Sliced::new("src/lib.rs".to_string(), parse_file("lib.rs")),
///     Sliced::new("src/utils.rs".to_string(), parse_file("utils.rs")),
/// ];
///
/// // When linking, you can report cross-module errors with file context
/// for module in &modules {
///     for item in &module.items {
///         if let Err(e) = resolve_item(item, &modules) {
///             eprintln!("Error in {}: {}", module.slice(), e);
///         }
///     }
/// }
/// ```
///
/// ## Mapping Values While Preserving Source
///
/// ```rust
/// use tokit::slice::Sliced;
///
/// let sliced_str = Sliced::new("input.txt", "42");
///
/// // Parse the string, keeping the same slice
/// let parsed: Sliced<i32, &str> = sliced_str.map_data(|s| s.parse::<i32>().unwrap());
///
/// assert_eq!(*parsed, 42);
/// assert_eq!(parsed.slice(), "input.txt");
/// ```
///
/// ## Building AST with File Context
///
/// ```rust,ignore
/// use tokit::slice::Sliced;
/// use tokit::{Span};
/// use tokit::span::Spanned;
/// use std::path::PathBuf;
///
/// // Combine Sliced and Spanned for complete location tracking
/// type Located<T> = Sliced<Spanned<T>, PathBuf>;
///
/// enum Expr {
///     Number(i64),
///     Call { func: String, args: Vec<Located<Expr>> },
/// }
///
/// // Each expression knows both which file it's in AND where in that file
/// let expr: Located<Expr> = Sliced::new(
///     PathBuf::from("src/main.rs"),
///     Spanned::new(
///         Span::new(100, 150),
///         Expr::Call {
///             func: "print".to_string(),
///             args: vec![/* ... */],
///         }
///     )
/// );
///
/// // Can report: "Error in src/main.rs at line 5, column 10"
/// ```
///
/// ## Error Reporting with Source Context
///
/// ```rust,ignore
/// fn type_error<T>(expected: &str, got: &Sliced<T, String>) -> Error
/// where
///     T: core::fmt::Debug
/// {
///     Error {
///         message: format!(
///             "Type error in {}: expected {}, got {:?}",
///             got.slice(),
///             expected,
///             got.data()
///         ),
///         slice: got.slice().clone(),
///     }
/// }
/// ```
///
/// ## Incremental Compilation
///
/// ```rust,ignore
/// use tokit::slice::Sliced;
/// use std::collections::HashMap;
/// use std::time::SystemTime;
///
/// struct CachedModule {
///     compiled: CompiledCode,
///     timestamp: SystemTime,
/// }
///
/// // Track which files need recompilation
/// let mut cache: HashMap<String, CachedModule> = HashMap::new();
///
/// fn compile_if_needed(file: Sliced<String, String>) -> CompiledCode {
///     let slice_file = file.slice();
///     let modified = fs::metadata(slice_file).unwrap().modified().unwrap();
///
///     if let Some(cached) = cache.get(slice_file) {
///         if cached.timestamp >= modified {
///             return cached.compiled.clone();
///         }
///     }
///
///     // Recompile because slice changed
///     let compiled = compile(&file.data);
///     cache.insert(slice_file.clone(), CachedModule {
///         compiled: compiled.clone(),
///         timestamp: modified,
///     });
///     compiled
/// }
/// ```
///
/// # Trait Implementations
///
/// - **`Deref` / `DerefMut`**: Access the inner data transparently
/// - **`Display`**: Delegates to the inner data's `Display` implementation
/// - **`IntoComponents`**: Destructure into `(Src, D)` tuple
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use tokit::slice::Sliced;
///
/// let sliced = Sliced::new("config.toml", "debug = true");
///
/// assert_eq!(sliced.slice(), "config.toml");
/// assert_eq!(sliced.data(), &"debug = true");
/// assert_eq!(*sliced, "debug = true"); // Via Deref
/// ```
///
/// ## Destructuring
///
/// ```rust
/// use tokit::slice::Sliced;
///
/// let sliced = Sliced::new("file.txt", 42);
///
/// let (slice, value) = sliced.into_components();
/// assert_eq!(slice, "file.txt");
/// assert_eq!(value, 42);
/// ```
///
/// ## Mutable Access
///
/// ```rust
/// use tokit::slice::Sliced;
///
/// let mut sliced = Sliced::new("input", 10);
///
/// // Modify the data
/// *sliced += 5;
/// assert_eq!(*sliced, 15);
///
/// // Modify the slice
/// *sliced.slice_mut() = "modified";
/// assert_eq!(sliced.slice(), "modified");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Sliced<D, Src = ()> {
  /// The slice covers the data.
  pub(crate) slice: Src,
  /// The wrapped data value.
  pub(crate) data: D,
}

impl<D, Src> AsRef<Src> for Sliced<D, Src> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &Src {
    self.slice_ref()
  }
}

impl<D, Src> core::ops::Deref for Sliced<D, Src> {
  type Target = D;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

impl<D, Src> core::ops::DerefMut for Sliced<D, Src> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.data
  }
}

impl<D, Src> core::fmt::Display for Sliced<D, Src>
where
  D: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.data.fmt(f)
  }
}

impl<D, Src> core::error::Error for Sliced<D, Src>
where
  D: core::error::Error,
  Src: core::fmt::Debug,
{
}

impl<D, Src> IntoComponents for Sliced<D, Src> {
  type Components = (Src, D);

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_components(self) -> Self::Components {
    (self.slice, self.data)
  }
}

impl<D, Src> Sliced<D, Src> {
  /// Create a new sliced value.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(slice: Src, data: D) -> Self {
    Self { slice, data }
  }

  /// Get a copy of the slice.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::slice::Sliced;
  ///
  /// let sliced = Sliced::new("file.rs", "data");
  /// assert_eq!(sliced.slice(), "file.rs");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn slice(&self) -> Src
  where
    Src: Copy,
  {
    self.slice
  }

  /// Get a reference to the slice.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::slice::Sliced;
  ///
  /// let sliced = Sliced::new("config.toml", "data");
  /// assert_eq!(sliced.slice_ref(), &"config.toml");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn slice_ref(&self) -> &Src {
    &self.slice
  }

  /// Get a mutable reference to the slice.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::slice::Sliced;
  ///
  /// let mut sliced = Sliced::new("old.txt", "data");
  /// *sliced.slice_mut() = "new.txt";
  /// assert_eq!(sliced.slice(), "new.txt");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn slice_mut(&mut self) -> &mut Src {
    &mut self.slice
  }

  /// Get a reference to the data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::slice::Sliced;
  ///
  /// let sliced = Sliced::new("file.txt", 42);
  /// assert_eq!(*sliced.data(), 42);
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
  /// use tokit::slice::Sliced;
  ///
  /// let mut sliced = Sliced::new("file.txt", 42);
  /// *sliced.data_mut() = 100;
  /// assert_eq!(*sliced.data(), 100);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data_mut(&mut self) -> &mut D {
    &mut self.data
  }

  /// Returns a reference to the slice and data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::slice::Sliced;
  ///
  /// let sliced = Sliced::new(String::from("file.txt"), String::from("hello"));
  /// let borrowed: Sliced<&String, &String> = sliced.as_ref();
  /// assert_eq!(borrowed.data(), &"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_ref(&self) -> Sliced<&D, &Src> {
    Sliced {
      slice: &self.slice,
      data: &self.data,
    }
  }

  /// Returns a mutable reference to the slice and data.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::slice::Sliced;
  ///
  /// let mut sliced = Sliced::new(String::from("file.txt"), String::from("hello"));
  /// let mut borrowed: Sliced<&mut String, &mut String> = sliced.as_mut();
  /// borrowed.data_mut().push_str(" world");
  /// assert_eq!(sliced.data(), &"hello world");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(&mut self) -> Sliced<&mut D, &mut Src> {
    Sliced {
      slice: &mut self.slice,
      data: &mut self.data,
    }
  }

  /// Consume the sliced value and return the slice.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_slice(self) -> Src {
    self.slice
  }

  /// Consume the sliced value and return the data.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_data(self) -> D {
    self.data
  }

  /// Decompose the sliced value into its slice and data.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (Src, D) {
    (self.slice, self.data)
  }

  /// Map the data to a new value, preserving the slice.
  #[inline]
  pub fn map_data<F, U>(self, f: F) -> Sliced<U, Src>
  where
    F: FnOnce(D) -> U,
  {
    Sliced {
      slice: self.slice,
      data: f(self.data),
    }
  }
}

#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;
  use std::{format, string::String, vec, vec::Vec};

  // --- Slice trait tests for &[u8] ---

  #[test]
  fn u8_slice_len() {
    let s: &[u8] = b"hello";
    assert_eq!(Slice::len(&s), 5);
  }

  #[test]
  fn u8_slice_is_empty() {
    let empty: &[u8] = b"";
    assert!(Slice::is_empty(&empty));
    let non_empty: &[u8] = b"a";
    assert!(!Slice::is_empty(&non_empty));
  }

  #[test]
  fn u8_slice_iter() {
    let s: &[u8] = b"abc";
    let chars: Vec<u8> = Slice::iter(&s).collect();
    assert_eq!(chars, vec![b'a', b'b', b'c']);
  }

  #[test]
  fn u8_slice_positioned_iter() {
    let s: &[u8] = b"ab";
    let items: Vec<(usize, u8)> = Slice::positioned_iter(&s).collect();
    assert_eq!(items, vec![(0, b'a'), (1, b'b')]);
  }

  // --- Slice trait tests for &str ---

  #[test]
  fn str_slice_len() {
    let s: &str = "hello";
    assert_eq!(Slice::len(&s), 5);
  }

  #[test]
  fn str_slice_len_multibyte() {
    let s: &str = "\u{00E9}"; // 2-byte char
    assert_eq!(Slice::len(&s), 2);
  }

  #[test]
  fn str_slice_is_empty() {
    let empty: &str = "";
    assert!(Slice::is_empty(&empty));
    let non_empty: &str = "a";
    assert!(!Slice::is_empty(&non_empty));
  }

  #[test]
  fn str_slice_iter() {
    let s: &str = "abc";
    let chars: Vec<char> = Slice::iter(&s).collect();
    assert_eq!(chars, vec!['a', 'b', 'c']);
  }

  #[test]
  fn str_slice_iter_multibyte() {
    let s: &str = "\u{00E9}x";
    let chars: Vec<char> = Slice::iter(&s).collect();
    assert_eq!(chars, vec!['\u{00E9}', 'x']);
  }

  #[test]
  fn str_slice_positioned_iter() {
    let s: &str = "ab";
    let items: Vec<(usize, char)> = Slice::positioned_iter(&s).collect();
    assert_eq!(items, vec![(0, 'a'), (1, 'b')]);
  }

  #[test]
  fn str_slice_positioned_iter_multibyte() {
    let s: &str = "\u{00E9}a"; // 2 bytes + 1 byte
    let items: Vec<(usize, char)> = Slice::positioned_iter(&s).collect();
    assert_eq!(items, vec![(0, '\u{00E9}'), (2, 'a')]);
  }

  // --- Sliced tests ---

  #[test]
  fn sliced_new_and_accessors() {
    let s = Sliced::new("file.rs", 42);
    assert_eq!(s.slice(), "file.rs");
    assert_eq!(*s.data(), 42);
  }

  #[test]
  fn sliced_slice_ref() {
    let s = Sliced::new("file.rs", 42);
    assert_eq!(s.slice_ref(), &"file.rs");
  }

  #[test]
  fn sliced_slice_mut() {
    let mut s = Sliced::new("old.rs", 42);
    *s.slice_mut() = "new.rs";
    assert_eq!(s.slice(), "new.rs");
  }

  #[test]
  fn sliced_data_mut() {
    let mut s = Sliced::new("file.rs", 42);
    *s.data_mut() = 100;
    assert_eq!(*s.data(), 100);
  }

  #[test]
  fn sliced_deref() {
    let s = Sliced::new("file.rs", 42i32);
    let val: &i32 = &s;
    assert_eq!(*val, 42);
  }

  #[test]
  fn sliced_deref_mut() {
    let mut s = Sliced::new("file.rs", 42i32);
    *s = 100;
    assert_eq!(*s, 100);
  }

  #[test]
  fn sliced_display() {
    let s = Sliced::new("file.rs", "hello");
    assert_eq!(format!("{s}"), "hello");
  }

  #[test]
  fn sliced_as_ref_borrowed() {
    let s = Sliced::new(String::from("file.rs"), String::from("data"));
    let borrowed = s.as_ref();
    assert_eq!(borrowed.data(), &&String::from("data"));
  }

  #[test]
  fn sliced_as_mut_borrowed() {
    let mut s = Sliced::new("file.rs", 42i32);
    {
      let m = s.as_mut();
      *m.data = 100;
    }
    assert_eq!(*s.data(), 100);
  }

  #[test]
  fn sliced_into_slice() {
    let s = Sliced::new("file.rs", 42);
    assert_eq!(s.into_slice(), "file.rs");
  }

  #[test]
  fn sliced_into_data() {
    let s = Sliced::new("file.rs", 42);
    assert_eq!(s.into_data(), 42);
  }

  #[test]
  fn sliced_into_components() {
    let s = Sliced::new("file.rs", 42);
    let (slice, data) = s.into_components();
    assert_eq!(slice, "file.rs");
    assert_eq!(data, 42);
  }

  #[test]
  fn sliced_into_components_trait() {
    let s = Sliced::new("file.rs", 42);
    let (slice, data) = IntoComponents::into_components(s);
    assert_eq!(slice, "file.rs");
    assert_eq!(data, 42);
  }

  #[test]
  fn sliced_map_data() {
    let s = Sliced::new("file.rs", "42");
    let mapped = s.map_data(|d| d.parse::<i32>().unwrap());
    assert_eq!(*mapped.data(), 42);
    assert_eq!(mapped.slice(), "file.rs");
  }

  #[test]
  fn sliced_as_ref_trait() {
    let s: Sliced<i32, &str> = Sliced::new("file.rs", 42);
    let r: &&str = <Sliced<i32, &str> as AsRef<&str>>::as_ref(&s);
    assert_eq!(r, &"file.rs");
  }
}
