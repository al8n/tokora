mod container;

// The handler traits exist only with `many`; these impls were in `combinator/many/handler`, in the
// `alloc`/`std` block beside the `VecDeque` ones, so their rustdoc banner reads `smallvec_1` and
// (`alloc` or `std`). This module is itself gated on `alloc` or `std` beside `smallvec_1` (see
// `integration/mod.rs`), so that gate excludes no build that reaches it; it stays so the banner
// does too.
#[cfg(feature = "many")]
#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
mod handler;
