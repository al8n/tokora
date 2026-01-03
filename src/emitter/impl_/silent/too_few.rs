use super::*;

impl<'a, O, L, E, Lang: ?Sized> TooFewEmitter<'a, O, L, Lang> for Silent<E, Lang>
where
  O: ?Sized,
  L: Lexer<'a>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_too_few(&mut self, _: TooFew<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  const fn assert_noop_too_few_emitter<'a, L, Any, Error, E>()
  where
    L: Lexer<'a>,
    E: TooFewEmitter<'a, Any, L, Error = Error>,
  {
  }

  assert_noop_too_few_emitter::<'_, DummyLexer, (), (), Silent<()>>();
};
