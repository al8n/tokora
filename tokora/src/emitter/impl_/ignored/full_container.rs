use super::*;

impl<'a, L, Lang: ?Sized> FullContainerEmitter<'a, L, Lang> for Ignored
where
  L: Lexer<'a>,
{
  #[inline(always)]
  fn emit_full_container(&mut self, _: FullContainer<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
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

  assert_noop_full_container_emitter::<'_, DummyLexer, (), Ignored>();
};
