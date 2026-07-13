#![allow(clippy::type_complexity)]

use core::{
  marker::PhantomData,
  ops::{Range, RangeBounds},
};

use generic_arraydeque::{GenericArrayDeque, typenum::U1};
use mayber::{Maybe, MaybeRef};

use crate::{
  ParseContext, Token, Window,
  cache::{CachedToken, CachedTokenOf, CachedTokenRefOf, MaybeRefCachedTokenOf, Peeked},
  emitter::Emitter,
  error::token::UnexpectedToken,
  span::Spanned,
  utils::Expected,
};

use super::{
  Cache, Checkpoint, Complete, Completeness, Cursor, Lexed, Lexer, Lineage, Source, Span,
  SurfaceIncomplete,
};

pub(crate) use session::Session;

mod consume_cached;
mod drop_policy;
mod fold;
mod peek;
mod pratt;
pub(crate) mod session;
mod skip_while;
#[cfg(any(feature = "std", feature = "alloc"))]
mod stacked;
mod sync;
mod sync_balanced;
mod sync_through;
mod sync_to;
#[cfg(feature = "trace")]
mod trace;
mod transaction;
mod try_expect;

pub use drop_policy::{Commit, DropPolicy, Rollback};
pub use sync_balanced::{Balance, DelimClass, Hole};
pub use transaction::Transaction;

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use stacked::{SavepointId, StackedTransaction};

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests;

#[cfg(all(test, feature = "logos", feature = "std"))]
mod partial_tests;

#[cfg(all(test, feature = "logos", feature = "std"))]
mod session_tests;

/// A reference to an `Input` instance.
pub struct InputRef<'inp, 'closure, L, Ctx, Lang: ?Sized = (), Cmpl = Complete>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  pub(super) input: &'closure &'inp L::Source,
  pub(super) state: &'closure mut L::State,
  pub(super) span: &'closure mut L::Span,
  pub(super) cache: &'closure mut Ctx::Cache,
  /// The completeness finality (`is_final`) flag, copied by value from the owning [`Input`] (a ZST
  /// for [`Complete`], a `bool` for [`Partial`]). The frontier rules read it only under
  /// [`Completeness::PARTIAL`]; [`set_final`](Self::set_final) updates it as chunks arrive.
  pub(super) finality: Cmpl::Finality,
  pub(super) emitted_error_end: &'closure mut L::Offset,
  pub(super) poison_boundary: &'closure mut Option<L::Offset>,
  /// The **session cell**: the input's lineage memos (the live-checkpoint stack, the pin set, and
  /// the cache-push/checkpoint-id/savepoint counters) together with the live
  /// [session points](Self::begin_point) opened on this handle.
  ///
  /// They are one cell because an abandoned session point has to release bookkeeping it does not
  /// own — the pin lives on the [`Input`](super::Input), which outlives this handle, while the
  /// point's [`Checkpoint`] lives here and dies with it. [`Session`]'s `Drop` reconciles them (see
  /// its [module docs](session) for that, and for why the destructor lives on this cell rather than
  /// on the handle: a `Drop` on `InputRef` would escape *every* field to the destructor and cost the
  /// scanner its registers).
  pub(super) session: Session<'inp, 'closure, L>,
  /// Trace nesting depth, borrowed from the owning [`Input`] (the `trace` feature). Its sole
  /// mutators are [`traced`](crate::traced)'s enter/exit hooks; internal leaf events only read
  /// it for indentation.
  #[cfg(feature = "trace")]
  pub(super) depth: &'closure mut usize,
  /// Debug-only witness of the input identity, for `restore`'s foreign-input check.
  #[cfg(all(
    debug_assertions,
    any(feature = "std", feature = "alloc"),
    target_has_atomic = "ptr"
  ))]
  pub(super) witness: &'closure super::Witness,
  pub(super) emitter: &'closure mut Ctx::Emitter,
  pub(super) _marker: PhantomData<Lang>,
}

