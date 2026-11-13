use super::Store;
use crate::emitter::DiagnosticKind;
use core::cell::Cell;
use std::{
  format,
  panic::{AssertUnwindSafe, catch_unwind},
  string::String,
  vec::Vec,
};

thread_local! {
  /// `Clone` calls seen since the last arm, and the call index that panics (0 = disarmed).
  static CLONES: Cell<usize> = const { Cell::new(0) };
  static CLONE_BOMB: Cell<usize> = const { Cell::new(0) };
  /// `Ord::cmp` calls seen since the last arm, and the call index that panics.
  static CMPS: Cell<usize> = const { Cell::new(0) };
  static CMP_BOMB: Cell<usize> = const { Cell::new(0) };
  /// `S::drop` calls seen since the last arm, and the call index that panics.
  static SPAN_DROPS: Cell<usize> = const { Cell::new(0) };
  static SPAN_DROP_BOMB: Cell<usize> = const { Cell::new(0) };
  /// `Error::drop` calls seen since the last arm, and the call index that panics.
  static PAYLOAD_DROPS: Cell<usize> = const { Cell::new(0) };
  static PAYLOAD_DROP_BOMB: Cell<usize> = const { Cell::new(0) };
  /// `PAYLOAD_DROPS` sampled at the instant a bomb fires, so a sweep can tell a destructor
  /// that ran *before* the panic from one the unwind dragged along behind it.
  static PAYLOAD_DROPS_AT_PANIC: Cell<usize> = const { Cell::new(0) };
}

fn arm_clone(at: usize) {
  CLONES.with(|c| c.set(0));
  CLONE_BOMB.with(|c| c.set(at));
}

fn arm_cmp(at: usize) {
  CMPS.with(|c| c.set(0));
  CMP_BOMB.with(|c| c.set(at));
}

fn arm_span_drop(at: usize) {
  SPAN_DROPS.with(|c| c.set(0));
  SPAN_DROP_BOMB.with(|c| c.set(at));
}

fn arm_payload_drop(at: usize) {
  PAYLOAD_DROPS.with(|c| c.set(0));
  PAYLOAD_DROP_BOMB.with(|c| c.set(at));
}

fn disarm() {
  CLONE_BOMB.with(|c| c.set(0));
  CMP_BOMB.with(|c| c.set(0));
  SPAN_DROP_BOMB.with(|c| c.set(0));
  PAYLOAD_DROP_BOMB.with(|c| c.set(0));
}

/// Counts one call of a bombed hook and panics if it is the armed one.
///
/// Exactly *one* index is ever armed, which is what keeps a destructor bomb usable at all: the
/// drops the unwind itself then runs are later indices, so they count and return, and the test
/// sees one panic rather than a double-panic abort.
///
/// Firing also samples the payload-destructor counter, which is what makes the *timing* of a
/// destructor observable and not just its effect.
fn tick(
  seen: &'static std::thread::LocalKey<Cell<usize>>,
  bomb: &'static std::thread::LocalKey<Cell<usize>>,
  what: &str,
) {
  let n = seen.with(|c| {
    let v = c.get() + 1;
    c.set(v);
    v
  });
  if n == bomb.with(Cell::get) {
    PAYLOAD_DROPS_AT_PANIC.with(|c| c.set(PAYLOAD_DROPS.with(Cell::get)));
    panic!("the armed {what} (#{n}) panics");
  }
}

/// A span whose `Clone`, `Ord` **and `Drop`** are user-code steps the store can run. `Drop` is
/// in here because a generic parameter is the caller's type in every position it appears: the
/// rollback path does not only compare and clone spans, it destroys them — when a group empties
/// and when the log entry naming it is popped.
#[derive(Debug, PartialEq, Eq)]
struct BombSpan(usize);

impl Drop for BombSpan {
  fn drop(&mut self) {
    tick(&SPAN_DROPS, &SPAN_DROP_BOMB, "span drop");
  }
}

/// The error payload, whose `Drop` is the *other* caller destructor a rollback runs: popping a
/// diagnostic off its group destroys the value the caller handed in.
///
/// Each record carries a distinct value, so the observable distinguishes *which* payload sits
/// in a slot — a rollback that moves values out rather than destroying them has to move the
/// right ones.
struct BombErr(u32);

impl core::fmt::Debug for BombErr {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "BombErr({})", self.0)
  }
}

impl Drop for BombErr {
  fn drop(&mut self) {
    tick(&PAYLOAD_DROPS, &PAYLOAD_DROP_BOMB, "payload drop");
  }
}

impl Clone for BombSpan {
  fn clone(&self) -> Self {
    tick(&CLONES, &CLONE_BOMB, "span clone");
    Self(self.0)
  }
}

impl PartialOrd for BombSpan {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for BombSpan {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    tick(&CMPS, &CMP_BOMB, "span comparison");
    self.0.cmp(&other.0)
  }
}

