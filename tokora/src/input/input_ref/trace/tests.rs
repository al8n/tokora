use super::{BoundedDebugOutcome, PREVIEW_WINDOW, bounded_debug};
use core::cell::Cell;

/// A `Debug` impl that deliberately breaks the streaming assumption [`bounded_debug`]'s
/// bound depends on: it scans its *entire* input into an owned buffer — counting every
/// character it touches along the way — and only then hands the finished buffer to the
/// formatter in a single `write_str` call. `crate::Slice`'s bound (`PartialEq + Eq + Debug`)
/// permits exactly this; nothing about implementing `Debug` requires interleaving writes
/// with the work that produces them. This is the shape the unbounded residual documented on
/// [`bounded_debug`] describes, made concrete.
struct FrontLoading<'a> {
  data: &'a str,
  touched: &'a Cell<usize>,
}

impl core::fmt::Debug for FrontLoading<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut built = std::string::String::with_capacity(self.data.len());
    for ch in self.data.chars() {
      // Recorded regardless of how much of `built` the caller ends up keeping — this is
      // exactly the work a streaming impl would stop doing once a sink starts refusing
      // writes, and exactly the work this one does not stop.
      self.touched.set(self.touched.get() + 1);
      built.push(ch);
    }
    // One write, issued only after the entire input has already been touched. The sink
    // below gets a chance to cut short the string it receives here, never the work that
    // built it.
    f.write_str(&built)
  }
}

/// The residual `bounded_debug`'s doc comment names as unbounded: a front-loading `Debug`
/// impl still yields the right preview, but the early-refusing sink never gets a chance to
/// shorten the work that produced it, because that work is already finished by the time the
/// sink sees anything at all.
///
/// Falsifying output: `touched` bounded near [`PREVIEW_WINDOW`] rather than equal to the
/// full input length — that would mean either this fixture stopped being front-loading (a
/// test bug) or `bounded_debug` gained a way to bound this shape after all, which would be
/// real news worth updating the doc comment and the changelog for, not just this assertion.
#[test]
fn bounded_debug_is_correct_but_unbounded_for_a_front_loading_debug_impl() {
  // Far longer than the window, so truncation is actually exercised rather than the
  // shorter-than-budget case `bounded_debug` also has to get right.
  let long: std::string::String = "a".repeat(PREVIEW_WINDOW * 10);
  let touched = Cell::new(0);
  let value = FrontLoading {
    data: &long,
    touched: &touched,
  };

  let (window, outcome) = bounded_debug(&value);

  // Correctness is unaffected by front-loading: everything a `Debug` impl writes, however
  // it was produced, is a genuine prefix of the untruncated dump the moment it reaches
  // `write_str` — so the sink still recovers exactly the first `PREVIEW_WINDOW` characters
  // and the right truncation decision.
  assert_eq!(window, "a".repeat(PREVIEW_WINDOW));
  assert_eq!(
    outcome,
    BoundedDebugOutcome::WindowTruncated,
    "content far longer than the window must report truncated"
  );

  // The residual: a streaming impl would have left `touched` at PREVIEW_WINDOW + 1 (the one
  // character whose push trips the sink's refusal). This impl built its entire output
  // before ever calling the formatter, so it touched every character of `long` regardless —
  // cost the sink never had an opportunity to bound.
  assert_eq!(
    touched.get(),
    long.chars().count(),
    "a front-loading impl costs the whole input no matter what the sink does"
  );
}

/// A `Debug` impl that writes a few characters — fewer than [`PREVIEW_WINDOW`], so nothing
/// about the sink can be why it fails — and then fails on its own account. Legal under the
/// trait: `fmt::Result` is a plain `Result`, and neither `Debug` nor `crate::Slice`'s bound
/// (`PartialEq + Eq + Debug`) requires every failure to originate from a formatter write.
struct FailsForAForeignReason;

impl core::fmt::Debug for FailsForAForeignReason {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("abc")?;
    Err(core::fmt::Error)
  }
}

/// The format-failure case: only three characters are ever offered to the sink here, so `self.len`
/// never reaches [`PREVIEW_WINDOW`] and the sink never refuses a write — `sink.truncated` stays
/// `false`. `write!` still fails, but for a reason of the `Debug` impl's own, and `"abc"` is that
/// impl's *complete* intended output, not a fragment of a longer one that got cut off. Folding the
/// write's own result into a single `truncated` bool with `||`, on the reasoning "incomplete is
/// incomplete, whatever the cause", is reasonable-sounding but wrong here: nothing is incomplete.
/// The write finished exactly as the impl meant it to. Reporting it truncated would append `…` to a
/// trace line for a value with no more content, claiming a continuation that does not exist — the
/// mirror of missing a real truncation. [`BoundedDebugOutcome::FormatFailed`] is the outcome that
/// makes neither claim: it says the renderer broke, and says nothing about how much of the value
/// that left out.
#[test]
fn bounded_debug_is_format_failed_when_debug_fails_for_a_reason_the_sink_did_not_cause() {
  let (window, outcome) = bounded_debug(&FailsForAForeignReason);

  // What was written before the failure is real output, kept exactly as written — the cell pins
  // how the outcome is classified, not what makes it into the prefix.
  assert_eq!(window, "abc");
  assert_eq!(
    outcome,
    BoundedDebugOutcome::FormatFailed,
    "a Debug impl that already wrote its complete, short rendering and then failed for its \
     own reason must be reported as a format failure, not window-truncated — truncated would \
     tell a reader there is more content past \"abc\" than this impl ever intended to write"
  );
}

