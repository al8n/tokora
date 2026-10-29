//! The write-chokepointed storage behind [`Verbose`](super::Verbose).
//!
//! # Why the storage is a child module
//!
//! Every one of `Verbose`'s emit channels must record on the shared emission log, because
//! that log is what [`checkpoint`](super::Emitter::checkpoint),
//! [`rewind`](super::Emitter::rewind) and [`diagnostics()`](super::Verbose::diagnostics) are
//! defined in terms of. A channel that appends to a payload map directly leaves the log
//! unmoved, and all three of those guarantees quietly stop holding for it — a defect that is
//! invisible at the write site and only shows up as a phantom diagnostic, an unmoved mark, or
//! a payload replayed against the wrong entry.
//!
//! Rust's own privacy rule makes that write unspellable rather than merely discouraged: a
//! private field is visible in its defining module and that module's *descendants*, so the
//! fields below are invisible to `store`'s siblings — the eleven channel files — and to its
//! parent. The only way in from a channel file is [`Store::record`] and its two mirrors, and
//! reaching for the raw map there is `error[E0616]`, not a review comment.

use std::{
  collections::{BTreeMap, btree_map::Entry},
  vec::Vec,
};

use crate::emitter::Severity;

use super::{Channel, Diagnostics};

/// What one [`pop_group`] step carries **out** of a channel rather than destroying in place: the
/// element the pop took off the group, and the whole map entry — key included — when the pop
/// emptied it.
///
/// Both halves are caller-supplied values, and the only reason to hand them back is that
/// destroying them is caller code. See [`pop_group`].
type Detached<S, T> = (Option<T>, Option<(S, Vec<T>)>);

