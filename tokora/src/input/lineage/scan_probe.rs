use core::cell::Cell;

thread_local! {
  static SCANNED: Cell<usize> = const { Cell::new(0) };
}

/// Zeroes the odometer and opens a fresh reading window.
#[cfg(feature = "logos_0_16")]
pub(crate) fn reset() {
  SCANNED.with(|c| c.set(0));
}

/// Elements inspected since the last [`reset`].
#[cfg(feature = "logos_0_16")]
pub(crate) fn scanned() -> usize {
  SCANNED.with(Cell::get)
}

#[inline(always)]
pub(crate) fn tick() {
  SCANNED.with(|c| c.set(c.get() + 1));
}