/// The four kinds of caller code a store call can run, as a sweepable axis: which counter to
/// reset and arm, and which one to read back when calibrating.
#[derive(Clone, Copy)]
enum Axis {
  Clone,
  Cmp,
  SpanDrop,
  PayloadDrop,
}

impl Axis {
  /// Resets this axis' counter and arms call `at`; `0` resets without arming, which is how a
  /// calibration run measures a path without tripping it.
  fn arm(self, at: usize) {
    match self {
      Axis::Clone => arm_clone(at),
      Axis::Cmp => arm_cmp(at),
      Axis::SpanDrop => arm_span_drop(at),
      Axis::PayloadDrop => arm_payload_drop(at),
    }
  }

  fn count(self) -> usize {
    match self {
      Axis::Clone => CLONES.with(Cell::get),
      Axis::Cmp => CMPS.with(Cell::get),
      Axis::SpanDrop => SPAN_DROPS.with(Cell::get),
      Axis::PayloadDrop => PAYLOAD_DROPS.with(Cell::get),
    }
  }

  const fn name(self) -> &'static str {
    match self {
      Axis::Clone => "S::Clone",
      Axis::Cmp => "S::Ord",
      Axis::SpanDrop => "S::Drop",
      Axis::PayloadDrop => "Error::Drop",
    }
  }
}

/// The three channels, each named by the accessor that must stay pristine.
#[derive(Clone, Copy)]
enum Channel {
  Error,
  Warning,
  Hole,
}

impl Channel {
  fn write(self, store: &mut Store<BombErr, BombSpan>, span: BombSpan, payload: u32) {
    match self {
      Channel::Error => store.record(span, BombErr(payload)),
      Channel::Warning => store.record_warning(span, BombErr(payload)),
      Channel::Hole => store.record_hole(span, payload as usize),
    }
  }

  /// The payload map's entry count — the observable a torn record corrupts.
  fn payload_len(self, store: &Store<BombErr, BombSpan>) -> usize {
    match self {
      Channel::Error => store.errors().len(),
      Channel::Warning => store.warnings().len(),
      Channel::Hole => store.skipped_regions().len(),
    }
  }

  const fn name(self) -> &'static str {
    match self {
      Channel::Error => "record",
      Channel::Warning => "record_warning",
      Channel::Hole => "record_hole",
    }
  }

  /// Everything a caller can see of this channel: the payload map, the **label map beside
  /// it**, the mark, the refused-error count and the ceiling, and the replay. The label map is
  /// in here deliberately — the record orphan is invisible to any check that reads only the
  /// payload side of a representation documented as parallel, which is exactly how it survived
  /// the previous matrix. The count and the ceiling are in here for the same reason: a sweep
  /// whose oracle is "nothing changed" checks only what this prints.
  fn observable(self, store: &Store<BombErr, BombSpan>) -> String {
    let (payload, labels) = match self {
      Channel::Error => (
        format!("{:?}", store.errors()),
        format!("{:?}", store.labels()),
      ),
      Channel::Warning => (
        format!("{:?}", store.warnings()),
        format!("{:?}", store.warning_labels()),
      ),
      Channel::Hole => (
        format!("{:?}", store.skipped_regions()),
        format!("{:?}", store.skipped_region_labels()),
      ),
    };
    let replay: Vec<String> = store
      .diagnostics()
      .map(|d| format!("{:?}@{:?}{:?}", d.kind(), d.span(), d.labels()))
      .collect();
    format!(
      "mark={} refused_errors={} max_diagnostics={} payload={payload} labels={labels} \
       replay={replay:?}",
      store.mark(),
      store.refused_errors(),
      store.max_diagnostics()
    )
  }
}

/// A store with one `channel` record at each span in `spans`, in order, and no ceiling.
fn seeded(channel: Channel, spans: &[usize]) -> Store<BombErr, BombSpan> {
  seeded_under(channel, spans, usize::MAX)
}

/// A store made by its own constructor with a ceiling of `max`: a `channel` record at each of
/// the first `max` spans, then an **error** at each span past them, which the full store refuses
/// and counts. So a store of any channel can carry refused errors above its kept records; with
/// `max` at `usize::MAX` this is `seeded`.
fn seeded_under(channel: Channel, spans: &[usize], max: usize) -> Store<BombErr, BombSpan> {
  let mut store = Store::<BombErr, BombSpan>::with_max_diagnostics(max);
  for (seq, &span) in spans.iter().enumerate() {
    let writer = if seq < max { channel } else { Channel::Error };
    writer.write(&mut store, BombSpan(span), seq as u32);
  }
  store
}

/// How many `axis` calls one record at `at_span` makes into a store seeded at `seed`, with
/// none of them armed. Both the sweeps and the zero-pin below start here, so "how many" is one
/// measurement rather than two that can drift apart.
fn record_calls(channel: Channel, axis: Axis, seed: &[usize], at_span: usize) -> usize {
  record_calls_under(channel, axis, seed, at_span, usize::MAX)
}