/// A `Debug` impl that writes past [`PREVIEW_WINDOW`] — so the sink genuinely refuses and
/// records `sink.truncated = true` — then discards that `Err` instead of propagating it with
/// `?`, and fails again anyway, for a reason that has nothing to do with the sink. Legal for
/// the same reason [`FailsForAForeignReason`] is: `fmt::Result` is a plain `Result`, nothing
/// obliges a `Debug` impl to propagate a formatter failure with `?`, and nothing obliges it
/// to stop trying once one write fails.
struct SwallowsTruncationThenFailsForItsOwnReason;

impl core::fmt::Debug for SwallowsTruncationThenFailsForItsOwnReason {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    // Longer than the window, so the sink genuinely refuses partway through this call —
    // and that `Err` is then thrown away rather than returned, standing in for a `Debug`
    // impl that does not treat a formatter failure as fatal to its own control flow.
    let _ = f.write_str(&"a".repeat(PREVIEW_WINDOW + 5));
    // Fails anyway, for a reason of its own, independent of the sink's refusal above.
    Err(core::fmt::Error)
  }
}

/// The residual documented on `bounded_debug` under "A residual this mechanism cannot
/// resolve": once the sink has refused, a `Debug` impl that swallows that refusal and then
/// fails again on its own account is absorbed into `WindowTruncated` — the same outcome as
/// an impl that only ever let the sink's `Err` propagate. This is not a bug to fix here; it
/// is pinning the documented limit of what `sink.truncated` and `wrote_fully` can tell
/// apart. A fourth, more specific outcome would need evidence neither flag carries.
#[test]
fn bounded_debug_absorbs_a_foreign_failure_after_truncation_into_window_truncated() {
  let (window, outcome) = bounded_debug(&SwallowsTruncationThenFailsForItsOwnReason);

  // The prefix is exactly what the sink kept before refusing — real content, the same as an
  // ordinary truncation with no failure of the impl's own behind it.
  assert_eq!(window, "a".repeat(PREVIEW_WINDOW));
  assert_eq!(
    outcome,
    BoundedDebugOutcome::WindowTruncated,
    "a Debug impl that swallows the sink's truncation Err and then fails again for its own, \
     unrelated reason is absorbed into WindowTruncated — indistinguishable from an impl that \
     only ever let the sink's Err propagate — because sink.truncated and wrote_fully carry \
     no evidence that anything happened after the sink refused; this is the documented \
     residual, not a defect"
  );
}

/// An independent reimplementation of the format-then-take preview: format the value, take
/// [`PREVIEW_WINDOW`] characters, and append the ellipsis if the full dump was longer than that —
/// the behaviour `bounded_debug` plus the `match` on [`BoundedDebugOutcome`] in `trace_preview`
/// must reproduce.
fn reference_preview_window<T: core::fmt::Debug + ?Sized>(value: &T) -> std::string::String {
  let dump = std::format!("{value:?}");
  let mut window: std::string::String = dump.chars().take(PREVIEW_WINDOW).collect();
  if dump.chars().count() > PREVIEW_WINDOW {
    window.push('\u{2026}');
  }
  window
}

/// Runs `value` through `bounded_debug` plus the same ellipsis step `trace_preview` applies
/// to its result, and through [`reference_preview_window`]'s independent reimplementation of
/// the old logic, then fails with `case` in the message if they disagree.
#[track_caller]
fn assert_matches_reference<T: core::fmt::Debug + ?Sized>(case: &str, value: &T) {
  let (mut window, outcome) = bounded_debug(value);
  if outcome == BoundedDebugOutcome::WindowTruncated {
    window.push('\u{2026}');
  }
  assert_eq!(
    window,
    reference_preview_window(value),
    "case {case:?}: bounded_debug (plus trace_preview's ellipsis step) disagreed with the \
     old format!-then-take(24)-then-ellipsis logic"
  );
}

