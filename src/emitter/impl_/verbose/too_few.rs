use super::*;

impl<'a, O, L, S, E, Lang: ?Sized> TooFewEmitter<'a, O, L, Lang> for Verbose<E, S, Lang>
where
  O: ?Sized,
  L: Lexer<'a, Span = S, Offset = S::Offset>,
  E: FromTooFewError<'a, O, L, Lang> + FromEmitterError<'a, L, Lang>,
  S: Span + Ord + Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, err: TooFew<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    self
      .errs
      .insert(err.span_ref().clone(), E::from_too_few(err));
    Ok(())
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::{BlackHole, DummyLexer};

  const fn assert_noop_too_few_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: TooFewEmitter<'a, Any, L, Error = Error>,
  {
  }

  assert_noop_too_few_emitter::<'_, DummyLexer, (), (), Verbose<()>>();
  assert_noop_too_few_emitter::<'_, DummyLexer, (), _, Verbose<BlackHole>>();
};
