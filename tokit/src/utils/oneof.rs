#[cfg(any(feature = "std", feature = "alloc"))]
use std::{borrow::Cow, vec::Vec};

#[cfg(not(any(feature = "std", feature = "alloc")))]
type Inner<'a, T> = &'a [T];

#[cfg(any(feature = "std", feature = "alloc"))]
type Inner<'a, T> = Cow<'a, [T]>;

/// Feature-aware slice with a unified API.
///
/// - **`no_std` + `no_alloc`**: stores a `&'a [T]`
/// - **`alloc` / `std`**: stores a `Cow<'a, [T]>`
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  derive_more::Display,
  derive_more::AsMut,
  derive_more::AsRef,
  derive_more::Deref,
  derive_more::DerefMut,
)]
#[cfg_attr(not(any(feature = "std", feature = "alloc")), derive(Copy))]
#[repr(transparent)]
#[display("{inner}")]
pub struct OneOf<'a, T: Clone> {
  #[deref]
  #[deref_mut]
  #[as_ref]
  #[as_mut]
  inner: Inner<'a, T>,
}

impl<'a, T: Clone> OneOf<'a, T> {
  /// Creates a new message from the provided representation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::OneOf;
  ///
  /// let msg = OneOf::new("greeting");
  /// assert_eq!(msg.as_str(), "greeting");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn new(inner: impl Into<Inner<'a, T>>) -> Self {
    Self {
      inner: inner.into(),
    }
  }

  /// Creates a message from a `'static` string without allocation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::OneOf;
  ///
  /// const MSG: OneOf = OneOf::from_static("hello");
  /// assert_eq!(MSG.as_str(), "hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_slice(value: &'a [T]) -> Self {
    Self {
      inner: Self::borrow_const(value),
    }
  }

  /// Returns the message as a string slice.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::OneOf;
  ///
  /// let msg = OneOf::from_static("world");
  /// assert_eq!(msg.as_str(), "world");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_slice(&self) -> &[T] {
    Self::as_inner_helper(&self.inner)
  }

  /// Borrows the inner representation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::OneOf;
  /// # #[cfg(any(feature = "std", feature = "alloc"))]
  /// # {
  /// use std::borrow::Cow;
  ///
  /// let msg = OneOf::from_static("inner");
  /// assert!(matches!(msg.as_inner(), &Cow::Borrowed("inner")));
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_inner(&self) -> &Inner<'a, T> {
    &self.inner
  }

  /// Consumes the message and returns the inner representation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use tokit::utils::OneOf;
  /// # #[cfg(any(feature = "std", feature = "alloc"))]
  /// # {
  /// use std::borrow::Cow;
  ///
  /// let msg = OneOf::from_static("consume");
  /// let inner = msg.into_inner();
  /// assert_eq!(inner, Cow::Borrowed("consume"));
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_inner(self) -> Inner<'a, T> {
    self.inner
  }
}

impl<'a, T: Clone> From<&'a [T]> for OneOf<'a, T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(value: &'a [T]) -> Self {
    Self {
      inner: Self::borrow_const(value),
    }
  }
}

impl<T: Clone> AsRef<[T]> for OneOf<'_, T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &[T] {
    self.as_inner()
  }
}

impl<T: Clone> core::borrow::Borrow<[T]> for OneOf<'_, T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn borrow(&self) -> &[T] {
    self.as_inner()
  }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
const _: () = {
  impl<'a, T: Clone> OneOf<'a, T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    const fn borrow_const(value: &'a [T]) -> Inner<'a, T> {
      value
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    const fn as_inner_helper(inner: &Inner<'a, T>) -> &'a [T] {
      inner
    }
  }

  impl<'a, T: Clone> From<OneOf<'a, T>> for &'a [T] {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: OneOf<'a, T>) -> Self {
      value.inner
    }
  }

  impl<'a, T: Clone> From<&OneOf<'a, T>> for &'a [T] {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: &OneOf<'a, T>) -> Self {
      value.inner
    }
  }
};

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  impl<'a, T: Clone> OneOf<'a, T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    const fn borrow_const(value: &'a [T]) -> Inner<'a, T> {
      Cow::Borrowed(value)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    const fn as_inner_helper<'b>(inner: &'b Inner<'a, T>) -> &'b [T] {
      match inner {
        Cow::Borrowed(value) => value,
        Cow::Owned(value) => value.as_slice(),
      }
    }

    /// Creates a message by taking ownership of a vector.
    ///
    /// ## Examples
    ///
    /// ```
    /// use tokit::utils::OneOf;
    ///
    /// let msg = OneOf::from_string(std::string::String::from("owned"));
    /// assert_eq!(msg.as_str(), "owned");
    /// ```
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn from_vec(value: Vec<T>) -> Self {
      Self {
        inner: Cow::Owned(value),
      }
    }

    /// Ensures the message is owned and returns a mutable reference.
    ///
    /// ## Examples
    ///
    /// ```
    /// use tokit::utils::OneOf;
    ///
    /// let mut msg = OneOf::from_static("grow");
    /// msg.to_mut().push('!');
    /// assert_eq!(msg.as_str(), "grow!");
    /// ```
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn to_mut(&mut self) -> &mut [T] {
      self.inner.to_mut()
    }
  }

  impl<T: Clone> From<Vec<T>> for OneOf<'_, T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: Vec<T>) -> Self {
      OneOf::from_vec(value)
    }
  }

  impl<'a, T: Clone> From<Cow<'a, [T]>> for OneOf<'a, T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: Cow<'a, [T]>) -> Self {
      Self { inner: value }
    }
  }

  impl<'a, T: Clone> From<OneOf<'a, T>> for Cow<'a, [T]> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: OneOf<'a, T>) -> Self {
      value.inner
    }
  }

  impl<'a, T: Clone> From<&OneOf<'a, T>> for Cow<'a, [T]> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: &OneOf<'a, T>) -> Self {
      value.inner.clone()
    }
  }
};
