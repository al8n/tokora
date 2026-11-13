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

// ── Skipped-region records: the third channel on the shared log ──────────────

fn hole(v: &mut Verbose<(), SimpleSpan>, span: SimpleSpan, skipped: usize) {
  let _ = <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::emit_skipped_region(
    v, span, skipped,
  );
}

// A hole record lands in `skipped_regions()` (with its label snapshot), advances the emission
// checkpoint, AND now surfaces through `diagnostics()` in its emitted position — interleaved with
// the payload channels rather than skipped.
//
// SANCTIONED TEST UPDATE (W7 Cluster A, item 1): the former assertion here — that `diagnostics()`
// yields only the payload channels (`spans == [a, b]`, hole excluded) — encoded the W3-era
// known limitation. The reshape resolves that seam, so this test is updated to assert the hole
// now appears in order as a `DiagnosticKind::SkippedRegion`.
#[test]
fn verbose_hole_records_interleave_into_diagnostics_in_emission_order() {
  let mut v = Verbose::<(), SimpleSpan>::new();
  let a = SimpleSpan::new(0usize, 1usize);
  let b = SimpleSpan::new(2usize, 5usize);

  enter(&mut v, "ctx");
  emit(&mut v, a); // log[0]: error at a
  hole(&mut v, b, 3); // log[1]: hole at b
  emit(&mut v, b); // log[2]: error at b — same span as the hole
  exit(&mut v);

  // Every record advanced the shared checkpoint.
  let ckp = <Verbose<(), SimpleSpan> as Emitter<'_, crate::lexer::DummyLexer>>::checkpoint(&mut v);
  assert_eq!(ckp, 3, "the hole record rides the shared emission log");

  // The hole is still readable through its own accessor, labels captured like any record.
  assert_eq!(v.skipped_regions().get(&b), Some(&vec![3usize]));
  assert_eq!(v.skipped_region_labels()[&b], vec![vec!["ctx"]]);

  // diagnostics() now replays ALL THREE records in emission order: error@a, hole@b, error@b.
  // The same-span error at `b` is still intact after the hole — the hole advances only its own
  // per-span cursor.
  let seq: Vec<(SimpleSpan, DiagnosticKind<'_, ()>)> =
    v.diagnostics().map(|d| (*d.span(), d.kind())).collect();
  assert_eq!(seq.len(), 3, "all three records replay");
  assert_eq!(seq[0].0, a);
  assert!(seq[0].1.is_error(), "log[0] is the error at a");
  assert_eq!(seq[1].0, b);
  assert!(
    seq[1].1.is_skipped_region(),
    "log[1] is the hole at b, in order"
  );
  match seq[1].1 {
    DiagnosticKind::SkippedRegion(skipped) => assert_eq!(skipped, 3, "the skipped-token count"),
    _ => unreachable!(),
  }
  assert_eq!(seq[2].0, b);
  assert!(
    seq[2].1.is_error(),
    "log[2] is the error at b, after the hole"
  );

  // Labels ride every record, the hole included.
  let labels: Vec<Vec<&'static str>> = v.diagnostics().map(|d| d.labels().to_vec()).collect();
  assert_eq!(labels, vec![vec!["ctx"], vec!["ctx"], vec!["ctx"]]);

  // The hole reports as a soft (Warning-tier) event; `kind()` distinguishes it from a real
  // warning payload.
  assert_eq!(
    v.diagnostics().nth(1).map(|d| d.severity()),
    Some(Severity::Warning)
  );
}

// The replay iterator reports its true remaining count at every step, so a renderer can size
// its own buffer up front — and stays fused once exhausted.
#[test]
fn diagnostics_len_is_exact_after_partial_consumption() {
  let mut v = Verbose::<(), SimpleSpan>::new();
  let a = SimpleSpan::new(0usize, 1usize);
  let b = SimpleSpan::new(2usize, 5usize);

  emit(&mut v, a); // log[0]: error at a
  hole(&mut v, b, 3); // log[1]: hole at b
  emit(&mut v, b); // log[2]: error at b

  let mut diags = v.diagnostics();
  assert_eq!(diags.len(), 3, "all three records are pending");
  assert_eq!(diags.size_hint(), (3, Some(3)));

  assert!(diags.next().is_some());
  assert_eq!(diags.len(), 2, "one consumed, two exact");
  assert_eq!(diags.size_hint(), (2, Some(2)));

  assert_eq!(diags.by_ref().count(), 2, "the rest replay");
  assert_eq!(diags.len(), 0);
  assert_eq!(diags.size_hint(), (0, Some(0)));

  // Fused: past the end it keeps returning `None` with an exact zero remaining.
  assert!(diags.next().is_none());
  assert!(diags.next().is_none());
  assert_eq!(diags.size_hint(), (0, Some(0)));
}

// ── The ceiling, through the emitter ─────────────────────────────────────────

type Dummy = crate::lexer::DummyLexer;

fn warn(v: &mut Verbose<(), SimpleSpan>, span: SimpleSpan) {
  let _ = <Verbose<(), SimpleSpan> as Emitter<'_, Dummy>>::emit_warning(v, Spanned::new(span, ()));
}

fn checkpoint(v: &mut Verbose<(), SimpleSpan>) -> u64 {
  <Verbose<(), SimpleSpan> as Emitter<'_, Dummy>>::checkpoint(v)
}

/// Drives `Emitter::rewind` the way the input layer does at a restore.
fn rewind(v: &mut Verbose<(), SimpleSpan>, mark: u64) {
  let origin = 0usize;
  <Verbose<(), SimpleSpan> as Emitter<'_, Dummy>>::rewind(
    v,
    crate::input::Cursor::from_ref(&origin),
    mark,
  );
}

