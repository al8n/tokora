use super::*;

impl<'a, O, L, S, E, Lang: ?Sized> FullContainerEmitter<'a, O, L, Lang> for Verbose<E, S, Lang>
where
  O: ?Sized,
  L: Lexer<'a, Span = S, Offset = S::Offset>,
  E: FromFullContainerError<'a, O, L, Lang>,
  S: Span + Ord + Clone,
  Verbose<E, S, Lang>: Emitter<'a, L, Lang, Error = E>,
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

  const fn assert_noop_full_container_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: FullContainerEmitter<'a, Any, L, Error = Error>,
  {
  }

  assert_noop_full_container_emitter::<'_, DummyLexer, (), (), Verbose<()>>();
  assert_noop_full_container_emitter::<'_, DummyLexer, (), _, Verbose<BlackHole>>();
};
