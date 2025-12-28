use super::*;

impl<'inp, L, Delim, S, E, Lang: ?Sized> DelimitedEmitter<'inp, Delim, L, Lang>
  for Verbose<E, S, Lang>
where
  L: Lexer<'inp, Span = S, Offset = S::Offset>,
  E: FromDelimitedError<'inp, Delim, L, Lang>,
  S: Span + Ord + Clone,
  Verbose<E, S, Lang>: Emitter<'inp, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unclosed(&mut self, err: Unclosed<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self
      .errs
      .insert(err.span_ref().clone(), E::from_unclosed(err));
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unopened(&mut self, err: Unopened<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self
      .errs
      .insert(err.span_ref().clone(), E::from_unopened(err));
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_undelimited(&mut self, err: Undelimited<Delim, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'inp>,
  {
    self
      .errs
      .insert(err.span_ref().clone(), E::from_undelimited(err));
    Ok(())
  }
}