/// `record_calls` over a store seeded under a ceiling of `max` (see `seeded_under`).
fn record_calls_under(
  channel: Channel,
  axis: Axis,
  seed: &[usize],
  at_span: usize,
  max: usize,
) -> usize {
  let mut cal = seeded_under(channel, seed, max);
  axis.arm(0);
  channel.write(&mut cal, BombSpan(at_span), 99);
  let calls = axis.count();
  disarm();
  calls
}

/// How many `axis` calls `rewind_to(keep)` makes over a store seeded at `spans`, none armed.
fn rewind_calls(channel: Channel, axis: Axis, spans: &[usize], keep: usize) -> usize {
  rewind_calls_under(channel, axis, spans, keep, usize::MAX)
}

/// `rewind_calls` over a store seeded under a ceiling of `max` (see `seeded_under`).
fn rewind_calls_under(
  channel: Channel,
  axis: Axis,
  spans: &[usize],
  keep: usize,
  max: usize,
) -> usize {
  let mut cal = seeded_under(channel, spans, max);
  axis.arm(0);
  cal.rewind_to(keep as u64);
  let calls = axis.count();
  disarm();
  calls
}

/// Arms every call of `axis` that one record at `at_span` makes into a store seeded at
/// `seed`, and asserts each one leaves the channel byte-for-byte as it was.
fn record_sweep(channel: Channel, axis: Axis, seed: &[usize], at_span: usize) {
  record_sweep_under(channel, axis, seed, at_span, usize::MAX);
}

/// `record_sweep` over a store seeded under a ceiling of `max` (see `seeded_under`). With `max`
/// at most `seed.len()` the store is full, and the write under test is refused.
fn record_sweep_under(channel: Channel, axis: Axis, seed: &[usize], at_span: usize, max: usize) {
  let calls = record_calls_under(channel, axis, seed, at_span, max);
  assert!(
    calls > 0,
    "`{}` runs no {} the sweep can arm, so this cell measures nothing",
    channel.name(),
    axis.name()
  );

  for at in 1..=calls {
    let mut store = seeded_under(channel, seed, max);
    let before = channel.observable(&store);
    axis.arm(at);
    let caught = catch_unwind(AssertUnwindSafe(|| {
      channel.write(&mut store, BombSpan(at_span), 99)
    }));
    disarm();
    assert!(
      caught.is_err(),
      "{} call #{at} of {calls} in `{}` did not panic when armed",
      axis.name(),
      channel.name()
    );
    assert_eq!(
      channel.observable(&store),
      before,
      "a panicking {} at call #{at} of {calls} left `{}` half-written: the record is not \
       panic-atomic",
      axis.name(),
      channel.name()
    );
  }
}

/// Arms every call of `axis` that `rewind_to(keep)` makes over a store seeded at `spans`.
///
/// Two assertions per armed call. The state after the caught unwind must equal the state a
/// clean rewind to the mark it actually reached would produce — the per-entry rollback either
/// happened or did not, and no group is left out of step with the log. And a retried rewind
/// must still land on the clean result: a rollback whose only ordering authority has already
/// been popped cannot be resumed, which is the shape of the defect this pins.
fn rewind_sweep(channel: Channel, axis: Axis, spans: &[usize], keep: usize) {
  rewind_sweep_under(channel, axis, spans, keep, usize::MAX);
}

/// `rewind_sweep` over a store seeded under a ceiling of `max` (see `seeded_under`), its clean
/// twin seeded under the same ceiling.
fn rewind_sweep_under(channel: Channel, axis: Axis, spans: &[usize], keep: usize, max: usize) {
  let clean = |to: u64| {
    let mut twin = seeded_under(channel, spans, max);
    twin.rewind_to(to);
    channel.observable(&twin)
  };
  let target = clean(keep as u64);
  let calls = rewind_calls_under(channel, axis, spans, keep, max);
  assert!(
    calls > 0,
    "`rewind_to` over the {} channel runs no {} the sweep can arm, so this cell measures \
     nothing",
    channel.name(),
    axis.name()
  );

  for at in 1..=calls {
    let mut store = seeded_under(channel, spans, max);
    axis.arm(at);
    let caught = catch_unwind(AssertUnwindSafe(|| store.rewind_to(keep as u64)));
    disarm();
    assert!(
      caught.is_err(),
      "{} call #{at} of {calls} in `rewind_to` did not panic when armed",
      axis.name()
    );
    assert_eq!(
      channel.observable(&store),
      clean(store.mark()),
      "a panicking {} at call #{at} of {calls} sheared the {} channel: the mark moved to {} \
       but the maps do not match a rewind to it",
      axis.name(),
      channel.name(),
      store.mark()
    );
    store.rewind_to(keep as u64);
    assert_eq!(
      channel.observable(&store),
      target,
      "after a panicking {} at call #{at} of {calls}, a retried rewind no longer reaches the \
       clean result",
      axis.name()
    );
  }
}

