use crate::{error::syntax::MissingSyntaxOf, utils::Spanned};

use super::super::*;

mod full_container;
mod too_few;
mod too_many;

/// An emitter that ignores all errors, and the error type is `()`.
///
/// If you want to preserve the error type, use [`Silent`](super::silent::Silent) emitter instead.
pub type Ignored = crate::utils::marker::Ignored<()>;

impl<'a, L, Lang: ?Sized> Emitter<'a, L, Lang> for Ignored
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
    _: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, _: &Cursor<'a, '_, L>)
  where
    L: Lexer<'a>,
  {
  }
}

impl<'a, L, Any, Lang: ?Sized> BatchEmitter<'a, L, Any, Lang> for Ignored
where
  L: Lexer<'a>,
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
    _: Spanned<Any, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_to_batch(&mut self, _: &L::Span, _: Spanned<Any, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_batch(&mut self, _: &L::Span) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn drop_batch(&mut self, _: &L::Span)
  where
    L: Lexer<'a>,
  {
  }
}

impl<'inp, L, O, Sep, Lang: ?Sized> SeparatedEmitter<'inp, O, Sep, L, Lang> for Ignored
where
  L: Lexer<'inp>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_separator(
    &mut self,
    _: MissingSeparatorOf<'inp, Sep, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_missing_element(
    &mut self,
    _: MissingSyntaxOf<'inp, O, L, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}

impl<'inp, L, Delim, Lang: ?Sized> DelimitedEmitter<'inp, Delim, L, Lang> for Ignored
where
  L: Lexer<'inp>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unclosed(&mut self, _: Unclosed<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unopened(&mut self, _: Unopened<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_undelimited(&mut self, _: Undelimited<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Ok(())
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

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

  assert_noop_batch_emitter::<'_, DummyLexer, (), (), Ignored>();
  assert_noop_separated_by_emitter::<'_, DummyLexer, (), DummySep, (), Ignored>();
};
