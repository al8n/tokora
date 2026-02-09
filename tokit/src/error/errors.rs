//! Error collection container that adapts to allocation environments.
//!
//! This module provides the `Errors` type for collecting multiple errors during parsing
//! or validation. The container automatically adapts based on available features:
//!
//! - **no_std (no alloc)**: Uses `ConstGenericArrayDeque<E, 2>` with fixed capacity of 2 errors
//! - **alloc/std**: Uses `VecDeque<E>` for unlimited error collection
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use tokit::error::Errors;
//!
//! let mut errors = Errors::new();
//! errors.push("First error");
//! errors.push("Second error");
//!
//! assert_eq!(errors.len(), 2);
//! assert!(!errors.is_empty());
//! ```
//!
//! ## Iteration
//!
//! ```rust
//! use tokit::error::Errors;
//!
//! let mut errors = Errors::new();
//! errors.push(1);
//! errors.push(2);
//!
//! let sum: i32 = errors.iter().sum();
//! assert_eq!(sum, 3);
//! ```

use core::fmt::{Debug, Display};

#[cfg(not(any(feature = "alloc", feature = "std")))]
use generic_arraydeque::ConstGenericArrayDeque;

#[cfg(any(feature = "alloc", feature = "std"))]
use std::collections::VecDeque;

/// Default error container for no-alloc environments.
///
/// Uses a stack-allocated `ConstGenericArrayDeque` with capacity for 2 errors.
/// When the capacity is exceeded, additional errors are dropped and
/// [`Errors::overflowed`](Errors::overflowed) becomes `true`.
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type DefaultContainer<E> = ConstGenericArrayDeque<E, 2>;

/// Default error container for alloc/std environments.
///
/// Uses a heap-allocated `VecDeque` for unlimited error collection.
#[cfg(any(feature = "alloc", feature = "std"))]
pub type DefaultContainer<E> = VecDeque<E>;

/// A collection of errors that adapts to the allocation environment.
///
/// This type is generic over both the error type `E` and the container `C`.
/// By default:
/// - In no-alloc environments: Uses `ConstGenericArrayDeque<E, 2>` (capacity of 2)
/// - In alloc/std environments: Uses `VecDeque<E>` (unlimited capacity)
///
/// # Type Parameters
///
/// - `E`: The error type to store
/// - `C`: The container type (defaults to environment-appropriate container)
///
/// # Examples
///
/// ## Using Default Container
///
/// ```rust
/// use tokit::error::Errors;
///
/// let mut errors = Errors::new();
/// errors.push("Error 1");
/// errors.push("Error 2");
///
/// assert_eq!(errors.len(), 2);
/// ```
///
/// ## Type Inference
///
/// ```rust
/// use tokit::error::Errors;
///
/// // Type inference works seamlessly
/// let mut errors = Errors::new();
/// errors.push("Error 1");
/// errors.push("Error 2");
///
/// let first: Option<&&str> = errors.front();
/// assert_eq!(first, Some(&"Error 1"));
/// ```
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  derive_more::Deref,
  derive_more::DerefMut,
  derive_more::AsRef,
  derive_more::AsMut,
)]
pub struct Errors<E, C = DefaultContainer<E>> {
  #[deref]
  #[deref_mut]
  #[as_ref]
  #[as_mut]
  container: C,
  overflowed_flag: bool,
  _phantom: core::marker::PhantomData<E>,
}

// Implementation for no-alloc environments (ConstGenericArrayDeque)
#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<E> Errors<E> {
  /// Creates a new empty error collection.
  ///
  /// In no-alloc environments, this creates a `ConstGenericArrayDeque` with capacity 2.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tokit::error::Errors;
  ///
  /// let errors: Errors<String> = Errors::new();
  /// assert!(errors.is_empty());
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self::new_in(DefaultContainer::new())
  }
}

