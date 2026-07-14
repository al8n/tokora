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
