use super::*;

impl<'inp, L, E, Delim, Lang: ?Sized> DelimitedEmitter<'inp, Delim, L, Lang> for Silent<E, Lang>
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
