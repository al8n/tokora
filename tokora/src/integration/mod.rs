//! Trait impls for the types of optional third-party crates: one module per crate and version,
//! each behind that crate's feature, written once, here. Nothing in this tree adds a public
//! item — a module is a set of impls of this crate's own traits.
//!
//! A family whose trait exists only under another feature (`CstText` under `rowan`, the
//! collection handlers under `many`, `SemanticPrefix` under `conformance`) carries that gate on
//! its own `mod` line, in the crate's `mod.rs`, exactly as the file the impls left carried it.

#[cfg(feature = "bstr_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bstr_1")))]
mod bstr_1;

#[cfg(feature = "bytes_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "bytes_1")))]
mod bytes_1;

#[cfg(feature = "heapless_0_9")]
#[cfg_attr(docsrs, doc(cfg(feature = "heapless_0_9")))]
mod heapless_0_9;

#[cfg(feature = "hipstr_0_8")]
#[cfg_attr(docsrs, doc(cfg(feature = "hipstr_0_8")))]
mod hipstr_0_8;

#[cfg(feature = "smallvec_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "smallvec_1")))]
mod smallvec_1;

#[cfg(feature = "smol-bytes_0_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol-bytes_0_1")))]
mod smol_bytes_0_1;

#[cfg(feature = "smol-bytes_0_2")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol-bytes_0_2")))]
mod smol_bytes_0_2;

#[cfg(feature = "tinyvec_1")]
#[cfg_attr(docsrs, doc(cfg(feature = "tinyvec_1")))]
mod tinyvec_1;
