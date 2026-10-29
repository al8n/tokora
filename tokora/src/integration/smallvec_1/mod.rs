mod container;

// The handler traits exist only with `many`; these impls were in `combinator/many/handler`, in the
// `alloc`/`std` block beside the `VecDeque` ones, so their rustdoc banner reads `smallvec_1` and
// (`alloc` or `std`). `smallvec_1` already implies `alloc`, so that gate never excludes a build;
// it stays so the banner does too.
#[cfg(feature = "many")]
#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
mod handler;
