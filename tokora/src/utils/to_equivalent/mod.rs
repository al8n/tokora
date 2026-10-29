#![allow(single_use_lifetimes)]

pub(crate) mod sealed {
  pub trait Sealed<T: ?Sized> {}

  impl<T, O> Sealed<O> for &T where T: Sealed<O> + ?Sized {}
}

/// A trait for converting to an equivalent type.
///
/// e.g. `&[u8]` is equivalent to [`bytes::Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html)
pub trait ToEquivalent<T>: sealed::Sealed<T> {
  /// Converts this element to an equivalent type `T`.
  fn to_equivalent(&self) -> T;
}

impl<T, O> ToEquivalent<O> for &T
where
  T: ToEquivalent<O> + ?Sized,
{
  #[inline(always)]
  fn to_equivalent(&self) -> O {
    T::to_equivalent(self)
  }
}

impl sealed::Sealed<str> for str {}
impl<'de: 'a, 'a> sealed::Sealed<&'a str> for &'de str {}

impl sealed::Sealed<[u8]> for [u8] {}
impl<'de: 'a, 'a> sealed::Sealed<&'a [u8]> for &'de [u8] {}

impl<'de: 'a, 'a> ToEquivalent<&'a str> for &'de str {
  #[inline(always)]
  fn to_equivalent(&self) -> &'a str {
    self
  }
}

impl<'de: 'a, 'a> ToEquivalent<&'a [u8]> for &'de [u8] {
  #[inline(always)]
  fn to_equivalent(&self) -> &'a [u8] {
    self
  }
}

/// A trait for converting into an equivalent type.
///
/// e.g. `&[u8]` is equivalent to [`bytes::Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html)
///
/// ## Example
///
/// ```rust
/// use tokora::utils::IntoEquivalent;
///
/// let bytes: &[u8] = b"hello";
/// let str_ref: &str = "world";
///
/// // Identity conversions (consuming)
/// let _: &[u8] = bytes.into_equivalent();
/// let _: &str = str_ref.into_equivalent();
/// ```
pub trait IntoEquivalent<T>: sealed::Sealed<T> {
  /// Consumes this element and converts it into an equivalent type `T`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokora::utils::IntoEquivalent;
  ///
  /// let data: &[u8] = b"test";
  /// let result: &[u8] = data.into_equivalent();
  /// assert_eq!(result, b"test");
  /// ```
  fn into_equivalent(self) -> T;
}

impl<'de: 'a, 'a> IntoEquivalent<&'a str> for &'de str {
  #[inline(always)]
  fn into_equivalent(self) -> &'a str {
    self
  }
}

impl<'de: 'a, 'a> IntoEquivalent<&'a [u8]> for &'de [u8] {
  #[inline(always)]
  fn into_equivalent(self) -> &'a [u8] {
    self
  }
}

#[cfg(test)]
#[allow(warnings)]
mod tests;