impl<'inp, L, Ctx, Lang: ?Sized, Cmpl> InputRef<'inp, '_, L, Ctx, Lang, Cmpl>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  /// Returns a reference to the tokenizer's cache.
  ///
  /// The cache stores peeked tokens that have been lexed but not yet consumed.
  /// This can be useful for inspecting the cache state or implementing custom
  /// lookahead logic.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn cache(&self) -> &Ctx::Cache {
    self.cache
  }

  /// Returns a mutable reference to the tokenizer's cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn cache_mut(&mut self) -> &mut Ctx::Cache {
    self.cache
  }

  /// Pushes a lexed token onto the back of the cache, recording the accepted push on the lineage
  /// memos ([`Lineage::record_cache_push`](super::Lineage::record_cache_push)) so
  /// [`save`](Self::save) can snapshot the count and [`restore`](Self::restore) drop exactly the
  /// entries pushed since. A full cache hands the token back and records nothing.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn cache_push_back(&mut self, tok: CachedTokenOf<'inp, L>) -> Result<(), CachedTokenOf<'inp, L>> {
    match self.cache.push_back(tok) {
      Ok(_) => {
        self.session.lineage.record_cache_push();
        Ok(())
      }
      Err(tok) => Err(tok),
    }
  }

  /// Returns a reference to the underlying input source.
  ///
  /// This allows access to the raw source being tokenized, which is typically
  /// a `&str` or `&[u8]` depending on your Logos token definition.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn source(&self) -> &'inp L::Source {
    self.input
  }

  /// Returns a reference to the current lexer state (extras).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn state(&self) -> &L::State {
    self.state
  }

  /// Returns whether this input is **final** — the last chunk of a stream, or a
  /// [`Complete`](crate::input::Complete) input (always final).
  ///
  /// A [`Partial`](crate::input::Partial) input reports the runtime flag last set by
  /// [`set_final`](Self::set_final) (default `false`); a [`Complete`](crate::input::Complete) input
  /// is final by definition, so this returns `true` and the partial-input frontier rules are inert.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_final(&self) -> bool {
    Cmpl::is_final(&self.finality)
  }

  /// Marks whether this is the **final** chunk of a partial stream.
  ///
  /// Set `false` while more input may still arrive, `true` for the last chunk. While non-final, the
  /// frontier rules hold back any construct that later input could extend, surfacing an
  /// [`Incomplete`](crate::error::Incomplete); once final, a [`Partial`](crate::input::Partial)
  /// input behaves exactly like a [`Complete`](crate::input::Complete) one. On a
  /// [`Complete`](crate::input::Complete) input this is a no-op (its finality is a zero-sized type).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_final(&mut self, is_final: bool) {
    Cmpl::set_final(&mut self.finality, is_final);
  }

  /// Returns a mutable reference to the current lexer state (extras).
  ///
  /// # State replacement re-keys the input's forward-scanning facts
  ///
  /// Mutating the state through the returned reference can change how the region ahead of
  /// the cursor lexes, so this call **eagerly** re-keys every offset-dependent fact that
  /// governs forward scanning: the token cache is cleared (its entries were lexed under the
  /// old state and those offsets may lex differently now), the poison boundary is dropped,
  /// and the lexer-error dedup watermark is reset to the current committed cursor. The
  /// re-key runs before this returns, so it applies whether or not the caller ends up
  /// mutating through the `&mut`.
  ///
  /// Speculative peek-ahead diagnostics emitted under the old state for the region beyond
  /// the cursor stay in the emitter log, and the watermark reset makes that same region
  /// re-reportable once it re-lexes under the new state: state surgery with outstanding
  /// speculative diagnostics may re-report the re-lexed region under the new regime, so
  /// callers should complete or roll back speculation before replacing state.
  ///
  /// # Transactional: checkpoints survive state surgery
  ///
  /// The re-key is itself **transactional**, not invalidating. A [`Checkpoint`] pure-copies
  /// every fact the re-key touches — regime, poison boundary, dedup watermark, cursor/span,
  /// and the cache-push counter — so restoring one saved *before* the surgery simply undoes
  /// it: the pre-surgery regime, boundary, watermark, and position all return, and the cache
  /// re-lexes under the restored regime. Outstanding checkpoints therefore **remain valid**
  /// across state surgery — a raw [`restore`](Self::restore), an [`attempt`](Self::attempt)
  /// rollback, and a [`StackedTransaction`] savepoint taken before the surgery all roll back
  /// across it cleanly.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn state_mut(&mut self) -> &mut L::State {
    self.rekey_offset_facts();
    self.state
  }

  /// Manually sets the lexer state (for context-sensitive lexing).
  ///
  /// # State replacement re-keys the input's forward-scanning facts
  ///
  /// Replacing the state can change how the region ahead of the cursor lexes, so this call
  /// re-keys every offset-dependent fact that governs forward scanning: the token cache is
  /// cleared (its entries were lexed under the old state and those offsets may lex
  /// differently now), the poison boundary is dropped, and the lexer-error dedup watermark
  /// is reset to the current committed cursor. Dropping the poison boundary is the
  /// documented limit-recovery path — swap in a fresh or bigger-budget state and scanning
  /// resumes past the old boundary.
  ///
  /// Speculative peek-ahead diagnostics emitted under the old state for the region beyond
  /// the cursor stay in the emitter log, and the watermark reset makes that same region
  /// re-reportable once it re-lexes under the new state: state surgery with outstanding
  /// speculative diagnostics may re-report the re-lexed region under the new regime, so
  /// callers should complete or roll back speculation before replacing state.
  ///
  /// # Transactional: checkpoints survive state surgery
  ///
  /// The re-key is itself **transactional**, not invalidating. A [`Checkpoint`] pure-copies
  /// every fact the re-key touches — regime, poison boundary, dedup watermark, cursor/span,
  /// and the cache-push counter — so restoring one saved *before* the surgery simply undoes
  /// it: the pre-surgery regime, boundary, watermark, and position all return, and the cache
  /// re-lexes under the restored regime. Outstanding checkpoints therefore **remain valid**
  /// across state surgery — a raw [`restore`](Self::restore), an [`attempt`](Self::attempt)
  /// rollback, and a [`StackedTransaction`] savepoint taken before the surgery all roll back
  /// across it cleanly.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_state(&mut self, state: L::State) {
    self.rekey_offset_facts();
    *self.state = state;
  }

  /// Re-keys every offset-dependent fact to the current committed cursor — the shared body
  /// of the public state-surgery APIs [`set_state`](Self::set_state) and
  /// [`state_mut`](Self::state_mut).
  ///
  /// Replacing the lexer state changes how the region ahead of the cursor lexes, so every
  /// fact keyed to the dead regime's offsets is discarded:
  ///
  /// - the **token cache** is cleared — its entries were lexed under the old state and those
  ///   offsets may lex differently now;
  /// - the **poison boundary** is dropped — a latched limit belonged to the old regime, and
  ///   replacing the state is the documented limit-recovery path (a caller swaps in a
  ///   fresh/bigger-budget state and scanning resumes);
  /// - the **lexer-error dedup watermark** is reset to the current committed cursor — not
  ///   zero: forward scanning never revisits the region behind the committed cursor (a
  ///   consume only advances), so its already-reported errors stay deduplicated, while the
  ///   region ahead must be re-evaluatable under the new regime.
  ///
  /// The cache is cleared first so the cursor reads the committed position (the end of the
  /// last consumed token), which is exactly where a re-lex now resumes. The cache-push
  /// counter is deliberately left untouched: future saves snapshot its current value.
  ///
  /// # Restoring across this re-key is consistent (state surgery is transactional)
  ///
  /// A [`Checkpoint`] saved before the surgery restores cleanly across it — walk each fact it
  /// carries against this re-key and [`restore_unchecked`](Self::restore_unchecked):
  ///
  /// - **cursor / span / state (the regime)**: pure-copied back; the cursor follows from the
  ///   restored span and the emptied cache.
  /// - **poison boundary** and **dedup watermark**: pure-copied back, overwriting the
  ///   surgery's `None` / committed-cursor reset with the saved values.
  /// - **cache-push counter**: the surgery cleared the cache (`len 0`) but left the counter,
  ///   so restore's tail-drop (`min(cache.len(), pushes − saved)`) drops nothing, and the
  ///   pure-copy re-anchors the counter to the saved value — future deltas stay exact.
  /// - **cache contents**: emptied by the surgery, so restore re-lexes the region on demand
  ///   under the RESTORED state — the old regime — which the restored state field makes
  ///   correct.
  ///
  /// Every fact therefore returns to its pre-surgery value: the surgery is simply undone,
  /// like any other post-save mutation, so outstanding checkpoints remain valid across it.
  ///
  /// This re-key is exclusive to the public state-surgery APIs. Internal state *threading* —
  /// [`restore`](Self::restore)'s copy-back, the scan/consume paths writing
  /// `*self.state = lexer.into_state()`, and the cached-consume state adoption — is
  /// lineage-consistent by construction and never routes through here.
  #[cfg_attr(not(tarpaulin), inline)]
  fn rekey_offset_facts(&mut self) {
    self.cache_mut().clear();
    *self.poison_boundary = None;
    let committed = self.cursor().as_inner().clone();
    *self.emitted_error_end = committed;
  }

  /// Returns a mutable reference to the emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn emitter(&mut self) -> &mut Ctx::Emitter {
    self.emitter
  }

  /// Emits a lexer error unless the same region has already been reported.
  ///
  /// Peeking a window larger than the cache lexes past the cached region and emits
  /// any lexer errors it finds right away, so a peek-and-stop caller never loses
  /// them. Consuming that region later re-lexes it; this dedup — keyed on the error
  /// span's end against a high-water mark — guarantees every lexer error is reported
  /// exactly once, whether it is peeked, consumed, or both.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error_deduped(
    &mut self,
    err: Spanned<<L::Token as Token<'inp>>::Error, L::Span>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let end = err.span_ref().end_ref().clone();
    if end <= *self.emitted_error_end {
      return Ok(());
    }
    *self.emitted_error_end = end;
    self.emitter().emit_lexer_error(err)
  }

  /// Returns `true` if the input is poisoned by a sticky limit error.
  ///
  /// True whenever a poison boundary is latched, regardless of the current lex
  /// position. The *positional* question a scanner asks ("has my lex position reached the
  /// boundary?") is [`reached_boundary`](Self::reached_boundary); a poisoned input can
  /// still lex strictly before its boundary (e.g. to replay a drained prefix).
  ///
  /// Test-support observability: gated to exactly the feature set of its callers (the
  /// `logos` + `std` guard test suites), so it exists precisely when they do and is never
  /// dead code under `--tests` with leaner feature combinations.
  #[cfg(all(test, feature = "logos", feature = "std"))]
  pub(super) fn is_poisoned(&self) -> bool {
    self.poison_boundary.is_some()
  }

  /// Returns `true` if `pos` — the offset a scan would lex its next token at — has
  /// reached the poison boundary (a smaller boundary is more poisoned). At or past
  /// it a scanner yields its poisoned outcome without rebuilding a lexer; strictly
  /// before it, lexing proceeds normally.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn reached_boundary(&self, pos: &L::Offset) -> bool {
    matches!(self.poison_boundary.as_ref(), Some(b) if pos >= b)
  }

  /// Lexes the next token unless doing so would cross the poison boundary.
  ///
  /// Once the position the next token would be lexed at (`lex_at`, threaded by the
  /// caller and advanced to each token's end) reaches the boundary, returns `None`
  /// so the caller's end-of-input handling produces the poisoned outcome — the
  /// tripping token and everything after it is never re-scanned. With no boundary
  /// (or strictly before it) this is exactly [`Lexed::lex_spanned`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lex_within_boundary(
    &self,
    lexer: &mut L,
    lex_at: &mut L::Offset,
  ) -> Option<Spanned<Lexed<'inp, L::Token>, L::Span>> {
    if self.reached_boundary(lex_at) {
      return None;
    }
    let lexed = Lexed::<L::Token>::lex_spanned(lexer)?;
    // Lexer contract: every lexed item has a nonempty span. The span wraps both the
    // `Token` and `Error` variants here, and this is the input layer's only lexing
    // site, so this one check guards every scanner and peek path. A zero-width span at
    // the poison boundary would be excluded by the positional gate yet advance nothing,
    // silently breaking replay and termination; catch it loudly in debug builds.
    debug_assert!(
      lexed.span_ref().end_ref() > lexed.span_ref().start_ref(),
      "lexer contract violation: zero-width token span {:?}",
      lexed.span_ref(),
    );
    *lex_at = lexed.span_ref().end_ref().clone();
    Some(lexed)
  }

  /// Latches the input-level poison boundary if `lexer`'s state has tripped a limit
  /// error, recording `boundary` — the durable frontier (the offset up to which the
  /// pre-trip tokens stay reproducible by re-lexing) — as the trip position.
  ///
  /// A limit-class error is sticky: it manifests as a failing
  /// [`check`](crate::Lexer::check) (the exact condition the lexer's own latch keys
  /// on). Because `InputRef` rebuilds a fresh lexer per operation, that per-lexer
  /// latch would be lost; recording the frontier here bounds the work a recovering
  /// caller can trigger by re-entering a scanner. Returns whether it latched. A
  /// plain (non-limit) lexer error leaves `check()` `Ok` and does not latch, so the
  /// caller keeps scanning for the next valid token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn latch_if_limit_tripped(&mut self, lexer: &L, boundary: L::Offset) -> bool {
    if lexer.check().is_err() {
      // A trip can only maintain or increase poison: clamp to the more-poisoned
      // (smaller) of any existing frontier and this one. In practice a live scan
      // never reaches a trip past an already-latched boundary (it stops at the
      // boundary first), so this only ever records the frontier or lowers it.
      match self.poison_boundary.as_ref() {
        Some(existing) if *existing <= boundary => {}
        _ => *self.poison_boundary = Some(boundary),
      }
      true
    } else {
      false
    }
  }

  /// Returns `true` if reached the end of input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[doc(alias = "is_eof")]
  #[doc(alias = "end_of_input")]
  pub fn is_eoi(&self) -> bool {
    self.offset().ge(&self.input.len())
  }

  /// Creates a lexer positioned at the end of the cache or current cursor.
  ///
  /// This internal method constructs a fresh Logos lexer with the current state and
  /// positions it to continue lexing from where the cache ends (or from the cursor
  /// if the cache is empty).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn lexer(&self) -> L
  where
    L::State: Clone,
  {
    let mut lexer = L::with_state(self.input, self.state.clone());
    lexer.bump(self.offset());
    lexer
  }

  /// Sets the cursor to the specified position, clamped to the input length.
  ///
  /// This ensures the cursor never exceeds the bounds of the input source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn set_span(&mut self, new: MaybeRef<'_, L::Span>) {
    let end = self.input.len();
    *self.span = if new.end_ref().le(&end) {
      to_owned(new)
    } else {
      L::Span::new(new.start_ref().clone(), end)
    };
  }

  /// Records the span of the just-consumed token as the current input span.
  ///
  /// `span()`/`slice()` therefore report the most recently consumed token even
  /// when the cache still holds later peeked tokens. The span is clamped to the
  /// input length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn set_span_after_consume(&mut self, new: MaybeRef<'_, L::Span>) {
    self.set_span(new);
  }
}