/// Pops the newest entry of an **already-resolved** channel group and hands back everything it
/// detached — the shared per-channel step of [`rewind_to`](Store::rewind_to)'s newest-first
/// unwind.
///
/// Takes a resolved [`Entry`] rather than a map and a key, and that is half the point: every
/// `S: Ord` comparison this step needs was already spent by the descent that produced the
/// handle, and [`Vec::pop`] and
/// [`OccupiedEntry::remove_entry`](std::collections::btree_map::OccupiedEntry::remove_entry)
/// both work off that handle without comparing again. Reaching for the map by key here instead
/// (`get_mut` then `remove`, as this did) puts a second descent *between* two halves of one
/// rollback, which is precisely the tear [`rewind_to`](Store::rewind_to) documents itself as not
/// having.
///
/// The other half is that **nothing is dropped here.** A generic parameter is the caller's type
/// in every position it appears, and destruction is one of those positions: discarding what
/// `Vec::pop` returns runs `Error::drop`, and `remove()` — as opposed to
/// [`remove_entry`](std::collections::btree_map::OccupiedEntry::remove_entry) — runs `S::drop` on
/// the key it throws away. Both are caller code executing *on the spot*, between one map's
/// mutation and its twin's, so a rollback that drops in place is not panic-atomic however
/// carefully its descents are ordered: an unwinding destructor leaves the payload group short
/// while the log still names the slot, or the payload key gone while the label map still has it,
/// and [`Diagnostics`] indexes by exactly those. Moving the values out instead makes the whole
/// commit run *no* caller code at all — no comparison, no clone, no destructor — and leaves the
/// caller to release them once the log has been popped and the channel is consistent again.
fn pop_group<S, T>(entry: Entry<'_, S, Vec<T>>, slot: usize) -> Detached<S, T>
where
  S: Ord,
{
  debug_assert_eq!(
    match &entry {
      Entry::Occupied(group) => Some(group.get().len()),
      Entry::Vacant(_) => None,
    },
    Some(slot + 1),
    "the newest-first unwind must meet the emit-time slot, in lockstep across a payload group \
     and its label snapshots"
  );
  match entry {
    Entry::Occupied(mut group) => {
      let popped = group.get_mut().pop();
      let removed = group.get().is_empty().then(|| group.remove_entry());
      (popped, removed)
    }
    // Unreachable above the assert, and reachable below it: `debug_assert` fails open, so a
    // release build that somehow logged a span this map does not have arrives here. There is no
    // group to pop, but the vacant handle owns the key it was built from, and letting *that*
    // fall out of scope would run `S::drop` inside the interval — the one destructor this arm
    // can reach. It leaves with everything else.
    Entry::Vacant(vacant) => (None, Some((vacant.into_key(), Vec::new()))),
  }
}

/// Unwinds the newest log entry off one channel's two parallel maps and the log — **one
/// complete panic-atomic interval**, from the first descent to the last release.
///
/// The shape is the same one [`record`](Store::record) uses, extended to cover destruction: take
/// every handle first, because a descent is where `S: Ord` and `S: Clone` run; then commit
/// through those handles, which compare nothing; then, with the maps and the log agreeing again,
/// let go of the values the commit detached, because releasing them is where `Error::drop` and
/// `S::drop` run.
///
/// **Why the release is inside this function rather than after the caller's loop.** The interval
/// is per unwound entry, and it has to stay that way. Collecting the detached values across a
/// whole suffix would fuse every entry into one interval: a panic on entry *k+1*'s descent would
/// unwind through entry *k*'s still-held values, running their destructors during an unwind that
/// is already in flight — a double panic, which aborts, and it would need an allocation to do
/// it. Releasing per entry means a destructor that unwinds strands nothing: the entries already
/// unwound are cleanly unwound, the one that panicked is fully unwound too, and a retried
/// `rewind_to` finishes the rest.
///
/// What it does **not** do is keep the panic inside. A destructor that unwinds here escapes
/// [`rewind_to`](Store::rewind_to) exactly as a panicking comparison does, and the only way to
/// stop that would be to hand the values out to `rewind_to`'s caller — a signature change, and
/// the caller has the same problem one frame up. See
/// [`Verbose::rewind`](super::Emitter::rewind) for what that costs mid-unwind, which this
/// change does not move: caller destructors could already escape this path, they merely
/// corrupted the channel on the way out.
fn unwind_one<S, T>(
  groups: &mut BTreeMap<S, Vec<T>>,
  labels: &mut BTreeMap<S, Vec<Vec<&'static str>>>,
  log: &mut Vec<LogEntry<S>>,
) where
  S: Ord + Clone,
{
  let newest = log
    .last()
    .expect("`rewind_to` only unwinds while the log is longer than the mark");
  let slot = newest.slot;
  // ── every fallible step, with nothing yet unwound ──
  let group_entry = groups.entry(newest.span.clone());
  let label_entry = labels.entry(newest.span.clone());
  // ── commit point: nothing below compares, clones or destroys ──
  let detached_group = pop_group(group_entry, slot);
  let detached_labels = pop_group(label_entry, slot);
  let popped_log = log.pop();
  // ── interval closed: both maps and the log name the same emissions again ──
  drop(detached_group);
  drop(detached_labels);
  drop(popped_log);
}

/// One emission-log entry: the channel tag, the span key, and the entry's slot within its
/// span's group in that channel at emit time.
///
/// The slot is what lets [`Diagnostics`] replay by direct index instead of rebuilding a
/// per-span cursor tree on every walk, and what [`rewind_to`](Store::rewind_to) debug-asserts
/// against: the log is popped newest-first, so a popped entry's slot always names its group's
/// current tail.
#[derive(Debug, Clone)]
pub(super) struct LogEntry<S> {
  pub(super) channel: Channel,
  pub(super) span: S,
  pub(super) slot: usize,
}

/// [`Verbose`](super::Verbose)'s collected diagnostics and the emission log that orders them.
///
/// Every field is private to this module by design (see the module docs): the write surface is
/// [`record`](Self::record) / [`record_warning`](Self::record_warning) /
/// [`record_hole`](Self::record_hole) and nothing else.
#[derive(Debug)]
pub(super) struct Store<Error, S> {
  errs: BTreeMap<S, Vec<Error>>,
  /// Parallel to `errs`: the open-label snapshot captured when each error was
  /// recorded, kept in lockstep with the error groups (same span keys, same
  /// per-span `Vec` lengths). `label_snapshots[span][i]` is the *"while parsing X"*
  /// context stack that was open when `errs[span][i]` was emitted. A separate map
  /// (rather than pairing the label into the error) keeps
  /// [`errors()`](super::Verbose::errors) returning exactly `&BTreeMap<S, Vec<Error>>`.
  ///
  /// That choice is paid for rather than free, and the price is an ordering discipline: two
  /// maps agree key-for-key only if every write and every rollback takes **both** descents
  /// before it touches either one. [`record`](Self::record) and
  /// [`rewind_to`](Self::rewind_to) are the only two places that can, and both do — a third
  /// that grew a group and then went looking for its twin would publish a span key through
  /// one accessor that the other does not have.
  label_snapshots: BTreeMap<S, Vec<Vec<&'static str>>>,
  /// The warning channel: mirrors `errs` in shape but is fed by
  /// [`emit_warning`](super::Emitter::emit_warning) rather than the error emit paths. Kept
  /// separate so [`errors()`](super::Verbose::errors) and
  /// [`warnings()`](super::Verbose::warnings) each return a clean `&BTreeMap<S, Vec<Error>>`
  /// for their own [`Severity`] tier.
  warns: BTreeMap<S, Vec<Error>>,
  /// Parallel to `warns`, exactly as `label_snapshots` is parallel to `errs`.
  warn_label_snapshots: BTreeMap<S, Vec<Vec<&'static str>>>,
  /// The skipped-region channel: one entry per recovery hole recorded by
  /// [`emit_skipped_region`](super::Emitter::emit_skipped_region), keyed by the hole span,
  /// each entry the skipped-token count. Payload-less (no `Error` value), which is why it is
  /// its own map rather than a third `Severity` tier.
  holes: BTreeMap<S, Vec<usize>>,
  /// Parallel to `holes`, exactly as `label_snapshots` is parallel to `errs`.
  hole_label_snapshots: BTreeMap<S, Vec<Vec<&'static str>>>,
  /// The `(channel, span)` of every emission, in emission order — the single ordering
  /// authority across *all* channels. An entry's index in this log is its monotonic sequence
  /// number; [`mark`](Self::mark) is the log length and [`rewind_to`](Self::rewind_to) unwinds
  /// the tail back to a mark, popping the matching record — and its label snapshot — off the
  /// channel named by the entry's [`Channel`] tag. This is what lets rewind drop a speculative
  /// zero-width diagnostic while keeping an earlier one at the same span, and what lets
  /// [`diagnostics()`](Self::diagnostics) reconstruct the true interleaving of the payload
  /// channels — a distinction span-ordered storage alone cannot make.
  log: Vec<LogEntry<S>>,
  /// The currently-open label stack, pushed by
  /// [`enter_label`](super::Emitter::enter_label) and popped by
  /// [`exit_label`](super::Emitter::exit_label). Snapshotted (cloned) into the recording
  /// channel at each emit; a push/pop never allocates and an empty snapshot clones for free.
  stack: Vec<&'static str>,
}

impl<Error, S> Default for Store<Error, S> {
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

impl<Error, S> Clone for Store<Error, S>
where
  Error: Clone,
  S: Clone,
{
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      errs: self.errs.clone(),
      label_snapshots: self.label_snapshots.clone(),
      warns: self.warns.clone(),
      warn_label_snapshots: self.warn_label_snapshots.clone(),
      holes: self.holes.clone(),
      hole_label_snapshots: self.hole_label_snapshots.clone(),
      log: self.log.clone(),
      stack: self.stack.clone(),
    }
  }
}