/// The two record shapes, by what the span key does to each map descent: a **fresh** span
/// makes both descents vacant (the pair of insertions the orphan came from), an **existing**
/// one makes both occupied. The seed is four spans deep so a descent costs more than the one
/// comparison a single-key map would.
const SEED: &[usize] = &[10, 20, 30, 40];
const FRESH: usize = 25;
const EXISTING: usize = 30;

#[test]
fn record_is_atomic_under_a_panicking_span_clone() {
  record_sweep(Channel::Error, Axis::Clone, SEED, FRESH);
  record_sweep(Channel::Error, Axis::Clone, SEED, EXISTING);
}

#[test]
fn record_warning_is_atomic_under_a_panicking_span_clone() {
  record_sweep(Channel::Warning, Axis::Clone, SEED, FRESH);
  record_sweep(Channel::Warning, Axis::Clone, SEED, EXISTING);
}

#[test]
fn record_hole_is_atomic_under_a_panicking_span_clone() {
  record_sweep(Channel::Hole, Axis::Clone, SEED, FRESH);
  record_sweep(Channel::Hole, Axis::Clone, SEED, EXISTING);
}

#[test]
fn record_is_atomic_under_a_panicking_ord() {
  record_sweep(Channel::Error, Axis::Cmp, SEED, FRESH);
  record_sweep(Channel::Error, Axis::Cmp, SEED, EXISTING);
}

#[test]
fn record_warning_is_atomic_under_a_panicking_ord() {
  record_sweep(Channel::Warning, Axis::Cmp, SEED, FRESH);
  record_sweep(Channel::Warning, Axis::Cmp, SEED, EXISTING);
}

#[test]
fn record_hole_is_atomic_under_a_panicking_ord() {
  record_sweep(Channel::Hole, Axis::Cmp, SEED, FRESH);
  record_sweep(Channel::Hole, Axis::Cmp, SEED, EXISTING);
}

/// The three rollback shapes, by what the unwound entry does to its group: **`LAST_ONLY`**
/// empties it (the removal path), **`LAST_OF_TWO`** merely shortens it (the pop-only path),
/// and **`SUFFIX`** unwinds several entries so a panic can land after earlier ones have
/// already committed.
const LAST_ONLY: (&[usize], usize) = (&[10, 20, 30, 40], 3);
const LAST_OF_TWO: (&[usize], usize) = (&[10, 20, 30, 30], 3);
const SUFFIX: (&[usize], usize) = (&[10, 20, 30, 30, 40, 20], 1);

fn rewind_shapes(channel: Channel, axis: Axis) {
  for (spans, keep) in [LAST_ONLY, LAST_OF_TWO, SUFFIX] {
    rewind_sweep(channel, axis, spans, keep);
  }
}

#[test]
fn rewind_is_atomic_under_a_panicking_ord() {
  rewind_shapes(Channel::Error, Axis::Cmp);
}

#[test]
fn rewind_warning_is_atomic_under_a_panicking_ord() {
  rewind_shapes(Channel::Warning, Axis::Cmp);
}

#[test]
fn rewind_hole_is_atomic_under_a_panicking_ord() {
  rewind_shapes(Channel::Hole, Axis::Cmp);
}

/// The rollback path resolves its groups through owned keys, so `S::Clone` is caller code it
/// runs too and gets the same sweep as `S::Ord` — the axis a matrix written against the
/// comparison defect alone would have left unpinned.
#[test]
fn rewind_is_atomic_under_a_panicking_span_clone() {
  rewind_shapes(Channel::Error, Axis::Clone);
  rewind_shapes(Channel::Warning, Axis::Clone);
  rewind_shapes(Channel::Hole, Axis::Clone);
}

/// The rollback **destroys** caller values as well as comparing and cloning them: a group that
/// empties gives its span key back to be dropped, and so does the log entry that named it.
/// This is the axis a census of "`S::Clone` ×2, `S::Ord` (2 descents)" leaves out, and it is
/// the one that reached every shape and every channel — including the payload-less
/// skipped-region channel, whose maps are still keyed by `S`.
#[test]
fn rewind_is_atomic_under_a_panicking_span_drop() {
  rewind_shapes(Channel::Error, Axis::SpanDrop);
  rewind_shapes(Channel::Warning, Axis::SpanDrop);
  rewind_shapes(Channel::Hole, Axis::SpanDrop);
}

/// Unwinding a diagnostic destroys the **payload** the caller handed in, which is a second
/// caller destructor and lands between the payload group's pop and its label group's — the
/// narrowest window in the whole rollback, and enough to leave `Diagnostics` indexing a slot
/// the group no longer has.
#[test]
fn rewind_is_atomic_under_a_panicking_payload_drop() {
  rewind_shapes(Channel::Error, Axis::PayloadDrop);
  rewind_shapes(Channel::Warning, Axis::PayloadDrop);
}

