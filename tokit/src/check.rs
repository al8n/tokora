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
mod tests {
  use super::*;

  #[test]
  fn fn_check_impl() {
    let checker = |x: &i32| *x > 5;
    assert!(checker.check(&10));
    assert!(!checker.check(&3));
  }

  #[test]
  fn by_ref_check() {
    let checker = |x: &i32| *x > 5;
    let by_ref = checker.by_ref();
    assert!(by_ref.check(&10));
    assert!(!by_ref.check(&3));
  }

  #[test]
  fn by_ref_ref_check() {
    let checker = |x: &i32| *x == 42;
    let by_ref: &ByRef<_> = checker.by_ref();
    let ref_to_by_ref: &&ByRef<_> = &by_ref;
    assert!(Check::<i32, bool>::check(ref_to_by_ref, &42));
    assert!(!Check::<i32, bool>::check(ref_to_by_ref, &0));
  }

  #[test]
  fn check_with_custom_output() {
    let checker = |x: &str| x.len();
    assert_eq!(checker.check("hello"), 5);
    assert_eq!(checker.check(""), 0);
  }
}
