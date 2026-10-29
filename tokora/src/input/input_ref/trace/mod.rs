//! `InputRef` tracing hooks — the `trace` feature.
//!
//! The enter / exit / leaf event emitters and the source preview they print. Kept beside
//! [`InputRef`](super::InputRef) so they can reach its `pub(super)` `depth` field. Every line
//! is handed to [`crate::trace::write_line`], which routes it out of band (stderr, or the
//! test capture buffer) — never through the emitter.

use super::{Completeness, InputRef};
use crate::{Lexer, ParseContext, source::Source};

/// How many characters of `{rest:?}` a trace preview shows before truncating. Was, and stays,
/// a plain `24` in the output — pulled out here only because [`bounded_debug`] needs the same
/// number the old inline arithmetic used, and one named constant can't drift out of sync with
/// itself the way two copies of `24` could.
const PREVIEW_WINDOW: usize = 24;

impl<'inp, L, Ctx, Lang: ?Sized, Cmpl> InputRef<'inp, '_, L, Ctx, Lang, Cmpl>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  /// A short, generic preview of the source at the cursor: the current offset plus a debug
  /// window of the remaining source. The window is followed by `…` when the window cut real
  /// content off, or by a distinct failure marker when `T`'s `Debug` impl broke on its own
  /// account instead — see [`BoundedDebugOutcome`] for the difference between the two, and the
  /// `FormatFailed` arm below for why that marker is a plain-text convention for a human
  /// reader, not reserved syntax, and can collide with a value's own rendering. Cheap
  /// when the [`Source`]'s `Slice` type has a `Debug` impl that writes incrementally — see
  /// [`bounded_debug`] for what that means, which in-tree backings qualify, and the residual
  /// shapes where cost can still track the remaining source (one of them unbounded, not merely
  /// narrower).
  fn trace_preview(&self) -> std::string::String {
    let off = self.offset();
    match self.source().slice(off.clone()..) {
      Some(rest) => {
        let (mut window, outcome) = bounded_debug(&rest);
        match outcome {
          BoundedDebugOutcome::Complete => {}
          BoundedDebugOutcome::WindowTruncated => window.push('\u{2026}'),
          // Deliberately not `…`: that glyph promises more of the value exists. This says
          // the renderer itself broke instead, which is a different fact calling for a
          // different reaction from whoever reads the trace.
          //
          // This text is a convention for a human reading the trace, not a reserved syntax.
          // It is appended into the same channel as `window` itself, so a value whose own
          // `Debug` happens to render this exact text (or any text containing it) inside the
          // window is indistinguishable, in the line this function returns, from a value that
          // rendered less and then failed here. Nothing in this crate parses a trace line back
          // apart, and nothing that consumes trace output should try to either.
          BoundedDebugOutcome::FormatFailed => window.push_str(" <fmt error>"),
        }
        std::format!("@{off:?} {window}")
      }
      None => std::format!("@{off:?} <eof>"),
    }
  }

  /// Emits a leaf event naming an instrumented combinator at the current depth. Reached only
  /// through the `trace_event!` macro, so with the feature off it has no caller.
  pub(crate) fn trace_leaf(&self, name: &str) {
    crate::trace::write_line(std::format!(
      "{}\u{b7} {name}  {}",
      "  ".repeat(*self.depth),
      self.trace_preview()
    ));
  }

  /// Drops the depth by one **without emitting**, for a [`traced`](crate::traced) scope that is
  /// leaving by an unwind rather than by one of its exit arms.
  ///
  /// The arms decrement and then emit; an unwound scope prints no exit line (trace events are
  /// out of band, and there is no outcome to report), so its guard needs the decrement alone.
  /// The field is `pub(super)` to `input_ref` and `crate::trace` is a sibling module, so the
  /// guard reaches it through here.
  pub(crate) fn trace_unwind_pop(&mut self) {
    *self.depth = (*self.depth).saturating_sub(1);
  }

  /// Emits an `enter` event, then bumps the depth so nested events indent beneath it.
  pub(crate) fn trace_enter(&mut self, name: &str) {
    crate::trace::write_line(std::format!(
      "{}> {name}  {}",
      "  ".repeat(*self.depth),
      self.trace_preview()
    ));
    *self.depth += 1;
  }

  /// Emits an `ok` exit carrying the span consumed since `start`, dropping the depth first.
  pub(crate) fn trace_exit_ok(&mut self, name: &str, start: &L::Offset) {
    *self.depth = (*self.depth).saturating_sub(1);
    let end = self.offset().clone();
    crate::trace::write_line(std::format!(
      "{}< {name}  ok  {start:?}..{end:?}",
      "  ".repeat(*self.depth)
    ));
  }

  /// Emits a non-`ok` exit (`err` or `decline`), dropping the depth first.
  pub(crate) fn trace_exit(&mut self, name: &str, outcome: &str) {
    *self.depth = (*self.depth).saturating_sub(1);
    crate::trace::write_line(std::format!(
      "{}< {name}  {outcome}",
      "  ".repeat(*self.depth)
    ));
  }
}