/// A record over a span the channel already has destroys two span keys —
/// [`BTreeMap::entry`] drops the key it was handed whenever the descent finds one — so the
/// destructor axis reaches the write path too.
#[test]
fn record_is_atomic_under_a_panicking_span_drop() {
  record_sweep(Channel::Error, Axis::SpanDrop, SEED, EXISTING);
  record_sweep(Channel::Warning, Axis::SpanDrop, SEED, EXISTING);
  record_sweep(Channel::Hole, Axis::SpanDrop, SEED, EXISTING);
}

/// A panic escaping mid-suffix must not drag a caller destructor out with it.
///
/// Where the detached values are released is a decision, not a detail, and the state oracle
/// above cannot see it: holding every entry's payload until the whole loop finished would
/// satisfy every assertion in [`rewind_sweep`] — the maps and the log still agree at each catch
/// point — while moving all of those destructors *into* the unwind, which is the one place a
/// panic is a second panic and aborts the process instead of being catchable. Fusing the loop
/// into a single interval would also need an allocation to hold the suffix.
///
/// So the property is about *timing*, and a `SUFFIX` unwind is where it bites: by the time
/// entry *k+1*'s descent runs, entry *k*'s payload must already be destroyed. Arming each
/// comparison in turn, the payload-destructor count sampled at the instant the panic is raised
/// must already equal the count once it is caught — an unwind out of `rewind_to` destroys no
/// payload on its way. Under a fused loop the two diverge by exactly the number of entries the
/// suffix had already unwound.
#[test]
fn an_escaping_panic_destroys_no_payload_on_the_way_out() {
  let (spans, keep) = SUFFIX;
  for channel in [Channel::Error, Channel::Warning] {
    let calls = rewind_calls(channel, Axis::Cmp, spans, keep);
    assert!(
      calls > 0,
      "no comparison to arm: this cell measures nothing"
    );
    for at in 1..=calls {
      let mut store = seeded(channel, spans);
      arm_payload_drop(0);
      PAYLOAD_DROPS_AT_PANIC.with(|c| c.set(usize::MAX));
      arm_cmp(at);
      let caught = catch_unwind(AssertUnwindSafe(|| store.rewind_to(keep as u64)));
      disarm();
      assert!(
        caught.is_err(),
        "S::Ord call #{at} of {calls} in `rewind_to` did not panic when armed"
      );
      assert_eq!(
        PAYLOAD_DROPS_AT_PANIC.with(Cell::get),
        PAYLOAD_DROPS.with(Cell::get),
        "a panicking comparison at call #{at} of {calls} over the {} channel unwound through \
         a payload the rollback was still holding: entry k's detached values must be released \
         before entry k+1 begins, or the whole suffix is one interval and the destructors run \
         mid-unwind",
        channel.name()
      );
    }
  }
}

/// The cells where an axis has **nothing to arm**, pinned at zero instead of left out.
///
/// "Measures nothing" and "measures a property" must not look alike from the outside: the
/// destructor axis was missing from the previous matrix precisely because nobody had asked how
/// many destructors these paths run, and a cell quietly finding no calls is how that question
/// stays unasked. Each row below is a claim about the code, and a change that makes one of
/// them destroy something reds here rather than going unswept:
///
/// - a record **moves** every caller value it touches — the payload into the group, the span
///   into two handles and a log entry — so over a *fresh* span it destroys nothing at all;
/// - a record never destroys its payload on any shape, fresh or existing;
/// - the skipped-region channel stores a `usize` count rather than an `Error`, so no rollback
///   on it can run a payload destructor, on any of the three rollback shapes.
#[test]
fn the_cells_with_no_caller_destructor_have_none() {
  for channel in [Channel::Error, Channel::Warning, Channel::Hole] {
    assert_eq!(
      record_calls(channel, Axis::SpanDrop, SEED, FRESH),
      0,
      "`{}` over a fresh span destroys a span key: it is meant to move every one of them",
      channel.name()
    );
    for at_span in [FRESH, EXISTING] {
      assert_eq!(
        record_calls(channel, Axis::PayloadDrop, SEED, at_span),
        0,
        "`{}` destroys its payload: it is meant to move it into the group",
        channel.name()
      );
    }
  }
  for (spans, keep) in [LAST_ONLY, LAST_OF_TWO, SUFFIX] {
    assert_eq!(
      rewind_calls(Channel::Hole, Axis::PayloadDrop, spans, keep),
      0,
      "the skipped-region channel ran an `Error` destructor, but it stores no `Error`"
    );
  }
}

