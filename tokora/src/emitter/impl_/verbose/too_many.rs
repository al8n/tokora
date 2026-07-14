use super::*;

impl<'a, L, S, E, Lang: ?Sized> TooManyEmitter<'a, L, Lang> for Verbose<E, S, Lang>
where
  L: Lexer<'a, Span = S, Offset = S::Offset>,
  E: FromTooManyError<'a, L, Lang> + FromEmitterError<'a, L, Lang>,
  S: Span + Ord + Clone,
{
  #[inline(always)]
  fn emit_too_many(&mut self, err: TooMany<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    self
      .errs
      .entry(err.span_ref().clone())
      .or_default()
      .push(E::from_too_many(err));
    Ok(())
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  const fn assert_noop_too_many_emitter<'a, L, Error, E>()
  where
    L: Lexer<'a>,
    E: TooManyEmitter<'a, L, Error = Error>,
  {
  }

  assert_noop_too_many_emitter::<'_, DummyLexer, (), Verbose<()>>();
};
