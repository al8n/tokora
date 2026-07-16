//! The rewindable event sink: [`CstSink`] wraps any inner emitter, buffers the CST event
//! stream, and rewinds it under the **same** mark that rewinds diagnostics — one timeline,
//! every channel (see the [`event`](super::event) module for the vocabulary and its laws).
//!
//! # CELL_CENSUS — every mutable cell of the sink, and its class
//!
//! The input layer's cell taxonomy (see `input::lineage`) governs the sink's cells too: a
//! new cell lands here classified, or [`census`] fails to compile. The classes:
//!
//! | # | Cell | Class | Rewind/restore semantics |
//! |---|------|-------|--------------------------|
//! | E1 | [`CstSink::events`] | **ground truth** (a second emission log) | append + suffix-truncate to the mark — the same two verbs as `Verbose`'s log (plus the one censused prefix-preserving splice of the hole wrap, entirely above every live mark) |
//! | E2 | [`CstSink::journal`] | **undo journal** (the Verbose-parallel-maps discipline lifted to events) | rewind pops entries written above the mark, reverse order, restoring each overwritten `forward_parent`; never grows on rewind |
//! | E3 | [`CstSink::ledger`] | **monotone era source + truncation witness** | rewind APPENDS to it (a rewind *is* a truncation) and never removes; rewinding it would false-accept a stale mark |
//! | E4 | [`CstSink::rows`] | **release stack + per-checkpoint depth ledger** | push at `checkpoint()`, pop at `release()` (kept) and `rewind()` (spent); depth entries are frozen facts about prefixes, never live counters |
//! | E5 | [`CstSink::diag_index`] | **derived memo of E1's `Diag` slots** | truncated with E1; never checkpointed separately |
//! | — | [`CstSink::floor`] | derived memo (the newest released row) | reset to the surviving top row when a rewind drops below it |
//! | — | [`CstSink::base_inner`] | derived memo (the inner's reading at first use) | captured once, never restored |
//! | — | `inner`, `mapper`, `error_kind`, `gap_kind`, `trivia` | configuration / the wrapped emitter | never touched by rewind (the inner rewinds through its own contract) |
//! | — | `witness` | debug witness (sink identity) | never restored |
//!
//! Open-node **depth is derived, never cached**: there is no live depth counter anywhere —
//! [`checkpoint`](Emitter::checkpoint) snapshots a frozen per-row depth, and every query
//! recounts the suffix above the nearest frozen fact. A cached counter would need its own
//! restore rule; a derived one is restored by truncation for free.

use core::{cell::RefCell, marker::PhantomData, num::NonZeroU32};

use std::vec::Vec;

use crate::{
  Lexer,
  emitter::{
    CstEmitter, Emitter, FullContainerEmitter, MissingLeadingSeparatorEmitter,
    MissingTrailingSeparatorEmitter, PrattEmitter, SeparatedEmitter, TooFewEmitter, TooManyEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  error::{
    UnexpectedEoLhs, UnexpectedEoRhs,
    syntax::{FullContainer, MissingSyntaxOf, TooFew, TooMany},
    token::{MissingTokenOf, UnexpectedTokenOf},
  },
  input::Cursor,
  span::{Span, Spanned},
  token::Token,
  utils::CowStr,
};

use super::event::{Event, EventMark, TOMBSTONE, TruncationLedger};

/// How the sink places trivia tokens at materialization.
///
/// The default — and, in this version, only — policy is the provable one:
/// **innermost-open-node-at-commit** (call-site placement). A committed trivia token
/// materializes into whichever node was open when it settled, which is deterministic (a
/// function of the event prefix), cache-transparent (the scanner is origin-blind), and
/// exactly what capturing padded atoms already encode. This is deliberately **not** the
/// Roslyn/Swift leading-attaches-forward policy; a token-attached view is a later
/// materialization-time extension, which is why the enum exists at all.
#[derive(
  Debug, Default, Clone, Copy, PartialEq, Eq, Hash, derive_more::IsVariant, derive_more::Display,
)]
#[display("{}", self.as_str())]
#[non_exhaustive]
pub enum TriviaPolicy {
  /// Trivia tokens land exactly where they were emitted: inside the innermost node open
  /// at their commit position.
  #[default]
  AsEmitted,
}