/// The keep-green control: with nothing armed, all three channels record normally.
#[test]
fn unarmed_records_are_unaffected() {
  for channel in [Channel::Error, Channel::Warning, Channel::Hole] {
    let mut store = Store::<BombErr, BombSpan>::new();
    channel.write(&mut store, BombSpan(1), 1);
    assert_eq!(
      channel.payload_len(&store),
      1,
      "{} records normally when nothing is armed",
      channel.name()
    );
  }
}

// ── The ceiling: a refusal's atomicity, and rollbacks over refused errors ─────────────────

/// A full store: its ceiling is `SEED`'s length, so a write after the seed is refused. One short
/// of it, `SEED`'s last span is itself an error the store refused (see `seeded_under`), so the
/// write under test meets a full store that has already counted one.
const FULL: usize = SEED.len();

/// Past the ceiling a write is refused, and the only caller code a refusal runs is the
/// destructors of what it was handed and does not keep. Arming each of them in turn over a
/// full store, with and without an error already refused, on every channel that has the
/// destructor: an unwinding destructor leaves the store exactly as it was, the mark and the
/// count included, which is the oracle `record_sweep` already holds every write to.
#[test]
fn a_refusal_is_atomic_under_a_panicking_span_drop() {
  for channel in [Channel::Error, Channel::Warning, Channel::Hole] {
    for max in [FULL, FULL - 1] {
      for at_span in [FRESH, EXISTING] {
        record_sweep_under(channel, Axis::SpanDrop, SEED, at_span, max);
      }
    }
  }
}

/// The payload's destructor, the first a refusal runs: the hole channel has no payload, so its
/// cell is the zero pin below.
#[test]
fn a_refusal_is_atomic_under_a_panicking_payload_drop() {
  for channel in [Channel::Error, Channel::Warning] {
    for max in [FULL, FULL - 1] {
      for at_span in [FRESH, EXISTING] {
        record_sweep_under(channel, Axis::PayloadDrop, SEED, at_span, max);
      }
    }
  }
}

/// What a refusal runs, counted rather than assumed: exactly one span destructor on every
/// channel, exactly one payload destructor on the two payload channels and none on the hole
/// channel, and no span comparison and no span clone on any of them.
#[test]
fn a_refusal_runs_only_the_destructors_of_what_it_drops() {
  for channel in [Channel::Error, Channel::Warning, Channel::Hole] {
    let payloads = usize::from(!matches!(channel, Channel::Hole));
    for max in [FULL, FULL - 1] {
      for at_span in [FRESH, EXISTING] {
        let calls = |axis: Axis| record_calls_under(channel, axis, SEED, at_span, max);
        assert_eq!(
          calls(Axis::SpanDrop),
          1,
          "a refused `{}` must destroy exactly the span it was handed",
          channel.name()
        );
        assert_eq!(
          calls(Axis::PayloadDrop),
          payloads,
          "a refused `{}` must destroy exactly the payload it was handed, and a hole has none",
          channel.name()
        );
        assert_eq!(
          calls(Axis::Clone),
          0,
          "a refused `{}` cloned a span: a refusal stores nothing, so it clones nothing",
          channel.name()
        );
        assert_eq!(
          calls(Axis::Cmp),
          0,
          "a refused `{}` compared a span: a refusal descends no map",
          channel.name()
        );
      }
    }
  }
}

/// Six emissions under a ceiling of four: a store seeded with them (see `seeded_under`) keeps
/// four records of its channel and has refused two errors above them, so its mark is six.
const REFUSING: &[usize] = &[10, 20, 30, 30, 40, 20];
const REFUSING_MAX: usize = 4;

/// Undoing a refused error is a subtraction from a count, so a rewind that undoes refused errors
/// and nothing else runs no caller code: zero on all four axes, on every channel, whether it
/// undoes one of them or both.
#[test]
fn undoing_a_refused_error_runs_no_caller_code() {
  for channel in [Channel::Error, Channel::Warning, Channel::Hole] {
    for keep in [REFUSING_MAX, REFUSING_MAX + 1] {
      for axis in [Axis::Clone, Axis::Cmp, Axis::SpanDrop, Axis::PayloadDrop] {
        assert_eq!(
          rewind_calls_under(channel, axis, REFUSING, keep, REFUSING_MAX),
          0,
          "undoing refused errors down to mark {keep} over the {} channel ran {}",
          channel.name(),
          axis.name()
        );
      }
    }
  }
}

/// A rewind that crosses from refused errors into kept records undoes the refused errors first
/// and then unwinds kept entries as a store with no ceiling does, so it is atomic at each armed
/// call exactly as that store's rewind is, against a clean twin seeded under the same ceiling.
#[test]
fn a_rewind_across_refused_errors_into_kept_records_is_atomic() {
  for channel in [Channel::Error, Channel::Warning, Channel::Hole] {
    for axis in [Axis::Cmp, Axis::Clone, Axis::SpanDrop] {
      rewind_sweep_under(channel, axis, REFUSING, 1, REFUSING_MAX);
    }
  }
  for channel in [Channel::Error, Channel::Warning] {
    rewind_sweep_under(channel, Axis::PayloadDrop, REFUSING, 1, REFUSING_MAX);
  }
  assert_eq!(
    rewind_calls_under(Channel::Hole, Axis::PayloadDrop, REFUSING, 1, REFUSING_MAX),
    0,
    "the skipped-region channel ran an `Error` destructor, but it stores no `Error`"
  );
}