/// The outcome of trying to capture up to [`PREVIEW_WINDOW`] characters of `T`'s `Debug`
/// rendering through [`bounded_debug`]'s sink.
///
/// Two independent facts a caller needs kept apart, not folded into one bool: whether the
/// returned prefix might be missing real content the window cut off
/// ([`WindowTruncated`](Self::WindowTruncated)), and whether `T`'s own `Debug` impl finished
/// without failing on its own account (it did not, exactly when
/// [`FormatFailed`](Self::FormatFailed)). Conflating these is the bug this type exists to rule
/// out — see [`bounded_debug`]'s doc comment for how the two are told apart and why that is
/// reliable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundedDebugOutcome {
  /// The write ran to completion and the sink never refused it: `prefix` is the complete
  /// rendering.
  Complete,
  /// The sink refused a write once its window was already full. `prefix` is a genuine prefix
  /// of the untruncated dump: real content, but not all of it.
  ///
  /// Also reported — indistinguishably — when a `Debug` impl swallows that refusal and then
  /// fails again afterward for a reason of its own. See "A residual this mechanism cannot
  /// resolve" on [`bounded_debug`]'s doc comment for why that case cannot be told apart from
  /// an ordinary truncation.
  WindowTruncated,
  /// `write!` returned `Err` and the sink itself never refused anything, so `T`'s `Debug` impl
  /// failed on its own account. `prefix` is whatever it wrote before failing — which can be
  /// its complete intended output — with no evidence either way that more was coming.
  FormatFailed,
}

