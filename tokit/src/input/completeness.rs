//! The input **completeness** typestate: [`Complete`] (the default — a whole, closed input)
//! and [`Partial`] (a growing chunk of a Sans-I/O stream that may still be extended).
//!
//! The typestate is a zero-sized type parameter on the input types ([`InputRef`](crate::InputRef)
//! and the crate-internal `Input`), defaulted to [`Complete`], so it is a **non-breaking** addition:
//! every existing spelling `InputRef<'inp, '_, L, Ctx, Lang>` still names the complete input, and
//! its generated code is unchanged. The partial-input frontier rules live behind the
//! [`Completeness::PARTIAL`] associated constant, so [`Complete`] monomorphizes them away entirely
//! (the const is `false`, and every rule is written `if Cmpl::PARTIAL && …`, so dead-code
//! elimination erases the whole check — no runtime branch on the hot path).
//!
//! # What each mode means
//!
//! - [`Complete`] — the input is the *entire* source. End of input is genuine, a lexer error at the
//!   very end is genuine, and every lexed token is final. This is today's behaviour, bit for bit.
//! - [`Partial`] — the input is a *prefix* of a stream that may still grow (`is_final == false`) or
//!   the last chunk (`is_final == true`). While non-final, three conservative *frontier rules*
//!   apply at the scan chokepoint so a construct that might still be extended by later input is
//!   never mistaken for a finished one; each surfaces an
//!   [`Incomplete`](crate::error::Incomplete) instead. See
//!   [`InputRef::next`](crate::InputRef::next) and the [`input`](crate::input) module docs for the
//!   frontier rules and the resumption pattern. With `is_final == true`, [`Partial`] behaves
//!   exactly like [`Complete`].

use core::fmt::Debug;

use crate::{Emitter, Lexer, ParseContext, error::Incomplete};

pub(crate) mod sealed {
  /// Seals [`Completeness`](super::Completeness): only [`Complete`](super::Complete) and
  /// [`Partial`](super::Partial), defined in this crate, implement it, so the set of input modes
  /// is closed and no downstream type can add one.
  pub trait Sealed {}
}

/// The completeness typestate of an input: whether the source is the whole input ([`Complete`]) or
/// a still-growable chunk of a stream ([`Partial`]).
///
/// A **sealed** trait with exactly two implementors. It is a compile-time selector, never a runtime
/// flag: the associated const [`PARTIAL`](Self::PARTIAL) gates every partial-input rule, so the
/// [`Complete`] path compiles to identical code with the rules eliminated (see the
/// [`input`](crate::input) module docs).
pub trait Completeness: sealed::Sealed + Sized {
  /// `true` for [`Partial`], `false` for [`Complete`]. Every frontier rule is written
  /// `if Cmpl::PARTIAL && …`, so this constant is what erases the rules from the complete path at
  /// monomorphization.
  const PARTIAL: bool;

  /// Per-input storage for the runtime `is_final` flag. A **zero-sized** `()` for [`Complete`] (so
  /// carrying it costs nothing and never grows the input) and a `bool` for [`Partial`].
  type Finality: Copy + Debug;

  /// Builds the finality storage for a freshly-constructed input. [`Complete`] ignores the flag;
  /// [`Partial`] records it.
  fn finality(is_final: bool) -> Self::Finality;

  /// Reads whether the input is final. Always `true` for [`Complete`] (a whole input is final by
  /// definition — every frontier rule is thereby inert); the stored flag for [`Partial`].
  fn is_final(finality: &Self::Finality) -> bool;

  /// Sets the runtime `is_final` flag. A no-op for [`Complete`] (its finality is the ZST `()`); it
  /// records the flag for [`Partial`].
  fn set_final(finality: &mut Self::Finality, is_final: bool);
}

/// The default completeness: the input is the **whole** source.
///
/// End of input, an end-of-input lexer error, and every lexed token are all genuine — there is no
/// "more may arrive". Selecting this (the default) reproduces today's behaviour with identical
/// generated code: [`Completeness::PARTIAL`] is `false`, so the partial-input frontier rules are
/// eliminated at monomorphization and the finality storage is the zero-sized `()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Complete;

/// The partial completeness: the input is a **prefix** of a stream that may still grow.
///
/// Carries a runtime `is_final: bool` the caller sets as chunks arrive. While non-final the
/// [frontier rules](crate::input#partial-input-sans-io-mode) hold back any construct that might be
/// extended by later input, surfacing an [`Incomplete`](crate::error::Incomplete) so the caller can
/// refill and re-drive; once final it behaves exactly like [`Complete`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Partial;

impl sealed::Sealed for Complete {}
impl Completeness for Complete {
  const PARTIAL: bool = false;
  type Finality = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn finality(_is_final: bool) -> Self::Finality {}

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_final(_finality: &Self::Finality) -> bool {
    // A complete input is final by definition, so every frontier rule is inert regardless.
    true
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn set_final(_finality: &mut Self::Finality, _is_final: bool) {}
}

impl sealed::Sealed for Partial {}
impl Completeness for Partial {
  const PARTIAL: bool = true;
  type Finality = bool;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn finality(is_final: bool) -> Self::Finality {
    is_final
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_final(finality: &Self::Finality) -> bool {
    *finality
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn set_final(finality: &mut Self::Finality, is_final: bool) {
    *finality = is_final;
  }
}

/// Constructs the emitter's error as an [`Incomplete`] partial-input sentinel — the
/// incomplete-surfacing mechanism the frontier rules call at the scan chokepoint.
///
/// This is the **minimal least-surface bound** that keeps the complete path bound-free. The scan
/// methods need to build the user's error type as an incomplete, but requiring that everywhere
/// would force the bound onto every complete-mode caller. Routing it through this trait — which
/// [`Completeness`] extends — makes the requirement *conditional on the typestate*:
///
/// - [`Complete`] implements it **unconditionally**, with an `unreachable!()` body. Complete mode
///   never surfaces an incomplete ([`Completeness::PARTIAL`] is `false`, so every call site is
///   dead-code-eliminated), so it needs no construction and imposes **no new bound** — a complete
///   parse compiles exactly as before.
/// - [`Partial`] implements it only where `<Ctx::Emitter>::Error: From<Incomplete<L::Offset>>`, so
///   that `From` requirement lands **only** on partial-mode parses.
///
/// The trait is sealed through its [`Completeness`] supertrait (only [`Complete`] and [`Partial`]
/// exist), and the orphan rule keeps downstream from implementing it for either.
pub trait SurfaceIncomplete<'inp, L, Ctx, Lang: ?Sized>: Completeness
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Builds the emitter's error as an [`Incomplete`] at `offset` — the offset the input ran out
  /// at (the frontier). Called only in partial, non-final mode.
  fn surface_incomplete(offset: L::Offset) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error;
}

impl<'inp, L, Ctx, Lang: ?Sized> SurfaceIncomplete<'inp, L, Ctx, Lang> for Complete
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn surface_incomplete(_offset: L::Offset) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error {
    // Unreachable by construction: `Complete::PARTIAL` is `false`, so every frontier rule is
    // written `if Cmpl::PARTIAL && …` and this call is eliminated at monomorphization. Providing
    // an unconditional (bound-free) impl is what keeps `From<Incomplete>` off the complete path.
    unreachable!("Complete-mode input never surfaces Incomplete (PARTIAL == false)")
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> SurfaceIncomplete<'inp, L, Ctx, Lang> for Partial
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<Incomplete<L::Offset>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn surface_incomplete(offset: L::Offset) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error {
    Incomplete::new(offset).into()
  }
}
