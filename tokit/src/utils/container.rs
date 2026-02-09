use derive_more::{From, Into};

use std::vec::Vec;

/// A small vector which inlines 1 element to avoid allocations.
///
/// ## Example
///
/// ```rust
/// use tokit::utils::container::OneOrMore;
///
/// let mut vec = OneOrMore::new();
/// vec.push(42);
/// vec.push(100);
/// assert_eq!(vec.len(), 2);
/// ```
pub type OneOrMore<T> = SmallVec<T, 1>;

/// A small vector which inlines up to 2 elements to avoid allocations.
///
/// ## Example
///
/// ```rust
/// use tokit::utils::container::TwoOrMore;
///
/// let mut vec = TwoOrMore::new();
/// vec.push(1);
/// vec.push(2);
/// vec.push(3);
/// assert_eq!(vec.len(), 3);
/// ```
pub type TwoOrMore<T> = SmallVec<T, 2>;

/// A small vector which inlines up to 4 elements to avoid allocations.
///
/// ## Example
///
/// ```rust
/// use tokit::utils::container::FourOrMore;
///
/// let vec = FourOrMore::from_const([1, 2, 3, 4]);
/// assert_eq!(vec.len(), 4);
/// ```
pub type FourOrMore<T> = SmallVec<T, 4>;

/// A small vector which inlines up to `N` elements to avoid allocations.
/// It uses the [`smallvec`](https://docs.rs/smallvec) crate internally.
///
/// ## Example
///
/// ```rust
/// use tokit::utils::container::SmallVec;
///
/// // Create a SmallVec that inlines 4 elements
/// let mut vec = SmallVec::<i32, 4>::new();
/// vec.push(1);
/// vec.push(2);
/// assert_eq!(vec.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct SmallVec<T, const N: usize>(smallvec::SmallVec<[T; N]>);

impl<T, const N: usize> Default for SmallVec<T, N> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl<T, const N: usize> core::ops::Deref for SmallVec<T, N> {
  type Target = smallvec::SmallVec<[T; N]>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T, const N: usize> core::ops::DerefMut for SmallVec<T, N> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T, const N: usize> AsRef<[T]> for SmallVec<T, N> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &[T] {
    &self.0
  }
}

impl<T, const N: usize> AsMut<[T]> for SmallVec<T, N> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_mut(&mut self) -> &mut [T] {
    &mut self.0
  }
}

impl<T, const N: usize> core::iter::FromIterator<T> for SmallVec<T, N> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    Self(iter.into_iter().collect())
  }
}

impl<T, const N: usize> core::iter::Extend<T> for SmallVec<T, N> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    self.0.extend(iter);
  }
}

impl<T, const N: usize> IntoIterator for SmallVec<T, N> {
  type Item = T;
  type IntoIter = smallvec::IntoIter<[T; N]>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl<'a, T, const N: usize> core::iter::IntoIterator for &'a SmallVec<T, N> {
  type Item = &'a T;
  type IntoIter = core::slice::Iter<'a, T>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.0.iter()
  }
}

// #[cfg(feature = "chumsky")]
// impl<T, const N: usize> chumsky::container::Container<T> for SmallVec<T, N> {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn with_capacity(n: usize) -> Self {
//     Self::with_capacity(n)
//   }

//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn push(&mut self, item: T) {
//     self.0.push(item);
//   }
// }

impl<T, const N: usize> From<Vec<T>> for SmallVec<T, N> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(value: Vec<T>) -> Self {
    Self(smallvec::SmallVec::from_vec(value))
  }
}

impl<T, const N: usize> From<SmallVec<T, N>> for Vec<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(value: SmallVec<T, N>) -> Self {
    value.0.into_vec()
  }
}

impl<T, const N: usize> SmallVec<T, N> {
  /// Creates a new empty `SmallVec`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::utils::container::SmallVec;
  ///
  /// let vec = SmallVec::<i32, 4>::new();
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self(smallvec::SmallVec::new_const())
  }

  /// Creates a new `SmallVec` with the given capacity.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::utils::container::SmallVec;
  ///
  /// let vec = SmallVec::<i32, 2>::with_capacity(10);
  /// assert_eq!(vec.len(), 0);
  /// assert!(vec.capacity() >= 10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_capacity(capacity: usize) -> Self {
    Self(smallvec::SmallVec::with_capacity(capacity))
  }

  /// The array passed as an argument is moved to be an inline version of SmallVec.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::utils::container::SmallVec;
  ///
  /// let vec = SmallVec::from_const([1, 2, 3]);
  /// assert_eq!(vec.len(), 3);
  /// assert_eq!(vec[0], 1);
  /// assert_eq!(vec[1], 2);
  /// assert_eq!(vec[2], 3);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_const(vec: [T; N]) -> Self {
    Self(smallvec::SmallVec::from_const(vec))
  }
}
