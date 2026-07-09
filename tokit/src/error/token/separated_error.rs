//! Separator-position-tagged unexpected-token error for separated sequences.
//!
//! [`SeparatedError`] wraps an [`UnexpectedToken`] together with the
//! [`SeparatorPosition`] at which it occurred. Carrying the position as **data**
//! (rather than encoding it in the `Lang` type parameter of `UnexpectedToken`)
//! lets a downstream error type absorb leading / trailing separator errors
//! through a single `From<SeparatedError<..>>` impl and tell them apart via the
//! [`position`](SeparatedError::position) field.

use crate::{Lexer, Token, error::token::UnexpectedToken, span::SimpleSpan};

/// Where, within a separated sequence, a separator-related error occurred.
///
/// Used as a data field on [`SeparatedError`] instead of overloading the `Lang`
/// type slot of [`UnexpectedToken`] with position marker types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display, derive_more::IsVariant)]
#[display("{}", self.as_str())]
pub enum SeparatorPosition {
  /// The error occurred where a sequence **element** was expected — for example
  /// a separator found in place of an element (a repeated / duplicate
  /// separator, or a missing element between two separators).
  Element,
  /// The error occurred at a **leading** separator — one appearing before the
  /// first element, where the sequence's policy does not permit it.
  Leading,
  /// The error occurred at a **trailing** separator — one appearing after the
  /// last element, where the sequence's policy does not permit it.
  Trailing,
}

impl SeparatorPosition {
  /// Returns the static, lowercase string name of this position
  /// (`"element"`, `"leading"`, or `"trailing"`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Element => "element",
      Self::Leading => "leading",
      Self::Trailing => "trailing",
    }
  }
}

/// A type alias for a [`SeparatedError`] for a given lexer and language.
pub type SeparatedErrorOf<'inp, L, Lang = ()> = SeparatedError<
  'inp,
  <L as Lexer<'inp>>::Token,
  <<L as Lexer<'inp>>::Token as Token<'inp>>::Kind,
  <L as Lexer<'inp>>::Span,
  Lang,
>;

/// An [`UnexpectedToken`] error tagged with the [`SeparatorPosition`] at which
/// it was produced within a separated sequence.
///
/// This is the payload the separator emitter conversion traits speak: the
/// leading / trailing separator emitters wrap the offending token here and
/// stamp the position, so a downstream error type distinguishes the cases by
/// reading [`position`](Self::position) rather than by matching distinct types.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SeparatedError<'a, T, Kind: Clone, S = SimpleSpan, Lang: ?Sized = ()> {
  position: SeparatorPosition,
  inner: UnexpectedToken<'a, T, Kind, S, Lang>,
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> SeparatedError<'a, T, Kind, S, Lang> {
  /// Creates a new `SeparatedError` at `position` wrapping `inner`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    position: SeparatorPosition,
    inner: UnexpectedToken<'a, T, Kind, S, Lang>,
  ) -> Self {
    Self { position, inner }
  }

  /// Creates a `SeparatedError` at the [`Leading`](SeparatorPosition::Leading) position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn leading(inner: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    Self::new(SeparatorPosition::Leading, inner)
  }

  /// Creates a `SeparatedError` at the [`Trailing`](SeparatorPosition::Trailing) position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn trailing(inner: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    Self::new(SeparatorPosition::Trailing, inner)
  }

  /// Creates a `SeparatedError` at the [`Element`](SeparatorPosition::Element) position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn element(inner: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    Self::new(SeparatorPosition::Element, inner)
  }

  /// Returns the position at which this separator error occurred.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn position(&self) -> SeparatorPosition {
    self.position
  }

  /// Returns a reference to the wrapped [`UnexpectedToken`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn inner_ref(&self) -> &UnexpectedToken<'a, T, Kind, S, Lang> {
    &self.inner
  }

  /// Returns a mutable reference to the wrapped [`UnexpectedToken`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn inner_mut(&mut self) -> &mut UnexpectedToken<'a, T, Kind, S, Lang> {
    &mut self.inner
  }

  /// Consumes the error, returning the wrapped [`UnexpectedToken`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_inner(self) -> UnexpectedToken<'a, T, Kind, S, Lang> {
    self.inner
  }

  /// Consumes the error, returning its position and wrapped [`UnexpectedToken`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (SeparatorPosition, UnexpectedToken<'a, T, Kind, S, Lang>) {
    (self.position, self.inner)
  }
}

// Allow unit to be used as an error sink for tests and no-op emitters.
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, Kind, S, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: SeparatedError<'a, T, Kind, S, Lang>) -> Self {}
}

impl<T, Kind: Clone, S, Lang: ?Sized> core::fmt::Debug for SeparatedError<'_, T, Kind, S, Lang>
where
  T: core::fmt::Debug,
  Kind: core::fmt::Debug,
  S: core::fmt::Debug,
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SeparatedError")
      .field("position", &self.position)
      .field("span", self.inner.span_ref())
      .field("found", &self.inner.found())
      .field("expected", &self.inner.expected())
      .finish()
  }
}
