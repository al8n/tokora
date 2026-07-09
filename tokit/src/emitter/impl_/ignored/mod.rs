use crate::{error::syntax::MissingSyntaxOf, span::Spanned};

use super::super::*;

mod full_container;
mod missing_leading_separator;
mod missing_trailing_separator;
mod pratt;
mod separator;
mod too_few;
mod too_many;
mod unexpected_leading_separator;
mod unexpected_trailing_separator;

/// An emitter that ignores all errors, and the error type is `()`.
///
/// If you want to preserve the error type, use [`Silent`](super::silent::Silent) emitter instead.
pub type Ignored = crate::utils::marker::Ignored<()>;

impl<'a, L, Lang: ?Sized> Emitter<'a, L, Lang> for Ignored {
  type Error = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    _: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, _: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_token(&mut self, _: UnexpectedTokenOf<'a, L, Lang>) -> Result<(), Self::Error>
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

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  struct DummySep;

  const fn assert_noop_separated_by_emitter<'a, L, Sep, Error, E>()
  where
    L: Lexer<'a>,
    E: SeparatedEmitter<'a, Sep, L, Error = Error>,
  {
  }

  assert_noop_separated_by_emitter::<'_, DummyLexer, DummySep, (), Ignored>();
};

#[cfg(test)]
mod tests;
