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

/// A span with no meaningful offsets, used as a type marker.
pub type PhantomSpan = super::Span<()>;

impl PhantomSpan {
  /// A zero-sized span for phantom usage.
  pub const PHANTOM: Self = Self { start: (), end: () };
}

/// A sliced value with no meaningful slice, used as a type marker.
pub type PhantomSliced = super::Sliced<(), ()>;

impl PhantomSliced {
  /// A zero-sized sliced value for phantom usage.
  pub const PHANTOM: Self = Self {
    slice: (),
    data: (),
  };
}

/// A located value with no meaningful source or span, used as a type marker.
pub type PhantomLocated = super::Located<(), (), ()>;

impl PhantomLocated {
  /// A zero-sized located value for phantom usage.
  pub const PHANTOM: Self = Self {
    slice: (),
    span: (),
    data: (),
  };
}
