use crate::{error::token::UnexpectedToken, utils::Spanned};

use super::*;

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
  E: From<<L::Token as Token<'a>>::Error>
    + From<UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>>
    + From<UnexpectedEot<L::Span, Lang>>,
{
  type Error = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    Spanned { data: err, .. }: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Self::Error> {
    Err(err.into())
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
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, _: &Cursor<'a, '_, L>)
  where
    L: Lexer<'a>,
  {
  }
}

impl<'a, O, L, E, Lang: ?Sized> RepeatedEmitter<'a, O, L, Lang> for Fatal<E, Lang>
where
  O: ?Sized,
  L: Lexer<'a>,
  E: From<TooFew<O, L::Span, Lang>>
    + From<TooMany<O, L::Span, Lang>>
    + From<FullContainer<O, L::Span, Lang>>,
  Fatal<E, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, err: TooFew<O, <L>::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_many(&mut self, err: TooMany<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_full_container(&mut self, err: FullContainer<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(err.into())
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

impl<'inp, L, O, Sep, E, Lang: ?Sized> SeparatedByEmitter<'inp, O, Sep, L, Lang> for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: From<MissingSeparatorOf<'inp, Sep, L, Lang>>
    + From<MissingSyntaxOf<'inp, O, L, Lang>>
    + From<MissingLeadingOf<'inp, Sep, L, Lang>>
    + From<MissingTrailingOf<'inp, Sep, L, Lang>>
    + From<UnexpectedLeadingOf<'inp, Sep, L, Lang>>
    + From<UnexpectedTrailingOf<'inp, Sep, L, Lang>>
    + From<UnexpectedRepeatedOf<'inp, Sep, L, Lang>>
    + From<TooFew<O, L::Span, Lang>>
    + From<TooMany<O, L::Span, Lang>>
    + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>,
  Fatal<E, Lang>: Emitter<'inp, L, Lang, Error = E>
    + BatchEmitter<'inp, L, UnexpectedLeadingOf<'inp, Sep, L, Lang>>
    + BatchEmitter<'inp, L, UnexpectedTrailingOf<'inp, Sep, L, Lang>>
    + BatchEmitter<'inp, L, UnexpectedRepeatedOf<'inp, Sep, L, Lang>>
    + BatchEmitter<'inp, L, <L::Token as Token<'inp>>::Error, Lang>
    + RepeatedEmitter<'inp, O, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    err: MissingSeparatorOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    err: MissingSyntaxOf<'inp, O, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_leading_separator(
    &mut self,
    err: MissingLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_trailing_separator(
    &mut self,
    err: MissingTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_repeated_separator(
    &mut self,
    err: UnexpectedRepeatedOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    err: UnexpectedLeadingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    err: UnexpectedTrailingOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
  }
}

impl<'inp, L, Delim, E, Lang: ?Sized> DelimiterEmitter<'inp, Delim, L, Lang> for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: From<Unclosed<Delim, L::Span, Lang>>
    + From<Unopened<Delim, L::Span, Lang>>
    + From<Undelimited<Delim, L::Span, Lang>>,
  Fatal<E, Lang>: Emitter<'inp, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unclosed(&mut self, err: Unclosed<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unopened(&mut self, err: Unopened<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_undelimited(&mut self, err: Undelimited<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(err.into())
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

  const fn assert_noop_repeated_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: RepeatedEmitter<'a, Any, L, Error = Error>,
  {
  }

  const fn assert_noop_separated_by_emitter<'a, L, O, Sep, Error, E>()
  where
    L: Lexer<'a>,
    E: SeparatedByEmitter<'a, O, Sep, L, Error = Error>,
  {
  }

  assert_noop_batch_emitter::<'_, DummyLexer, (), (), Fatal<()>>();
  assert_noop_batch_emitter::<'_, DummyLexer, (), BlackHole, Fatal<BlackHole>>();

  assert_noop_repeated_emitter::<'_, DummyLexer, (), (), Fatal<()>>();

  assert_noop_separated_by_emitter::<'_, DummyLexer, (), DummySep, (), Fatal<()>>();
};