/// Debug-formats `value`, returning at most [`PREVIEW_WINDOW`] characters of
/// `format!("{value:?}")` alongside a [`BoundedDebugOutcome`] describing how the write ended.
/// The prefix is exactly the `window` half of the `(window, needs_ellipsis)` pair
/// `trace_preview` used to compute by building the complete dump and then discarding everything
/// past the 24th character; the outcome replaces the old `needs_ellipsis` bool with the
/// three-way distinction `trace_preview` needs to render a preview honestly.
///
/// # Telling the sink's own refusal apart from a foreign `Debug` failure
///
/// `write!(sink, "{value:?}")`'s `Result` comes back `Err` in exactly two situations: the
/// sink's own refusal once it already holds a full window, or `T`'s `Debug` impl returning
/// `Err` on its own account, for a reason that has nothing to do with the sink. Both are legal:
/// `fmt::Result` is a plain `Result`, and neither the `Debug` trait nor [`crate::Slice`]'s own
/// bound (`PartialEq + Eq + Debug`) requires every `Err` to originate from a formatter write. A
/// reader of the preview needs the two told apart: one means "there is more of this value than
/// shown", the other means "this value's renderer broke, independent of how much fit" — and
/// they call for different next steps.
///
/// The two are told apart by a fact `PreviewSink` itself records, not by inspecting `T` or
/// guessing at intent. `PreviewSink` implements only `write_str`; `Write::write_char` and
/// `write_fmt` are inherited default methods that both bottom out in it, so `write_str` is the
/// *only* place this sink can ever fail, at any nesting depth (a nested value's own
/// `Debug`/`Display` call still writes through the same `Formatter`, hence the same sink). That
/// method has exactly one `Err`-returning branch, and it sets `self.truncated = true` in the
/// same statement, before returning — there is no other `Err` anywhere in the sink. So
/// `sink.truncated == true` is necessary *and* sufficient evidence that the sink produced an
/// `Err` somewhere during the call, and `sink.truncated == false` proves it never did — meaning
/// an `Err` from `write!` while `sink.truncated` is still `false` cannot be the sink's; it was
/// synthesized by `T::fmt` itself (or something it called), independent of the window. That is
/// what makes the distinction reliable: it reads a fact recorded at the one point the sink is
/// able to fail, rather than inferring anything about `T`'s `Debug` impl.
///
/// ```text
/// sink.truncated                   -> WindowTruncated  (regardless of write!'s own result)
/// !sink.truncated &&  wrote_fully  -> Complete
/// !sink.truncated && !wrote_fully  -> FormatFailed
/// ```
///
/// `WindowTruncated` is checked first because the two facts are not mutually exclusive: the
/// ordinary case is the sink's `Err` propagating straight out of `write!` (`wrote_fully =
/// false`), but a `Debug` impl may also swallow that `Err` and report overall `Ok` anyway
/// (legal — nothing requires propagating a formatter `Result` with `?`). Either way the sink
/// already knows it cut something off, so this stays truncated no matter what `write!` itself
/// says. `FormatFailed` is what remains once `WindowTruncated` is ruled out: `write!` failed,
/// and by the proof above that failure cannot be the sink's, so `prefix` is whatever `T::fmt`
/// wrote before failing on its own account — which can be its complete, intended output, short
/// and otherwise unremarkable (see `FailsForAForeignReason` in this module's tests) — so this
/// must not be reported the same way as a real truncation, which would claim content beyond
/// `prefix` that the impl never intended to write.
///
/// This is a correctness property, independent of the cost discussion below: it holds
/// unconditionally, whether `T`'s `Debug` impl streams or front-loads.
///
/// # A residual this mechanism cannot resolve
///
/// The proof above tells `sink.truncated` apart from a foreign `Debug` failure whenever
/// exactly one of them happened. It says nothing about the case where both do, in the same
/// call: a `Debug` impl writes past the window — the sink refuses and sets `sink.truncated =
/// true` — swallows that `Err` instead of propagating it with `?`, and then fails again
/// afterward for a reason of its own, unrelated to the sink. Two facts are true there: the
/// window really did cut off genuine content, *and* the impl broke afterward. Only the first
/// is reported. With `sink.truncated` already `true`, `WindowTruncated` is checked first (as
/// above) and this case is `WindowTruncated`, indistinguishable from an impl that did nothing
/// but let the sink's own `Err` propagate.
///
/// This is not an artifact of checking `sink.truncated` before `wrote_fully`, and no
/// reordering of the two, nor any other reading of them, recovers it. Once the sink has
/// refused, these two histories:
///
/// - the impl let the sink's `Err` propagate straight out — ordinary truncation, nothing else
///   happened
/// - the impl swallowed the sink's `Err` and then failed again afterward, on its own account
///
/// leave identical evidence behind: `sink.truncated == true` and `wrote_fully == false`, in
/// both. Nothing records "and something else went wrong after the sink refused," so a fourth
/// [`BoundedDebugOutcome`] variant could not be populated honestly here — there is no fact left
/// to populate it from, only two histories that read the same. A richer sink protocol —
/// recording not just *that* it refused but everything offered to it afterward — could
/// recover this, but that is materially more machinery than a debug preview earns.
///
/// The loss is narrow in practice. `WindowTruncated`'s `prefix` is a genuine, incomplete slice
/// of the value in both histories, so the `…` a reader sees is warranted either way; what does
/// not survive is the extra fact that the impl was *also* broken. See
/// `bounded_debug_absorbs_a_foreign_failure_after_truncation_into_window_truncated` in this
/// module's tests for this exact case, pinned as `WindowTruncated`.
///
/// # Why the old shape (`format!` then `.chars().take(24)`) is the bug, and why this isn't
///
/// `format!("{rest:?}")` fully escapes and allocates the debug rendering of *all* of `rest` —
/// the entire remaining source — before a single byte of that is kept; `dump.chars().count()`
/// then walks it again. Both are `O(remaining source)`, unconditionally, on every traced event.
///
/// The fix this file would like to make is "ask for a shorter `rest` before formatting it at
/// all," but that lever isn't reachable from here. [`Source::slice`] only accepts a range in
/// `L::Offset`, and `Lexer::Offset` is deliberately left with no arithmetic and no conversion
/// to `usize` (`Default + Debug + Ord + Clone + Hash` — see the trait) so a third-party
/// `Source` is never forced to expose byte-offset structure it may not have; there is no
/// `off + 24` to compute anywhere generic code can reach. [`crate::Slice`] — what
/// `Source::slice` hands back — offers iteration (`iter`, `positioned_iter`, `len`), not
/// sub-slicing, so there is no smaller `Self::Slice` to ask for either. Adding either
/// capability would mean widening `Lexer`/`Source`/`Slice` themselves, which is a far bigger
/// and riskier change than this fix, for every caller of those public traits — not something
/// to do as a side effect of a trace-preview cost fix.
///
/// What *is* reachable generically: a `Debug` impl only ever produces output by calling
/// `Formatter::write_str`/`write_char`, and the `?` those calls are always written behind
/// means a sink that starts refusing them aborts the whole call chain immediately, no matter
/// how deep into its own loop the impl is. Everything written before that point is, byte for
/// byte, a genuine prefix of the untruncated dump — that is what "the first writes of a
/// formatting call" means, regardless of how an impl batches them internally — so the window
/// and the ellipsis decision below come from real output, never a reconstruction of it.
///
/// # What this does and does not bound
///
/// This removes the one guarantee the old code had: that a preview costs the *entire*
/// remaining source, every time, unconditionally. What replaces it is conditional, not
/// universal: **the bound holds for a `Debug` impl that writes incrementally** — one whose
/// calls to `write_str`/`write_char` interleave with whatever scanning, escaping or
/// allocation produces each piece of output, so a sink that starts refusing mid-stream stops
/// that work too, not merely the writes themselves. **It does not hold for an impl that
/// front-loads** — builds its entire rendering into an owned buffer first and hands it to the
/// formatter in a single `write_str` call — because that cost has already run by the time the
/// sink is ever invoked.
///
/// Every `Source`/`Slice` pairing this crate itself ships was audited against that line, and
/// the guarantee above is asserted for exactly these, not for [`Source`] in general. What the
/// audit found, worst first:
///
/// - **Unbounded, and the only one of the three reachable without touching this crate at
///   all.** [`Source::Slice`] is a public associated type any downstream crate can implement
///   (`Self::Slice<'source>: Slice<'source>`), and [`crate::Slice`]'s own bound (`PartialEq +
///   Eq + Debug`) carries no streaming requirement, so nothing stops a conforming third-party
///   impl from scanning, escaping and allocating its *entire* rendering into a temporary and
///   calling `write_str` once — no unsafe, no misuse of this crate, just an ordinary trait
///   impl. This sink cannot shorten work that already ran before it was ever called, so the
///   cost stays the same `O(remaining source)` this fix exists to remove. Output is still
///   correct (a front-loaded impl's single write still *starts* with a genuine prefix of the
///   untruncated dump, so the window and the ellipsis decision are unaffected) — only the cost
///   guarantee does not reach this far. No in-tree backing does this today (see the rest of
///   this list). See `bounded_debug_is_correct_but_unbounded_for_a_front_loading_debug_impl`
///   in this module's tests for a fixture that exercises exactly this shape.
/// - `[u8]`-shaped backings (`[u8]` itself, and `HipByt`/the `smol-bytes` byte types, which
///   delegate to it) go through `Formatter::debug_list`, which drives its element iterator
///   with the write failure recorded in an internal `Result` rather than by breaking its own
///   loop — so it keeps *pulling* elements after this sink starts refusing them, even though
///   it stops writing. That bounds the allocation and the escaping work to the budget (still a
///   large, measured win — see the fix's cost evidence), but the iteration stays
///   `O(remaining elements)`.
/// - `str`-shaped backings (`str` itself, and `HipStr`/`Utf8Bytes`, which delegate to it)
///   accumulate a run of characters that need no escaping before their first write, so a sink
///   can only cut in *between* such runs, not partway through accumulating one. Ordinary text
///   ends a run at the next newline, quote, backslash, control character or non-printable code
///   point — a handful of bytes — which is what turns the reported cost from scaling with the
///   remaining source into not scaling with it at all. An input engineered to have a long run
///   of nothing-needs-escaping right at the cursor — e.g. this crate's own
///   `int_run_source`/`comma_list_source` bench fixtures, which are digits and spaces for
///   ~128 KiB at a stretch — still costs `O(that run)` here. Found while fixing the reported
///   cost; not fixed by it.
/// - `bytes::Bytes` and `bstr::BStr` hand-write their `Debug` as a loop with `?` on every
///   write, so a refusing sink stops them outright: genuinely `O(1)` here, independent of
///   content.
fn bounded_debug<T: core::fmt::Debug + ?Sized>(
  value: &T,
) -> (std::string::String, BoundedDebugOutcome) {
  use core::fmt::Write as _;

  /// The sink itself: accepts characters until [`PREVIEW_WINDOW`] of them have been kept, then
  /// starts returning `Err` and keeps a record that it did.
  struct PreviewSink {
    prefix: std::string::String,
    len: usize,
    truncated: bool,
  }

  impl core::fmt::Write for PreviewSink {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
      for c in s.chars() {
        if self.len == PREVIEW_WINDOW {
          self.truncated = true;
          return Err(core::fmt::Error);
        }
        self.prefix.push(c);
        self.len += 1;
      }
      Ok(())
    }
  }

  // `* 4`: up to `PREVIEW_WINDOW` *characters* can be kept, and a `char` is up to 4 UTF-8
  // bytes — sized to avoid a reallocation on non-ASCII content, not because more than the
  // budget is ever kept.
  let mut sink = PreviewSink {
    prefix: std::string::String::with_capacity(PREVIEW_WINDOW * 4),
    len: 0,
    truncated: false,
  };
  // `write!`'s `Result` can be `Err` for a reason `sink.truncated` never sees: `T`'s `Debug`
  // impl failing on its own account, not because this sink refused anything. See "Telling the
  // sink's own refusal apart from a foreign `Debug` failure" above for the proof that
  // `sink.truncated` alone tells the two apart reliably.
  let wrote_fully = write!(sink, "{value:?}").is_ok();
  // `WindowTruncated` is checked first: it can coexist with `wrote_fully` (a `Debug` impl may
  // swallow the sink's `Err` and report `Ok` anyway), and whenever the sink has something to
  // report, its bookkeeping wins over what `write!` itself says. Only what is left —
  // `write!` failed, and the sink did not cause it — is `FormatFailed`.
  let outcome = if sink.truncated {
    BoundedDebugOutcome::WindowTruncated
  } else if wrote_fully {
    BoundedDebugOutcome::Complete
  } else {
    BoundedDebugOutcome::FormatFailed
  };
  (sink.prefix, outcome)
}

#[cfg(test)]
mod tests;
