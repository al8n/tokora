//! A tiny, dependency-free deterministic PRNG for the fuzz harness.
//!
//! [`Rng`] is a SplitMix64 generator: a single `u64` of state, advanced by a fixed odd
//! increment and finalized with the standard SplitMix64 avalanche. It has no external entropy —
//! every value derives from the explicit seed the caller supplies, so a run is fully reproducible
//! from its `u64` seed (see the [module docs](crate::fuzz) for the reproduce-with-seed workflow).
//! It is *not* cryptographic; it only needs to be well-distributed and portable.

/// A deterministic SplitMix64 PRNG seeded by an explicit `u64`.
#[derive(Debug, Clone)]
pub(crate) struct Rng {
  state: u64,
}

impl Rng {
  /// The SplitMix64 increment (the fractional bits of the golden ratio), a fixed odd constant.
  const INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;

  /// Creates a generator from an explicit seed. Every subsequent value is a pure function of
  /// this seed, so the whole run reproduces from it.
  #[inline]
  pub(crate) const fn new(seed: u64) -> Self {
    Self { state: seed }
  }

  /// Advances the state and returns the next 64-bit value (SplitMix64).
  #[inline]
  pub(crate) fn next_u64(&mut self) -> u64 {
    self.state = self.state.wrapping_add(Self::INCREMENT);
    let mut z = self.state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
  }

  /// A uniformly-distributed value in `0..n` (via Lemire-style multiply-shift; `n` must be
  /// non-zero). Good enough for selection; not perfectly unbiased, which fuzz selection does not
  /// need.
  #[inline]
  pub(crate) fn below(&mut self, n: usize) -> usize {
    debug_assert!(n > 0, "Rng::below requires a non-zero bound");
    ((self.next_u64() as u128 * n as u128) >> 64) as usize
  }

  /// A `bool` true with probability `num / den`.
  #[inline]
  pub(crate) fn chance(&mut self, num: u32, den: u32) -> bool {
    debug_assert!(den > 0, "Rng::chance requires a non-zero denominator");
    (self.next_u64() % den as u64) < num as u64
  }
}
