use crate::parser::ByRef;

/// A trait for checking
pub trait Check<T: ?Sized, O = bool> {
  /// Check against the target.
  fn check(&self, target: &T) -> O;

  /// Create a reference check wrapper.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn by_ref(&self) -> &ByRef<Self> {
    ByRef::from_ref(self)
  }
}

impl<F, T, O> Check<T, O> for F
where
  F: ?Sized + Fn(&T) -> O,
  T: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn check(&self, target: &T) -> O {
    (self)(target)
  }
}

impl<T: ?Sized, Target: ?Sized, O> Check<Target, O> for &ByRef<T>
where
  T: Check<Target, O>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn check(&self, target: &Target) -> O {
    (**self).check(target)
  }
}

#[cfg(test)]
#[allow(warnings)]
mod tests;