impl<Error, S> Store<Error, S> {
  /// An empty store: no diagnostics, no log, no open labels.
  #[inline(always)]
  pub(super) const fn new() -> Self {
    Self {
      errs: BTreeMap::new(),
      label_snapshots: BTreeMap::new(),
      warns: BTreeMap::new(),
      warn_label_snapshots: BTreeMap::new(),
      holes: BTreeMap::new(),
      hole_label_snapshots: BTreeMap::new(),
      log: Vec::new(),
      stack: Vec::new(),
    }
  }

  /// Records `err` in the **error** channel at `span`, appending it to the span's group and
  /// logging the emission (tagged [`Severity::Error`]) so a later
  /// [`rewind_to`](Self::rewind_to) can undo it precisely.
  ///
  /// A snapshot of the currently-open label stack is captured alongside the error,
  /// into `label_snapshots` at the same span/index — this is the *capture-at-emit*
  /// point for diagnostic labels. Cloning an empty stack does not allocate, so an
  /// unlabelled emission pays nothing beyond the parallel bookkeeping.
  ///
  /// The debug assert is the parallel-map tripwire: the payload group and its label
  /// snapshots are only ever grown here, one element each, so a span where they have drifted
  /// apart means a write that went round this chokepoint — which is exactly the class of
  /// defect that voids the mark, the unwind and the replay for the channel that did it.
  ///
  /// # A record is panic-atomic
  ///
  /// Every step that can run **caller code** runs before the first observable write, so a panic
  /// anywhere in this body leaves no payload, no label entry and no log entry:
  /// [`errors`](Self::errors), [`labels`](Self::labels), replay, [`mark`](Self::mark) and
  /// [`rewind_to`](Self::rewind_to) all behave as if the call never happened. The caller code a
  /// record can run is exactly two `S: Clone`s, the `S: Ord` comparisons of the two map descents,
  /// and — when the span is one the channel already holds — the two `S::drop`s that
  /// [`entry`](BTreeMap::entry) runs on the key it was handed and did not need to store. All of
  /// it is above the commit point. The destructors belong in that list even though this path
  /// happens to keep them above the line: a generic parameter is the caller's type wherever it
  /// appears, and a census that lists only comparison and cloning is the census that let the
  /// *rollback* destroy an `Error` mid-commit while reading as complete.
  ///
  /// Below the commit point the body **moves** every caller value it still holds — the payload
  /// into the group, the snapshot into the labels, the last span clone into the log entry — so
  /// there is nothing left down there to destroy.
  ///
  /// The load-bearing move is that a [`BTreeMap`] **descent and its insertion are separate
  /// operations**, and only the descent runs caller code.
  /// [`entry`](BTreeMap::entry) searches and hands back a handle without touching the map — a
  /// [`Vacant`](Entry::Vacant) handle that is *dropped* rather than filled inserts nothing — and
  /// [`or_default`](Entry::or_default) then fills it with no further comparison. So both
  /// descents are taken first, and only once both have succeeded does either map grow. Calling
  /// `entry(..).or_default()` on the label map *before* descending the payload map — as this did
  /// — interleaves them the other way, and a panicking `S: Ord` on the second descent leaves an
  /// empty group under the new span key in `label_snapshots` that
  /// [`Verbose::labels`](super::Verbose::labels) publishes and no
  /// [`rewind_to`](Self::rewind_to) can reach, because no log entry names it. Reversing the two
  /// maps only moves that orphan into [`errors()`](super::Verbose::errors); taking both handles
  /// first is what removes it.
  ///
  /// One ordering below the commit point is still deliberate: **the log last.** The log entry is
  /// appended after both maps, so it can never name a slot that was never filled — a discipline
  /// that costs nothing here and is what keeps replay, which indexes by the logged slot, total.
  ///
  /// Allocation is the one thing that remains below the commit point, and it cannot tear a
  /// record. Allocation *failure* aborts rather than unwinding, and the reserves are ordered so
  /// that the one case that does unwind — a `Vec::reserve` capacity overflow at
  /// `len ≈ isize::MAX`, in a process already out of address space — happens after **both**
  /// groups exist, leaving the two maps holding a matching pair of empty groups rather than one
  /// orphan.
  #[inline(always)]
  pub(super) fn record(&mut self, span: S, err: Error)
  where
    S: Ord + Clone,
  {
    // ── every fallible step, with nothing yet written ──
    let snapshot = self.stack.clone();
    let label_key = span.clone();
    let log_key = span.clone();
    self.log.reserve(1);
    let label_entry = self.label_snapshots.entry(label_key);
    let group_entry = self.errs.entry(span);
    // ── commit point: nothing below compares or clones ──
    let labels = label_entry.or_default();
    let group = group_entry.or_default();
    labels.reserve(1);
    group.reserve(1);
    let slot = group.len();
    debug_assert_eq!(
      labels.len(),
      slot,
      "the error group and its label snapshots must be parallel before a record: a span \
       where they have drifted apart was written outside this chokepoint"
    );
    group.push(err);
    labels.push(snapshot);
    self.log.push(LogEntry {
      channel: Channel::Diagnostic(Severity::Error),
      span: log_key,
      slot,
    });
  }