impl TriviaPolicy {
  /// The canonical name of this policy.
  #[inline(always)]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::AsEmitted => "as_emitted",
    }
  }
}

/// One row of the sink's mark stack: an emitter checkpoint capture, with the derived
/// open-node depth frozen at capture time. Rows are spent by exactly one of
/// [`release`](Emitter::release) (the branch was kept) or [`rewind`](Emitter::rewind) (the
/// branch was abandoned) — the settle discipline the input layer's RELEASE_CENSUS locks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MarkRow {
  /// The captured mark: the event-log length at capture time.
  mark: u64,
  /// The derived open-node depth at capture time — a frozen fact about the `mark`-length
  /// prefix, not a live counter.
  depth: i64,
}

impl MarkRow {
  /// The empty-buffer baseline: depth 0 at length 0.
  const ZERO: Self = Self { mark: 0, depth: 0 };
}

/// One undo-journal entry: an in-place `forward_parent` write performed by
/// [`cst_start_at`](CstEmitter::cst_start_at) on its target tombstone, recorded so a rewind
/// can reverse it. In-place event mutation is otherwise banned by law; this one acceleration
/// field is the single exception, and it is legal *only because* every write is journaled.
#[derive(Debug, Clone, Copy)]
struct JournalEntry {
  /// The event-log length immediately after the append that carried this write (the
  /// `StartAt`'s own index + 1). A rewind to any mark below it must reverse the write.
  at_len: u64,
  /// The absolute index of the mutated tombstone.
  index: u64,
  /// The `forward_parent` value the write overwrote.
  old_forward_parent: Option<NonZeroU32>,
}

/// One entry of the derived `Diag`-slot index: the buffer position of a
/// [`Event::Diag`] slot and the inner mark it recorded, kept beside the buffer so rewind
/// recovery is a stack read instead of a backward scan.
#[derive(Debug, Clone, Copy)]
struct DiagSlot {
  /// The buffer index of the `Diag` event.
  pos: u64,
  /// The inner emitter's mark immediately after the forwarded emission.
  inner_mark_after: u64,
}

/// Mints a process-unique sink witness id (1-based; 0 is the inert mark's reserved id).
#[cfg(all(
  debug_assertions,
  any(feature = "std", feature = "alloc"),
  target_has_atomic = "ptr"
))]
fn next_sink_witness() -> usize {
  use core::sync::atomic::{AtomicUsize, Ordering};
  static NEXT: AtomicUsize = AtomicUsize::new(1);
  NEXT.fetch_add(1, Ordering::Relaxed)
}

