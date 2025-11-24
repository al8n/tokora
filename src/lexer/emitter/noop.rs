use crate::utils::Spanned;

use super::{super::Noop, Emitter, Token, Lexer};

impl<'a, L, E> Emitter<'a, L> for Noop<E>
where
  L: Lexer<'a>,
  E: From<<L::Token as Token<'a>>::Error>,
{
  type Error = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_token_error(
    &mut self,
    Spanned { span, data: err }: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Spanned<Self::Error, L::Span>> {
    Err(Spanned {
      span,
      data: err.into(),
    })
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, err: Spanned<Self::Error, L::Span>) -> Result<(), Spanned<Self::Error, L::Span>> {
    Err(err)
  }
}
