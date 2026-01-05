/// A reference wrapper.
#[repr(transparent)]
pub struct ByRef<T: ?Sized>(T);

impl<T: ?Sized> core::ops::Deref for ByRef<T> {
  type Target = T;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.inner()
  }
}

impl<T: ?Sized> core::ops::DerefMut for ByRef<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.inner_mut()
  }
}

impl<'a, T: ?Sized> ByRef<T> {
  /// Create a new reference check wrapper.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn from_ref(target: &'a T) -> &'a Self {
    // SAFETY: This is safe because `ByRef` is `repr(transparent)` over `T`.
    unsafe { &*(target as *const T as *const Self) }
  }

  /// Create a new mutable reference check wrapper.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn from_ref_mut(target: &'a mut T) -> &'a mut Self {
    // SAFETY: This is safe because `ByRef` is `repr(transparent)` over `T`.
    unsafe { &mut *(target as *mut T as *mut Self) }
  }

  /// Returns the inner reference.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn inner(&self) -> &T {
    &self.0
  }

  /// Returns the inner mutable reference.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn inner_mut(&mut self) -> &mut T {
    &mut self.0
  }
}