fn at(offset: usize) -> SimpleSpan {
  SimpleSpan::new(offset, offset + 1)
}

/// Past the ceiling every route still answers `Ok(())`, so the parse goes on: an error route,
/// the warning route and the hole route, each over a full `Verbose`. Only the error route's
/// refusal is counted, and none of the three is kept.
#[test]
fn every_route_answers_ok_past_the_ceiling() {
  type V = Verbose<(), SimpleSpan>;
  let mut v = V::with_max_diagnostics(1);
  emit(&mut v, at(0));

  assert_eq!(
    <V as Emitter<'_, Dummy>>::emit_error(&mut v, Spanned::new(at(1), ())),
    Ok(()),
    "the error route answers `Ok(())` past the ceiling"
  );
  assert_eq!(
    <V as Emitter<'_, Dummy>>::emit_lexer_error(&mut v, Spanned::new(at(2), ())),
    Ok(()),
    "so does the lexer-error route"
  );
  assert_eq!(
    <V as Emitter<'_, Dummy>>::emit_warning(&mut v, Spanned::new(at(3), ())),
    Ok(()),
    "the warning route answers `Ok(())` past the ceiling"
  );
  assert_eq!(
    <V as Emitter<'_, Dummy>>::emit_skipped_region(&mut v, at(4), 2),
    Ok(()),
    "the hole route answers `Ok(())` past the ceiling"
  );

  assert_eq!(v.diagnostics().len(), 1, "only the first emission is kept");
  assert_eq!(v.errors().len(), 1);
  assert!(v.warnings().is_empty() && v.skipped_regions().is_empty());
  assert_eq!(
    v.refused_errors(),
    2,
    "the two error routes' refusals are counted; the warning's and the hole's are not"
  );
}

