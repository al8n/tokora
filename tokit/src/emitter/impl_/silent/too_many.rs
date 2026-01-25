use super::*;

impl<'a, L, E, Lang: ?Sized> TooManyEmitter<'a, L, Lang> for Silent<E, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_many(&mut self, _: TooMany<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
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

  assert_noop_too_many_emitter::<'_, DummyLexer, (), Silent<()>>();
};
