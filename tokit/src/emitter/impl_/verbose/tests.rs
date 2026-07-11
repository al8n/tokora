use super::*;
use std::{format, vec};

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

// ── Diagnostic labels: capture-at-emit snapshots ──────────────────────────────

fn enter(v: &mut Verbose<(), SimpleSpan>, label: &'static str) {
  <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::enter_label(v, label);
}

fn exit(v: &mut Verbose<(), SimpleSpan>) {
  <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::exit_label(v);
}

fn emit(v: &mut Verbose<(), SimpleSpan>, span: SimpleSpan) {
  let _ = <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::emit_error(
    v,
    Spanned::new(span, ()),
  );
}

// An inner emission snapshots outer+inner; after the inner scope closes, a later emission
// snapshots the outer label only — the stack follows the nesting of the `labelled` scopes.
#[test]
fn verbose_nested_labels_snapshot_outer_then_outer_and_inner_then_outer() {
  let mut v = Verbose::<(), SimpleSpan>::new();
  let a = SimpleSpan::new(0usize, 1usize);
  let b = SimpleSpan::new(1usize, 2usize);
  let c = SimpleSpan::new(2usize, 3usize);

  enter(&mut v, "outer");
  emit(&mut v, a); // open labels: [outer]
  enter(&mut v, "inner");
  emit(&mut v, b); // open labels: [outer, inner]
  exit(&mut v); // inner scope closes
  emit(&mut v, c); // open labels: [outer]
  exit(&mut v); // outer scope closes

  assert_eq!(v.labels()[&a], vec![vec!["outer"]]);
  assert_eq!(v.labels()[&b], vec![vec!["outer", "inner"]]);
  assert_eq!(v.labels()[&c], vec![vec!["outer"]]);
}

// An emission made with no open label snapshots an empty stack (and never allocates).
#[test]
fn verbose_unlabelled_emission_snapshots_empty_stack() {
  let mut v = Verbose::<(), SimpleSpan>::new();
  let a = SimpleSpan::new(0usize, 1usize);
  emit(&mut v, a);
  assert_eq!(v.labels()[&a], vec![Vec::<&'static str>::new()]);
}

// `labels()` is parallel to `errors()`: same span keys, same per-span group lengths, so
// same-span diagnostics each keep their own snapshot in emission order.
#[test]
fn verbose_labels_parallel_to_errors_same_span_group() {
  let mut v = Verbose::<(), SimpleSpan>::new();
  let a = SimpleSpan::new(0usize, 1usize);
  enter(&mut v, "x");
  emit(&mut v, a);
  emit(&mut v, a); // second diagnostic at the SAME span, still under [x]
  exit(&mut v);

  assert_eq!(v.errors().len(), v.labels().len(), "same span keys");
  assert_eq!(v.errors()[&a].len(), 2, "two diagnostics at the span");
  assert_eq!(
    v.labels()[&a],
    vec![vec!["x"], vec!["x"]],
    "one snapshot per diagnostic"
  );
}
