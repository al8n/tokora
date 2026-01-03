use super::*;

impl<'a, O, L, E, Lang: ?Sized> TooFewEmitter<'a, O, L, Lang> for Fatal<E, Lang>
where
  O: ?Sized,
  L: Lexer<'a>,
  E: FromTooFewError<'a, O, L, Lang> + FromEmitterError<'a, L, Lang>,
  Fatal<E, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, err: TooFew<O, <L>::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(E::from_too_few(err))
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  const fn assert_noop_too_few_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: TooFewEmitter<'a, Any, L, Error = Error>,
  {
  }

  assert_noop_too_few_emitter::<'_, DummyLexer, (), (), Fatal<()>>();
};
