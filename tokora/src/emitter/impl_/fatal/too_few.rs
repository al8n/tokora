use super::*;

impl<'a, L, E, Lang: ?Sized> TooFewEmitter<'a, L, Lang> for Fatal<E, Lang>
where
  E: FromTooFewError<'a, L, Lang> + FromEmitterError<'a, L, Lang>,
  Fatal<E, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[inline(always)]
  fn emit_too_few(&mut self, err: TooFew<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(E::from_too_few(err))
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

  assert_noop_too_few_emitter::<'_, DummyLexer, (), Fatal<()>>();
};