/// The recording CST emitter: wraps an inner emitter `E`, forwards every diagnostic to it,
/// and buffers the event stream — one rewindable timeline for tree and diagnostics alike.
///
/// # One mark, every channel
///
/// [`checkpoint`](Emitter::checkpoint) is the event-log length: one positional mark over one
/// unified log, exactly `Verbose`'s architecture. Every diagnostic forwarded to the inner
/// emitter occupies a [`Diag`](super::event) slot *inside* the event buffer (appended by one
/// census-marked helper, on `Ok` and `Err` alike), so [`rewind`](Emitter::rewind) is:
/// truncate the buffer to the mark, reverse-replay the undo journal, and rewind the inner
/// emitter to the reading of the last surviving slot. No per-checkpoint side table exists;
/// the mark stack holds exactly the live captures because every capture is spent by exactly
/// one of `rewind` (abandoned) or [`release`](Emitter::release) (kept).
///
/// # Composition
///
/// The sink forwards the **entire** emitter trait family — core [`Emitter`], the atomic
/// capability traits ([`TooFewEmitter`], [`TooManyEmitter`], [`FullContainerEmitter`],
/// [`SeparatedEmitter`] and its four leading/trailing refinements), and [`PrattEmitter`] —
/// so any context bound satisfied by `E` is satisfied by `CstSink<E>`. It exposes the inner
/// emitter by shared reference only ([`inner_ref`](Self::inner_ref)); there is **no** `&mut`
/// accessor, because a caller who could rewind the inner emitter directly would shear the
/// event log from the diagnostic log with no witness. Materialization
/// (`finish` / `finish_partial`, landing with materialization) consumes the sink
/// and returns the inner emitter with the tree.
///
/// # Construction
///
/// [`new`](Self::new) takes the wrapped emitter, the dialect's token mapper
/// (`fn(&L::Token) -> u16` into the dialect's unified kind space — no kind bound leaks into
/// core), the `error_kind` used to wrap recovery holes, and the `gap_kind` used to tile
/// uncovered source bytes at materialization (what makes `tree.text() == source` structural
/// for every input, lexer errors included).
pub struct CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
{
  /// The wrapped emitter every diagnostic forwards to.
  inner: E,
  /// E1 — the event buffer: the second emission log (ground truth).
  events: Vec<Event<L::Span>>,
  /// E2 — the undo journal for the `forward_parent` acceleration writes.
  journal: Vec<JournalEntry>,
  /// E4 — the mark stack: one row per live checkpoint capture, holding the frozen depth.
  /// Interior mutability because [`Emitter::checkpoint`] is `&self` by contract; every
  /// borrow is method-local and non-reentrant, and the `&mut` paths use `get_mut` (no
  /// runtime flag traffic).
  rows: RefCell<Vec<MarkRow>>,
  /// The newest *released* row: a frozen `(mark, depth)` fact that keeps depth derivation
  /// O(events-since-last-settle) instead of O(buffer) across commit-heavy loops.
  floor: MarkRow,
  /// E5 — the derived index of the buffer's `Diag` slots.
  diag_index: Vec<DiagSlot>,
  /// E3 — the monotone era source and truncation witness backing mark validation.
  ledger: TruncationLedger,
  /// The inner emitter's mark as of this sink's first operation — the rewind recovery
  /// value when no `Diag` slot survives below the mark. Captured lazily (the inner is
  /// unreachable from outside once moved in, so first-use equals construction).
  base_inner: Option<u64>,
  /// The dialect's token mapper into the unified u16 kind space.
  mapper: fn(&L::Token) -> u16,
  /// The node kind that wraps a recovery hole's skipped tokens.
  error_kind: u16,
  /// The token kind that tiles source bytes no committed token covers.
  gap_kind: u16,
  /// The materialization-time trivia placement policy.
  trivia: TriviaPolicy,
  /// The sink's debug identity, stamped into every mark it mints.
  #[cfg(all(
    debug_assertions,
    any(feature = "std", feature = "alloc"),
    target_has_atomic = "ptr"
  ))]
  witness: usize,
  _lexer: PhantomData<&'inp L>,
}

impl<'inp, L, E> core::fmt::Debug for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: core::fmt::Debug,
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("CstSink")
      .field("inner", &self.inner)
      .field("events", &self.events.len())
      .field("live_marks", &self.rows.borrow().len())
      .field("error_kind", &self.error_kind)
      .field("gap_kind", &self.gap_kind)
      .field("trivia", &self.trivia)
      .finish_non_exhaustive()
  }
}

/// CELL_CENSUS — the structural tripwire for the sink's cells, in the exact shape of the
/// input layer's guardian (`input::lineage::census`): it destructures [`CstSink`]
/// exhaustively — no `..` — so adding a field fails to compile *here*, at the table that
/// asks which class the new cell is in and what a rewind must do to it. Generic and never
/// instantiated: type-checked in every build, monomorphized in none.
#[allow(dead_code)]
pub(crate) fn census<'inp, L, E>(sink: &CstSink<'inp, L, E>)
where
  L: Lexer<'inp>,
{
  let CstSink {
    // — the wrapped emitter: rewinds through its own contract, driven only by this sink.
    inner: _,
    // — E1, ground truth: append + suffix-truncate (+ the censused hole-wrap splice).
    events: _,
    // — E2, undo journal: rewind reverse-replays and truncates by `at_len`.
    journal: _,
    // — E4, release stack + depth ledger: push at checkpoint, pop at release/rewind.
    rows: _,
    // — derived memo: the newest released row; reset when a rewind drops below it.
    floor: _,
    // — E5, derived memo of E1's Diag slots: truncated with E1.
    diag_index: _,
    // — E3, monotone era source + truncation witness: NEVER rewound.
    ledger: _,
    // — derived memo: captured once at first use, never restored.
    base_inner: _,
    // — configuration: fixed for the sink's life.
    mapper: _,
    error_kind: _,
    gap_kind: _,
    trivia: _,
    // — witness: sink identity, never restored.
    #[cfg(all(
      debug_assertions,
      any(feature = "std", feature = "alloc"),
      target_has_atomic = "ptr"
    ))]
      witness: _,
    _lexer: _,
  } = sink;
}