impl<'inp, 'closure, L, Ctx, Lang: ?Sized, Cmpl> InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  /// Attempts to parse with the given function, rolling back on failure.
  ///
  /// A checkpoint is saved before `f` runs. If `f` returns `Some`, its progress is
  /// kept. If it returns `None`, the input rolls back to the checkpoint — position,
  /// lexer state, diagnostics emitted inside the attempt, the dedup watermark, and
  /// the poison boundary all return to their pre-attempt values.
  ///
  /// This is the recommended way to backtrack: the save/restore pair is scoped to the
  /// closure, so the last-in, first-out discipline documented on [`restore`](Self::restore)
  /// holds by construction, even under nesting.
  ///
  /// # Contract: the closure owns its span of the timeline
  ///
  /// The attempt saves at entry and settles at exit — commit-shaped on `Some`, restore-shaped
  /// on `None` — so the last-in, first-out law holds structurally and a declined attempt leaves
  /// **no trace** (the rewind story above: position, lexer state, emissions, watermark, poison
  /// boundary). One violation remains expressible, only under `unstable-raw`: a raw
  /// [`restore`](Self::restore) inside `f` to a checkpoint saved *before* the attempt began
  /// would tear out the attempt's own begin point (it pops it off the live lineage). Allocator
  /// builds pin that begin point, so such a restore **panics at the restore** — its message
  /// names a live transaction guard or attempt — rather than letting `f` continue on a torn
  /// foundation and detecting it only at the decline. A LIFO-clean raw save/restore pair taken
  /// and released entirely inside `f`, above the attempt's checkpoint, is unaffected.
  /// Allocator-less targets keep no pin set, so this mixing is unspecified-but-bounded there.
  /// Enforcing tests (in `src/input/input_ref/tests.rs`):
  /// `attempt_inner_raw_restore_below_checkpoint_panics_at_restore`,
  /// `attempt_inner_lifo_clean_raw_pair_is_legal`, and
  /// `attempt_backtrack_over_trip_reemits_diagnostic_exactly_once`.
  ///
  /// For fallible closures that carry an error value, see
  /// [`try_attempt`](Self::try_attempt).
  pub fn attempt<F, R>(&mut self, f: F) -> Option<R>
  where
    F: FnOnce(&mut Self) -> Option<R>,
  {
    trace_event!(self, "attempt");
    let ckp = self.save();
    // Pin the attempt's begin point: a raw restore below it inside `f` panics at the restore.
    #[cfg(any(feature = "std", feature = "alloc"))]
    self.pin_checkpoint(ckp.ckp_id);

    match f(self) {
      Some(result) => {
        // Progress kept: the checkpoint is dropped without restoring, so unpin and drop its
        // lineage id too rather than leaving either to grow the live stack.
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
          self.unpin_checkpoint(ckp.ckp_id);
          self.forget_checkpoint(ckp.ckp_id);
        }
        Some(result)
      }
      None => {
        // Roll back to the attempt's own checkpoint. Unpin it FIRST so the restore below does
        // not see the attempt's own begin point as pinned (rolling back to it is legal). A raw
        // restore *below* this checkpoint through `f` would already have panicked at that
        // restore (detect-at-cause), so the stale assert here is now an unreachable backstop —
        // kept for defense in depth and for the allocator-less path, which pins nothing.
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
          self.unpin_checkpoint(ckp.ckp_id);
          assert!(
            self.live_contains(ckp.ckp_id),
            "attempt checkpoint is stale (invalidated by an earlier restore)"
          );
        }
        trace_event!(self, "rollback");
        self.restore(ckp);
        None
      }
    }
  }

  /// Attempts to parse with a fallible function, rolling back on error.
  ///
  /// The `Result`-shaped sibling of [`attempt`](Self::attempt), for recovery- and
  /// pratt-style flows that need the failure value. A checkpoint is saved before `f`
  /// runs.
  ///
  /// - If `f` returns `Ok`, its progress is kept and the value is returned.
  /// - If `f` returns `Err`, the input rolls back to the checkpoint and the error is
  ///   returned to the caller. Everything the attempt touched returns to its
  ///   pre-attempt value: the position, the lexer state, the diagnostics emitted
  ///   inside the attempt, the dedup watermark, and the poison boundary.
  ///
  /// Like `attempt`, this is a structural way to backtrack: the save/restore pair is
  /// scoped to the closure, so the last-in, first-out discipline documented on
  /// [`restore`](Self::restore) holds by construction, even under nesting.
  ///
  /// # Contract: the closure owns its span of the timeline
  ///
  /// Exactly [`attempt`](Self::attempt)'s contract with `Err` as the declining shape: the
  /// last-in, first-out law holds structurally, a failed attempt leaves no trace, and the one
  /// remaining violation — a raw [`restore`](Self::restore) inside `f` to a checkpoint saved
  /// *before* the attempt (`unstable-raw` only) — **panics at the restore** in allocator
  /// builds, which pin the attempt's begin point, rather than letting `f` continue on a torn
  /// foundation. Allocator-less targets are unspecified-but-bounded there. Enforcing tests
  /// (in `src/input/input_ref/tests.rs`): `try_attempt_err_rolls_back_everything`,
  /// `try_attempt_nested_lifo`, and
  /// `try_attempt_inner_raw_restore_below_checkpoint_panics_at_restore`.
  pub fn try_attempt<F, T, E>(&mut self, f: F) -> Result<T, E>
  where
    F: FnOnce(&mut Self) -> Result<T, E>,
  {
    trace_event!(self, "try_attempt");
    let ckp = self.save();
    // Pin the attempt's begin point: a raw restore below it inside `f` panics at the restore.
    #[cfg(any(feature = "std", feature = "alloc"))]
    self.pin_checkpoint(ckp.ckp_id);

    match f(self) {
      Ok(result) => {
        // Progress kept: unpin and drop the checkpoint's lineage id (see `attempt`).
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
          self.unpin_checkpoint(ckp.ckp_id);
          self.forget_checkpoint(ckp.ckp_id);
        }
        Ok(result)
      }
      Err(e) => {
        // See `attempt`: unpin FIRST (rolling back to the attempt's own base is legal), then the
        // now-unreachable stale backstop. A raw restore below this checkpoint through `f` would
        // already have panicked at that restore (detect-at-cause).
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
          self.unpin_checkpoint(ckp.ckp_id);
          assert!(
            self.live_contains(ckp.ckp_id),
            "attempt checkpoint is stale (invalidated by an earlier restore)"
          );
        }
        trace_event!(self, "rollback");
        self.restore(ckp);
        Err(e)
      }
    }
  }

  /// Starts a transaction: a scoped, compile-time-safe form of [`save`](Self::save)
  /// and [`restore`](Self::restore).
  ///
  /// The returned [`Transaction`] guard mutably borrows this input; parse through the
  /// guard (it dereferences to `InputRef`), then decide with
  /// [`commit`](Transaction::commit) (keep the progress) or
  /// [`rollback`](Transaction::rollback) (return to the begin point). Dropping the
  /// guard without deciding rolls back — uncommitted speculative work is discarded, as
  /// in a database transaction. For a guard that instead *keeps* progress on drop
  /// (commit-by-default), use [`begin_with::<Commit>`](Self::begin_with).
  ///
  /// Prefer this for imperative flows with several exits (loops, `match` arms);
  /// [`attempt`](Self::attempt)/[`try_attempt`](Self::try_attempt) for single-closure
  /// speculation; raw `save`/`restore` (feature `unstable-raw`) only where no guard shape fits.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn begin(&mut self) -> Transaction<'_, 'inp, 'closure, L, Ctx, Lang, Rollback, Cmpl> {
    self.begin_with::<Rollback>()
  }

  /// Starts a transaction with an explicit [`DropPolicy`] — the canonical generic form of
  /// [`begin`](Self::begin).
  ///
  /// The type parameter `D` fixes what an *undecided* guard does on drop:
  ///
  /// - [`Rollback`] — restore to the begin point (the speculative default that
  ///   [`begin`](Self::begin) selects; drop discards the speculative work);
  /// - [`Commit`] — keep the progress (commit-by-default, the dual a Pratt-style operator
  ///   loop wants: keep progress on every success and every `?`-propagation, and roll back
  ///   explicitly only on the branches that back out).
  ///
  /// [`commit`](Transaction::commit) and [`rollback`](Transaction::rollback) are available
  /// on either flavour; only the *drop* behaviour differs.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn begin_with<D: DropPolicy>(
    &mut self,
  ) -> Transaction<'_, 'inp, 'closure, L, Ctx, Lang, D, Cmpl> {
    trace_event!(self, "begin");
    let ckp = self.save();
    // Pin the begin point: a raw restore below it (through the guard's `DerefMut`) now panics at
    // the restore. Every settle path (commit, rollback, Drop — both policy flavors) unpins.
    #[cfg(any(feature = "std", feature = "alloc"))]
    self.pin_checkpoint(ckp.ckp_id);
    Transaction {
      input: self,
      ckp: Some(ckp),
      _policy: PhantomData,
    }
  }

  /// Starts a transaction that can hold several internal savepoints at once — the
  /// multi-fallback-point form of [`begin`](Self::begin).
  ///
  /// [`savepoint`](StackedTransaction::savepoint) marks a position;
  /// [`rollback_to`](StackedTransaction::rollback_to) returns to a mark, destroying every
  /// younger savepoint while the mark itself stays valid;
  /// [`release`](StackedTransaction::release) forgets savepoints while keeping the parsed
  /// progress; [`commit`](StackedTransaction::commit) /
  /// [`rollback`](StackedTransaction::rollback) decide the whole transaction. Savepoints
  /// follow SQL database semantics: rolling back to an older savepoint always destroys
  /// the newer ones — out-of-order revival is impossible by construction. A misused
  /// [`SavepointId`] is caught in layers: a temporally-misused id (kept past its
  /// transaction) at compile time via its lifetime brand, and a foreign or a stale id by a
  /// runtime check in every build; see [`SavepointId`].
  ///
  /// Raw [`save`](Self::save) / [`restore`](Self::restore), state replacement, and nested
  /// transactions are all reachable through the guard's deref; see the mixing rules on
  /// [`StackedTransaction`] for the one combination that invalidates a savepoint (a raw
  /// restore below it — it panics as stale in every build) and which are always legal (state
  /// surgery, nested speculation, and a LIFO-clean raw pair above the savepoints).
  ///
  /// Reach for the backtracking tools in order of shape:
  ///
  /// - [`begin`](Self::begin) / [`Transaction`] — a single speculative alternative with
  ///   several imperative exits (loops, `match` arms);
  /// - [`begin_stacked`](Self::begin_stacked) / [`StackedTransaction`] — **several live
  ///   fallback points at once** (best/longest-match selection: a savepoint after each
  ///   parsed stage, then `rollback_to` the best-scoring one);
  /// - [`attempt`](Self::attempt) / [`try_attempt`](Self::try_attempt) — closure-shaped
  ///   speculation;
  /// - [`begin_with::<Commit>`](Self::begin_with) — commit-by-default flows where progress
  ///   is kept on most exits;
  /// - [`begin_point`](Self::begin_point) session points — **non-lexical** speculation a driver
  ///   opens in one call and settles in a later one (the shape a borrowing guard cannot express).
  ///
  /// Raw [`save`](Self::save) / [`restore`](Self::restore) sit beneath all of these as the
  /// `unstable-raw` escape hatch — reachable only with that feature, for the rare shape no guard
  /// or session point fits.
  ///
  /// Dropping an undecided guard rolls back to the begin point; for a stacked guard that
  /// instead keeps its progress on drop, use
  /// [`begin_stacked_with::<Commit>`](Self::begin_stacked_with).
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn begin_stacked(
    &mut self,
  ) -> StackedTransaction<'_, 'inp, 'closure, L, Ctx, Lang, Rollback, Cmpl> {
    self.begin_stacked_with::<Rollback>()
  }

  /// Starts a stacked transaction with an explicit [`DropPolicy`] — the canonical generic
  /// form of [`begin_stacked`](Self::begin_stacked) (see
  /// [`begin_with`](Self::begin_with) for the policy meanings).
  ///
  /// `D` fixes what an *undecided* guard does on drop: [`Rollback`] rolls back to the begin
  /// point, discarding all savepoints (the default [`begin_stacked`](Self::begin_stacked)
  /// selects); [`Commit`] keeps the parsed progress. The savepoint operations and
  /// `commit`/`rollback` are identical for either flavour.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn begin_stacked_with<D: DropPolicy>(
    &mut self,
  ) -> StackedTransaction<'_, 'inp, 'closure, L, Ctx, Lang, D, Cmpl> {
    trace_event!(self, "begin_stacked");
    // Nonce = the address of this Input's `poison_boundary` field, an Input-owned slot the
    // `InputRef` holds a `&mut` to. Two simultaneously-live Inputs are distinct structs at
    // distinct addresses (the field is never zero-sized), so their nonces differ and a
    // cross-parser id is caught at runtime; the `'txn` brand on `SavepointId` — not this
    // address — rules out the address-reuse case where a dropped Input's slot is later
    // reallocated. NOT the source pointer: two Inputs can share one `&str`.
    let nonce = core::ptr::from_ref(&*self.poison_boundary).addr();
    let base = self.save();
    // Pin the begin point (only the base — savepoints keep their detect-at-use staleness rule):
    // a raw restore below the base now panics at the restore. Every whole-transaction settle
    // path (commit, rollback, Drop) unpins the base.
    self.pin_checkpoint(base.ckp_id);
    StackedTransaction {
      input: self,
      base: Some(base),
      saves: Default::default(),
      nonce,
      _policy: PhantomData,
    }
  }

  /// Opens a **session point**: saves a checkpoint of the current position onto the input's
  /// internal point stack and **pins** its lineage id, exactly as a transaction guard pins its
  /// begin point. Returns nothing — and that is the whole feature.
  ///
  /// # The shape the guards cannot express
  ///
  /// Every guard ([`begin`](Self::begin), [`begin_stacked`](Self::begin_stacked)) and both
  /// attempts are **lexical**: the guard *is* a borrow of this input, so while one is alive the
  /// input is not, and the speculative scope can only end where the borrow does — inside one
  /// expression, one block, one call. A driver that is stepped across *separate method calls* — a
  /// REPL, an IDE that parses a fragment, speculates, and decides on a later call — cannot hold a
  /// guard beside the input it borrows: that value would be self-referential.
  ///
  /// A session point is a **value on the input**, not a borrow of it. `begin_point` takes
  /// `&mut self`, pushes, and returns; the borrow ends with the call, so the whole consume surface
  /// ([`next`](Self::next), [`peek`](Self::peek), [`try_expect`](Self::try_expect), any parser you
  /// hand this input to) stays callable *with the point still open*, in this call and in later
  /// ones:
  ///
  /// ```ignore
  /// inp.begin_point();          // mark — nothing is borrowed afterwards
  /// let t = inp.next()?;        // parse, in this call or a later one
  /// let u = inp.next()?;        // …and again
  /// inp.rollback_point();       // unmark: cursor, span, state, cache, diagnostics all return
  /// ```
  ///
  /// Settle the point with [`commit_point`](Self::commit_point) (keep the progress) or
  /// [`rollback_point`](Self::rollback_point) (return to it). The stack *is* the last-in,
  /// first-out order — points settle newest-first — so nesting is structural and needs no id.
  /// [`points`](Self::points) is the live depth.
  ///
  /// # A point pins its base
  ///
  /// A session point is the base of a speculative scope, so it carries the same hazard a guard
  /// base does until it is settled: a rewind reaching *below* it would tear its foundation out.
  /// The pin makes such a rewind **panic where it is requested** rather than corrupt the timeline
  /// silently. Two ways to reach it, both caller bugs:
  ///
  /// - a raw [`restore`](Self::restore) below the point (reachable only under `unstable-raw`);
  /// - leaving a point open across the end of an enclosing guard or attempt, whose own settle then
  ///   rewinds below it.
  ///
  /// Settle your points before the scope that opened them ends and neither can arise.
  ///
  /// # Contract: a point is scoped to *this handle*, and never outlives it
  ///
  /// A session point is non-lexical — it outlives the *call* that opened it — but it is **not**
  /// unbounded: it lives on this `InputRef` and dies with it. It cannot be carried to another
  /// handle, not even one taken from the same input, and this is a *law*, not a convention.
  ///
  /// The reason is what a [`Checkpoint`] carries. Among its facts is the **emitter's emission
  /// mark** — an index into the log of *the emitter this handle borrows*
  /// ([`Emitter::checkpoint`](crate::emitter::Emitter::checkpoint)), which
  /// [`rollback_point`](Self::rollback_point) replays into
  /// [`Emitter::rewind`](crate::emitter::Emitter::rewind). A point saved while emitter *A* was
  /// borrowed and settled while emitter *B* is would truncate *B*'s log at *A*'s mark: a diagnostic
  /// count from one timeline, applied to another. So a checkpoint is only meaningful within the one
  /// emitter borrow that produced it, and a session point — a checkpoint held across calls — must be
  /// scoped to that borrow.
  ///
  /// That scope *is* this handle: `as_ref` takes the emitter borrow, the handle holds it, and the
  /// borrow ends when the handle dies. The type system enforces it — the `'closure` brand on
  /// [`Checkpoint`] (and [`Cursor`]) is invariant in the emitter-borrow lifetime, so a checkpoint
  /// cannot even be *held* across the moment a second handle is taken from the same input; the
  /// attempt is a borrow error, not a runtime surprise. The point stack is therefore a field of the
  /// handle rather than of the input, on purpose.
  ///
  /// # Dropping the handle with points open: pins released, progress kept, nothing rewound
  ///
  /// Unlike a guard — whose drop rolls back (or, under [`Commit`], keeps) its undecided scope —
  /// dropping the handle with live session points performs **no rollback**. Their speculative work
  /// is *kept*: every token consumed, every diagnostic emitted, and every state change made through
  /// an open point stands, exactly as if each had been committed. A session ends *explicitly*;
  /// rolling an abandoned one back implicitly would silently paper over a driver that lost track of
  /// its own points — the deliberate opposite of a guard's drop policy — so the end is left explicit
  /// to surface that bug instead.
  ///
  /// What the drop *does* do is **release the bookkeeping**: each remaining point's pin and its
  /// live-checkpoint lineage entry are dropped from the input's lineage memos (see [`InputRef`]'s
  /// `Drop`). It has to, precisely because the point is split across two lifetimes — the
  /// [`Checkpoint`] dies with the handle, but the pin lives on the *input*, which does not. A pin
  /// left behind would stand for a point nobody can ever settle, so the pin set would no longer hold
  /// exactly the live begin points and would grow for the life of the input. Enforcing tests (in
  /// `src/input/input_ref/session_tests.rs`): `dropping_the_handle_releases_the_open_points`,
  /// `dropping_the_handle_keeps_the_progress_of_the_open_points`, and
  /// `a_second_handle_rewinds_across_an_abandoned_point`.
  ///
  /// # Fuzz coverage
  ///
  /// The abandon path is in the fuzz alphabet as `Op::SessionAbandon` (`session.abandon(drop)`);
  /// see `OP_SURFACE_CENSUS` in `src/fuzz/ops.rs`.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn begin_point(&mut self) {
    trace_event!(self, "begin_point");
    let ckp = self.save();
    // Pin the base exactly like a guard: a rewind reaching below this point now panics at that
    // rewind instead of silently invalidating the session's foundation. Every settle path unpins —
    // `commit_point`, `rollback_point`, and the handle's `Drop` for a point abandoned outright.
    self.pin_checkpoint(ckp.ckp_id);
    self.session.points.push(ckp);
  }

  /// Settles the newest session point by **committing** it: pops it off the internal stack,
  /// releases its pin, and keeps every bit of progress made since it opened — the consuming
  /// [`commit`](Self::commit) that releases the checkpoint's lineage entry.
  ///
  /// # Panics
  ///
  /// Panics with a message prefixed `no live session point` when there is no open point to
  /// commit.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn commit_point(&mut self) {
    trace_event!(self, "commit_point");
    let ckp = self
      .session
      .points
      .pop()
      .expect("no live session point to commit");
    // Kept, not restored: unpin the base, then the raw consuming commit keeps the progress and
    // releases the lineage entry.
    self.unpin_checkpoint(ckp.ckp_id);
    self.commit(ckp);
  }

  /// Settles the newest session point by **rolling back** to it: pops it off the internal stack,
  /// releases its pin **first** — so restoring to the point does not trip its own pin, mirroring
  /// the guards' settle ordering — then performs the checked [`restore`](Self::restore). Position,
  /// span, lexer state, token cache, emission log, dedup watermark, and poison boundary all return
  /// to where the point opened.
  ///
  /// # Panics
  ///
  /// Panics with a message prefixed `no live session point` when there is no open point to roll
  /// back.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn rollback_point(&mut self) {
    trace_event!(self, "rollback_point");
    let ckp = self
      .session
      .points
      .pop()
      .expect("no live session point to roll back");
    // Unpin the base FIRST so the checked restore below does not see the point's own begin point
    // as pinned — rolling back to it is legal. A rewind *below* it would already have panicked at
    // that rewind (the pin's detect-at-cause check).
    self.unpin_checkpoint(ckp.ckp_id);
    self.restore(ckp);
  }

  /// The number of live session points — the depth of the speculation stack
  /// [`begin_point`](Self::begin_point) pushes onto, for a driver tracking where it sits in a
  /// nested speculation.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn points(&self) -> usize {
    self.session.points.len()
  }

  /// Hands out the next input-global savepoint sequence number; see
  /// [`Lineage::next_savepoint_seq`](super::Lineage::next_savepoint_seq) for the uniqueness
  /// invariant it maintains.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub(super) fn next_savepoint_seq(&mut self) -> u64 {
    self.session.lineage.next_savepoint_seq()
  }

  /// Drops `id` from the live-checkpoint lineage stack because its checkpoint was kept
  /// (committed) rather than restored — see [`Transaction::commit`], [`attempt`](Self::attempt),
  /// [`try_attempt`](Self::try_attempt), and the [`StackedTransaction`] release/commit paths.
  /// See [`Lineage::forget`](super::Lineage::forget) for the bounding invariant and its cost.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub(crate) fn forget_checkpoint(&mut self, id: u64) {
    self.session.lineage.forget(id);
  }

  /// Returns whether `id` is still live on the lineage stack; see
  /// [`Lineage::contains`](super::Lineage::contains).
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub(super) fn live_contains(&self, id: u64) -> bool {
    self.session.lineage.contains(id)
  }

  /// Pops the lineage stack down through `id` inclusive on restore; see
  /// [`Lineage::pop_through`](super::Lineage::pop_through).
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  fn live_pop_through(&mut self, id: u64) {
    self.session.lineage.pop_through(id);
  }

  /// Pins `id` — the begin-point checkpoint of a transaction guard, an
  /// [`attempt`](Self::attempt), or a [session point](Self::begin_point) — so a raw
  /// [`restore`](Self::restore) reaching below it panics at the restore. Every guard constructor
  /// ([`begin_with`](Self::begin_with), [`begin_stacked_with`](Self::begin_stacked_with)),
  /// [`attempt`](Self::attempt)/[`try_attempt`](Self::try_attempt), and
  /// [`begin_point`](Self::begin_point) pins on entry; the matching
  /// [`unpin_checkpoint`](Self::unpin_checkpoint) runs on every settle path. See
  /// [`Lineage::pin`](super::Lineage::pin) for the borrowck-serialization argument (session points
  /// are serialized instead by their own last-in, first-out stack).
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub(crate) fn pin_checkpoint(&mut self, id: u64) {
    self.session.lineage.pin(id);
  }

  /// Removes `id` from the pin set when its guard, attempt, or session point settles; see
  /// [`Lineage::unpin`](super::Lineage::unpin). Called on **every** settle path (commit, explicit
  /// rollback, `Drop`, both closure arms of the attempts, and both session-point verbs). A session
  /// point abandoned with the handle settles through this handle's `Drop`, which reaches
  /// [`Lineage::unpin`](super::Lineage::unpin) directly — a `Drop` impl may not add the
  /// `L::State: Clone` bound this method's impl block carries.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub(crate) fn unpin_checkpoint(&mut self, id: u64) {
    self.session.lineage.unpin(id);
  }

  /// Panics if restoring to `target_id` would pop a **pinned** checkpoint off the live lineage —
  /// the detect-at-cause check that refuses a raw restore below a live guard/attempt begin point,
  /// in every allocator build. See
  /// [`Lineage::assert_restore_preserves_pins`](super::Lineage::assert_restore_preserves_pins)
  /// for why a guard's own settle, a savepoint `rollback_to`, and a dead target never trip it.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  fn assert_restore_preserves_pins(&self, target_id: u64) {
    self
      .session
      .lineage
      .assert_restore_preserves_pins(target_id);
  }

  /// The number of live checkpoints — test-only observability for the no-growth
  /// guarantee that committing (and a success-path [`Recover`](crate::parser::Recover))
  /// gives the lineage stack (see [`Lineage::live_len`](super::Lineage::live_len)).
  ///
  /// The stack it measures is maintained in every allocator build, so this accessor is gated
  /// only to its callers — the `logos` + `std` guard and recover test suites — and *not* to
  /// `debug_assertions` or `target_has_atomic = "ptr"`, so the no-growth cases can run under the
  /// release profile too. Keeping the `logos` + `std` constraint (rather than the looser
  /// `any(std, alloc)`) keeps the method from being dead code under
  /// `cargo hack --each-feature --tests`, whose single-feature combinations never enable both
  /// `logos` and `std` and so compile neither this method nor its callers.
  #[cfg(all(test, feature = "logos", feature = "std"))]
  pub(crate) fn live_checkpoints_len(&self) -> usize {
    self.session.lineage.live_len()
  }

  /// Returns a slice of the current token from the input source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice(&self) -> <L::Source as Source<L::Offset>>::Slice<'inp> {
    self
      .input
      .slice(self.span.start()..self.span.end())
      .expect("lexer should guarantee slice")
  }

  /// Returns a slice of the input source from the given cursor to the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice_since(
    &self,
    cursor: &Cursor<'inp, 'closure, L>,
  ) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    let end = self.cursor();
    self
      .input
      .slice(cursor.as_inner().clone()..end.as_inner().clone())
  }

  /// Returns a slice of the input source from the given cursor to the end of the input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice_from(
    &self,
    cursor: &Cursor<'inp, 'closure, L>,
  ) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    let start = cursor.as_inner().clone();
    self.input.slice(start..)
  }

  /// Returns a slice of the input source for the given cursor range.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice_range<'r, R>(
    &self,
    range: R,
  ) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>>
  where
    R: RangeBounds<&'r Cursor<'inp, 'closure, L>>,
    'closure: 'r,
  {
    let start = range.start_bound().map(|c| c.as_inner().clone());
    let end = range.end_bound().map(|c| c.as_inner().clone());
    // SAFETY: The range is guaranteed to be within bounds as both cursors are within input length and comes from the same input.
    self.input.slice((start, end))
  }

  /// Returns the span of the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> &L::Span {
    self.span
  }

  /// Returns a span from the given cursor to the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span_since(&self, cursor: &Cursor<'inp, 'closure, L>) -> L::Span {
    Span::new(cursor.as_inner().clone(), self.cursor().as_inner().clone())
  }

  /// Returns a span from the given cursor to the end of the input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span_from(&self, cursor: &Cursor<'inp, 'closure, L>) -> L::Span {
    Span::new(cursor.as_inner().clone(), self.input.len())
  }

  /// Returns a span for the given cursor range.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span_range(&self, range: Range<&Cursor<'inp, 'closure, L>>) -> L::Span {
    Span::new(range.start.as_inner().clone(), range.end.as_inner().clone())
  }

  /// Saves the current state as a [`Checkpoint`] for backtracking.
  ///
  /// # Unstable: feature-gated raw API
  ///
  /// `save` is one third of the raw checkpoint triple (`save` / [`restore`](Self::restore) /
  /// [`commit`](Self::commit)) and is public **only** under the `unstable-raw` feature; without
  /// it the method is crate-internal, so a [`Checkpoint`] can be neither obtained nor consumed
  /// from another crate. The supported backtracking surface is the transaction guards
  /// ([`begin`](Self::begin) / [`begin_stacked`](Self::begin_stacked)), the
  /// [session points](Self::begin_point), and
  /// [`attempt`](Self::attempt)/[`try_attempt`](Self::try_attempt) — together these cover every
  /// legal backtracking shape. The last-in, first-out / lineage contract documented here and on
  /// [`restore`](Self::restore) governs the raw triple unchanged whenever the feature is on.
  ///
  /// The checkpoint captures the cursor, the last-consumed span, the lexer state, the
  /// emitter's emission mark, the lexer-error dedup watermark, and the poison
  /// boundary — everything [`restore`](Self::restore) needs to make this exact moment
  /// the live state again.
  ///
  /// Saving is amortized O(1): it clones the lexer state and a few offsets, and — in
  /// allocator builds — records the checkpoint's id on the input's live-checkpoint
  /// lineage stack (one `Vec` push) so restore ordering and savepoint validity can be
  /// tracked in every build; allocator-less builds allocate nothing. Saving never
  /// invalidates other checkpoints; only restoring does (see [`Checkpoint`]'s validity
  /// section).
  ///
  /// Every checkpoint `save` returns should end in exactly one of [`restore`](Self::restore)
  /// (abandon this branch and rewind) or [`commit`](Self::commit) (keep this branch's progress
  /// and release the checkpoint's lineage entry); a checkpoint merely dropped keeps its progress
  /// but strands that lineage entry until an older restore pops through it.
  ///
  /// Prefer [`attempt`](Self::attempt)/[`try_attempt`](Self::try_attempt) when the
  /// save/restore pair brackets a single speculative computation — they enforce the
  /// restore discipline by construction.
  #[cfg(feature = "unstable-raw")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable-raw")))]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn save(&mut self) -> Checkpoint<'inp, 'closure, L> {
    self.save_checkpoint()
  }

  /// The crate-internal raw `save`, used when the `unstable-raw` valve is off — the primitive the
  /// transaction guards, [`attempt`](Self::attempt)/[`try_attempt`](Self::try_attempt), and the
  /// [session points](Self::begin_point) build on. Same body as the public flavor;
  /// only its visibility differs.
  #[cfg(not(feature = "unstable-raw"))]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn save(&mut self) -> Checkpoint<'inp, 'closure, L> {
    self.save_checkpoint()
  }

  /// Shared body of the [`save`](Self::save) twins.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn save_checkpoint(&mut self) -> Checkpoint<'inp, 'closure, L> {
    // Open a lineage entry (every allocator build): take a fresh id, record it on the
    // live-checkpoint stack, and stamp it into the checkpoint. `restore` pops the stack down
    // through that id, and a `StackedTransaction` checks the id is still present before honoring
    // a savepoint — the check that makes stale savepoints panic on release and no-ptr targets.
    #[cfg(any(feature = "std", feature = "alloc"))]
    let ckp_id = self.session.lineage.open();
    Checkpoint::new(
      self.cursor().clone(),
      self.span.clone(),
      self.state.clone(),
      self.emitter.checkpoint(),
      self.emitted_error_end.clone(),
      self.poison_boundary.clone(),
      self.session.lineage.cache_pushes(),
      #[cfg(all(
        debug_assertions,
        any(feature = "std", feature = "alloc"),
        target_has_atomic = "ptr"
      ))]
      self.witness.input_id(),
      #[cfg(any(feature = "std", feature = "alloc"))]
      ckp_id,
    )
  }

  /// Returns the current cursor position.
  ///
  /// If there are cached tokens, the cursor points to the start
  /// of the first cached token; otherwise, it points to the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn cursor(&self) -> &Cursor<'inp, 'closure, L> {
    Cursor::from_ref(
      self
        .cache()
        .front_span()
        .map(|span| span.start_ref())
        .unwrap_or_else(|| self.span.end_ref()),
    )
  }

  /// Returns the current offset of the tokenizer.
  ///
  /// This is the end of the last lexed token (cached or otherwise).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn offset(&self) -> &L::Offset {
    self
      .cache()
      .back_span()
      .map(|s| s.end_ref())
      .unwrap_or_else(|| self.span.end_ref())
  }

  /// Rewinds the input to `checkpoint`'s save point.
  ///
  /// # Unstable: feature-gated raw API
  ///
  /// `restore` is part of the raw checkpoint triple ([`save`](Self::save) / `restore` /
  /// [`commit`](Self::commit)) and is public **only** under the `unstable-raw` feature; without
  /// it the method is crate-internal. The supported backtracking surface is the transaction
  /// guards ([`begin`](Self::begin) / [`begin_stacked`](Self::begin_stacked)), the
  /// [session points](Self::begin_point), and
  /// [`attempt`](Self::attempt)/[`try_attempt`](Self::try_attempt); each enforces the
  /// last-in, first-out discipline below by construction. That contract applies to the raw
  /// triple unchanged whenever the feature is on.
  ///
  /// After a restore, the input behaves exactly as it did the moment the checkpoint
  /// was taken:
  ///
  /// - the cursor, last-consumed span, and lexer state are restored; consuming
  ///   resumes from the saved position. Cached tokens appended after the save belong to
  ///   the abandoned continuation and are dropped so their region re-lexes (re-emitting
  ///   any lexer error it held); tokens cached before the save re-lex identically — this
  ///   includes a pre-save cached token the abandoned branch already consumed out of the
  ///   cache: it is re-lexed on demand after the restore. By the `Lexer` determinism
  ///   contract that replay is identical (the same token and span, its diagnostics
  ///   exactly once, an in-`State` limiter recounting the same), while scan-count
  ///   instrumentation held outside the lexer state will observe the additional scans;
  /// - diagnostics emitted after the save are rolled back — the emitter's emission
  ///   log is truncated to the saved mark (see
  ///   [`Emitter::rewind`](crate::emitter::Emitter::rewind));
  /// - the lexer-error dedup watermark returns to its saved value: an error whose
  ///   emission was just rolled back becomes re-emittable — exactly once — if the
  ///   resumed parse reaches it again, while errors retained from before the save
  ///   stay deduplicated;
  /// - the poison boundary returns to its saved value: an input unpoisoned at save
  ///   time is unpoisoned again (a rolled-back limit trip re-trips and re-diagnoses
  ///   if re-reached); an input poisoned at save time gets the saved boundary and its
  ///   retained diagnostic back, still paired.
  ///
  /// # Contract: restores are last-in, first-out
  ///
  /// Restoring this checkpoint **invalidates every checkpoint saved after it**.
  /// Equivalently: with several live checkpoints, always restore the youngest one you
  /// intend to return to; never restore a checkpoint after restoring one older than
  /// it.
  ///
  /// Both of these are fine:
  ///
  /// ```ignore
  /// // Nested speculation — inner ended before outer (each ends in commit or restore):
  /// let outer = input.save();
  /// let inner = input.save();
  /// if try_variant_a(input) { input.commit(inner) } else { input.restore(inner) } // youngest first
  /// if try_variant_b(input) { input.commit(outer) } else { input.restore(outer) } // then the older
  ///
  /// // Retry loop — a fresh checkpoint per iteration:
  /// loop {
  ///   let ckp = input.save();
  ///   match try_parse(input) {
  ///     Ok(v) => { input.commit(ckp); break v }          // success: keep progress, release the id
  ///     Err(_) => input.restore(ckp),                    // failure: the youngest live one
  ///   }
  /// }
  /// ```
  ///
  /// This is a contract violation:
  ///
  /// ```ignore
  /// let a = input.save();
  /// let b = input.save();   // b is younger than a
  /// input.restore(a);       // rolls history back past b's save point:
  ///                         // b now refers to a lineage that no longer exists
  /// input.restore(b);       // ✗ contract violation
  /// ```
  ///
  /// The reason is structural, not stylistic: restoring `a` truncated the emission
  /// log below `b`'s mark and un-lexed the tokens `b`'s position depends on. A
  /// truncated log cannot be rebuilt, so there is *no correct state* the second
  /// restore could produce.
  ///
  /// # Debug builds
  ///
  /// Debug builds track live checkpoints exactly and **panic** on any out-of-order
  /// restore (message begins `non-LIFO checkpoint restore`), and on restoring a
  /// checkpoint into an input that did not create it. `cargo test` compiles with
  /// debug assertions by default, so exercising your parser's backtracking paths in
  /// tests surfaces violations immediately.
  ///
  /// # Release builds
  ///
  /// Release builds do not check. An out-of-order restore leaves the input in an
  /// **unspecified but bounded** state. Even then, all of the following still hold:
  /// no undefined behavior, no leak, no panic originating in this crate, every scan
  /// terminates (the resource-limiter state travels inside the checkpoint, so a
  /// re-reached limit re-trips instead of rescanning without bound), and the input
  /// remains usable.
  ///
  /// What is **not** guaranteed after a violation: diagnostics may be missing or
  /// attributed to the wrong branch, and the replayed token stream may differ from
  /// what was visible at the save. The only well-specified use of a checkpoint is
  /// restoring it while it is still valid.
  #[cfg(feature = "unstable-raw")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable-raw")))]
  #[doc(alias = "rewinds")]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn restore(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    self.restore_checked(checkpoint)
  }

  /// The crate-internal raw `restore`, used when the `unstable-raw` valve is off. Same body as
  /// the public flavor; only its visibility differs. The transaction guards, the
  /// [session points](Self::begin_point), and
  /// [`attempt`](Self::attempt)/[`try_attempt`](Self::try_attempt) rewind through it.
  #[cfg(not(feature = "unstable-raw"))]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn restore(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    self.restore_checked(checkpoint)
  }

  /// Shared body of the [`restore`](Self::restore) twins: verifies the last-in, first-out and
  /// foreign-input discipline (debug + ptr builds) and refuses a restore that would tear out a
  /// pinned guard/attempt begin point (every allocator build), then rewinds.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn restore_checked(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    // Verify the last-in, first-out discipline exactly, before any mutation: the
    // checkpoint must belong to this input, and it must still be live (restoring an
    // older checkpoint invalidates every one saved after it). Release and no-ptr builds
    // omit these panics; the lineage stack itself is still maintained in every allocator
    // build inside `restore_unchecked`.
    #[cfg(all(
      debug_assertions,
      any(feature = "std", feature = "alloc"),
      target_has_atomic = "ptr"
    ))]
    {
      assert!(
        checkpoint.input_id == self.witness.input_id(),
        "checkpoint restored into a foreign input: this checkpoint was created by a different input"
      );
      assert!(
        self.live_contains(checkpoint.ckp_id),
        "non-LIFO checkpoint restore: this checkpoint was invalidated by restoring an older one (restores must be last-in, first-out)"
      );
    }
    // Detect-at-cause, in EVERY allocator build (unlike the debug-only misuse panics above):
    // refuse a restore that would tear the begin point out from under a live transaction guard
    // or attempt — a raw restore below its pinned base. A guard's own settle unpins its held id
    // before reaching here, so this never trips a guard rolling back to its own base.
    #[cfg(any(feature = "std", feature = "alloc"))]
    self.assert_restore_preserves_pins(checkpoint.ckp_id);

    self.restore_unchecked(checkpoint);
  }

  /// Commits `checkpoint`: keeps every bit of progress made since its save and releases the
  /// checkpoint's lineage entry. This is the success-path counterpart to
  /// [`restore`](Self::restore) — the verb for a speculative branch that *worked out*.
  ///
  /// # Unstable: feature-gated raw API
  ///
  /// `commit` is part of the raw checkpoint triple ([`save`](Self::save) /
  /// [`restore`](Self::restore) / `commit`) and is public **only** under the `unstable-raw`
  /// feature; without it the method is crate-internal. The supported backtracking surface is the
  /// transaction guards ([`begin`](Self::begin) / [`begin_stacked`](Self::begin_stacked)), the
  /// [session points](Self::begin_point), and
  /// [`attempt`](Self::attempt)/[`try_attempt`](Self::try_attempt); the lineage contract below
  /// applies to the raw triple unchanged whenever the feature is on.
  ///
  /// # Contract: end each checkpoint in exactly one of restore or commit
  ///
  /// A saved [`Checkpoint`] should end its life in exactly one of two ways: hand it to
  /// [`restore`](Self::restore) to abandon the branch and rewind, or hand it to `commit` to
  /// keep the branch's progress. A checkpoint that is merely **dropped** keeps the progress
  /// too — dropping rewinds nothing — but in allocator builds its id lingers on the input's
  /// live-checkpoint lineage stack until an older [`restore`](Self::restore) happens to pop
  /// through it. Repeated successful speculation that drops rather than commits therefore grows
  /// that stack for the life of the input; `commit` is what keeps it bounded. (The stranded ids
  /// are inert lineage bookkeeping, not unsafety: every restore still replays its lineage
  /// exactly.)
  ///
  /// A retry loop keeps its progress by committing the youngest live checkpoint on success:
  ///
  /// ```ignore
  /// loop {
  ///   let ckp = input.save();
  ///   match try_parse(input) {
  ///     Ok(v) => { input.commit(ckp); break v }   // success: keep progress, release the id
  ///     Err(_) => input.restore(ckp),             // failure: rewind to the save
  ///   }
  /// }
  /// ```
  ///
  /// Releasing is `O(1)` when `checkpoint` is the youngest live checkpoint — the common
  /// retry-loop case — and a linear removal otherwise (e.g. a younger raw checkpoint was dropped
  /// above it); the rest of the stack keeps its order either way, so an older restore still pops
  /// cleanly through the gap. Committing an already-invalidated checkpoint — one an older
  /// [`restore`](Self::restore) already popped off the lineage — is a harmless **no-op**: its id
  /// is simply absent, so nothing is released and no state changes (no panic, in any build).
  ///
  /// Allocator-less builds keep no lineage stack, so `commit` there merely drops the checkpoint;
  /// the growth it prevents cannot arise without a stack to grow.
  #[cfg(feature = "unstable-raw")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable-raw")))]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn commit(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    self.commit_checkpoint(checkpoint)
  }

  /// The crate-internal raw `commit`, used when the `unstable-raw` valve is off. Same body as the
  /// public flavor; only its visibility differs. Its sole in-crate caller is the allocator-gated
  /// [`commit_point`](Self::commit_point) (the guards release their kept
  /// begin points through unpin/forget directly), so in an allocator-less valve-off build it is
  /// deliberately uncalled — kept defined so the raw triple stays whole in every configuration.
  #[cfg(not(feature = "unstable-raw"))]
  #[cfg_attr(not(any(feature = "std", feature = "alloc")), allow(dead_code))]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn commit(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    self.commit_checkpoint(checkpoint)
  }

  /// Shared body of the [`commit`](Self::commit) twins. Reachable only through them, so in the
  /// one configuration where both are uncalled (valve off, no allocator — see the twin above) it
  /// shares their deliberate-dead-code allowance.
  #[cfg_attr(
    all(
      not(feature = "unstable-raw"),
      not(any(feature = "std", feature = "alloc"))
    ),
    allow(dead_code)
  )]
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn commit_checkpoint(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    // Cheap sanity in debug + ptr builds, mirroring `restore`'s foreign-input guard: a
    // checkpoint may only be committed into the input that created it. Presence is NOT
    // asserted — committing a dead checkpoint is the documented no-op handled below.
    #[cfg(all(
      debug_assertions,
      any(feature = "std", feature = "alloc"),
      target_has_atomic = "ptr"
    ))]
    assert!(
      checkpoint.input_id == self.witness.input_id(),
      "checkpoint committed into a foreign input: this checkpoint was created by a different input"
    );

    // Keep all progress; release ONLY the lineage entry, never the pin set. `forget_checkpoint`
    // is `O(1)` at the stack top and pops nothing for an already-invalidated id (the no-op case).
    //
    // No pin check is needed, and none could ever trip: a pinned id is the begin point of a live
    // transaction guard or `attempt`, which holds that begin-point `Checkpoint` internally and
    // never hands it out. A caller can only reach a checkpoint's id THROUGH a `Checkpoint` value,
    // and this method consumes one it was given — so the committed id is a raw, unpinned
    // checkpoint by construction. There is no reachable way to commit a guard's pinned base and
    // unpin-bypass it, and `forget_checkpoint` leaves `pinned` untouched regardless.
    #[cfg(any(feature = "std", feature = "alloc"))]
    self.forget_checkpoint(checkpoint.ckp_id);

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    let _ = checkpoint;
  }

  /// Rewinds to `checkpoint` without the debug raw-misuse panics, the shared primitive behind
  /// the checked [`restore`](Self::restore) and the drop-path
  /// [`restore_unchecked_if_live`](Self::restore_unchecked_if_live). A rolling-back `Drop`
  /// reaches it through the latter and must stay silent: `Drop` may run while already unwinding,
  /// and `no_std` has no `thread::panicking()` to guard a drop-bomb, so a debug assert firing
  /// here would abort. It still maintains the lineage stack (popping through the restored id if
  /// present) and replays the saved lineage exactly, identically to [`restore`](Self::restore)
  /// in release. Its own base is usually the oldest live checkpoint, but a raw restore below it
  /// through the guard can invalidate it first — which is why the drop path consults liveness
  /// before calling in (skipping a dead base), and an explicit
  /// [`rollback`](Transaction::rollback) restores through the checked [`restore`](Self::restore),
  /// panicking on that stale case since it never runs during an unwind.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn restore_unchecked(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    // Maintain the lineage stack in every allocator build: pop it down through the restored
    // id (invalidating it and every younger checkpoint). An absent id is a no-op — a raw
    // restore to a checkpoint an earlier restore already invalidated (release's
    // unspecified-but-bounded posture; `restore` asserts presence in debug + ptr).
    #[cfg(any(feature = "std", feature = "alloc"))]
    self.live_pop_through(checkpoint.ckp_id);

    self.cache_mut().rewind(&checkpoint);
    // Drop the cache entries pushed after the save. They were lexed on the continuation
    // this restore abandons, and the cache memoizes only their token *values*, not the
    // scan side effects of the region they came from (a lexer error emitted while lexing
    // across it). Leaving them would let a later drain jump over a rewound error instead
    // of re-lexing — and re-emitting — its region, so drop them here on every restore path.
    //
    // The push count is per-lineage state the copy-back below rewinds to its saved value on
    // every restore, exactly like the dedup watermark and the poison boundary. So the count
    // always describes the CURRENT lineage, and `cache_pushes - saved` counts the pushes
    // since this checkpoint within that lineage only. Pushes only ever append to the back and
    // evictions only ever pop the front or clear the whole cache, so the live cache is a
    // contiguous run of the push sequence in push order and the post-save entries are its
    // tail. `min(len, ..)` discounts post-save entries a front eviction or a consume already
    // removed — those lower the survivor count below `cache_pushes - saved` — so dropping
    // that many from the back removes exactly the ones still resident.
    //
    // Rewinding the count is what makes nested last-in, first-out restores compose. Take the
    // sequence: prefill one cached token, save outer, save inner, peek more, restore inner,
    // restore outer. The inner restore drops its post-save tail and rewinds the count to the
    // inner save's value; nothing was pushed between the two saves, so that equals the outer
    // save's value. The outer restore's cursor equals the cache front, so the rewind above
    // no-ops, and `cache_pushes - saved` is now zero: it drops nothing and retains the
    // prefilled pre-save token. A never-rewound count would still read stale-high here and
    // over-drop that token, forcing a re-lex whose scan side effects belong to the abandoned
    // lineage (advancing shared lexer/limit state, latching a poison the checkpoint predates).
    let post_save = self
      .session
      .lineage
      .cache_pushes()
      .saturating_sub(checkpoint.cache_pushes);
    let survivors = (self.cache.len() as u64).min(post_save);
    for _ in 0..survivors {
      self.cache.pop_back();
    }
    let cur = checkpoint.cursor();
    self.emitter().rewind(cur, checkpoint.emitter_checkpoint);
    // The push count, the dedup watermark, and the poison boundary are facts about the saved
    // lineage. Under the last-in, first-out contract the restore returns to that lineage
    // exactly, so all three copy back verbatim: the count is restored to the push history of
    // the lineage now live (the tail-drop above already consumed its pre-rewind value), and a
    // saved boundary's diagnostic predates the saved emitter mark and therefore survives the
    // rewind above, keeping poison and its diagnostic paired.
    self
      .session
      .lineage
      .restore_cache_pushes(checkpoint.cache_pushes);
    *self.emitted_error_end = checkpoint.emitted_error_end;
    *self.poison_boundary = checkpoint.poison_boundary;
    self.set_span((&checkpoint.span).into());
    *self.state = checkpoint.state;
  }

  /// Drop-path rewind that never resurrects a dead base. Used by the transaction guards'
  /// rolling-back [`Drop`], whose held begin-point checkpoint a raw restore below it (through
  /// the guard's `DerefMut`) may have popped off the live lineage.
  ///
  /// If the checkpoint is still live it rewinds exactly as
  /// [`restore_unchecked`](Self::restore_unchecked). If an earlier restore already invalidated
  /// it, the input already sits where that older restore left it, so this skips the rewind
  /// rather than copying the stale saved state back over it. It never panics: a `Drop` may run
  /// while already unwinding, so it must stay silent.
  ///
  /// # Now a backstop
  ///
  /// The guards pin their begin point, so in allocator builds a raw restore that would pop it
  /// off the lineage panics **at the restore** ([`restore`](Self::restore)'s pin check) — the
  /// base can no longer go stale while its guard is live, so the skip branch here is
  /// unreachable and this always rewinds. The skip is retained as **defense in depth** and for
  /// allocator-less builds, which keep no pin set and no lineage stack: there the rewind always
  /// proceeds regardless, unspecified-but-bounded on misuse as documented on the guards. Reads
  /// the lineage stack without popping — the pop-through happens only inside the rewind it
  /// forwards to.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn restore_unchecked_if_live(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    #[cfg(any(feature = "std", feature = "alloc"))]
    {
      if !self.live_contains(checkpoint.ckp_id) {
        return;
      }
    }
    self.restore_unchecked(checkpoint);
  }

  /// Advances the cursor and returns the next valid token, emitting errors encountered on the way.
  ///
  /// Skips over lexer errors, emitting them through the provided emitter.
  /// Non-fatal errors are emitted and the method continues to the next token.
  ///
  /// # Partial-input frontier (`Partial`, non-final)
  ///
  /// On a [`Partial`](crate::input::Partial) input that is not yet final
  /// ([`is_final`](Self::is_final) `== false`), three conservative rules keep a construct that
  /// later input could still extend from being mistaken for a finished one — each surfaces an
  /// [`Incomplete`](crate::error::Incomplete) on the `Err` channel instead:
  ///
  /// 1. **Frontier holdback** — a token whose span **end touches the buffer end** is not yielded;
  ///    it may be a prefix of a longer token once more input arrives.
  /// 2. **Frontier error** — a lexer error whose span **touches the buffer end** is not emitted; it
  ///    may be a truncation artifact.
  /// 3. **Non-final EOF** — lexer exhaustion is not treated as genuine end of input; more may come.
  ///
  /// A genuine limit trip (poison boundary) still yields `Ok(None)` — it is terminal, not
  /// incomplete. With [`is_final`](Self::is_final) `== true`, or on a
  /// [`Complete`](crate::input::Complete) input, all three rules are off and `next` behaves
  /// identically to before this typestate existed (the checks are eliminated at monomorphization).
  /// The frontier holdback means the last token only becomes visible after more input arrives or
  /// the input is marked final — a **one-token latency** that is correct by construction. See the
  /// [`input`](crate::input) module docs for the Sans-I/O resumption loop.
  #[allow(clippy::should_implement_trait)]
  pub fn next(
    &mut self,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Cmpl: SurfaceIncomplete<'inp, L, Ctx, Lang>,
  {
    if let Some(cached_token) = self.cache_mut().pop_front() {
      let (spanned_lexed, extras) = cached_token.into_components();
      let (span, lexed) = spanned_lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = extras;
      return Ok(Some(Spanned::new(span, lexed)));
    }

    // A sticky limit trip latches a poison boundary: once the cache is drained and
    // the cursor has reached the durable frontier, stop without rebuilding a lexer
    // or rescanning the tripping token. Strictly before it, `next()` re-lexes (e.g.
    // to replay a drained prefix after a restore).
    if self.reached_boundary(self.offset()) {
      return Ok(None);
    }

    // `next()` commits no progress before a poisoned or exhausted outcome, so it
    // latches at the cursor and yields `None` on both a trip and end of input.
    let mut lex_at = self.offset().clone();
    let mut lexer = self.lexer();
    match self.scan_with(&mut lexer, &mut lex_at, &mut AtCursor)? {
      Scan::Token(tok) => {
        self.set_span_after_consume(tok.span_ref().into());
        *self.state = lexer.into_state();
        Ok(Some(tok))
      }
      Scan::Tripped | Scan::Eof => Ok(None),
    }
  }

  /// Internal implementation for syncing tokens in the cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn sync_matched_in_cache<P, Exp>(
    &mut self,
    mut pred: P,
    mut exp: Exp,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    P: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
  {
    let matched = core::cell::RefCell::new(false);
    // pop from cache if not matching
    while let Some(tok) = self.cache.pop_front_if(|t| {
      let span = t.token().span();
      *matched.borrow_mut() = pred(Spanned::new(span, t.token().data()));
      !*matched.borrow()
    }) {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;

      // if matched, we stop here
      if *matched.borrow() {
        return Ok(Some(Spanned::new(span, tok)));
      }

      // Note: cursor/state are updated before emission. If emission fails,
      // the error token has still been consumed (no backtracking here).

      self
        .emitter()
        .emit_unexpected_token(UnexpectedToken::maybe_expected_of(span, exp()).with_found(tok))?;
    }
    Ok(None)
  }

  /// Runs the shared scanner loop: lex within the poison boundary and handle
  /// every lexer error in one place — latch the durable frontier on a limit
  /// trip, deduplicate-and-emit the diagnostic, and take the identical fatal
  /// exit when the emitter rejects it.
  ///
  /// Returns to the caller only on an event it must decide: a valid
  /// [`Scan::Token`] (the caller applies its per-path policy and either commits
  /// or keeps scanning), a [`Scan::Tripped`] limit trip (already latched and
  /// emitted), or [`Scan::Eof`]. `frontier` chooses where a trip latches —
  /// [`AtCursor`] for scans that commit no progress first, [`AtFrontier`] for
  /// scans that consume tokens as they go — and advances over each error the
  /// loop skips on the way to the next event.
  ///
  /// # The partial-input frontier rules live here
  ///
  /// This is the sole driver of the single lexing site
  /// ([`lex_within_boundary`](Self::lex_within_boundary)), so the three partial-input frontier
  /// rules are applied here once for every consume path (`next`, `try_expect*`, `skip_while`, the
  /// `sync` family) rather than scattered across them. In [`Partial`](crate::input::Partial)
  /// non-final mode they surface an [`Incomplete`](crate::error::Incomplete) on the `Err` channel,
  /// which every `scan_with(..)?` caller propagates unchanged:
  ///
  /// - **frontier holdback / frontier error** — a lexed item (token *or* error) whose span end
  ///   touches the buffer end is withheld, since more input could extend it;
  /// - **non-final EOF** — lexer exhaustion that is *not* a poison-boundary trip surfaces
  ///   Incomplete, since more input may still arrive.
  ///
  /// All three are written `if Cmpl::PARTIAL && …`; on a [`Complete`](crate::input::Complete)
  /// input `Cmpl::PARTIAL` is a `false` constant, so the whole block is eliminated at
  /// monomorphization and this compiles to the pre-typestate scanner byte for byte.
  #[cfg_attr(not(tarpaulin), inline)]
  fn scan_with<Fr>(
    &mut self,
    lexer: &mut L,
    lex_at: &mut L::Offset,
    frontier: &mut Fr,
  ) -> Result<Scan<'inp, L>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Fr: Frontier<'inp, L>,
    Cmpl: SurfaceIncomplete<'inp, L, Ctx, Lang>,
  {
    while let Some(Spanned { span, data: tok }) = self.lex_within_boundary(lexer, lex_at) {
      // Frontier holdback (rule 1) and frontier error (rule 2): in partial, non-final mode a
      // lexed item whose span END touches the buffer end may be a prefix of a longer construct
      // once more input arrives, so it is neither yielded nor emitted — surface Incomplete
      // instead. Const-gated: `Complete::PARTIAL` is `false`, so this is dead-code-eliminated and
      // `is_final()` is never even evaluated on the complete path.
      if Cmpl::PARTIAL && !self.is_final() && span.end_ref() >= &self.input.len() {
        return Err(Cmpl::surface_incomplete(span.end_ref().clone()));
      }
      match tok {
        Lexed::Error(err) => {
          let boundary = frontier.boundary(self.offset());
          let limit_hit = self.latch_if_limit_tripped(lexer, boundary);
          match self.emit_lexer_error_deduped(Spanned::new(span, err)) {
            Ok(()) => {
              if limit_hit {
                return Ok(Scan::Tripped);
              }
              // Non-limit error: skip over it and keep scanning for a token.
              frontier.advance(lexer);
            }
            Err(e) => {
              self.set_span_after_consume(lexer.span().into());
              *self.state = lexer.state().clone();
              return Err(e);
            }
          }
        }
        Lexed::Token(tok) => return Ok(Scan::Token(Spanned::new(span, tok))),
      }
    }

    // Non-final EOF (rule 3): the lexer is exhausted, but in partial non-final mode more input
    // may still arrive, so this is not genuine end of input — surface Incomplete. A poison-boundary
    // trip is exempt: it is a terminal limit outcome (re-lexing the same prefix re-trips), so it
    // stands as `Eof`. Const-gated, so `Complete` never reaches this and yields `Eof` as before.
    if Cmpl::PARTIAL && !self.is_final() && !self.reached_boundary(lex_at) {
      return Err(Cmpl::surface_incomplete(lex_at.clone()));
    }

    Ok(Scan::Eof)
  }
}

