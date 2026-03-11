use core::marker::PhantomData;

use derive_more::From;

pub use missing_token::*;
pub use unexpected_repeated_token::*;
pub use unexpected_token::*;

mod missing_token;
mod unexpected_repeated_token;
mod unexpected_token;

/// A marker type representing trailing tokens.
#[derive(Debug, Default, From, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Trailing<T: ?Sized, Lang: ?Sized = ()> {
  _marker: PhantomData<T>,
  _lang: PhantomData<Lang>,
}

impl<T: ?Sized> Trailing<T> {
  /// Creates a new `Trailing`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self
  where
    T: Sized,
  {
    Self::of()
  }
}

impl<T: ?Sized, Lang: ?Sized> Trailing<T, Lang> {
  /// Creates a new `Trailing` for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of() -> Self
  where
    T: Sized,
  {
    Self {
      _marker: PhantomData,
      _lang: PhantomData,
    }
  }
}

/// A marker type representing leading tokens.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Leading<T: ?Sized, Lang: ?Sized = ()> {
  _marker: PhantomData<T>,
  _lang: PhantomData<Lang>,
}

impl<T: ?Sized> Leading<T> {
  /// Creates a new `Leading`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self
  where
    T: Sized,
  {
    Self::of()
  }
}

impl<T: ?Sized, Lang: ?Sized> Leading<T, Lang> {
  /// Creates a new `Leading` for the given language
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of() -> Self
  where
    T: Sized,
  {
    Self {
      _marker: PhantomData,
      _lang: PhantomData,
    }
  }
}

/// A marker type representing repeated tokens.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RepeatedWhile<T: ?Sized, Lang: ?Sized = ()> {
  _marker: PhantomData<T>,
  _lang: PhantomData<Lang>,
}

impl<T: ?Sized> RepeatedWhile<T> {
  /// Creates a new `RepeatedWhile`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self
  where
    T: Sized,
  {
    Self::of()
  }
}

impl<T: ?Sized, Lang: ?Sized> RepeatedWhile<T, Lang> {
  /// Creates a new `RepeatedWhile` for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of() -> Self
  where
    T: Sized,
  {
    Self {
      _marker: PhantomData,
      _lang: PhantomData,
    }
  }
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
  use super::*;
  use std::format;

  #[test]
  fn trailing_new_and_of() {
    let t: Trailing<u32> = Trailing::new();
    let t2: Trailing<u32, ()> = Trailing::of();
    assert_eq!(t, t2);
  }

  #[test]
  fn trailing_default_debug_clone() {
    let t: Trailing<u32> = Default::default();
    let t2 = t.clone();
    assert_eq!(format!("{:?}", t), format!("{:?}", t2));
  }

  #[test]
  fn leading_new_and_of() {
    let l: Leading<u32> = Leading::new();
    let l2: Leading<u32, ()> = Leading::of();
    assert_eq!(l, l2);
  }

  #[test]
  fn leading_default_debug_clone() {
    let l: Leading<u32> = Default::default();
    let l2 = l.clone();
    assert_eq!(format!("{:?}", l), format!("{:?}", l2));
  }

  #[test]
  fn repeated_while_new_and_of() {
    let r: RepeatedWhile<u32> = RepeatedWhile::new();
    let r2: RepeatedWhile<u32, ()> = RepeatedWhile::of();
    assert_eq!(r, r2);
  }

  #[test]
  fn repeated_while_default_debug_clone() {
    let r: RepeatedWhile<u32> = Default::default();
    let r2 = r.clone();
    assert_eq!(format!("{:?}", r), format!("{:?}", r2));
  }
}