impl<'inp, L, E> CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
{
  /// Creates a recording sink around `inner`.
  ///
  /// - `mapper` maps each committed token into the dialect's unified u16 kind space (the
  ///   [`TOMBSTONE`] value is reserved; emission debug-asserts it, materialization rejects
  ///   it);
  /// - `error_kind` is the node kind wrapped around a recovery hole's skipped tokens;
  /// - `gap_kind` is the token kind tiled over source bytes no committed token covers at
  ///   materialization, making `tree.text() == source` structural for every input.
  #[inline]
  pub fn new(inner: E, mapper: fn(&L::Token) -> u16, error_kind: u16, gap_kind: u16) -> Self {
    Self {
      inner,
      events: Vec::new(),
      journal: Vec::new(),
      rows: RefCell::new(Vec::new()),
      floor: MarkRow::ZERO,
      diag_index: Vec::new(),
      ledger: TruncationLedger::new(),
      base_inner: None,
      mapper,
      error_kind,
      gap_kind,
      trivia: TriviaPolicy::AsEmitted,
      #[cfg(all(
        debug_assertions,
        any(feature = "std", feature = "alloc"),
        target_has_atomic = "ptr"
      ))]
      witness: next_sink_witness(),
      _lexer: PhantomData,
    }
  }

  /// Sets the materialization-time trivia policy (builder form).
  #[inline(always)]
  #[must_use]
  pub fn with_trivia_policy(mut self, policy: TriviaPolicy) -> Self {
    self.trivia = policy;
    self
  }

  /// The wrapped emitter, by shared reference.
  ///
  /// Deliberately no `&mut` counterpart: a caller who could drive the inner emitter's
  /// `rewind` directly would shear the event log from the diagnostic log with no witness.
  /// The mutable path to the inner emitter is the sink's own trait surface; ownership
  /// comes back from `finish` / `finish_partial` (the materialization half).
  #[inline(always)]
  pub const fn inner_ref(&self) -> &E {
    &self.inner
  }

  /// The configured recovery-hole node kind.
  #[inline(always)]
  pub const fn error_kind(&self) -> u16 {
    self.error_kind
  }

  /// The configured gap-tile token kind.
  #[inline(always)]
  pub const fn gap_kind(&self) -> u16 {
    self.gap_kind
  }

  /// The configured trivia policy.
  #[inline(always)]
  pub const fn trivia_policy(&self) -> TriviaPolicy {
    self.trivia
  }

  /// Derives the open-node depth of the current buffer: the nearest frozen `(mark, depth)`
  /// fact (the newest of the released floor and the innermost live row) plus the summed
  /// deltas of the events above it. Depth is **never** cached live — this recount is the
  /// restore rule (truncation restores it for free).
  fn derived_depth(&self) -> i64 {
    let top = self.rows.borrow().last().copied();
    let base = match top {
      Some(row) if row.mark >= self.floor.mark => row,
      _ => self.floor,
    };
    let from = (base.mark as usize).min(self.events.len());
    base.depth
      + self.events[from..]
        .iter()
        .map(Event::depth_delta)
        .sum::<i64>()
  }

  /// Validates a mark before a spend — the panic-in-every-build wall.
  ///
  /// A mark is live iff its index is in bounds, the slot still holds a tombstone, no
  /// truncation younger than the mark's era reached its index, and (debug) it was minted
  /// by this sink. Anything else is a parser bug: the branch that conceived the wrap was
  /// rolled back, and silently wrapping whatever regrew at that index is the wrong-tree
  /// class nothing downstream can detect.
  fn validate_mark(&self, mark: &EventMark) {
    #[cfg(all(
      debug_assertions,
      any(feature = "std", feature = "alloc"),
      target_has_atomic = "ptr"
    ))]
    {
      assert!(
        mark.sink() == self.witness,
        "EventMark was minted by a different sink (or by a no-event emitter's defaulted \
         cst_mark): marks are only spendable on the sink that issued them"
      );
    }
    let index = mark.index();
    let in_bounds = (index as usize) < self.events.len();
    let is_tombstone = in_bounds && self.events[index as usize].is_tombstone();
    let is_current = !self.ledger.is_stale(mark.era(), index);
    assert!(
      in_bounds && is_tombstone && is_current,
      "stale EventMark: the tombstone this mark named no longer exists on the live \
       timeline (a rewind truncated it{}). The wrap intent died with the branch that \
       conceived it; spending the mark anyway would wrap an unrelated region.",
      if in_bounds {
        " and the buffer regrew over its index"
      } else {
        ""
      },
    );
  }

  /// The hole wrap: brackets the already-buffered token events of a recovery hole in a
  /// `Start(error_kind) … Finish` pair at the recovery site.
  ///
  /// The hole's tokens are the buffer's suffix by construction (they settled during the
  /// scan, after every live mark was captured, and the scanner runs no user code), so the
  /// wrap is a prefix-preserving splice: one insert at the first hole token, one appended
  /// finish. Interleaved `Diag` slots (lexer errors crossed while skipping) ride inside
  /// the wrap unchanged — they are invisible to materialization. If no buffered token
  /// event lies inside the hole span (no auto-emission configured, or a direct call),
  /// there is nothing to wrap and no node is made.
  fn wrap_hole(&mut self, span: &L::Span) {
    let mut wrap_start: Option<usize> = None;
    for (idx, ev) in self.events.iter().enumerate().rev() {
      match ev {
        Event::Diag { .. } => continue,
        Event::Token { span: s, .. }
          if s.start_ref() >= span.start_ref() && s.end_ref() <= span.end_ref() =>
        {
          wrap_start = Some(idx);
        }
        _ => break,
      }
    }
    let Some(at) = wrap_start else {
      return;
    };

    // The splice preserves every prefix a live mark can name: the first hole token
    // postdates every live capture (scan discipline), so nothing below `at` moves.
    debug_assert!(
      self
        .rows
        .borrow()
        .last()
        .is_none_or(|row| row.mark <= at as u64),
      "hole wrap would splice below a live checkpoint mark; the scan discipline \
       guarantees the hole's tokens postdate every live capture"
    );

    self.events.insert(
      at,
      Event::StartNode {
        kind: self.error_kind,
        forward_parent: None,
      },
    );
    self.events.push(Event::FinishNode);

    // Keep the derived memos exact across the splice: positions at or above the insert
    // point shift by one. (Journal entries cannot reference the spliced region — marks
    // and their tombstones all predate the scan — but the bump is exact anyway.)
    for slot in &mut self.diag_index {
      if slot.pos >= at as u64 {
        slot.pos += 1;
      }
    }
    for entry in &mut self.journal {
      if entry.at_len > at as u64 {
        entry.at_len += 1;
      }
      if entry.index >= at as u64 {
        entry.index += 1;
      }
    }
  }
}

