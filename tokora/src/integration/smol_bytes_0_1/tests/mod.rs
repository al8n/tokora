#[cfg(any(feature = "std", feature = "alloc"))]
mod slice;
#[cfg(any(feature = "std", feature = "alloc"))]
mod source;

// Gated by `cfg(test)` alone, as `utils/to_equivalent`'s tests were: these need neither this
// crate's `std` nor its `alloc`.
mod to_equivalent;
