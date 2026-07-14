use super::*;
use crate::lexer::DummyToken;

#[test]
fn token_ref_delegation_kind() {
  let tok = DummyToken;
  let r: &DummyToken = &tok;
  assert_eq!(Token::kind(&r), DummyToken);
}

#[test]
fn token_ref_delegation_is_trivia() {
  let tok = DummyToken;
  let r: &DummyToken = &tok;
  assert!(Token::is_trivia(&r));
}