impl<'inp, L, E> CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
{
  /// The inner emitter's reading as of this sink's first operation. Lazy on purpose: the
  /// inner is unreachable from outside once moved into the sink, so the first sink
  /// operation observes exactly the construction-time state — and capturing here keeps
  /// the constructor free of emitter bounds.
  fn base_inner_mark<Lang>(&mut self) -> u64
  where
    Lang: ?Sized,
    E: Emitter<'inp, L, Lang>,
  {
    match self.base_inner {
      Some(mark) => mark,
      None => {
        let mark = <E as Emitter<'inp, L, Lang>>::checkpoint(&self.inner);
        self.base_inner = Some(mark);
        mark
      }
    }
  }

  /// CST_FORWARD_CENSUS — the ONE helper every forwarded diagnostic routes through: call
  /// the inner emitter, then append a `Diag` slot carrying the inner's mark **regardless
  /// of the verdict** (record-then-propagate: transaction guards rewind during fatal
  /// unwinds, so a slot skipped on the `Err` edge would skew every later recovery).
  ///
  /// Every `emit_*` of every implemented emitter trait calls this; none touches
  /// `self.inner` directly. The source census test locks the discipline.
  fn forward_diag<Lang, R>(&mut self, forward: impl FnOnce(&mut E) -> R) -> R
  where
    Lang: ?Sized,
    E: Emitter<'inp, L, Lang>,
  {
    // The base must predate the first forwarded emission.
    let _ = self.base_inner_mark::<Lang>();
    let out = forward(&mut self.inner);
    let inner_mark_after = <E as Emitter<'inp, L, Lang>>::checkpoint(&self.inner);
    let pos = self.events.len() as u64;
    self.events.push(Event::Diag { inner_mark_after });
    self.diag_index.push(DiagSlot {
      pos,
      inner_mark_after,
    });
    out
  }
}