  /// Records `warning` in the **warning** channel at `span` — the exact mirror of
  /// [`record`](Self::record), but into `warns`/`warn_label_snapshots` and logging the
  /// emission tagged [`Severity::Warning`]. The shared `log` keeps both channels on one
  /// emission timeline, so a [`rewind_to`](Self::rewind_to) unwinds warnings and errors
  /// together in reverse emission order.
  #[inline(always)]
  pub(super) fn record_warning(&mut self, span: S, warning: Error)
  where
    S: Ord + Clone,
  {
    // Panic-atomic by the same recipe as [`record`](Self::record): both descents first, then a
    // commit that neither compares nor clones.
    let snapshot = self.stack.clone();
    let label_key = span.clone();
    let log_key = span.clone();
    self.log.reserve(1);
    let label_entry = self.warn_label_snapshots.entry(label_key);
    let group_entry = self.warns.entry(span);
    // ── commit point ──
    let labels = label_entry.or_default();
    let group = group_entry.or_default();
    labels.reserve(1);
    group.reserve(1);
    let slot = group.len();
    debug_assert_eq!(
      labels.len(),
      slot,
      "the warning group and its label snapshots must be parallel before a record: a span \
       where they have drifted apart was written outside this chokepoint"
    );
    group.push(warning);
    labels.push(snapshot);
    self.log.push(LogEntry {
      channel: Channel::Diagnostic(Severity::Warning),
      span: log_key,
      slot,
    });
  }