/// `Emitter::checkpoint` reads the kept records plus the refused errors, and `Emitter::rewind`
/// to a mark undoes the refused errors above it before it unwinds a kept record.
#[test]
fn checkpoint_and_rewind_through_the_emitter_carry_the_refused_errors() {
  let mut v = Verbose::<(), SimpleSpan>::with_max_diagnostics(2);
  emit(&mut v, at(0));
  hole(&mut v, at(1), 1);
  let full = checkpoint(&mut v);
  assert_eq!(full, 2, "two kept records");

  emit(&mut v, at(2));
  warn(&mut v, at(3));
  emit(&mut v, at(4));
  assert_eq!(
    checkpoint(&mut v),
    4,
    "two kept records and two refused errors; the refused warning does not move the mark"
  );
  assert_eq!(v.refused_errors(), 2);

  rewind(&mut v, full);
  let mark = checkpoint(&mut v);
  assert_eq!(
    (v.diagnostics().len(), v.refused_errors(), mark),
    (2, 0, 2),
    "the rewind to the full mark uncounts both refused errors and unwinds no kept record"
  );

  rewind(&mut v, 1);
  let mark = checkpoint(&mut v);
  assert_eq!(
    (v.diagnostics().len(), v.refused_errors(), mark),
    (1, 0, 1),
    "below the refused errors, a kept record unwinds as it always has"
  );
  assert!(
    v.skipped_regions().is_empty(),
    "the kept hole was the newest record"
  );
}

/// A `Verbose` with no ceiling is 0.13.0's: through `new()`, `Default` and
/// `with_max_diagnostics(usize::MAX)` alike, every emission on every channel is kept and moves
/// the mark by exactly one, nothing is refused, and the ceiling reads `usize::MAX`.
#[test]
fn no_ceiling_is_0_13_0() {
  assert_eq!(
    format!("{:?}", Verbose::<(), SimpleSpan>::new()),
    format!(
      "{:?}",
      Verbose::<(), SimpleSpan>::with_max_diagnostics(usize::MAX)
    ),
    "`with_max_diagnostics(usize::MAX)` is the same value as `new()`"
  );

  for (name, mut v) in [
    ("new()", Verbose::<(), SimpleSpan>::new()),
    ("default()", Verbose::<(), SimpleSpan>::default()),
    (
      "with_max_diagnostics(usize::MAX)",
      Verbose::<(), SimpleSpan>::with_max_diagnostics(usize::MAX),
    ),
  ] {
    assert_eq!(v.max_diagnostics(), usize::MAX, "{name} sets no ceiling");
    for offset in 0..6usize {
      match offset % 3 {
        0 => emit(&mut v, at(offset)),
        1 => warn(&mut v, at(offset)),
        _ => hole(&mut v, at(offset), offset),
      }
      assert_eq!(
        checkpoint(&mut v),
        offset as u64 + 1,
        "{name}: every emission moves the mark by one"
      );
      assert_eq!(v.refused_errors(), 0, "{name} refuses nothing");
    }
    assert_eq!(v.diagnostics().len(), 6, "{name} keeps every diagnostic");
  }
}

/// `Clone` and `Debug` carry the ceiling and the count, and a clone's count is its own: each of
/// the two rewinds independently, and the clone goes on refusing at the same ceiling.
#[test]
fn a_clone_carries_the_ceiling_and_the_count() {
  let mut original = Verbose::<(), SimpleSpan>::with_max_diagnostics(1);
  emit(&mut original, at(0));
  let kept = checkpoint(&mut original);
  emit(&mut original, at(1));
  emit(&mut original, at(2));

  let mut twin = original.clone();
  assert_eq!(
    (twin.max_diagnostics(), twin.refused_errors()),
    (1, 2),
    "the clone carries the ceiling and the count"
  );
  let shown = format!("{twin:?}");
  assert!(
    shown.contains("max_diagnostics: 1, refused_errors: 2 }"),
    "`Debug` shows the ceiling and the count: {shown}"
  );

  rewind(&mut original, kept);
  assert_eq!(
    (original.refused_errors(), twin.refused_errors()),
    (0, 2),
    "rewinding the original leaves the clone's count alone"
  );
  rewind(&mut twin, kept + 1);
  assert_eq!(
    (original.refused_errors(), twin.refused_errors()),
    (0, 1),
    "and rewinding the clone leaves the original's"
  );

  emit(&mut twin, at(3));
  assert_eq!(
    (twin.diagnostics().len(), twin.refused_errors()),
    (1, 2),
    "the clone's ceiling still refuses"
  );
}
