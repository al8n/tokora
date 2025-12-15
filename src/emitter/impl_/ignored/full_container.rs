use super::*;

impl<'a, O, L, Lang: ?Sized> FullContainerEmitter<'a, O, L, Lang> for Ignored
where
  O: ?Sized,
  L: Lexer<'a>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_full_container(&mut self, _: FullContainer<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  struct DummySep;

  const fn assert_noop_full_container_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: FullContainerEmitter<'a, Any, L, Error = Error>,
  {
  }

  assert_noop_full_container_emitter::<'_, DummyLexer, (), (), Ignored>();
};
