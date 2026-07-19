use super::*;

use crate::error::Unclosed;

impl<'a, L, E, Lang: ?Sized> UnclosedEmitter<'a, L, Lang> for Silent<E, Lang>
where
  L: Lexer<'a>,
{
  #[inline(always)]
  fn emit_unclosed<Delimiter>(
    &mut self,
    _: Unclosed<Delimiter, L::Span, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
    Self::Error: From<Unclosed<Delimiter, L::Span, Lang>>,
  {
    Ok(())
  }
}
