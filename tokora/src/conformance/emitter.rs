//! The source-binding emitter conformance kit. Its documentation is the source-binding emitter
//! kit section of the [`conformance`](super) module docs, where docs.rs renders it: this module
//! is crate-private, and its items are reached as `tokora::conformance::Item`.

use crate::{Emitter, Lexer, source::SourceIdentity};

/// Asserts that `emitter` reports the source it was built over.
///
/// Call this from a `#[test]` in the crate that defines a source-binding emitter, passing the
/// same source value the emitter was constructed with.
///
/// # Panics
///
/// If the emitter answers `None` (it inherited [`Emitter::bound_source`]'s default while
/// actually binding a source), or answers a different identity than `source`'s.
#[track_caller]
pub fn assert_binds_source<'inp, L, Lang, E>(emitter: &E, source: &'inp L::Source)
where
  L: Lexer<'inp>,
  Lang: ?Sized,
  E: Emitter<'inp, L, Lang>,
{
  let expected = SourceIdentity::of(source);
  match emitter.bound_source() {
    None => panic!(
      "this emitter binds a source but inherits `Emitter::bound_source`'s `None` default. \
       The parse entry then has nothing to compare, so a parse driven over a *different* \
       buffer is accepted and produces a tree whose text is yours and whose structure is the \
       parse's. Override it: `fn bound_source(&self) -> Option<SourceIdentity> {{ \
       Some(SourceIdentity::of(self.source)) }}`"
    ),
    Some(got) => assert!(
      got == expected,
      "this emitter reports a different source ({got}) than the one it was given \
       ({expected}). `bound_source` must project the source the emitter will actually slice, \
       not a copy of the reference to it — for a sized `L::Source` the two are different \
       addresses"
    ),
  }
}

/// Asserts that `wrapper` reports the same bound source as the `inner` emitter it wraps.
///
/// # Panics
///
/// If the wrapper answers `None` where the inner answers `Some`, or answers a different
/// identity.
#[track_caller]
pub fn assert_forwards_bound_source<'inp, L, Lang, Inner, Wrapper>(wrapper: &Wrapper, inner: &Inner)
where
  L: Lexer<'inp>,
  Lang: ?Sized,
  Inner: Emitter<'inp, L, Lang>,
  Wrapper: Emitter<'inp, L, Lang>,
{
  let expected = Emitter::<'inp, L, Lang>::bound_source(inner);
  let got = Emitter::<'inp, L, Lang>::bound_source(wrapper);
  assert!(
    got == expected,
    "this wrapper does not forward `bound_source` (wrapper: {got:?}, inner: {expected:?}). \
     A wrapper that forwards every emission but not this one silently disables the \
     source-identity check for the emitter it wraps — the same forwarding obligation \
     `checkpoint` / `rewind` / `release` carry"
  );
}
