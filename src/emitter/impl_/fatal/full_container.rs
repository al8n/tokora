use super::*;

impl<'a, O, L, E, Lang: ?Sized> FullContainerEmitter<'a, O, L, Lang> for Fatal<E, Lang>
where
  O: ?Sized,
  L: Lexer<'a>,
  E: FromFullContainerError<'a, O, L, Lang>,
  Fatal<E, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_full_container(&mut self, err: FullContainer<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(E::from_full_container(err))
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::{BlackHole, DummyLexer};

  struct DummySep;

  const fn assert_noop_full_container_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: FullContainerEmitter<'a, Any, L, Error = Error>,
  {
  }

  assert_noop_full_container_emitter::<'_, DummyLexer, (), (), Fatal<()>>();
  assert_noop_full_container_emitter::<'_, DummyLexer, (), _, Fatal<BlackHole>>();
};