  /// Records a recovery hole in the **skipped-region** channel at `span` — the same shape as
  /// [`record_warning`](Self::record_warning), but the payload is the skipped-token count and
  /// the log entry is tagged [`Channel::SkippedRegion`]. The shared `log` keeps all record
  /// kinds on one emission timeline, so a [`rewind_to`](Self::rewind_to) unwinds hole records
  /// together with diagnostics in reverse emission order.
  #[inline(always)]
  pub(super) fn record_hole(&mut self, span: S, skipped: usize)
  where
    S: Ord + Clone,
  {
    // Panic-atomic by the same recipe as [`record`](Self::record); the payload is a `usize`,
    // so this body has one fewer fallible input than its two siblings.
    let snapshot = self.stack.clone();
    let label_key = span.clone();
    let log_key = span.clone();
    self.log.reserve(1);
    let label_entry = self.hole_label_snapshots.entry(label_key);
    let group_entry = self.holes.entry(span);
    // ── commit point ──
    let labels = label_entry.or_default();
    let group = group_entry.or_default();
    labels.reserve(1);
    group.reserve(1);
    let slot = group.len();
    debug_assert_eq!(
      labels.len(),
      slot,
      "the hole group and its label snapshots must be parallel before a record: a span \
       where they have drifted apart was written outside this chokepoint"
    );
    group.push(skipped);
    labels.push(snapshot);
    self.log.push(LogEntry {
      channel: Channel::SkippedRegion,
      span: log_key,
      slot,
    });
  }