impl<'inp, L, E, Lang> Emitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  type Error = E::Error;

  #[inline]
  fn emit_lexer_error(
    &mut self,
    err: Spanned<<L::Token as Token<'inp>>::Error, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_lexer_error(err))
  }

  #[inline]
  fn emit_unexpected_token(
    &mut self,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_unexpected_token(err))
  }

  #[inline]
  fn emit_error(&mut self, err: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_error(err))
  }

  #[inline]
  fn emit_warning(&mut self, warning: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_warning(warning))
  }

  /// Wraps the hole's already-buffered token events in an `error_kind` node at the
  /// recovery site (empty holes and token-less holes produce no node), then forwards the
  /// one-per-hole diagnostic to the inner emitter through the census helper.
  fn emit_skipped_region(&mut self, span: L::Span, skipped: usize) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    if skipped > 0 {
      self.wrap_hole(&span);
    }
    self.forward_diag::<Lang, _>(|inner| inner.emit_skipped_region(span, skipped))
  }

  /// One positional mark over one unified log: the event-buffer length. The capture also
  /// pushes a mark-stack row freezing the derived depth, so
  /// [`cst_finish`](CstEmitter::cst_finish) can assert against the innermost live capture
  /// and [`release`](Emitter::release) has a row to reclaim.
  fn checkpoint(&self) -> u64 {
    let mark = self.events.len() as u64;
    let depth = self.derived_depth();
    self.rows.borrow_mut().push(MarkRow { mark, depth });
    mark
  }

  /// Truncate + reverse-replay + inner rewind: drop the events above the mark, undo the
  /// journaled `forward_parent` writes whose `StartAt`s died, record the truncation in
  /// the era ledger (marks into the dropped region are stale forever), and rewind the
  /// inner emitter to the reading of the last surviving `Diag` slot (the sink's base
  /// reading when none survives). Out-of-range marks clamp (the `Verbose` posture).
  fn rewind(&mut self, cursor: &Cursor<'inp, '_, L>, checkpoint: u64)
  where
    L: Lexer<'inp>,
  {
    let base = self.base_inner_mark::<Lang>();
    let len = self.events.len() as u64;
    let mark = checkpoint.min(len);

    // Spend the captures at or above the mark: everything strictly above dies with the
    // branch; the newest capture at exactly the mark is the one being rewound to.
    {
      let rows = self.rows.get_mut();
      while rows.last().is_some_and(|row| row.mark > mark) {
        rows.pop();
      }
      if rows.last().map(|row| row.mark) == Some(mark) {
        rows.pop();
      }
      if self.floor.mark > mark {
        self.floor = rows.last().copied().unwrap_or(MarkRow::ZERO);
      }
    }

    if mark < len {
      self.events.truncate(mark as usize);
      // Reverse-replay the undo journal: every forward_parent write carried by a
      // truncated StartAt is reversed, newest first, restoring the overwritten value —
      // the Verbose parallel-maps pop discipline lifted to events. A write whose target
      // slot was itself truncated has nothing left to restore; its entry just pops.
      while self.journal.last().is_some_and(|entry| entry.at_len > mark) {
        let entry = self.journal.pop().expect("guarded by the loop condition");
        if let Some(Event::StartNode { forward_parent, .. }) =
          self.events.get_mut(entry.index as usize)
        {
          *forward_parent = entry.old_forward_parent;
        }
      }
      while self.diag_index.last().is_some_and(|slot| slot.pos >= mark) {
        self.diag_index.pop();
      }
      self.ledger.record_truncation(mark);
    }

    let inner_mark = match self.diag_index.last() {
      Some(slot) => slot.inner_mark_after,
      None => base,
    };
    self.inner.rewind(cursor, inner_mark);
  }

  /// Pops the kept capture's row off the mark stack — the eviction dual of
  /// [`checkpoint`](Self::checkpoint) that keeps the stack at exactly the live captures
  /// (commit-heavy loops would otherwise strand one dead row per committed guard, and a
  /// stale row is exactly the aliased-mark state the length-mark design must never
  /// consult). The popped row becomes the derived-depth floor: a frozen fact that keeps
  /// depth recounts short across commit-heavy loops. Marks arrive newest-first on the
  /// crate's paths (O(1) top pop); a mark already gone is a no-op, per the trait's
  /// advisory contract.
  fn release(&mut self, checkpoint: u64) {
    let rows = self.rows.get_mut();
    let row = if rows.last().map(|row| row.mark) == Some(checkpoint) {
      rows.pop()
    } else {
      rows
        .iter()
        .rposition(|row| row.mark == checkpoint)
        .map(|pos| rows.remove(pos))
    };
    if let Some(row) = row {
      if row.mark >= self.floor.mark {
        self.floor = row;
      }
    }
  }

  #[inline]
  fn enter_label(&mut self, label: &'static str) {
    // Labels are not emissions: the inner's live stack follows the wrapper scopes, and
    // snapshots ride the inner's own entries — no order fact belongs in the event log.
    self.inner.enter_label(label);
  }

  #[inline]
  fn exit_label(&mut self) {
    self.inner.exit_label();
  }
}

