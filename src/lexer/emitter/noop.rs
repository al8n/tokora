use crate::{
  error::token::UnexpectedToken,
  utils::{Spanned, marker::Noop},
};

use super::*;

impl<'a, L, E> Emitter<'a, L> for Noop<E>
where
  L: Lexer<'a>,
  E: From<<L::Token as Token<'a>>::Error>
    + From<UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span>>,
{
  type Error = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    Spanned { span, data: err }: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>> {
    Err(Spanned {
      span,
      data: err.into(),
    })
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(
    &mut self,
    err: Spanned<Self::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>> {
    Err(err)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_token(
    &mut self,
    err: UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span>,
  ) -> Result<(), Spanned<Self::Error, <L>::Span>>
  where
    L: Lexer<'a>,
  {
    Err(Spanned::new(err.span_ref().clone(), err.into()))
  }
}

impl<'a, O, L, E> RepeatedEmitter<'a, O, L> for Noop<E>
where
  L: Lexer<'a>,
  E: From<TooFew<O, L::Span>> + From<TooMany<O, L::Span>>,
  Noop<E>: Emitter<'a, L, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, err: TooFew<O, L::Span>) -> Result<(), Spanned<Self::Error, <L>::Span>>
  where
    L: Lexer<'a>,
  {
    Err(Spanned::new(err.span().clone(), err.into()))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_many(
    &mut self,
    err: TooMany<O, L::Span>,
  ) -> Result<(), Spanned<Self::Error, <L>::Span>>
  where
    L: Lexer<'a>,
  {
    Err(Spanned::new(err.span().clone(), err.into()))
  }
}

impl<'a, L, Any, E> BatchEmitter<'a, L, Any> for Noop<E>
where
  L: Lexer<'a>,
  E: From<Any>,
  Noop<E>: Emitter<'a, L, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch(&mut self, _: <L>::Span, _: Message)
  where
    L: Lexer<'a>,
  {}

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn create_batch_with_error(
    &mut self,
    _: Message,
    err: Spanned<Any, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Err(err.map_data(Into::into))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_to_batch(
    &mut self,
    _: &<L>::Span,
    err: Spanned<Any, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>>
  where
    L: Lexer<'a>,
  {
    Err(err.map_data(Into::into))
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
  {}
}
