use super::*;
use crate::lexer::DummyLexer;
use crate::span::SimpleSpan;

#[test]
fn ignored_emit_lexer_error_returns_ok() {
  let mut ign = Ignored::default();
  let spanned = Spanned::new(SimpleSpan::new(0usize, 5usize), ());
  let result = <Ignored as Emitter<'_, DummyLexer>>::emit_lexer_error(&mut ign, spanned);
  assert!(result.is_ok());
}

#[test]
fn ignored_emit_error_returns_ok() {
  let mut ign = Ignored::default();
  let spanned = Spanned::new(SimpleSpan::new(0usize, 5usize), ());
  let result = <Ignored as Emitter<'_, DummyLexer>>::emit_error(&mut ign, spanned);
  assert!(result.is_ok());
}

#[test]
fn ignored_from_unit() {
  let ign: Ignored = ().into();
  let _ = ign;
}
