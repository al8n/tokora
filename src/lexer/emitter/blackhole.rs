use crate::utils::Spanned;

use super::{super::BlackHole, Emitter, Token};

impl<'a, T, S> Emitter<'a, T, S> for BlackHole
where
  T: Token<'a>,
{
  type Error = core::convert::Infallible;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_token_error(&mut self, _err: Spanned<T::Error, S>) -> Result<(), Self::Error> {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, _err: Spanned<Self::Error, S>) -> Result<(), Self::Error> {
    Ok(())
  }
}
