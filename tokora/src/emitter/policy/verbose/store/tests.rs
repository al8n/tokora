use super::Store;
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
  /// it**, the mark, and the replay. The label map is in here deliberately — the record
  /// orphan is invisible to any check that reads only the payload side of a representation
  /// documented as parallel, which is exactly how it survived the previous matrix.
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
      "mark={} payload={payload} labels={labels} replay={replay:?}",
      store.mark()
    )
  }
}

/// A store with one `channel` record at each span in `spans`, in order.
fn seeded(channel: Channel, spans: &[usize]) -> Store<BombErr, BombSpan> {
  let mut store = Store::<BombErr, BombSpan>::new();
  for (seq, &span) in spans.iter().enumerate() {
    channel.write(&mut store, BombSpan(span), seq as u32);
  }
  store
}

/// How many `axis` calls one record at `at_span` makes into a store seeded at `seed`, with
/// none of them armed. Both the sweeps and the zero-pin below start here, so "how many" is one
/// measurement rather than two that can drift apart.
fn record_calls(channel: Channel, axis: Axis, seed: &[usize], at_span: usize) -> usize {
  let mut cal = seeded(channel, seed);
  axis.arm(0);
  channel.write(&mut cal, BombSpan(at_span), 99);
  let calls = axis.count();
  disarm();
  calls
}

/// How many `axis` calls `rewind_to(keep)` makes over a store seeded at `spans`, none armed.
fn rewind_calls(channel: Channel, axis: Axis, spans: &[usize], keep: usize) -> usize {
  let mut cal = seeded(channel, spans);
  axis.arm(0);
  cal.rewind_to(keep as u64);
  let calls = axis.count();
  disarm();
  calls
}

/// Arms every call of `axis` that one record at `at_span` makes into a store seeded at
/// `seed`, and asserts each one leaves the channel byte-for-byte as it was.
fn record_sweep(channel: Channel, axis: Axis, seed: &[usize], at_span: usize) {
  let calls = record_calls(channel, axis, seed, at_span);
  assert!(
    calls > 0,
    "`{}` runs no {} the sweep can arm, so this cell measures nothing",
    channel.name(),
    axis.name()
  );

  for at in 1..=calls {
    let mut store = seeded(channel, seed);
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
  let clean = |to: u64| {
    let mut twin = seeded(channel, spans);
    twin.rewind_to(to);
    channel.observable(&twin)
  };
  let target = clean(keep as u64);
  let calls = rewind_calls(channel, axis, spans, keep);
  assert!(
    calls > 0,
    "`rewind_to` over the {} channel runs no {} the sweep can arm, so this cell measures \
     nothing",
    channel.name(),
    axis.name()
  );

  for at in 1..=calls {
    let mut store = seeded(channel, spans);
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