// ── The ceiling: what a full store keeps, refuses and counts ──────────────────────────────

/// One replayed record: its channel (`'E'`, `'W'` or `'H'`), its span, and its payload (the
/// error value, or a hole's skipped count).
type Kept = (char, usize, usize);

/// The kept records, replayed in emission order.
fn kept(store: &Store<BombErr, BombSpan>) -> Vec<Kept> {
  store
    .diagnostics()
    .map(|d| match d.kind() {
      DiagnosticKind::Error(e) => ('E', d.span().0, e.0 as usize),
      DiagnosticKind::Warning(e) => ('W', d.span().0, e.0 as usize),
      DiagnosticKind::SkippedRegion(skipped) => ('H', d.span().0, skipped),
    })
    .collect()
}

/// What a rewind must restore: the kept records, the refused-error count and the mark.
type State = (Vec<Kept>, usize, u64);

fn state(store: &Store<BombErr, BombSpan>) -> State {
  (kept(store), store.refused_errors(), store.mark())
}

/// The `State` holding `records`, with `refused` errors counted and the mark at `mark`.
fn holding(records: &[Kept], refused: usize, mark: u64) -> State {
  (records.to_vec(), refused, mark)
}

/// A sequence over all three channels. Under a ceiling of two it keeps an error and a hole, then
/// refuses an error, a warning, an error, a hole and an error.
const MIXED: &[(Channel, usize)] = &[
  (Channel::Error, 10),
  (Channel::Hole, 20),
  (Channel::Error, 30),
  (Channel::Warning, 40),
  (Channel::Error, 50),
  (Channel::Hole, 60),
  (Channel::Error, 70),
];

/// A store with a ceiling of `max` that has taken the first `upto` emissions of `MIXED`, each
/// carrying its index as its payload.
fn mixed(max: usize, upto: usize) -> Store<BombErr, BombSpan> {
  let mut store = Store::<BombErr, BombSpan>::with_max_diagnostics(max);
  for (seq, &(channel, span)) in MIXED[..upto].iter().enumerate() {
    channel.write(&mut store, BombSpan(span), seq as u32);
  }
  store
}

/// Ceiling three: an error, a hole and a warning, then an error, an error, a hole and a warning.
/// The first three are kept in order, the two later errors are counted, and the later hole and
/// warning leave no trace. Refuses evicting the oldest records, counting a refused warning or
/// hole, and a ceiling for each channel.
#[test]
fn a_full_store_refuses_and_counts_errors() {
  let mut store = Store::<BombErr, BombSpan>::with_max_diagnostics(3);
  let emissions: [(Channel, usize); 7] = [
    (Channel::Error, 10),
    (Channel::Hole, 20),
    (Channel::Warning, 30),
    (Channel::Error, 40),
    (Channel::Error, 50),
    (Channel::Hole, 60),
    (Channel::Warning, 70),
  ];
  for (seq, (channel, span)) in emissions.into_iter().enumerate() {
    channel.write(&mut store, BombSpan(span), seq as u32);
  }
  assert_eq!(
    state(&store),
    holding(&[('E', 10, 0), ('H', 20, 1), ('W', 30, 2)], 2, 5),
    "the first three are kept in emission order, only the two errors past them are counted, \
     and the mark is the three kept plus the two refused"
  );
  assert_eq!(
    (
      store.errors().len(),
      store.warnings().len(),
      store.skipped_regions().len()
    ),
    (1, 1, 1),
    "each channel's map holds its one kept record and nothing refused"
  );
}

/// On each channel, below the ceiling and past it: a kept record moves the mark by one, a
/// refused error moves it by one, and a refused warning or hole leaves it where it was. Refuses
/// a mark that is the log's length, and one that moves on a refused warning or hole.
#[test]
fn the_mark_counts_kept_records_and_refused_errors() {
  for channel in [Channel::Error, Channel::Warning, Channel::Hole] {
    let counted = u64::from(matches!(channel, Channel::Error));
    let mut store = Store::<BombErr, BombSpan>::with_max_diagnostics(2);
    for (seq, span) in [10, 20, 30, 40].into_iter().enumerate() {
      let before = store.mark();
      channel.write(&mut store, BombSpan(span), seq as u32);
      let (side, want) = if seq < 2 {
        ("below", 1)
      } else {
        ("past", counted)
      };
      assert_eq!(
        store.mark() - before,
        want,
        "`{}` #{seq}, {side} the ceiling of two, moved the mark by the wrong amount",
        channel.name()
      );
    }
    assert_eq!(
      (store.diagnostics().len(), store.refused_errors() as u64),
      (2, 2 * counted),
      "`{}`: two kept, and the two past the ceiling counted only if they are errors",
      channel.name()
    );
  }
}

