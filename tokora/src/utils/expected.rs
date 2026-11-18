//! Expected value enumeration for parser error messages.
//!
//! This module provides the [`Expected`] type, which is used to represent what
//! token or value was expected during parsing when an error occurs.

use derive_more::{From, IsVariant, TryUnwrap, Unwrap};

/// The set of alternatives an [`Expected::OneOf`] carries: a slice that is borrowed, or, with
/// `alloc` or `std`, owned.
///
/// Without `alloc` or `std` this is `cower::Cow<'a, [T], cower::BorrowedPolicy>`: it stores the
/// `&'a [T]` itself, has the layout of one, is `Copy` for every `T`, and has no owned form. With
/// `alloc` or `std` it is `cower::Cow<'a, [T]>`, backed by `std::borrow::Cow<'a, [T]>`, so it can
/// also own a `Vec<T>`. It is one name for two types because no single `cower` policy keeps both
/// tiers: the default policy stores a slice without `alloc` only for `T: 'static`, and the
/// borrowed-only policy cannot own.
///
/// `from_ref` builds a borrowed one in every tier, in a `const` context too:
///
/// ```
/// use tokora::utils::CowSlice;
///
/// const KEYWORDS: CowSlice<'static, &str> = CowSlice::<&str>::from_ref(&["if", "while"]);
/// assert_eq!(KEYWORDS.as_slice(), &["if", "while"]);
/// assert!(KEYWORDS.is_borrowed());
/// ```
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub type CowSlice<'a, T> = cower::Cow<'a, [T], cower::BorrowedPolicy>;

/// The set of alternatives an [`Expected::OneOf`] carries: a slice that is borrowed, or, with
/// `alloc` or `std`, owned.
///
/// Without `alloc` or `std` this is `cower::Cow<'a, [T], cower::BorrowedPolicy>`: it stores the
/// `&'a [T]` itself, has the layout of one, is `Copy` for every `T`, and has no owned form. With
/// `alloc` or `std` it is `cower::Cow<'a, [T]>`, backed by `std::borrow::Cow<'a, [T]>`, so it can
/// also own a `Vec<T>`. It is one name for two types because no single `cower` policy keeps both
/// tiers: the default policy stores a slice without `alloc` only for `T: 'static`, and the
/// borrowed-only policy cannot own.
///
/// `from_ref` builds a borrowed one in every tier, in a `const` context too:
///
/// ```
/// use tokora::utils::CowSlice;
///
/// const KEYWORDS: CowSlice<'static, &str> = CowSlice::<&str>::from_ref(&["if", "while"]);
/// assert_eq!(KEYWORDS.as_slice(), &["if", "while"]);
/// assert!(KEYWORDS.is_borrowed());
/// ```
#[cfg(any(feature = "std", feature = "alloc"))]
pub type CowSlice<'a, T> = cower::Cow<'a, [T]>;

/// An enumeration representing expected tokens or values in parsing contexts.
///
/// This type is used to express what token or value was expected during parsing or validation.
/// It can represent either a single expected value or multiple alternative values.
///
/// # Examples
///
/// ```
/// use tokora::utils::{CowSlice, Expected};
///
/// // A single expected token
/// let single = Expected::One("identifier");
/// assert_eq!(format!("{}", single), "expected 'identifier'");
///
/// // Multiple expected tokens
/// let multiple =
///   Expected::OneOf(CowSlice::<&str>::from_ref(&["identifier", "number", "string"]));
/// assert_eq!(format!("{}", multiple), "expected one of: 'identifier', 'number', 'string'");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, IsVariant, Unwrap, TryUnwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum Expected<'a, T: Clone> {
  /// A single expected token or value.
  One(T),
  /// Multiple alternative expected tokens or values.
  OneOf(CowSlice<'a, T>),
}

impl<'a, T: Clone> Expected<'a, T> {
  /// Creates a new `Expected` variant with a single expected value.
  ///
  /// This is equivalent to `Expected::One(expected)` but provides a more ergonomic API.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokora::utils::Expected;
  ///
  /// let expected = Expected::one("identifier");
  /// if let Expected::One(value) = expected {
  ///     assert_eq!(value, "identifier");
  /// }
  /// ```
  #[inline]
  pub const fn one(expected: T) -> Self {
    Self::One(expected)
  }

  /// Creates a new `Expected` variant with multiple alternative expected values.
  ///
  /// This is equivalent to `Expected::OneOf(expected)` but provides a more ergonomic API.
  ///
  /// # Examples
  ///
  /// ```
  /// use tokora::utils::Expected;
  ///
  /// let expected = Expected::one_of(&["if", "while", "for"]);
  /// if let Expected::OneOf(values) = expected {
  ///     assert_eq!(values.as_slice(), &["if", "while", "for"]);
  /// }
  /// ```
  #[inline]
  pub const fn one_of(expected: &'a [T]) -> Self {
    Self::OneOf(CowSlice::<'a, T>::from_ref(expected))
  }
}

impl<T: core::fmt::Display + Clone> core::fmt::Display for Expected<'_, T> {
  #[inline(always)]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::One(expected) => write!(f, "expected '{expected}'"),
      Self::OneOf(expected) => {
        write!(f, "expected one of: ")?;
        for (i, exp) in expected.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "'{}'", exp)?;
        }
        Ok(())
      }
    }
  }
}
