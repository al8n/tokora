//! Flexible message container that adapts to `no_std` and `alloc` environments.
//!
//! `CowStr` provides a single abstraction for short, human-readable strings that may be
//! either static literals or owned allocations depending on the available features. In
//! `no_std` + `no_alloc` builds the type degenerates to a lightweight wrapper around a
//! `&'static str`, while in `alloc`/`std` builds it stores a `Cow<'static, str>` to balance
//! zero-copy ergonomics with configurability.

/// Feature-aware message container with a unified API.
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub type CowStr = cower::Cow<'static, str>;

/// Feature-aware message container with a unified API.
#[cfg(any(feature = "std", feature = "alloc"))]
pub type CowStr = cower::Cow<'static, str>;