/// Ceiling two, and the state after every emission of `MIXED`, which refuses past the ceiling on
/// all three channels. Rewound from every point of the sequence — its end, its middle, its
/// start — to the mark taken at every earlier point, and once more step by step down one store:
/// the kept records, the count and the mark are exactly what they were when that mark was
/// taken. Refuses a count that a rewind does not restore, and a kept record unwound while a
/// refused error is left above it.
#[test]
fn a_rewind_undoes_refused_errors_first_and_restores_the_count() {
  let taken: Vec<State> = (0..=MIXED.len())
    .map(|upto| state(&mixed(2, upto)))
    .collect();
  assert_eq!(
    taken.last(),
    Some(&holding(&[('E', 10, 0), ('H', 20, 1)], 3, 5)),
    "the sequence ends with three refused errors above a full log"
  );

  for from in 0..=MIXED.len() {
    for (to, then) in taken.iter().enumerate().take(from + 1) {
      let mut store = mixed(2, from);
      store.rewind_to(then.2);
      assert_eq!(
        &state(&store),
        then,
        "a rewind from after emission {from} to the mark taken after emission {to}"
      );
    }
  }

  let mut store = mixed(2, MIXED.len());
  for (to, then) in taken.iter().enumerate().rev().skip(1) {
    store.rewind_to(then.2);
    assert_eq!(
      &state(&store),
      then,
      "step by step down one store, the rewind to the mark taken after emission {to}"
    );
  }
}

/// An error refused, rewound and emitted again is counted once: the rewind uncounted it.
#[test]
fn an_error_refused_rewound_and_emitted_again_is_counted_once() {
  let mut store = Store::<BombErr, BombSpan>::with_max_diagnostics(1);
  Channel::Error.write(&mut store, BombSpan(10), 0);
  let before = store.mark();
  Channel::Error.write(&mut store, BombSpan(20), 1);
  assert_eq!(
    state(&store),
    holding(&[('E', 10, 0)], 1, 2),
    "refused, and counted"
  );
  store.rewind_to(before);
  assert_eq!(
    state(&store),
    holding(&[('E', 10, 0)], 0, 1),
    "uncounted by the rewind"
  );
  Channel::Error.write(&mut store, BombSpan(20), 1);
  assert_eq!(
    state(&store),
    holding(&[('E', 10, 0)], 1, 2),
    "emitted again, it is counted once"
  );
}

/// Fill the store, refuse two errors, rewind below the ceiling and emit an error: it is kept,
/// and nothing is counted as refused. Refuses a store that goes on refusing once it has
/// refused, and a count that survives a rewind below the ceiling.
#[test]
fn a_store_below_its_ceiling_after_a_rewind_keeps_again() {
  let mut store = Store::<BombErr, BombSpan>::with_max_diagnostics(2);
  Channel::Error.write(&mut store, BombSpan(10), 0);
  let below = store.mark();
  Channel::Warning.write(&mut store, BombSpan(20), 1);
  Channel::Error.write(&mut store, BombSpan(30), 2);
  Channel::Error.write(&mut store, BombSpan(40), 3);
  assert_eq!(
    state(&store),
    holding(&[('E', 10, 0), ('W', 20, 1)], 2, 4),
    "full, with two errors refused"
  );
  store.rewind_to(below);
  assert_eq!(
    state(&store),
    holding(&[('E', 10, 0)], 0, 1),
    "below the ceiling, with nothing refused"
  );
  Channel::Error.write(&mut store, BombSpan(50), 4);
  assert_eq!(
    state(&store),
    holding(&[('E', 10, 0), ('E', 50, 4)], 0, 2),
    "the error is kept, and nothing is counted as refused"
  );
}

/// A ceiling of zero keeps nothing: every error is counted, every warning and hole is dropped,
/// and a rewind only uncounts. Refuses reading zero as no ceiling, and keeping a first record.
#[test]
fn a_ceiling_of_zero_keeps_nothing() {
  let mut store = mixed(0, MIXED.len());
  assert_eq!(
    state(&store),
    holding(&[], 4, 4),
    "all four of `MIXED`'s errors are counted, and nothing is kept"
  );
  assert!(
    store.errors().is_empty()
      && store.labels().is_empty()
      && store.warnings().is_empty()
      && store.warning_labels().is_empty()
      && store.skipped_regions().is_empty()
      && store.skipped_region_labels().is_empty(),
    "no channel map holds anything"
  );
  store.rewind_to(1);
  assert_eq!(
    state(&store),
    holding(&[], 1, 1),
    "a rewind uncounts the errors refused after its mark"
  );
  store.rewind_to(0);
  assert_eq!(state(&store), holding(&[], 0, 0));
}
