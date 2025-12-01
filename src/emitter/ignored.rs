use crate::utils::Spanned;

use super::*;

/// An emitter that ignores all errors, and the error type is `()`.
///
/// If you want to preserve the error type, use [`Silent`](super::silent::Silent) emitter instead.
pub type Ignored = crate::utils::marker::Ignored<()>;

impl<'a, L> Emitter<'a, L> for Ignored
where
  L: Lexer<'a>,
{
  type Error = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    _: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Self::Error> {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, _: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error> {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }
}

impl<'a, O, L> RepeatedEmitter<'a, O, L> for Ignored
where
  O: ?Sized,
  L: Lexer<'a>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, _: TooFew<O, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_many(&mut self, _: TooMany<O, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_full_container(&mut self, _: FullContainer<O, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }
}

impl<'a, L, Any> BatchEmitter<'a, L, Any> for Ignored
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
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_to_batch(&mut self, _: &<L>::Span, _: Spanned<Any, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_batch(&mut self, _: &<L>::Span) -> Result<(), Self::Error>
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

impl<'inp, L, O, Sep> SeparatedByEmitter<'inp, O, Sep, L> for Ignored
where
  L: Lexer<'inp>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(&mut self, _: MissingTokenOf<'inp, Sep, L>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(&mut self, _: MissingSyntaxOf<'inp, O, L>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_leading_separator(
    &mut self,
    _: MissingLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_trailing_separator(
    &mut self,
    _: MissingTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_repeated_separator(
    &mut self,
    _: UnexpectedRepeatedOf<'inp, Sep, L>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_leading_separator(
    &mut self,
    _: UnexpectedLeadingOf<'inp, Sep, L>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: UnexpectedTrailingOf<'inp, Sep, L>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}

#[cfg(test)]
const _: () = {
  use crate::DummyLexer;

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

  assert_noop_batch_emitter::<'_, DummyLexer, (), (), Ignored>();
  assert_noop_repeated_emitter::<'_, DummyLexer, (), (), Ignored>();

  assert_noop_separated_by_emitter::<'_, DummyLexer, (), DummySep, (), Ignored>();
};
