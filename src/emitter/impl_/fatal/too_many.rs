use super::*;

impl<'a, O, L, E, Lang: ?Sized> TooManyEmitter<'a, O, L, Lang> for Fatal<E, Lang>
where
  O: ?Sized,
  L: Lexer<'a>,
  E: FromTooManyError<'a, O, L, Lang> + FromFullContainerError<'a, O, L, Lang>,
  Fatal<E, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_many(&mut self, err: TooMany<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(E::from_too_many(err))
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::{BlackHole, DummyLexer};

  struct DummySep;

  const fn assert_noop_too_many_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: TooManyEmitter<'a, Any, L, Error = Error>,
  {
  }

  assert_noop_too_many_emitter::<'_, DummyLexer, (), (), Fatal<()>>();
  assert_noop_too_many_emitter::<'_, DummyLexer, (), _, Fatal<BlackHole>>();
};
