use super::*;

impl<'a, L, E, Lang: ?Sized> TooManyEmitter<'a, L, Lang> for Fatal<E, Lang>
where
  E: FromTooManyError<'a, L, Lang> + FromEmitterError<'a, L, Lang>,
  Fatal<E, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[inline(always)]
  fn emit_too_many(&mut self, err: TooMany<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(E::from_too_many(err))
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

  assert_noop_too_many_emitter::<'_, DummyLexer, (), Fatal<()>>();
};
