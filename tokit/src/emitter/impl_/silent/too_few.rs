use super::*;

impl<'a, L, E, Lang: ?Sized> TooFewEmitter<'a, L, Lang> for Silent<E, Lang> {
  #[inline(always)]
  fn emit_too_few(&mut self, _: TooFew<L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
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

  assert_noop_too_few_emitter::<'_, DummyLexer, (), Silent<()>>();
};
