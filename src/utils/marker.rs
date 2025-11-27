/// A no-operation marker type.
pub struct Noop<T: ?Sized>(core::marker::PhantomData<T>);

impl<T: ?Sized> Noop<T> {
  /// Creates a new `Noop`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self(core::marker::PhantomData)
  }
}

impl<T: ?Sized> Default for Noop<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

impl<T: ?Sized> core::fmt::Debug for Noop<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Noop")
  }
}

impl<T: ?Sized> Clone for Noop<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: ?Sized> Copy for Noop<T> {}

/// A marker type representing ignored values.
pub struct Ignored<T: ?Sized>(core::marker::PhantomData<T>);

impl<T> From<T> for Ignored<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: T) -> Self {
    Self(core::marker::PhantomData)
  }
}

impl<T: ?Sized> Default for Ignored<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

impl<T: ?Sized> core::fmt::Debug for Ignored<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Ignored")
  }
}

impl<T: ?Sized> Clone for Ignored<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: ?Sized> Copy for Ignored<T> {}