// Implementation for alloc/std environments (VecDeque)
#[cfg(any(feature = "alloc", feature = "std"))]
impl<E> Errors<E> {
  /// Creates a new empty error collection.
  ///
  /// In alloc/std environments, this creates an empty `VecDeque`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tokit::error::Errors;
  ///
  /// let errors: Errors<String> = Errors::new();
  /// assert!(errors.is_empty());
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self::new_in(VecDeque::new())
  }

  /// Returns the number of errors the collection can hold without reallocating.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tokit::error::Errors;
  ///
  /// let errors: Errors<String> = Errors::with_capacity(10);
  /// assert_eq!(errors.capacity(), 10);
  /// ```
  #[inline]
  pub fn capacity(&self) -> usize {
    self.container.capacity()
  }

  /// Reserves capacity for at least `additional` more errors.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tokit::error::Errors;
  ///
  /// let mut errors: Errors<String> = Errors::new();
  /// errors.reserve(10);
  /// assert!(errors.capacity() >= 10);
  /// ```
  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    self.container.reserve(additional);
  }
}

impl<E, Container> Errors<E, Container>
where
  Container: super::ErrorContainer<E>,
{
  /// Pushes an error into the collection, marking `overflowed` if it doesn't fit.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn push(&mut self, error: E) {
    let _ = self.try_push(error);
  }

  /// Attempts to push an error, returning it back if capacity is exhausted.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_push(&mut self, error: E) -> Result<(), E> {
    match super::ErrorContainer::try_push(&mut self.container, error) {
      Ok(()) => Ok(()),
      Err(err) => {
        self.overflowed_flag = true;
        Err(err)
      }
    }
  }

  /// Returns `true` if any error has been dropped because of limited capacity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn overflowed(&self) -> bool {
    self.overflowed_flag
  }

  /// Reports the remaining capacity when the backing container is bounded.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn remaining_capacity(&self) -> Option<usize> {
    super::ErrorContainer::remaining_capacity(&self.container)
  }

  /// Returns `true` when a bounded container cannot accept more errors.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_full(&self) -> bool {
    matches!(self.remaining_capacity(), Some(0))
  }

  /// Creates a new empty error collection with the specified capacity.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tokit::error::Errors;
  ///
  /// let errors: Errors<String> = Errors::with_capacity(5);
  /// assert_eq!(errors.len(), 0);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_capacity(capacity: usize) -> Self {
    Self::new_in(Container::with_capacity(capacity))
  }
}

impl<E, Container> Errors<E, Container> {
  #[inline]
  const fn new_in(container: Container) -> Self {
    Self {
      container,
      overflowed_flag: false,
      _phantom: core::marker::PhantomData,
    }
  }
}

// Default trait implementations
impl<E, Container> Default for Errors<E, Container>
where
  Container: Default,
{
  #[inline]
  fn default() -> Self {
    Self::new_in(Container::default())
  }
}

// AsRef and AsMut implementations
impl<E, C> AsRef<[E]> for Errors<E, C>
where
  C: AsRef<[E]>,
{
  #[inline]
  fn as_ref(&self) -> &[E] {
    self.container.as_ref()
  }
}

impl<E, C> AsMut<[E]> for Errors<E, C>
where
  C: AsMut<[E]>,
{
  #[inline]
  fn as_mut(&mut self) -> &mut [E] {
    self.container.as_mut()
  }
}

// Display implementation for better error reporting
impl<E, C> Display for Errors<E, C>
where
  E: Display,
  C: AsRef<[E]>,
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let errors = self.container.as_ref();

    if errors.is_empty() {
      return Ok(());
    }

    if errors.len() == 1 {
      write!(f, "{}", errors[0])
    } else {
      writeln!(f, "{} errors:", errors.len())?;
      for (i, error) in errors.iter().enumerate() {
        write!(f, "  {}. {}", i + 1, error)?;
        if i < errors.len() - 1 {
          writeln!(f)?;
        }
      }
      Ok(())
    }
  }
}

