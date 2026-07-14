use super::*;
use crate::lexer::DummyToken;

// DummyToken implements PunctuatorToken with all defaults (returning None)

#[test]
fn default_all_punctuator_kinds_none() {
  assert!(DummyToken::open_angle().is_none());
  assert!(DummyToken::close_angle().is_none());
  assert!(DummyToken::open_brace().is_none());
  assert!(DummyToken::close_brace().is_none());
  assert!(DummyToken::open_paren().is_none());
  assert!(DummyToken::close_paren().is_none());
  assert!(DummyToken::open_bracket().is_none());
  assert!(DummyToken::close_bracket().is_none());
  assert!(DummyToken::comma().is_none());
  assert!(DummyToken::dot().is_none());
  assert!(DummyToken::colon().is_none());
  assert!(DummyToken::semicolon().is_none());
  assert!(DummyToken::plus().is_none());
  assert!(DummyToken::minus().is_none());
  assert!(DummyToken::asterisk().is_none());
  assert!(DummyToken::slash().is_none());
  assert!(DummyToken::equal().is_none());
  assert!(DummyToken::exclamation().is_none());
  assert!(DummyToken::question().is_none());
  assert!(DummyToken::hash().is_none());
  assert!(DummyToken::at().is_none());
  assert!(DummyToken::pipe().is_none());
  assert!(DummyToken::ampersand().is_none());
  assert!(DummyToken::caret().is_none());
  assert!(DummyToken::tilde().is_none());
  assert!(DummyToken::underscore().is_none());
  assert!(DummyToken::dollar().is_none());
  assert!(DummyToken::percent().is_none());
  assert!(DummyToken::backslash().is_none());
}

#[test]
fn default_is_punctuator_false() {
  let tok = DummyToken;
  assert!(!tok.is_punctuator());
}

#[test]
fn default_is_predicates_false() {
  let tok = DummyToken;
  assert!(!tok.is_dot());
  assert!(!tok.is_comma());
  assert!(!tok.is_colon());
  assert!(!tok.is_semicolon());
  assert!(!tok.is_plus());
  assert!(!tok.is_minus());
  assert!(!tok.is_asterisk());
  assert!(!tok.is_slash());
  assert!(!tok.is_equal());
  assert!(!tok.is_open_paren());
  assert!(!tok.is_close_paren());
  assert!(!tok.is_open_brace());
  assert!(!tok.is_close_brace());
  assert!(!tok.is_open_bracket());
  assert!(!tok.is_close_bracket());
  assert!(!tok.is_open_angle());
  assert!(!tok.is_close_angle());
}

#[test]
fn ext_aliases() {
  let tok = DummyToken;
  assert!(!tok.is_less_than());
  assert!(!tok.is_greater_than());
  assert!(!tok.is_bang());
  assert!(!tok.is_hyphen());
  assert!(!tok.is_thin_arrow());
  assert!(!tok.is_add_assign());
  assert!(!tok.is_sub_assign());
  assert!(!tok.is_mul_assign());
  assert!(!tok.is_div_assign());
  assert!(!tok.is_exponentiation_assign());
  assert!(!tok.is_bitand_assign());
  assert!(!tok.is_bitor_assign());
  assert!(!tok.is_bitxor_assign());
  assert!(!tok.is_shl_assign());
  assert!(!tok.is_shr_assign());
  assert!(!tok.is_sar_assign());
}
