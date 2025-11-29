use crate::utils::Spanned;

use super::*;

/// A silent emitter that treats all errors as non-fatal, and ignores them.
/// 
/// Compared to [`Ignored`](super::ignored::Ignored) emitter, the error type is preserved.
pub struct Silent<T: ?Sized>(core::marker::PhantomData<T>);

impl<T: ?Sized> Silent<T> {
  /// Creates a new `Silent`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self(core::marker::PhantomData)
  }
}

impl<T: ?Sized> Default for Silent<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

impl<T: ?Sized> core::fmt::Debug for Silent<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Silent")
  }
}

impl<T: ?Sized> Clone for Silent<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: ?Sized> Copy for Silent<T> {}

impl<'a, L, E> Emitter<'a, L> for Silent<E>
where
  L: Lexer<'a>,
{
  type Error
    = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    _: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>> {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(
    &mut self,
    _: Spanned<Self::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>> {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span>,
  ) -> Result<(), Spanned<Self::Error, <L>::Span>>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }
}

impl<'a, O, L, E> RepeatedEmitter<'a, O, L> for Silent<E>
where
  O: ?Sized,
  L: Lexer<'a>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, _: TooFew<O, L::Span>) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_many(&mut self, _: TooMany<O, L::Span>) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_full_container(
    &mut self,
    _: FullContainer<O, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }
}

impl<'a, L, Any, E> BatchEmitter<'a, L, Any> for Silent<E>
where
  L: Lexer<'a>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch(&mut self, _: <L>::Span, _: Message)
  where
    L: Lexer<'a>,
  {
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch_with_error(
    &mut self,
    _: Message,
    _: Spanned<Any, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_to_batch(
    &mut self,
    _: &<L>::Span,
    _: Spanned<Any, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_batch(&mut self, _: &<L>::Span) -> Result<(), Spanned<Self::Error, L::Span>>
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

impl<'inp, L, O, Sep, E> SeparatedByEmitter<'inp, O, Sep, L> for Silent<E>
where
  L: Lexer<'inp>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    _: MissingTokenOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    _: MissingSyntaxOf<'inp, O, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_leading_separator(
    &mut self,
    _: MissingLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_trailing_separator(
    &mut self,
    _: MissingTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_repeated_separator(
    &mut self,
    _: UnexpectedRepeatedOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    _: UnexpectedLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: UnexpectedTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}

#[cfg(test)]
const _: () = {
  use crate::{BlackHole, Check, DummyLexer, DummyToken, SeqSepAction};

  struct DummySep;

  impl<'inp> Check<DummyToken, SeqSepAction<'inp, DummyToken>> for DummySep {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn check(&self, _: &DummyToken) -> SeqSepAction<'inp, DummyToken> {
      unimplemented!()
    }
  }

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

  assert_noop_batch_emitter::<'_, DummyLexer, (), (), Silent<()>>();
  assert_noop_batch_emitter::<'_, DummyLexer, (), BlackHole, Silent<BlackHole>>();

  assert_noop_repeated_emitter::<'_, DummyLexer, (), (), Silent<()>>();

  assert_noop_separated_by_emitter::<'_, DummyLexer, (), DummySep, (), Silent<()>>();
};
