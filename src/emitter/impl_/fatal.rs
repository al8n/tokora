use crate::{
  error::{
    syntax::MissingSyntaxOf,
    token::{UnexpectedLeadingOf, UnexpectedToken, UnexpectedTrailingOf},
  },
  span::Spanned,
};

use super::super::{
  separated::{
    FromUnexpectedLeadingSeparatorError, FromUnexpectedTrailingSeparatorError,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  *,
};

mod delimiter;
mod full_container;
mod separator;
mod too_few;
mod too_many;
mod unexpected_leading_separator;
mod unexpected_trailing_separator;

/// A fatal emitter that treats all errors as fatal.
///
/// This will make the parser stop at the first error encountered, so it is a fail-fast emitter,
/// suitable for scenarios where error recovery is not desired.
///
/// `Fatal` is a **complete implementation** of all atomic emitter traits, providing a pre-built bundle
/// for fail-fast parsing. It implements all emitter traits ([`Emitter`](super::super::Emitter),
/// [`TooFewEmitter`](super::super::TooFewEmitter), [`TooManyEmitter`](super::super::TooManyEmitter),
/// [`DelimitedEmitter`](super::super::DelimitedEmitter), etc.) with consistent fail-fast behavior.
///
/// For custom error handling, you can implement only the atomic emitter traits you need rather than
/// using this pre-built bundle.
pub struct Fatal<T: ?Sized, Lang: ?Sized = ()> {
  _e: core::marker::PhantomData<T>,
  _lang: core::marker::PhantomData<Lang>,
}

impl<T: ?Sized, Lang: ?Sized> Clone for Fatal<T, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: ?Sized, Lang: ?Sized> Copy for Fatal<T, Lang> {}

impl<T: ?Sized> Default for Fatal<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl<T: ?Sized> core::fmt::Debug for Fatal<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Fatal")
  }
}

impl<T: ?Sized> Fatal<T> {
  /// Creates a new `Fatal`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self::of()
  }
}

impl<T: ?Sized, Lang: ?Sized> Fatal<T, Lang> {
  /// Creates a new `Fatal` for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of() -> Self {
    Self {
      _e: core::marker::PhantomData,
      _lang: core::marker::PhantomData,
    }
  }
}

impl<'a, L, E, Lang: ?Sized> Emitter<'a, L, Lang> for Fatal<E, Lang>
where
  L: Lexer<'a>,
  E: FromEmitterError<'a, L, Lang>,
{
  type Error = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    err: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Self::Error> {
    Err(E::from_lexer_error(err))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, err: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(err.into_data())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_token(
    &mut self,
    err: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(E::from_unexpected_token(err))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, _: &Cursor<'a, '_, L>)
  where
    L: Lexer<'a>,
  {
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  struct DummySep;

  const fn assert_noop_separated_by_emitter<'a, L, O, Sep, Error, E>()
  where
    L: Lexer<'a>,
    E: SeparatedEmitter<'a, O, Sep, L, Error = Error>,
  {
  }

  assert_noop_separated_by_emitter::<'_, DummyLexer, (), DummySep, (), Fatal<()>>();
};