/// **Characterization test, not a specification.** It pins `bounded_debug`'s output — plus
/// the ellipsis step `trace_preview` applies on top of it — as byte-identical to
/// `format!("{value:?}")` + `.chars().take(24)` + conditional-ellipsis logic (reimplemented
/// independently in [`reference_preview_window`]), across a spread of lengths and escape-heavy
/// content, so nothing in the suite misses a regression of that equivalence. It documents today's
/// behaviour, not a contract: a future, *deliberate* change to what `bounded_debug` or
/// `trace_preview` returns is expected to update this test alongside it, not be blocked by it.
///
/// The reference is a stand-in for the old logic, and the old logic is a faithful reference
/// only for a **streaming** `Debug` impl — one whose writes interleave with the work that
/// produces them. `str` and `[u8]`, used throughout below, both qualify (see `bounded_debug`'s
/// doc comment for exactly how, and for the residual cost gaps that make them worth calling
/// out explicitly even though their *output* is unaffected). A front-loading impl legitimately
/// costs more without differing in output — that is
/// `bounded_debug_is_correct_but_unbounded_for_a_front_loading_debug_impl` above, not this
/// test.
#[test]
fn bounded_debug_matches_the_old_format_and_take_24_logic() {
  assert_matches_reference("empty", "");
  assert_matches_reference("shorter than the window", "hi");
  assert_matches_reference("exactly at the window", &"a".repeat(22));
  assert_matches_reference("one character past the window", &"a".repeat(23));
  assert_matches_reference("far longer than the window", &"a".repeat(500));
  assert_matches_reference("newlines", "first line\nsecond line\nthird line");
  assert_matches_reference("tabs", "col1\tcol2\tcol3");
  assert_matches_reference("double quotes", "she said \"hello\" and left");
  assert_matches_reference("single quotes", "it's fine, isn't it");
  assert_matches_reference("backslashes", r"a\b\c\path\to\thing");
  assert_matches_reference("non-ASCII", "héllo wörld — 你好，世界 🎉");
  assert_matches_reference(
    "a multi-byte scalar straddling the cut",
    &std::format!("{}€{}", "a".repeat(22), "b".repeat(10)),
  );
  assert_matches_reference(
    "control characters",
    "\u{0}\u{1}\u{7}\u{8}\u{b}\u{c}\u{1b}\u{7f}",
  );
  let bytes: std::vec::Vec<u8> = (0..40).collect();
  assert_matches_reference("the &[u8] Debug convention", bytes.as_slice());

  // The two boundary cases, pinned directly against `bounded_debug`'s own return value, not
  // only cross-checked against `reference_preview_window` above: an off-by-one shared by both
  // implementations would make them agree with each other while both are wrong.
  let (window, outcome) = bounded_debug(&"a".repeat(22));
  assert_eq!(window, std::format!("\"{}\"", "a".repeat(22)));
  assert_eq!(
    outcome,
    BoundedDebugOutcome::Complete,
    "a dump exactly PREVIEW_WINDOW characters long is not truncated"
  );

  let (window, outcome) = bounded_debug(&"a".repeat(23));
  assert_eq!(window, std::format!("\"{}", "a".repeat(23)));
  assert_eq!(
    outcome,
    BoundedDebugOutcome::WindowTruncated,
    "a dump one character past PREVIEW_WINDOW is truncated"
  );
}

/// A `Debug` impl whose complete, correct rendering happens to be the exact text
/// `trace_preview` appends for [`BoundedDebugOutcome::FormatFailed`]. Nothing is wrong with
/// this impl or its output; it simply renders, as ordinary content, the same characters the
/// marker uses.
struct RendersTheFmtErrorMarkerVerbatim;

impl core::fmt::Debug for RendersTheFmtErrorMarkerVerbatim {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("abc <fmt error>")
  }
}

/// Pins the collision the `FormatFailed` arm in `trace_preview` documents: the marker is
/// appended into the same text channel as the preview, so a value's own, complete rendering
/// can be byte-identical to the rendered line for a value that failed. `trace_preview` itself
/// is a method on a live `InputRef`, which this module's tests do not otherwise construct, so
/// its one-line `FormatFailed` step (`push_str(" <fmt error>")`) is reproduced directly here
/// against [`FailsForAForeignReason`]'s own output rather than called.
#[test]
fn fmt_error_marker_collides_with_a_debug_impl_that_renders_the_same_text() {
  let (window, outcome) = bounded_debug(&RendersTheFmtErrorMarkerVerbatim);
  assert_eq!(window, "abc <fmt error>");
  assert_eq!(
    outcome,
    BoundedDebugOutcome::Complete,
    "this value's Debug impl writes its complete output in one call and returns Ok — nothing \
     about it fails"
  );

  // trace_preview's own FormatFailed rendering step, reproduced rather than called: append
  // the marker text to whatever prefix a foreign failure left behind.
  let (failure_prefix, failure_outcome) = bounded_debug(&FailsForAForeignReason);
  assert_eq!(failure_outcome, BoundedDebugOutcome::FormatFailed);
  let mut rendered_after_failure = failure_prefix;
  rendered_after_failure.push_str(" <fmt error>");

  assert_eq!(
    window, rendered_after_failure,
    "a value that legitimately renders \"abc <fmt error>\" as its complete output is \
     byte-identical, in the rendered trace line, to a value that rendered \"abc\" and then \
     failed to format at all — the marker is a convention for a human reader, not reserved \
     syntax, and it collides with arbitrary Debug output"
  );
}
