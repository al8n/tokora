use super::*;
use crate::span::SimpleSpan;

type Urt = UnexpectedRepeatedToken<u8, SimpleSpan>;

#[test]
fn new_and_accessors() {
  let span = SimpleSpan::const_new(0, 5);
  let err = Urt::new(span, 3);
  assert_eq!(err.span(), span);
  assert_eq!(err.count(), 3);
}

#[test]
fn of_with_lang() {
  let span = SimpleSpan::const_new(1, 4);
  let err = UnexpectedRepeatedToken::<u8, SimpleSpan, ()>::of(span, 2);
  assert_eq!(err.span(), span);
  assert_eq!(err.count(), 2);
}

#[test]
fn span_ref_and_span_mut() {
  let span = SimpleSpan::const_new(0, 5);
  let mut err = Urt::new(span, 1);
  assert_eq!(*err.span_ref(), span);
  *err.span_mut() = SimpleSpan::const_new(1, 6);
  assert_eq!(err.span(), SimpleSpan::const_new(1, 6));
}

#[test]
fn expand() {
  let mut err = Urt::new(SimpleSpan::const_new(0, 3), 1);
  err.expand(SimpleSpan::const_new(3, 7), 2);
  assert_eq!(err.span(), SimpleSpan::const_new(0, 7));
  assert_eq!(err.count(), 3);
}

#[test]
fn derive_traits() {
  let err = Urt::new(SimpleSpan::const_new(0, 1), 1);
  let err2 = err.clone();
  assert_eq!(err, err2);
  let _ = format!("{:?}", err);
}

#[test]
fn from_into_unit() {
  let err = Urt::new(SimpleSpan::const_new(0, 1), 1);
  let _: () = err.into();
}
