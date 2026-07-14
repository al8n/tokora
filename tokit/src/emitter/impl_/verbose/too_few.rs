use super::*;

impl<'a, L, S, E, Lang: ?Sized> TooFewEmitter<'a, L, Lang> for Verbose<E, S, Lang>
where
  L: Lexer<'a, Span = S, Offset = S::Offset>,
  E: FromTooFewError<'a, L, Lang> + FromEmitterError<'a, L, Lang>,
  S: Span + Ord + Clone,
{
  #[inline(always)]
  fn emit_too_few(&mut self, err: TooFew<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    self
      .errs
      .entry(err.span_ref().clone())
      .or_default()
      .push(E::from_too_few(err));
    Ok(())
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  const fn assert_noop_too_few_emitter<'a, L, Error, E>()
  where
    L: Lexer<'a>,
    E: TooFewEmitter<'a, L, Error = Error>,
  {
  }

  assert_noop_too_few_emitter::<'_, DummyLexer, (), Verbose<()>>();
};
