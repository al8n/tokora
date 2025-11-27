use crate::{error::token::UnexpectedToken, utils::Spanned};

use super::{super::Noop, *};

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
