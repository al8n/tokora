use crate::utils::Spanned;

use super::{super::Noop, Emitter, Token};

impl<'a, T, S, E> Emitter<'a, T, S> for Noop<E>
where
  T: Token<'a>,
  E: From<T::Error>,
{
  type Error = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_token_error(&mut self, Spanned { span, data: err }: Spanned<T::Error, S>) -> Result<(), Spanned<Self::Error, S>> {
    Err(Spanned {
      span,
      data: err.into(),
    })
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, err: Spanned<Self::Error, S>) -> Result<(), Spanned<Self::Error, S>> {
    Err(err)
  }
}