#[cfg_attr(not(tarpaulin), inline(always))]
fn to_owned<T>(maybe: MaybeRef<'_, T>) -> T
where
  T: Clone,
{
  match maybe {
    MaybeRef::Ref(r) => r.clone(),
    MaybeRef::Owned(o) => o,
  }
}

/// The event the shared scanner loop ([`InputRef::scan_with`]) stops on.
enum Scan<'inp, L>
where
  L: Lexer<'inp>,
{
  /// A valid token; the caller applies its per-path policy (commit, put back,
  /// consume-and-report, …) and either stops or keeps scanning.
  Token(Spanned<L::Token, L::Span>),
  /// A limit trip: the durable frontier is already latched and the diagnostic
  /// emitted. The caller yields its poisoned outcome.
  Tripped,
  /// The input is exhausted (or the boundary was already reached). The caller
  /// yields its end-of-input outcome.
  Eof,
}

/// Where a scan latches the poison boundary on a limit trip, and how it advances
/// that frontier over each error it skips.
///
/// Two shapes cover the eight scanner paths: a scan that commits no progress
/// before its poisoned/exhausted outcome latches at the cursor ([`AtCursor`]); a
/// scan that consumes tokens as it goes latches at — and later commits — the end
/// of the last consumed token ([`AtFrontier`]).
trait Frontier<'inp, L: Lexer<'inp>> {
  /// The offset a trip latches as the durable frontier. `cursor` is the current
  /// scan position, used by scans that accumulate no progress of their own.
  fn boundary(&self, cursor: &L::Offset) -> L::Offset;

  /// Advances the frontier past a token or error the scan has skipped over.
  fn advance(&mut self, lexer: &L);
}

/// Frontier for scans that commit no progress before stopping (`next`,
/// `try_expect*`): a trip latches at the cursor and nothing accumulates, so
/// advancing is a no-op.
struct AtCursor;

impl<'inp, L: Lexer<'inp>> Frontier<'inp, L> for AtCursor {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn boundary(&self, cursor: &L::Offset) -> L::Offset {
    cursor.clone()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn advance(&mut self, _lexer: &L) {}
}

/// Frontier for scans that consume tokens as they go (`skip_while`, `sync_to`,
/// `sync_through`): a trip latches at — and the scan commits — the end of the
/// last consumed token, tracked here as its span and the lexer state that
/// produced it.
struct AtFrontier<S, St> {
  span: S,
  state: St,
}

impl<'inp, L: Lexer<'inp>> Frontier<'inp, L> for AtFrontier<L::Span, L::State>
where
  L::State: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn boundary(&self, _cursor: &L::Offset) -> L::Offset {
    self.span.end_ref().clone()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn advance(&mut self, lexer: &L) {
    self.span = lexer.span();
    self.state = lexer.state().clone();
  }
}
