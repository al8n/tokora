//! Flexible message container that adapts to `no_std` and `alloc` environments.
//!
//! `CowStr` provides a single abstraction for short, human-readable strings that may be
//! either static literals or owned allocations depending on the available features. In
//! `no_std` + `no_alloc` builds the type degenerates to a lightweight wrapper around a
//! `&'static str`, while in `alloc`/`std` builds it stores a `Cow<'static, str>` to balance
//! zero-copy ergonomics with configurability.

/// A short, human-readable string: a `&'static str`, or with `alloc` or `std` also an owned
/// `String`.
///
/// Without `alloc` or `std` this is `cower::Cow<'static, str, cower::BorrowedPolicy>`: it holds
/// the `&'static str` itself, is `Copy`, and has no owned form. With `alloc` or `std` it is
/// `cower::Cow<'static, str>`, backed by `std::borrow::Cow<'static, str>`.
///
/// The first tier names its policy rather than taking cower's default, because the default
/// policy's storage follows cower's own `alloc` and `std` features, and Cargo unifies those
/// across a build: another crate enabling `cower/alloc` would turn this tier's `CowStr` into a
/// standard `Cow`, which is not `Copy`. `CowStr::from_static` builds one in every tier, in a
/// `const` context too:
///
/// ```
/// use tokora::utils::CowStr;
///
/// const NAME: CowStr = CowStr::from_static("block");
/// assert_eq!(NAME.as_str(), "block");
/// assert!(NAME.is_borrowed());
/// ```
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub type CowStr = cower::Cow<'static, str, cower::BorrowedPolicy>;

/// A short, human-readable string: a `&'static str`, or with `alloc` or `std` also an owned
/// `String`.
///
/// Without `alloc` or `std` this is `cower::Cow<'static, str, cower::BorrowedPolicy>`: it holds
/// the `&'static str` itself, is `Copy`, and has no owned form. With `alloc` or `std` it is
/// `cower::Cow<'static, str>`, backed by `std::borrow::Cow<'static, str>`.
///
/// The first tier names its policy rather than taking cower's default, because the default
/// policy's storage follows cower's own `alloc` and `std` features, and Cargo unifies those
/// across a build: another crate enabling `cower/alloc` would turn this tier's `CowStr` into a
/// standard `Cow`, which is not `Copy`. `CowStr::from_static` builds one in every tier, in a
/// `const` context too:
///
/// ```
/// use tokora::utils::CowStr;
///
/// const NAME: CowStr = CowStr::from_static("block");
/// assert_eq!(NAME.as_str(), "block");
/// assert!(NAME.is_borrowed());
/// ```
#[cfg(any(feature = "std", feature = "alloc"))]
pub type CowStr = cower::Cow<'static, str>;
