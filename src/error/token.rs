use core::marker::PhantomData;

use derive_more::From;

pub use missing_token::*;
pub use unexpected_token::*;

mod missing_token;
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
pub struct Repeated<T: ?Sized, Lang: ?Sized = ()> {
  _marker: PhantomData<T>,
  _lang: PhantomData<Lang>,
}

impl<T: ?Sized> Repeated<T> {
  /// Creates a new `Repeated`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self
  where
    T: Sized,
  {
    Self::of()
  }
}

impl<T: ?Sized, Lang: ?Sized> Repeated<T, Lang> {
  /// Creates a new `Repeated` for the given language.
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
pub struct Separator<T: ?Sized, Lang: ?Sized = ()> {
  _marker: PhantomData<T>,
  _lang: PhantomData<Lang>,
}

impl<T: ?Sized> Separator<T> {
  /// Creates a new `Separator`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self
  where
    T: Sized,
  {
    Self::of()
  }
}

impl<T: ?Sized, Lang: ?Sized> Separator<T, Lang> {
  /// Creates a new `Separator` for the given language.
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
