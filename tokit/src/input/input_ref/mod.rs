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

use super::{Cache, Checkpoint, Cursor, Lexed, Lexer, Source, Span};

mod consume_cached;
mod drop_policy;
mod fold;
mod peek;
mod pratt;
mod skip_while;
#[cfg(any(feature = "std", feature = "alloc"))]
mod stacked;
mod sync_through;
mod sync_to;
mod transaction;
mod try_expect;

pub use drop_policy::{Commit, DropPolicy, Rollback};
pub use transaction::Transaction;

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use stacked::{SavepointId, StackedTransaction};

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests;

/// A reference to an `Input` instance.
pub struct InputRef<'inp, 'closure, L, Ctx, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  pub(super) input: &'closure &'inp L::Source,
  pub(super) state: &'closure mut L::State,
  pub(super) span: &'closure mut L::Span,
  pub(super) cache: &'closure mut Ctx::Cache,
  pub(super) emitted_error_end: &'closure mut L::Offset,
  pub(super) poison_boundary: &'closure mut Option<L::Offset>,
  /// The cache's monotone push count (see [`Input::cache_pushes`](super::Input)), bumped by
  /// [`cache_push_back`](InputRef::cache_push_back) and read by [`save`](InputRef::save) /
  /// [`restore`](InputRef::restore) to drop entries pushed on an abandoned continuation.
  pub(super) cache_pushes: &'closure mut u64,
  /// The input-global savepoint sequence counter (see
  /// [`Input::savepoint_seq`](super::Input)). Handed out monotonically by
  /// [`next_savepoint_seq`](InputRef::next_savepoint_seq) and never reset across the
  /// input's stacked transactions, so a [`SavepointId`]'s `seq` is unique for the whole
  /// life of the input.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub(super) savepoint_seq: &'closure mut u64,
  /// The input's live-checkpoint lineage stack (see [`Input::live_ckpts`](super::Input)),
  /// maintained by [`save`](InputRef::save) / [`restore`](InputRef::restore) in every
  /// allocator build and read by [`StackedTransaction`] to reject a stale savepoint.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub(super) live_ckpts: &'closure mut super::LineageStack,
  /// Monotone id source for [`live_ckpts`](Self::live_ckpts) (see
  /// [`Input::next_ckp_id`](super::Input)).
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub(super) next_ckp_id: &'closure mut u64,
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

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
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

  /// Pushes a lexed token onto the back of the cache, bumping the monotone push count on
  /// success. Every cache push flows through here (the peek fill and the `try_expect`
  /// put-backs), so the count tracks exactly the tokens the cache accepted: a full cache
  /// hands the token back and leaves the count unchanged, and a blackhole cache — which
  /// accepts no push — keeps its count at 0. [`save`](Self::save) records the count and
  /// [`restore`](Self::restore) uses the difference to drop entries pushed after a save.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn cache_push_back(&mut self, tok: CachedTokenOf<'inp, L>) -> Result<(), CachedTokenOf<'inp, L>> {
    match self.cache.push_back(tok) {
      Ok(_) => {
        *self.cache_pushes += 1;
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

  /// Returns a mutable reference to the current lexer state (extras).
  ///
  /// # Checkpoint invalidation
  ///
  /// Replacing the lexer state re-keys every offset-dependent fact the input tracks
  /// (cache spans, dedup watermark, poison boundary). All outstanding checkpoints are
  /// invalidated: the live-checkpoint lineage stack is cleared, so a later
  /// [`restore`](Self::restore) of an outstanding checkpoint no-ops the lineage pop and a
  /// [`StackedTransaction`] savepoint taken before this call panics as stale (every build).
  /// Restoring a checkpoint afterwards is a contract violation (debug builds also panic in
  /// `restore` itself).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn state_mut(&mut self) -> &mut L::State {
    #[cfg(any(feature = "std", feature = "alloc"))]
    self.live_ckpts.clear();
    self.state
  }

  /// Manually sets the lexer state (for context-sensitive lexing).
  ///
  /// # Checkpoint invalidation
  ///
  /// Replacing the lexer state re-keys every offset-dependent fact the input tracks
  /// (cache spans, dedup watermark, poison boundary). All outstanding checkpoints are
  /// invalidated: the live-checkpoint lineage stack is cleared, so a later
  /// [`restore`](Self::restore) of an outstanding checkpoint no-ops the lineage pop and a
  /// [`StackedTransaction`] savepoint taken before this call panics as stale (every build).
  /// Restoring a checkpoint afterwards is a contract violation (debug builds also panic in
  /// `restore` itself).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_state(&mut self, state: L::State) {
    #[cfg(any(feature = "std", feature = "alloc"))]
    self.live_ckpts.clear();
    *self.state = state;
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

impl<'inp, 'closure, L, Ctx, Lang: ?Sized> InputRef<'inp, 'closure, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
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
  /// For fallible closures that carry an error value, see
  /// [`try_attempt`](Self::try_attempt).
  pub fn attempt<F, R>(&mut self, f: F) -> Option<R>
  where
    F: FnOnce(&mut Self) -> Option<R>,
  {
    let ckp = self.save();

    match f(self) {
      Some(result) => {
        // Progress kept: the checkpoint is dropped without restoring, so drop its
        // lineage id too rather than leaving it to grow the live stack.
        #[cfg(any(feature = "std", feature = "alloc"))]
        self.forget_checkpoint(ckp.ckp_id);
        Some(result)
      }
      None => {
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
  pub fn try_attempt<F, T, E>(&mut self, f: F) -> Result<T, E>
  where
    F: FnOnce(&mut Self) -> Result<T, E>,
  {
    let ckp = self.save();

    match f(self) {
      Ok(result) => {
        // Progress kept: drop the checkpoint's lineage id (see `attempt`).
        #[cfg(any(feature = "std", feature = "alloc"))]
        self.forget_checkpoint(ckp.ckp_id);
        Ok(result)
      }
      Err(e) => {
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
  /// speculation; raw `save`/`restore` only where no guard shape fits.
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn begin(&mut self) -> Transaction<'_, 'inp, 'closure, L, Ctx, Lang, Rollback> {
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
  pub fn begin_with<D: DropPolicy>(&mut self) -> Transaction<'_, 'inp, 'closure, L, Ctx, Lang, D> {
    let ckp = self.save();
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
  /// [`StackedTransaction`] for which combinations invalidate a savepoint (a raw restore
  /// below it, or replacing the lexer state — both panic as stale in every build) and which
  /// are always legal (nested speculation, and a LIFO-clean raw pair above the savepoints).
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
  ///   is kept on most exits (raw [`save`](Self::save) / [`restore`](Self::restore) only
  ///   where no guard shape fits).
  ///
  /// Dropping an undecided guard rolls back to the begin point; for a stacked guard that
  /// instead keeps its progress on drop, use
  /// [`begin_stacked_with::<Commit>`](Self::begin_stacked_with).
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub fn begin_stacked(
    &mut self,
  ) -> StackedTransaction<'_, 'inp, 'closure, L, Ctx, Lang, Rollback> {
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
  ) -> StackedTransaction<'_, 'inp, 'closure, L, Ctx, Lang, D> {
    // Nonce = the address of this Input's `poison_boundary` field, an Input-owned slot the
    // `InputRef` holds a `&mut` to. Two simultaneously-live Inputs are distinct structs at
    // distinct addresses (the field is never zero-sized), so their nonces differ and a
    // cross-parser id is caught at runtime; the `'txn` brand on `SavepointId` — not this
    // address — rules out the address-reuse case where a dropped Input's slot is later
    // reallocated. NOT the source pointer: two Inputs can share one `&str`.
    let nonce = core::ptr::from_ref(&*self.poison_boundary).addr();
    let base = self.save();
    StackedTransaction {
      input: self,
      base: Some(base),
      saves: Default::default(),
      nonce,
      _policy: PhantomData,
    }
  }

  /// Hands out the next input-global savepoint sequence number, bumping the counter.
  ///
  /// The counter lives on the [`Input`](super::Input), not on any one transaction, so it
  /// is monotone across every stacked transaction of this input and never reset. That
  /// makes a [`SavepointId`]'s `seq` unique for the whole life of the input: an id that
  /// crosses transactions (a nested or a sequential one) can never collide with a live
  /// savepoint's `seq` in another transaction's stack, so the membership scan in
  /// [`rollback_to`](StackedTransaction::rollback_to) / [`release`](StackedTransaction::release)
  /// panics deterministically wherever the lifetime brand does not already reject the id.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub(super) fn next_savepoint_seq(&mut self) -> u64 {
    let seq = *self.savepoint_seq;
    *self.savepoint_seq += 1;
    seq
  }

  /// Drops `id` from the live-checkpoint lineage stack because its checkpoint was kept
  /// (committed) rather than restored — see [`Transaction::commit`],
  /// [`attempt`](Self::attempt), [`try_attempt`](Self::try_attempt), and the
  /// [`StackedTransaction`] release/commit paths.
  ///
  /// A restored checkpoint is popped off the stack by [`restore`](Self::restore); a
  /// *committed* one never reaches `restore`, so without this its id would linger and grow
  /// the stack across commit-heavy loops. Removing it keeps the lineage stack exact and
  /// bounded. `O(1)` when `id` is the stack top (the common case for a committed
  /// checkpoint); a linear removal otherwise (e.g. a raw checkpoint saved above it was
  /// dropped without restoring). Removing a non-top id keeps the rest of the stack in
  /// order, so an older restore still pops cleanly through it.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub(crate) fn forget_checkpoint(&mut self, id: u64) {
    if self.live_ckpts.last() == Some(&id) {
      self.live_ckpts.pop();
    } else if let Some(pos) = self.live_ckpts.iter().position(|&x| x == id) {
      self.live_ckpts.remove(pos);
    }
  }

  /// Returns whether `id` is still live on the lineage stack. Backs both the
  /// [`StackedTransaction`] savepoint-staleness check (every build) and, in debug + ptr
  /// builds, [`restore`](Self::restore)'s non-LIFO panic.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  pub(super) fn live_contains(&self, id: u64) -> bool {
    self.live_ckpts.contains(&id)
  }

  /// Pops the lineage stack down through `id` inclusive, invalidating it and every
  /// checkpoint saved after it. A no-op if `id` is already gone — a raw restore to a
  /// checkpoint an earlier restore or a state replacement already invalidated (release's
  /// unspecified-but-bounded posture; debug + ptr asserts presence in `restore` first).
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(not(tarpaulin), inline)]
  fn live_pop_through(&mut self, id: u64) {
    if let Some(pos) = self.live_ckpts.iter().position(|&x| x == id) {
      self.live_ckpts.truncate(pos);
    }
  }

  /// The number of live checkpoints — test-only observability for the no-growth
  /// guarantee that committing (and a success-path [`Recover`](crate::parser::Recover))
  /// gives the lineage stack.
  ///
  /// The stack it measures ([`live_ckpts`](super::Input::live_ckpts)) is maintained in every
  /// allocator build, so this accessor is gated only to its callers — the `logos` + `std`
  /// guard and recover test suites — and *not* to `debug_assertions` or
  /// `target_has_atomic = "ptr"`, so the no-growth cases can run under the release profile
  /// too. Keeping the `logos` + `std` constraint (rather than the looser `any(std, alloc)`)
  /// keeps the method from being dead code under `cargo hack --each-feature --tests`, whose
  /// single-feature combinations never enable both `logos` and `std` and so compile neither
  /// this method nor its callers.
  #[cfg(all(test, feature = "logos", feature = "std"))]
  pub(crate) fn live_checkpoints_len(&self) -> usize {
    self.live_ckpts.len()
  }

  /// Returns a slice of the current token from the input source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice(&self) -> <L::Source as Source<L::Offset>>::Slice<'inp> {
    self
      .input
      .slice(self.span.start_ref()..self.span.end_ref())
      .expect("lexer should guarantee slice")
  }

  /// Returns a slice of the input source from the given cursor to the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice_since(
    &self,
    cursor: &Cursor<'inp, 'closure, L>,
  ) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    let end = self.cursor();
    self.input.slice(cursor.as_inner()..end.as_inner())
  }

  /// Returns a slice of the input source from the given cursor to the end of the input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice_from(
    &self,
    cursor: &Cursor<'inp, 'closure, L>,
  ) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    let start = cursor.as_inner();
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
    let start = range.start_bound().map(|c| c.as_inner());
    let end = range.end_bound().map(|c| c.as_inner());
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
  /// Prefer [`attempt`](Self::attempt)/[`try_attempt`](Self::try_attempt) when the
  /// save/restore pair brackets a single speculative computation — they enforce the
  /// restore discipline by construction.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn save(&mut self) -> Checkpoint<'inp, 'closure, L> {
    // Record this checkpoint on the live-checkpoint lineage stack (every allocator build)
    // and stamp its fresh id into the checkpoint. `restore` pops the stack down through
    // that id, and a `StackedTransaction` checks the id is still present before honoring a
    // savepoint — the check that makes stale savepoints panic on release and no-ptr targets.
    #[cfg(any(feature = "std", feature = "alloc"))]
    let ckp_id = {
      let id = *self.next_ckp_id;
      *self.next_ckp_id += 1;
      self.live_ckpts.push(id);
      id
    };
    Checkpoint::new(
      self.cursor().clone(),
      self.span.clone(),
      self.state.clone(),
      self.emitter.checkpoint(),
      self.emitted_error_end.clone(),
      self.poison_boundary.clone(),
      *self.cache_pushes,
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
  /// After a restore, the input behaves exactly as it did the moment the checkpoint
  /// was taken:
  ///
  /// - the cursor, last-consumed span, and lexer state are restored; consuming
  ///   resumes from the saved position. Cached tokens appended after the save belong to
  ///   the abandoned continuation and are dropped so their region re-lexes (re-emitting
  ///   any lexer error it held); tokens cached before the save re-lex identically;
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
  /// // Nested speculation — inner restored (or dropped) before outer:
  /// let outer = input.save();
  /// let inner = input.save();
  /// if !try_variant_a(input) { input.restore(inner); }   // youngest first
  /// if !try_variant_b(input) { input.restore(outer); }   // then the older one
  ///
  /// // Retry loop — a fresh checkpoint per iteration:
  /// loop {
  ///   let ckp = input.save();
  ///   match try_parse(input) {
  ///     Ok(v) => break v,
  ///     Err(_) => input.restore(ckp),                    // always the youngest live one
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
  #[doc(alias = "rewinds")]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn restore(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
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
    self.restore_unchecked(checkpoint);
  }

  /// Rewinds to `checkpoint` without the debug raw-misuse panics, used by the transaction
  /// guards' `Drop`, whose base restore is internally managed — always the oldest live
  /// checkpoint — and must stay silent: `Drop` may run while already unwinding, and `no_std`
  /// has no `thread::panicking()` to guard a drop-bomb, so a debug assert firing here would
  /// abort. It still maintains the lineage stack (popping through the restored id if present)
  /// and replays the saved lineage exactly, identically to [`restore`](Self::restore) in
  /// release. (An explicit [`rollback`](Transaction::rollback) restores through the checked
  /// [`restore`](Self::restore), since it never runs during an unwind.)
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn restore_unchecked(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    // Maintain the lineage stack in every allocator build: pop it down through the restored
    // id (invalidating it and every younger checkpoint). An absent id is a no-op — a raw
    // restore to a checkpoint an earlier restore or a state replacement already invalidated
    // (release's unspecified-but-bounded posture; `restore` asserts presence in debug + ptr).
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
    let post_save = self.cache_pushes.saturating_sub(checkpoint.cache_pushes);
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
    *self.cache_pushes = checkpoint.cache_pushes;
    *self.emitted_error_end = checkpoint.emitted_error_end;
    *self.poison_boundary = checkpoint.poison_boundary;
    self.set_span((&checkpoint.span).into());
    *self.state = checkpoint.state;
  }

  /// Advances the cursor and returns the next valid token, emitting errors encountered on the way.
  ///
  /// Skips over lexer errors, emitting them through the provided emitter.
  /// Non-fatal errors are emitted and the method continues to the next token.
  #[allow(clippy::should_implement_trait)]
  pub fn next(
    &mut self,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
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
  #[cfg_attr(not(tarpaulin), inline)]
  fn scan_with<Fr>(
    &mut self,
    lexer: &mut L,
    lex_at: &mut L::Offset,
    frontier: &mut Fr,
  ) -> Result<Scan<'inp, L>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Fr: Frontier<'inp, L>,
  {
    while let Some(Spanned { span, data: tok }) = self.lex_within_boundary(lexer, lex_at) {
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
/// `try_expect*`, `sync_through`): a trip latches at the cursor and nothing
/// accumulates, so advancing is a no-op.
struct AtCursor;

impl<'inp, L: Lexer<'inp>> Frontier<'inp, L> for AtCursor {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn boundary(&self, cursor: &L::Offset) -> L::Offset {
    cursor.clone()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn advance(&mut self, _lexer: &L) {}
}

/// Frontier for scans that consume tokens as they go (`skip_while`, `sync_to`):
/// a trip latches at — and the scan commits — the end of the last consumed
/// token, tracked here as its span and the lexer state that produced it.
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
