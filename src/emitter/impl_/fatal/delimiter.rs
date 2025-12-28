use super::*;

impl<'inp, L, Delim, E, Lang: ?Sized> DelimitedEmitter<'inp, Delim, L, Lang> for Fatal<E, Lang>
where
  L: Lexer<'inp>,
  E: FromDelimitedError<'inp, Delim, L, Lang>,
  Fatal<E, Lang>: Emitter<'inp, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unclosed(&mut self, err: Unclosed<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unclosed(err))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unopened(&mut self, err: Unopened<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_unopened(err))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_undelimited(&mut self, err: Undelimited<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    Err(E::from_undelimited(err))
  }
}