impl<'a, E, Container> IntoIterator for &'a Errors<E, Container>
where
  &'a Container: IntoIterator<Item = &'a E>,
{
  type Item = &'a E;
  type IntoIter = <&'a Container as IntoIterator>::IntoIter;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    (&self.container).into_iter()
  }
}

impl<E, Container> IntoIterator for Errors<E, Container>
where
  Container: IntoIterator<Item = E>,
{
  type Item = E;
  type IntoIter = Container::IntoIter;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.container.into_iter()
  }
}

impl<E, Container> FromIterator<E> for Errors<E, Container>
where
  Container: FromIterator<E>,
{
  #[inline]
  fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
    Self::from_container(Container::from_iter(iter))
  }
}

impl<E, C> From<E> for Errors<E, C>
where
  C: FromIterator<E>,
{
  #[inline]
  fn from(error: E) -> Self {
    Self::from_iter(core::iter::once(error))
  }
}

impl<E, C> Errors<E, C> {
  /// Creates an `Errors` instance from an existing container.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// # #[cfg(any(feature = "alloc", feature = "std"))] {
  /// use tokit::error::{Errors, DefaultContainer};
  ///
  /// let errors = Errors::<&str, DefaultContainer<_>>::from_container(["Error 1", "Error 2"].into_iter().collect());
  /// assert_eq!(errors.len(), 2);
  /// # }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_container(container: C) -> Self {
    Self::new_in(container)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use generic_arraydeque::ConstGenericArrayDeque;

  #[test]
  fn test_new() {
    let _: Errors<&str> = Errors::new();
  }

  #[test]
  fn test_push_and_len() {
    let mut errors = Errors::new();
    errors.push("Error 1");
    assert_eq!(errors.len(), 1);
    errors.push("Error 2");
    assert_eq!(errors.len(), 2);
  }

  #[test]
  fn test_clear() {
    let mut errors = Errors::new();
    errors.push("Error");
    errors.clear();
    assert!(errors.is_empty());
  }

  #[test]
  fn test_iteration() {
    let mut errors = Errors::new();
    errors.push(1);
    errors.push(2);

    let sum: i32 = errors.iter().sum();
    assert_eq!(sum, 3);
  }

  #[test]
  fn test_overflow_tracking() {
    type SmallErrors<'a> = Errors<&'a str, ConstGenericArrayDeque<&'a str, 1>>;
    let mut errors: SmallErrors<'_> = Errors::from_container(ConstGenericArrayDeque::<_, 1>::new());

    assert!(!errors.overflowed());
    errors.push("first");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors.remaining_capacity(), Some(0));
    assert!(errors.is_full());

    errors.push("second");
    assert!(errors.overflowed());
    assert_eq!(errors.len(), 1);
  }

  #[test]
  fn test_try_push_reports_error() {
    type SmallErrors<'a> = Errors<&'a str, ConstGenericArrayDeque<&'a str, 1>>;
    let mut errors: SmallErrors<'_> = Errors::from_container(ConstGenericArrayDeque::<_, 1>::new());

    assert!(errors.try_push("first").is_ok());
    assert!(errors.try_push("second").is_err());
    assert!(errors.overflowed());
  }

  #[cfg(any(feature = "alloc", feature = "std"))]
  #[test]
  fn test_with_capacity() {
    let errors: Errors<&str> = Errors::with_capacity(10);
    assert_eq!(errors.capacity(), 10);
    assert!(errors.is_empty());
  }

  #[cfg(any(feature = "alloc", feature = "std"))]
  #[test]
  fn test_pop() {
    use crate::error::ErrorContainer;

    let mut errors = Errors::new();
    errors.push(1);
    errors.push(2);

    assert_eq!(errors.pop(), Some(1));
    assert_eq!(errors.pop(), Some(2));
    assert_eq!(errors.pop(), None);
  }
}
