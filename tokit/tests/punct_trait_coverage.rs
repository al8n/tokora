#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use tokit::{
  Lexer,
  punct::{Comma, Punctuator},
};

use common::TestLexer;

// ── Trait default methods ───────────────────────────────────────────────────

#[test]
fn punctuator_description_is_some() {
  let desc = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::description();
  assert!(desc.is_some());
  assert!(!desc.unwrap().as_str().is_empty());
}

#[test]
fn punctuator_name_is_populated() {
  let name = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::name();
  assert!(!name.as_str().is_empty());
}

#[test]
fn punctuator_eval_matches_kind() {
  use common::TokenKind;
  let kind = TokenKind::Comma;
  let matches = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::eval(&kind);
  assert!(matches);
}

#[test]
fn punctuator_eval_rejects_wrong_kind() {
  use common::TokenKind;
  let kind = TokenKind::Num;
  let matches = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::eval(&kind);
  assert!(!matches);
}

// ── Reference delegation ────────────────────────────────────────────────────

#[test]
fn ref_punctuator_name() {
  let name = <&Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::name();
  let orig = <Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::name();
  assert_eq!(name.as_str(), orig.as_str());
}

#[test]
fn ref_punctuator_eval() {
  use common::TokenKind;
  let kind = TokenKind::Comma;
  let matches = <&Comma<(), (), ()> as Punctuator<'_, TestLexer<'_>>>::eval(&kind);
  assert!(matches);
}