impl<'inp, L, E, Lang> CstEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  fn cst_start(&mut self, kind: u16)
  where
    L: Lexer<'inp>,
  {
    debug_assert!(
      kind != TOMBSTONE,
      "the tombstone kind (u16::MAX) is reserved; a dialect kind must never map to it"
    );
    self.events.push(Event::StartNode {
      kind,
      forward_parent: None,
    });
  }

  fn cst_token(&mut self, tok: &L::Token, span: &L::Span)
  where
    L: Lexer<'inp>,
  {
    let kind = (self.mapper)(tok);
    // Emission-time mapper validity (detect-at-cause): rowan would defer a bad kind to a
    // query-time panic arbitrarily far from the parse; materialization keeps the release
    // backstop.
    debug_assert!(
      kind != TOMBSTONE,
      "the dialect mapper produced the reserved tombstone kind (u16::MAX) for a committed \
       token"
    );
    self.events.push(Event::Token {
      kind,
      span: span.clone(),
    });
  }

  fn cst_finish(&mut self)
  where
    L: Lexer<'inp>,
  {
    // Detect-at-cause for the orphan-finish class: a finish must close a node opened
    // above the innermost LIVE capture (a finish crossing a live save boundary would be
    // truncated apart from its start by that capture's rewind). Debug-only — the raw
    // surface is sharp by contract, and materialization is the every-build wall.
    debug_assert!(
      {
        let baseline = self.rows.borrow().last().map_or(0, |row| row.depth);
        self.derived_depth() > baseline
      },
      "cst_finish with no open node above the innermost live checkpoint: the matching \
       start was rolled back (or never emitted), so this finish would close an enclosing \
       node instead"
    );
    self.events.push(Event::FinishNode);
  }

  fn cst_mark(&mut self) -> EventMark
  where
    L: Lexer<'inp>,
  {
    let index = self.events.len() as u64;
    self.events.push(Event::StartNode {
      kind: TOMBSTONE,
      forward_parent: None,
    });
    EventMark::new(
      index,
      self.ledger.era(),
      #[cfg(all(
        debug_assertions,
        any(feature = "std", feature = "alloc"),
        target_has_atomic = "ptr"
      ))]
      self.witness,
    )
  }

  fn cst_start_at(&mut self, mark: EventMark, kind: u16)
  where
    L: Lexer<'inp>,
  {
    self.validate_mark(&mark);
    debug_assert!(
      kind != TOMBSTONE,
      "the tombstone kind (u16::MAX) is reserved; a dialect kind must never map to it"
    );
    let target = mark.index();
    let new_index = self.events.len() as u64;
    self.events.push(Event::StartAt { kind, target });

    // The one journaled in-place write: point the tombstone's forward_parent at the
    // newest wrap. Materialization recovers every wrap from the StartAt events; the
    // pointer is an acceleration and an integrity canary (finish validates that a set
    // pointer names a live StartAt of this target). The journal is what keeps it honest
    // across rewinds — restoring the overwritten value is the pure-copy discipline.
    let relative = new_index - target;
    if let Ok(relative) = u32::try_from(relative) {
      if let Some(Event::StartNode { forward_parent, .. }) = self.events.get_mut(target as usize) {
        self.journal.push(JournalEntry {
          at_len: new_index + 1,
          index: target,
          old_forward_parent: *forward_parent,
        });
        *forward_parent = NonZeroU32::new(relative);
      }
    }
  }
}

