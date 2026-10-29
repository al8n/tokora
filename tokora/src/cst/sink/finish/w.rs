use core::cell::Cell;

thread_local! {
  static W: Cell<u64> = const { Cell::new(0) };
}

/// One unit of replay work.
#[inline(always)]
pub(crate) fn tick() {
  W.with(|w| w.set(w.get().saturating_add(1)));
}

/// `n` units at once — see [`w_tick_n`](super::w_tick_n).
#[inline(always)]
pub(crate) fn add(n: u64) {
  W.with(|w| w.set(w.get().saturating_add(n)));
}

/// Zeroes the counter; call immediately before the measured `finish`.
pub(crate) fn reset() {
  W.with(|w| w.set(0));
}

/// The accumulated tick count since the last [`reset`].
pub(crate) fn read() -> u64 {
  W.with(Cell::get)
}
