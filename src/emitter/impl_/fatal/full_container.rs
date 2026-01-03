use super::*;

impl<'a, L, E, Lang: ?Sized> FullContainerEmitter<'a, L, Lang> for Fatal<E, Lang>
where
  L: Lexer<'a>,
  E: FromFullContainerError<'a, L, Lang>,
  Fatal<E, Lang>: Emitter<'a, L, Lang, Error = E>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_full_container(&mut self, err: FullContainer<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Err(E::from_full_container(err))
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  const fn assert_noop_full_container_emitter<'a, L, Error, E>()
  where
    L: Lexer<'a>,
    E: FullContainerEmitter<'a, L, Error = Error>,
  {
  }

  assert_noop_full_container_emitter::<'_, DummyLexer, (), Fatal<()>>();
};