// ── The forwarded capability family ─────────────────────────────────────────────
//
// Every atomic emitter trait the crate ships forwards through the ONE census helper, so
// `CstSink<E>` satisfies every context bound `E` satisfies (the `ComposableEmitter`-shaped
// bundles downstream) and every forwarded diagnostic occupies a Diag slot in the unified
// log. CST_FORWARD_CENSUS locks the set.

impl<'inp, L, E, Lang> TooFewEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: TooFewEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline]
  fn emit_too_few(&mut self, err: TooFew<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_too_few(err))
  }
}

impl<'inp, L, E, Lang> TooManyEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: TooManyEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline]
  fn emit_too_many(&mut self, err: TooMany<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_too_many(err))
  }
}

impl<'inp, L, E, Lang> FullContainerEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: FullContainerEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline]
  fn emit_full_container(&mut self, err: FullContainer<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_full_container(err))
  }
}

impl<'inp, L, E, Lang> SeparatedEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: SeparatedEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline]
  fn emit_missing_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_missing_separator(name, err))
  }

  #[inline]
  fn emit_missing_element(&mut self, err: MissingSyntaxOf<'inp, L, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_missing_element(err))
  }
}

impl<'inp, L, E, Lang> MissingLeadingSeparatorEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: MissingLeadingSeparatorEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline]
  fn emit_missing_leading_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_missing_leading_separator(name, err))
  }
}

impl<'inp, L, E, Lang> MissingTrailingSeparatorEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: MissingTrailingSeparatorEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline]
  fn emit_missing_trailing_separator(
    &mut self,
    name: CowStr,
    err: MissingTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_missing_trailing_separator(name, err))
  }
}

impl<'inp, L, E, Lang> UnexpectedLeadingSeparatorEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: UnexpectedLeadingSeparatorEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline]
  fn emit_unexpected_leading_separator(
    &mut self,
    name: CowStr,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_unexpected_leading_separator(name, err))
  }
}

impl<'inp, L, E, Lang> UnexpectedTrailingSeparatorEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: UnexpectedTrailingSeparatorEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline]
  fn emit_unexpected_trailing_separator(
    &mut self,
    name: CowStr,
    err: UnexpectedTokenOf<'inp, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_unexpected_trailing_separator(name, err))
  }
}

impl<'inp, L, E, Lang> PrattEmitter<'inp, L, Lang> for CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
  E: PrattEmitter<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[inline]
  fn emit_unexpected_end_of_lhs(
    &mut self,
    err: UnexpectedEoLhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_unexpected_end_of_lhs(err))
  }

  #[inline]
  fn emit_unexpected_end_of_rhs(
    &mut self,
    err: UnexpectedEoRhs<L::Offset, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self.forward_diag::<Lang, _>(|inner| inner.emit_unexpected_end_of_rhs(err))
  }
}

// ── Test observability ──────────────────────────────────────────────────────────

#[cfg(test)]
impl<'inp, L, E> CstSink<'inp, L, E>
where
  L: Lexer<'inp>,
{
  /// The event-buffer view, for shape assertions.
  pub(crate) fn events(&self) -> &[Event<L::Span>] {
    &self.events
  }

  /// The number of live mark-stack rows (the release no-growth oracle).
  pub(crate) fn rows_len(&self) -> usize {
    self.rows.borrow().len()
  }

  /// The number of live undo-journal entries.
  pub(crate) fn journal_len(&self) -> usize {
    self.journal.len()
  }

  /// The tombstone's forward_parent pointer at `index`, if that slot is a start.
  pub(crate) fn forward_parent_at(&self, index: usize) -> Option<NonZeroU32> {
    match self.events.get(index) {
      Some(Event::StartNode { forward_parent, .. }) => *forward_parent,
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests;
