use core::marker::PhantomData;

use derive_more::From;

pub use missing_token::*;
pub use unexpected_token::*;

mod missing_token;
mod unexpected_token;

/// A marker type representing trailing tokens.
#[derive(Debug, Default, From, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Trailing<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> Trailing<T> {
  /// Creates a new `Trailing` from `T`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self
  where
    T: Sized,
  {
    Self(PhantomData)
  }
}

/// A marker type representing leading tokens.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Leading<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> Leading<T> {
  /// Creates a new `Leading` from `T`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self
  where
    T: Sized,
  {
    Self(PhantomData)
  }
}

/// A marker type representing repeated tokens.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Repeated<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> Repeated<T> {
  /// Creates a new `Leading` from `T`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self
  where
    T: Sized,
  {
    Self(PhantomData)
  }
}
