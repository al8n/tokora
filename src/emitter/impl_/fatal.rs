use crate::{
  error::{
    syntax::MissingSyntaxOf,
    token::{UnexpectedLeadingOf, UnexpectedToken, UnexpectedTrailingOf},
  },
  utils::Spanned,
};

use super::super::separated::{
  FromUnexpectedLeadingSeparatorError, FromUnexpectedTrailingSeparatorError,
  UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
};
use super::super::*;

mod full_container;
mod too_few;
mod too_many;

/// A fatal emitter that treats all errors as fatal.
///
/// This will make the parser stop at the first error encountered, so it is a fail-fast emitter,
/// suitable for scenarios where error recovery is not desired.
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

impl<'a, L, Any, E, Lang: ?Sized> BatchEmitter<'a, L, Any, Lang> for Fatal<E, Lang>
where
  L: Lexer<'a>,
  E: From<Any>,
  Fatal<E, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch(&mut self, _: L::Span, _: Message)
  where
    L: Lexer<'a>,
  {
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch_with_error(
    &mut self,
    _: Message,
    err: Spanned<Any, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(err.into_data().into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_to_batch(&mut self, _: &L::Span, err: Spanned<Any, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(err.into_data().into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_batch(&mut self, _: &L::Span) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn drop_batch(&mut self, _: &<L>::Span)
  where
    L: Lexer<'a>,
  {
  }
}

impl<'inp, L, O, Sep, E, Lang: ?Sized> SeparatedEmitter<'inp, O, Sep, L, Lang> for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: FromSeparatedError<'inp, O, Sep, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    err: MissingSeparatorOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_missing_separator(err))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, O, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_missing_element(err))
  }
}

impl<'inp, L, Delim, E, Lang: ?Sized> DelimitedEmitter<'inp, Delim, L, Lang> for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: FromDelimitedError<'inp, Delim, L, Lang>,
  Fatal<E, Lang>: Emitter<'inp, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unclosed(&mut self, err: Unclosed<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unclosed(err))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unopened(&mut self, err: Unopened<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unopened(err))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_undelimited(&mut self, err: Undelimited<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_undelimited(err))
  }
}

impl<'inp, L, O, Sep, E, Lang: ?Sized> UnexpectedLeadingSeparatorEmitter<'inp, O, Sep, L, Lang>
  for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: FromUnexpectedLeadingSeparatorError<'inp, O, Sep, L, Lang>,
  Fatal<E, Lang>: SeparatedEmitter<'inp, O, Sep, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    err: UnexpectedLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unexpected_leading_separator(err))
  }
}

impl<'inp, L, O, Sep, E, Lang: ?Sized> UnexpectedTrailingSeparatorEmitter<'inp, O, Sep, L, Lang>
  for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: FromUnexpectedTrailingSeparatorError<'inp, O, Sep, L, Lang>,
  Fatal<E, Lang>: SeparatedEmitter<'inp, O, Sep, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    err: UnexpectedTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unexpected_trailing_separator(err))
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::{BlackHole, DummyLexer};

  struct DummySep;

  const fn assert_noop_batch_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: BatchEmitter<'a, L, Any, Error = Error>,
  {
  }

  const fn assert_noop_separated_by_emitter<'a, L, O, Sep, Error, E>()
  where
    L: Lexer<'a>,
    E: SeparatedEmitter<'a, O, Sep, L, Error = Error>,
  {
  }

  assert_noop_batch_emitter::<'_, DummyLexer, (), (), Fatal<()>>();
  assert_noop_batch_emitter::<'_, DummyLexer, (), BlackHole, Fatal<BlackHole>>();
  assert_noop_separated_by_emitter::<'_, DummyLexer, (), DummySep, (), Fatal<()>>();
};
