use super::*;
use crate::lexer::DummyLexer;
use crate::span::SimpleSpan;
use std::format;

#[test]
fn silent_new() {
  let _s: Silent<()> = Silent::new();
}

#[test]
fn silent_default() {
  let _s: Silent<()> = Silent::default();
}

#[test]
fn silent_debug() {
  let s: Silent<()> = Silent::new();
  assert_eq!(format!("{:?}", s), "Silent");
}

#[test]
fn silent_clone_and_copy() {
  let s: Silent<()> = Silent::new();
  let s2 = s.clone();
  let s3 = s;
  let _ = (s2, s3);
}

#[test]
fn silent_emit_lexer_error_returns_ok() {
  let mut s: Silent<()> = Silent::new();
  let spanned = Spanned::new(SimpleSpan::new(0usize, 5usize), ());
  let result = <Silent<()> as Emitter<'_, DummyLexer>>::emit_lexer_error(&mut s, spanned);
  assert!(result.is_ok());
}

#[test]
fn silent_emit_error_returns_ok() {
  let mut s: Silent<()> = Silent::new();
  let spanned = Spanned::new(SimpleSpan::new(0usize, 5usize), ());
  let result = <Silent<()> as Emitter<'_, DummyLexer>>::emit_error(&mut s, spanned);
  assert!(result.is_ok());
}

#[test]
fn silent_emit_unexpected_token_returns_ok() {
  use crate::error::token::UnexpectedToken;
  use crate::lexer::DummyToken;

  let mut s: Silent<()> = Silent::new();
  let ut: UnexpectedToken<'_, DummyToken, DummyToken, SimpleSpan> =
    UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Silent<()> as Emitter<'_, DummyLexer>>::emit_unexpected_token(&mut s, ut);
  assert!(result.is_ok());
}

#[test]
fn silent_with_lang_type() {
  struct MyLang;
  let _s: Silent<(), MyLang> = Silent::new();
  let _s2: Silent<(), MyLang> = Silent::default();
  assert_eq!(format!("{:?}", _s), "Silent");
}