  /// The emission mark: the log's length, an opaque monotone counter that advances by exactly
  /// one per recorded emission on every channel.
  #[inline(always)]
  pub(super) fn mark(&self) -> u64 {
    self.log.len() as u64
  }

  /// Unwinds every emission recorded after `mark`, newest first.
  ///
  /// Each span's group grows in emission order, so the matching entry to drop is always its
  /// last one. The [`Channel`] tag names the maps the entry was recorded in, so the pop lands
  /// in the right channel. The dropped entry takes its label snapshot with it — labels
  /// captured into an entry are rewound together with it, and any later re-emission re-derives
  /// labels from the then-current stack. A mark beyond the current length is clamped: `Verbose`
  /// keeps no per-mark bookkeeping, so a stale mark can only name a suffix that is already gone.
  ///
  /// # One unwound entry is panic-atomic
  ///
  /// Each turn of the loop is one [`unwind_one`] interval, and the interval runs **no caller
  /// code at all**. Both channel handles are taken before anything moves, exactly as
  /// [`record`](Self::record) takes both descents before it inserts anything, because a
  /// [`BTreeMap`] descent is where the caller's `S: Ord` and `S: Clone` run. Everything after
  /// that — popping each group, taking out the entry when a group empties, popping the log —
  /// compares nothing, clones nothing, and **destroys nothing**: the detached payload, the
  /// removed keys and the popped [`LogEntry`] are carried out of the interval and released only
  /// once the maps and the log name the same emissions again (see [`pop_group`]). So a panicking
  /// comparison leaves the entry logged, both of its groups intact and [`mark`](Self::mark) where
  /// it was, and a panicking destructor leaves the entry cleanly and completely unwound.
  ///
  /// Three orderings are wrong here and it is worth naming all three, because the second is the
  /// obvious repair for the first and the third hid behind both. Popping the log *first* shears
  /// the mark away from maps that were never touched. Popping it *last* around map work that is
  /// still fallible is worse — it leaves the log naming a slot the payload group no longer has,
  /// and [`Diagnostics`] indexes by that slot. And committing through handles that *drop* what
  /// they detach reintroduces the same tear through the one kind of caller code a census of
  /// comparisons and clones does not list: `Error::drop` between the payload pop and the label
  /// pop, `S::drop` between one map's removal and its twin's.
  ///
  /// Two prices are worth naming. The span key is **cloned** per handle, so `S: Clone` joins the
  /// bound and `S::Clone` joins the caller code this path can run; in exchange the whole rollback
  /// spends exactly two descents per unwound entry rather than the two-to-four a
  /// `get_mut`-then-`remove` pair spends, and drops the two debug-only lookups entirely. And
  /// caller code on this path is *unavoidable* — comparison, clone and destructor alike — which
  /// matters because [`Emitter::rewind`](super::Emitter::rewind) may run from a guard's `Drop`
  /// while a panic is already unwinding, where any panic is a second panic and aborts. Atomicity
  /// is what this method can offer there; totality it cannot, and
  /// [`Verbose::rewind`](super::Emitter::rewind)'s own documentation says so rather than leaving
  /// the trait's "structurally non-panicking" reading to cover types it does not.
  #[inline(always)]
  pub(super) fn rewind_to(&mut self, mark: u64)
  where
    S: Ord + Clone,
  {
    let mark = (mark as usize).min(self.log.len());
    while self.log.len() > mark {
      match self
        .log
        .last()
        .expect("log length exceeds the mark")
        .channel
      {
        Channel::Diagnostic(Severity::Error) => {
          unwind_one(&mut self.errs, &mut self.label_snapshots, &mut self.log);
        }
        Channel::Diagnostic(Severity::Warning) => {
          unwind_one(
            &mut self.warns,
            &mut self.warn_label_snapshots,
            &mut self.log,
          );
        }
        Channel::SkippedRegion => {
          unwind_one(
            &mut self.holes,
            &mut self.hole_label_snapshots,
            &mut self.log,
          );
        }
      }
    }
  }

