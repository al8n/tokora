#![cfg(feature = "std")]

use tokit::Token;
use tokit::token::LitToken;

// ── Minimal token types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Kind;

impl core::fmt::Display for Kind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "kind")
  }
}

macro_rules! define_lit_tok {
  ($name:ident, $method:ident) => {
    #[derive(Debug, Clone)]
    struct $name;

    impl core::fmt::Display for $name {
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, stringify!($name))
      }
    }

    impl Token<'_> for $name {
      type Kind = Kind;
      type Error = ();
      fn kind(&self) -> Kind { Kind }
      fn is_trivia(&self) -> bool { false }
    }

    impl LitToken<'_> for $name {
      fn $method(&self) -> bool { true }
    }
  };
}

define_lit_tok!(DecTok, is_decimal_literal);
define_lit_tok!(HexTok, is_hexadecimal_literal);
define_lit_tok!(OctTok, is_octal_literal);
define_lit_tok!(BinTok, is_binary_literal);
define_lit_tok!(FloatTok, is_float_literal);
define_lit_tok!(HexFloatTok, is_hex_float_literal);
define_lit_tok!(InlineStrTok, is_inline_string_literal);
define_lit_tok!(MultilineStrTok, is_multiline_string_literal);
define_lit_tok!(RawStrTok, is_raw_string_literal);
define_lit_tok!(CharTok, is_char_literal);
define_lit_tok!(ByteTok, is_byte_literal);
define_lit_tok!(ByteStrTok, is_byte_string_literal);
define_lit_tok!(TrueTok, is_true_literal);
define_lit_tok!(FalseTok, is_false_literal);
define_lit_tok!(NullTok, is_null_literal);

// ── Composite default tests ─────────────────────────────────────────────────

#[test]
fn is_integer_literal_decimal() {
  assert!(DecTok.is_integer_literal());
  assert!(HexTok.is_integer_literal());
  assert!(OctTok.is_integer_literal());
  assert!(BinTok.is_integer_literal());
}

#[test]
fn is_float_literal_both() {
  assert!(FloatTok.is_float_literal());
  assert!(HexFloatTok.is_numeric_literal());
}

#[test]
fn is_numeric_literal_covers_all() {
  assert!(DecTok.is_numeric_literal());
  assert!(FloatTok.is_numeric_literal());
}

#[test]
fn is_string_literal_all_variants() {
  assert!(InlineStrTok.is_string_literal());
  assert!(MultilineStrTok.is_string_literal());
  assert!(RawStrTok.is_raw_string_literal());
}

#[test]
fn is_boolean_literal() {
  assert!(TrueTok.is_boolean_literal());
  assert!(FalseTok.is_boolean_literal());
}

#[test]
fn is_literal_covers_all() {
  assert!(DecTok.is_literal());
  assert!(FloatTok.is_literal());
  assert!(InlineStrTok.is_literal());
  assert!(CharTok.is_literal());
  assert!(ByteTok.is_literal());
  assert!(ByteStrTok.is_literal());
  assert!(TrueTok.is_literal());
  assert!(NullTok.is_literal());
}

// ── Reference delegation ────────────────────────────────────────────────────

#[test]
fn ref_delegates_is_decimal() {
  let tok = DecTok;
  let r: &DecTok = &tok;
  assert!(LitToken::is_decimal_literal(r));
  assert!(LitToken::is_integer_literal(r));
  assert!(LitToken::is_literal(r));
}

#[test]
fn ref_delegates_is_float() {
  let tok = FloatTok;
  let r: &FloatTok = &tok;
  assert!(LitToken::is_float_literal(r));
  assert!(LitToken::is_numeric_literal(r));
}

#[test]
fn ref_delegates_is_string() {
  let tok = InlineStrTok;
  let r: &InlineStrTok = &tok;
  assert!(LitToken::is_inline_string_literal(r));
  assert!(LitToken::is_string_literal(r));
}

#[test]
fn ref_delegates_is_char() {
  let tok = CharTok;
  assert!(LitToken::is_char_literal(&tok));
}

#[test]
fn ref_delegates_is_byte() {
  let tok = ByteTok;
  assert!(LitToken::is_byte_literal(&tok));
}

#[test]
fn ref_delegates_is_byte_string() {
  let tok = ByteStrTok;
  assert!(LitToken::is_byte_string_literal(&tok));
}

#[test]
fn ref_delegates_is_true() {
  let tok = TrueTok;
  assert!(LitToken::is_true_literal(&tok));
}

#[test]
fn ref_delegates_is_false() {
  let tok = FalseTok;
  assert!(LitToken::is_false_literal(&tok));
}

#[test]
fn ref_delegates_is_null() {
  let tok = NullTok;
  assert!(LitToken::is_null_literal(&tok));
}

// ── Default false tests ─────────────────────────────────────────────────────

#[test]
fn all_defaults_false() {
  let tok = DecTok;
  assert!(!tok.is_hexadecimal_literal());
  assert!(!tok.is_octal_literal());
  assert!(!tok.is_binary_literal());
  assert!(!tok.is_float_literal());
  assert!(!tok.is_hex_float_literal());
  assert!(!tok.is_inline_string_literal());
  assert!(!tok.is_multiline_string_literal());
  assert!(!tok.is_raw_string_literal());
  assert!(!tok.is_string_literal());
  assert!(!tok.is_char_literal());
  assert!(!tok.is_byte_literal());
  assert!(!tok.is_byte_string_literal());
  assert!(!tok.is_true_literal());
  assert!(!tok.is_false_literal());
  assert!(!tok.is_boolean_literal());
  assert!(!tok.is_null_literal());
}
