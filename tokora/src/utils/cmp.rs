/// A trait for cross-type equivalence checking.
///
/// `Equivalent` provides a way to compare values of different types for logical equivalence,
/// particularly useful when working with various string and byte slice types. Unlike `PartialEq`,
/// which requires both sides to be the same type, `Equivalent` allows comparison across
/// compatible but distinct types.
///
/// # Use Cases
///
/// - **Comparing `str` with `[u8]`**: Check if a string matches raw bytes
/// - **Comparing different string types**: `Bytes`, `HipStr`, `BStr`, etc.
/// - **Hash map lookups**: Use `str` to find entries keyed by `String`
/// - **Flexible APIs**: Accept multiple input types while comparing uniformly
///
/// # Provided Implementations
///
/// tokora provides implementations for:
/// - `str` ↔ `[u8]`: UTF-8 byte comparison
/// - `[u8]` ↔ `str`: Byte comparison
/// - `Bytes` ↔ `str`/`[u8]` (with `bytes` feature)
/// - `HipStr`/`HipByt` (with `hipstr` feature)
/// - `BStr` ↔ `str`/`[u8]`/`BStr` (with `bstr` feature)
/// - the `smol_bytes` family (with `smol-bytes` feature)
///
/// All comparisons work in both directions (`A` equivalent to `B` and `B` equivalent to `A`).
///
/// # Examples
///
/// ## Basic String/Bytes Comparison
///
/// ```rust
/// use tokora::utils::Equivalent;
///
/// let text = "hello";
/// let bytes = b"hello";
///
/// assert!(text.equivalent(bytes));
/// assert!(bytes.equivalent(text));
/// ```
///
/// ## Different String Types
///
/// ```rust,ignore
/// use tokora::utils::Equivalent;
/// use bytes::Bytes;
///
/// let static_str = "hello";
/// let owned_bytes = Bytes::from_static(b"hello");
///
/// assert!(owned_bytes.equivalent(static_str));
/// assert!(static_str.equivalent(owned_bytes.as_ref()));
/// ```
///
/// ## Case-Insensitive Comparison (Custom Implementation)
///
/// ```rust,ignore
/// use tokora::utils::Equivalent;
///
/// struct CaseInsensitive<'a>(&'a str);
///
/// impl Equivalent<str> for CaseInsensitive<'_> {
///     fn equivalent(&self, other: &str) -> bool {
///         self.0.eq_ignore_ascii_case(other)
///     }
/// }
///
/// let case_insensitive = CaseInsensitive("HELLO");
/// assert!(case_insensitive.equivalent("hello"));
/// assert!(case_insensitive.equivalent("HELLO"));
/// ```
///
/// # Design Notes
///
/// This trait is particularly useful for:
/// - **Hash map implementations** that need to look up keys by equivalent types
/// - **Parser APIs** that accept multiple input formats
/// - **Zero-copy comparisons** across different container types
///
/// The trait is sealed for reference types to provide blanket implementations
/// automatically.
pub trait Equivalent<T: ?Sized> {
  /// Returns `true` if `self` is logically equivalent to `other`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tokora::utils::Equivalent;
  ///
  /// assert!("hello".equivalent(b"hello"));
  /// assert!(b"world".equivalent("world"));
  /// ```
  fn equivalent(&self, other: &T) -> bool;
}

impl<T, O> Equivalent<O> for &T
where
  T: Equivalent<O> + ?Sized,
  O: ?Sized,
{
  #[inline(always)]
  fn equivalent(&self, other: &O) -> bool {
    T::equivalent(*self, other)
  }
}

impl<T, O> Equivalent<O> for &mut T
where
  T: Equivalent<O> + ?Sized,
  O: ?Sized,
{
  #[inline(always)]
  fn equivalent(&self, other: &O) -> bool {
    T::equivalent(*self, other)
  }
}

impl<T> Equivalent<T> for str
where
  T: AsRef<[u8]> + ?Sized,
{
  #[inline(always)]
  fn equivalent(&self, other: &T) -> bool {
    self.as_bytes().eq(other.as_ref())
  }
}

impl<T> Equivalent<T> for [u8]
where
  T: AsRef<[u8]> + ?Sized,
{
  #[inline(always)]
  fn equivalent(&self, other: &T) -> bool {
    self.eq(other.as_ref())
  }
}

pub(crate) const fn __assert_equivalent_impl<T, O>()
where
  O: Equivalent<T> + ?Sized,
  T: ?Sized,
{
}

const _: () = {
  __assert_equivalent_impl::<str, [u8]>();
  __assert_equivalent_impl::<[u8], str>();
  __assert_equivalent_impl::<str, str>();
  __assert_equivalent_impl::<[u8], [u8]>();
  __assert_equivalent_impl::<&str, &[u8]>();
  __assert_equivalent_impl::<&[u8], &str>();
  __assert_equivalent_impl::<&str, &str>();
  __assert_equivalent_impl::<&[u8], &[u8]>();
};
