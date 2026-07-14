use super::*;
use crate::lexer::DummyLexer;

#[test]
fn test_default_context() {
  fn assert_context<'inp, Ctx>()
  where
    Ctx: ParseContext<'inp, DummyLexer>,
  {
  }

  assert_context::<()>();
  assert_context::<FatalContext<'_, DummyLexer, ()>>();
}

#[test]
fn test_custom_context() {
  fn assert_context<'inp, Ctx>()
  where
    Ctx: ParseContext<'inp, DummyLexer>,
  {
  }

  assert_context::<(Fatal<()>, DefaultCache<'_, DummyLexer>)>();
}
