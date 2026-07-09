use super::*;
use std::format;

#[test]
fn verbose_new_is_empty() {
  let v = Verbose::<()>::new();
  assert!(v.errors().is_empty());
}

#[test]
fn verbose_default_is_empty() {
  let v = Verbose::<()>::default();
  assert!(v.errors().is_empty());
}

#[test]
fn verbose_clone() {
  let v = Verbose::<()>::new();
  let v2 = v.clone();
  assert!(v2.errors().is_empty());
}

#[test]
fn verbose_debug() {
  let v = Verbose::<()>::new();
  let dbg = format!("{:?}", v);
  assert!(dbg.contains("Verbose"));
}

#[test]
fn verbose_errors_returns_btreemap_ref() {
  let v = Verbose::<()>::new();
  let errs: &BTreeMap<SimpleSpan, Vec<()>> = v.errors();
  assert_eq!(errs.len(), 0);
}

#[test]
fn verbose_emit_error_same_span_accumulates() {
  let mut v = Verbose::<(), SimpleSpan>::new();
  let span = SimpleSpan::new(0usize, 5usize);
  let _ = <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::emit_error(
    &mut v,
    Spanned::new(span, ()),
  );
  let _ = <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::emit_error(
    &mut v,
    Spanned::new(span, ()),
  );
  // Two errors at the SAME span must both be retained (append, not overwrite).
  assert_eq!(v.errors().len(), 1, "one span key");
  assert_eq!(
    v.errors().get(&span).map(Vec::len),
    Some(2),
    "both same-span errors retained rather than overwritten"
  );
}

#[test]
fn verbose_emit_error_inserts() {
  let mut v = Verbose::<(), SimpleSpan>::new();
  let span = SimpleSpan::new(0usize, 5usize);
  let spanned_err = Spanned::new(span, ());
  let result = <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::emit_error(
    &mut v,
    spanned_err,
  );
  assert!(result.is_ok());
  assert_eq!(v.errors().len(), 1);
  assert!(v.errors().contains_key(&span));
}

#[test]
fn verbose_emit_error_multiple_spans() {
  let mut v = Verbose::<(), SimpleSpan>::new();
  let span1 = SimpleSpan::new(0usize, 5usize);
  let span2 = SimpleSpan::new(10usize, 15usize);
  let _ = <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::emit_error(
    &mut v,
    Spanned::new(span1, ()),
  );
  let _ = <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::emit_error(
    &mut v,
    Spanned::new(span2, ()),
  );
  assert_eq!(v.errors().len(), 2);
}

#[test]
fn verbose_emit_lexer_error_inserts() {
  let mut v = Verbose::<(), SimpleSpan>::new();
  let span = SimpleSpan::new(0usize, 5usize);
  let spanned_err = Spanned::new(span, ());
  let result = <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::emit_lexer_error(
    &mut v,
    spanned_err,
  );
  assert!(result.is_ok());
  assert_eq!(v.errors().len(), 1);
}