  /// Pushes a *"while parsing X"* label onto the open-label stack; the next recorded
  /// diagnostic snapshots it into the entry it emits.
  #[inline(always)]
  pub(super) fn enter_label(&mut self, label: &'static str) {
    self.stack.push(label);
  }

  /// Pops the innermost open label as its [`labelled`](crate::labelled) scope closes.
  #[inline(always)]
  pub(super) fn exit_label(&mut self) {
    self.stack.pop();
  }

  /// The error channel's span-keyed groups.
  #[inline(always)]
  pub(super) const fn errors(&self) -> &BTreeMap<S, Vec<Error>> {
    &self.errs
  }

  /// The error channel's per-diagnostic label snapshots.
  #[inline(always)]
  pub(super) const fn labels(&self) -> &BTreeMap<S, Vec<Vec<&'static str>>> {
    &self.label_snapshots
  }

  /// The warning channel's span-keyed groups.
  #[inline(always)]
  pub(super) const fn warnings(&self) -> &BTreeMap<S, Vec<Error>> {
    &self.warns
  }

  /// The warning channel's per-diagnostic label snapshots.
  #[inline(always)]
  pub(super) const fn warning_labels(&self) -> &BTreeMap<S, Vec<Vec<&'static str>>> {
    &self.warn_label_snapshots
  }

  /// The skipped-region channel's span-keyed skipped-token counts.
  #[inline(always)]
  pub(super) const fn skipped_regions(&self) -> &BTreeMap<S, Vec<usize>> {
    &self.holes
  }

  /// The skipped-region channel's per-hole label snapshots.
  #[inline(always)]
  pub(super) const fn skipped_region_labels(&self) -> &BTreeMap<S, Vec<Vec<&'static str>>> {
    &self.hole_label_snapshots
  }

  /// Replays every recorded diagnostic — errors, warnings and holes — in emission order.
  #[inline(always)]
  pub(super) fn diagnostics(&self) -> Diagnostics<'_, S, Error> {
    Diagnostics::new(
      &self.log,
      &self.errs,
      &self.label_snapshots,
      &self.warns,
      &self.warn_label_snapshots,
      &self.holes,
      &self.hole_label_snapshots,
    )
  }
}

/// The write paths are **panic-atomic**: a caught panic from caller code leaves the channel
/// exactly as it was, on both the record side and the rollback side.
///
/// The matrix is a sweep rather than a sample. A cell names a channel, a write path, and one of
/// the four kinds of caller code the store can run (`S: Clone`, `S: Ord`, `S::drop`,
/// `Error::drop`); it *measures* how many calls of that kind the path makes, then arms each of
/// them in turn and checks the whole observable surface — both maps, the mark, and the
/// [`Diagnostics`] replay — against what it was before. Sweeping matters because these defects
/// live at a *particular* call: the record orphan needed the last comparison of the second map
/// descent, and the rewind shear needed the comparison after the log had already moved. A matrix
/// that arms one hand-picked index measures the index, not the property.
///
/// The two destructor axes are here because **a generic parameter is the caller's type in every
/// position it appears**, and the axis list is the census that decides what a "commit" is
/// allowed to contain. A census taken over comparisons and clones alone called the rollback's
/// second half infallible and let it destroy an `Error` between one map's pop and its twin's;
/// the axes are enumerated by *what the code does to a caller value* — compare it, copy it,
/// destroy it — rather than by which of those has already gone wrong. Cells where an axis has
/// nothing to arm are pinned at zero by `the_cells_with_no_caller_destructor_have_none` rather
/// than omitted, so "this path destroys nothing" stays a checked claim.
///
/// The rollback cells assert two things at each armed call. The state after the caught unwind is
/// exactly the state a clean rewind to the mark it actually reached would have produced — so the
/// entries already unwound stay unwound and the one that panicked is untouched, with nothing
/// half-done in between. And a retried rewind still lands on exactly the clean result, because a
/// rollback that cannot be resumed is not a rollback.
#[cfg(all(test, feature = "std"))]
mod tests;
