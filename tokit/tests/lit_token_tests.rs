use core::fmt;
/// Tests for `token::LitToken` trait.
///
/// We define a small token type with distinct literal kinds, implement
/// `LitToken` with every override, then verify each predicate.
use tokit::{Token, token::LitToken};

// ── Token infrastructure ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
  Decimal,
  Hex,
  Octal,
  Binary,
  Float,
  HexFloat,
  InlineStr,
  MultilineStr,
  RawStr,
  Char,
  Byte,
  ByteStr,
  True,
  False,
  Null,
  Other,
}

impl fmt::Display for Kind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{self:?}")
  }
}

#[derive(Debug, Clone, PartialEq)]
struct Tok {
  kind: Kind,
}

impl Tok {
  fn new(kind: Kind) -> Self {
    Tok { kind }
  }
}

impl Token<'_> for Tok {
  type Kind = Kind;
  type Error = ();

  fn kind(&self) -> Kind {
    self.kind
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

impl LitToken<'_> for Tok {
  fn is_decimal_literal(&self) -> bool {
    self.kind == Kind::Decimal
  }
  fn is_hexadecimal_literal(&self) -> bool {
    self.kind == Kind::Hex
  }
  fn is_octal_literal(&self) -> bool {
    self.kind == Kind::Octal
  }
  fn is_binary_literal(&self) -> bool {
    self.kind == Kind::Binary
  }
  fn is_float_literal(&self) -> bool {
    self.kind == Kind::Float
  }
  fn is_hex_float_literal(&self) -> bool {
    self.kind == Kind::HexFloat
  }
  fn is_inline_string_literal(&self) -> bool {
    self.kind == Kind::InlineStr
  }
  fn is_multiline_string_literal(&self) -> bool {
    self.kind == Kind::MultilineStr
  }
  fn is_raw_string_literal(&self) -> bool {
    self.kind == Kind::RawStr
  }
  fn is_char_literal(&self) -> bool {
    self.kind == Kind::Char
  }
  fn is_byte_literal(&self) -> bool {
    self.kind == Kind::Byte
  }
  fn is_byte_string_literal(&self) -> bool {
    self.kind == Kind::ByteStr
  }
  fn is_true_literal(&self) -> bool {
    self.kind == Kind::True
  }
  fn is_false_literal(&self) -> bool {
    self.kind == Kind::False
  }
  fn is_null_literal(&self) -> bool {
    self.kind == Kind::Null
  }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn decimal_is_integer() {
  let t = Tok::new(Kind::Decimal);
  assert!(t.is_decimal_literal());
  assert!(t.is_integer_literal());
  assert!(t.is_numeric_literal());
  assert!(t.is_literal());
  assert!(!t.is_float_literal());
  assert!(!t.is_hex_float_literal());
}

#[test]
fn hex_is_integer() {
  let t = Tok::new(Kind::Hex);
  assert!(t.is_hexadecimal_literal());
  assert!(t.is_integer_literal());
  assert!(t.is_numeric_literal());
  assert!(t.is_literal());
  assert!(!t.is_decimal_literal());
}

#[test]
fn octal_is_integer() {
  let t = Tok::new(Kind::Octal);
  assert!(t.is_octal_literal());
  assert!(t.is_integer_literal());
  assert!(!t.is_binary_literal());
}

#[test]
fn binary_is_integer() {
  let t = Tok::new(Kind::Binary);
  assert!(t.is_binary_literal());
  assert!(t.is_integer_literal());
}

#[test]
fn float_is_numeric_not_integer() {
  let t = Tok::new(Kind::Float);
  assert!(t.is_float_literal());
  assert!(t.is_numeric_literal());
  assert!(!t.is_integer_literal());
}

#[test]
fn hex_float_is_numeric() {
  let t = Tok::new(Kind::HexFloat);
  assert!(t.is_hex_float_literal());
  assert!(t.is_numeric_literal());
  assert!(!t.is_float_literal());
}

#[test]
fn inline_string_is_string() {
  let t = Tok::new(Kind::InlineStr);
  assert!(t.is_inline_string_literal());
  assert!(t.is_string_literal());
  assert!(t.is_literal());
  assert!(!t.is_multiline_string_literal());
}

#[test]
fn multiline_string_is_string() {
  let t = Tok::new(Kind::MultilineStr);
  assert!(t.is_multiline_string_literal());
  assert!(t.is_string_literal());
  assert!(!t.is_inline_string_literal());
}

#[test]
fn raw_string_is_literal() {
  let t = Tok::new(Kind::RawStr);
  assert!(t.is_raw_string_literal());
  assert!(t.is_literal());
  assert!(!t.is_string_literal()); // raw is separate
}

#[test]
fn char_literal() {
  let t = Tok::new(Kind::Char);
  assert!(t.is_char_literal());
  assert!(t.is_literal());
}

#[test]
fn byte_literal() {
  let t = Tok::new(Kind::Byte);
  assert!(t.is_byte_literal());
  assert!(t.is_literal());
}

#[test]
fn byte_string_literal() {
  let t = Tok::new(Kind::ByteStr);
  assert!(t.is_byte_string_literal());
  assert!(t.is_literal());
}

#[test]
fn true_literal_is_boolean() {
  let t = Tok::new(Kind::True);
  assert!(t.is_true_literal());
  assert!(t.is_boolean_literal());
  assert!(t.is_literal());
  assert!(!t.is_false_literal());
}

#[test]
fn false_literal_is_boolean() {
  let t = Tok::new(Kind::False);
  assert!(t.is_false_literal());
  assert!(t.is_boolean_literal());
  assert!(!t.is_true_literal());
}

#[test]
fn null_literal() {
  let t = Tok::new(Kind::Null);
  assert!(t.is_null_literal());
  assert!(t.is_literal());
}

#[test]
fn other_is_not_literal() {
  let t = Tok::new(Kind::Other);
  assert!(!t.is_literal());
  assert!(!t.is_numeric_literal());
  assert!(!t.is_boolean_literal());
  assert!(!t.is_null_literal());
}

// ── Delegation through &T ────────────────────────────────────────────────────

#[test]
fn ref_delegates_decimal() {
  let t = Tok::new(Kind::Decimal);
  let r = &t;
  assert!(r.is_decimal_literal());
  assert!(r.is_integer_literal());
  assert!(r.is_numeric_literal());
  assert!(r.is_literal());
}

#[test]
fn ref_delegates_float() {
  let t = Tok::new(Kind::Float);
  let r = &t;
  assert!(r.is_float_literal());
  assert!(!r.is_integer_literal());
}

#[test]
fn ref_delegates_hex_float() {
  let t = Tok::new(Kind::HexFloat);
  let r = &t;
  assert!(r.is_hex_float_literal());
  assert!(r.is_numeric_literal());
}

#[test]
fn ref_delegates_inline_string() {
  let t = Tok::new(Kind::InlineStr);
  let r = &t;
  assert!(r.is_inline_string_literal());
  assert!(r.is_string_literal());
}

#[test]
fn ref_delegates_multiline_string() {
  let t = Tok::new(Kind::MultilineStr);
  let r = &t;
  assert!(r.is_multiline_string_literal());
}

#[test]
fn ref_delegates_raw_string() {
  let t = Tok::new(Kind::RawStr);
  let r = &t;
  assert!(r.is_raw_string_literal());
}

#[test]
fn ref_delegates_char() {
  let t = Tok::new(Kind::Char);
  assert!((&t).is_char_literal());
}

#[test]
fn ref_delegates_byte() {
  let t = Tok::new(Kind::Byte);
  assert!((&t).is_byte_literal());
}

#[test]
fn ref_delegates_byte_string() {
  let t = Tok::new(Kind::ByteStr);
  assert!((&t).is_byte_string_literal());
}

#[test]
fn ref_delegates_boolean() {
  let t = Tok::new(Kind::True);
  let r = &t;
  assert!(r.is_true_literal());
  assert!(r.is_boolean_literal());
}

#[test]
fn ref_delegates_false() {
  let t = Tok::new(Kind::False);
  let r = &t;
  assert!(r.is_false_literal());
  assert!(r.is_boolean_literal());
}

#[test]
fn ref_delegates_null() {
  let t = Tok::new(Kind::Null);
  assert!((&t).is_null_literal());
}

#[test]
fn ref_delegates_other_not_literal() {
  let t = Tok::new(Kind::Other);
  assert!(!(&t).is_literal());
}

// ── Default implementations (all false) ─────────────────────────────────────

/// A minimal token that does not override any LitToken method.
#[derive(Debug, Clone, PartialEq)]
struct MinimalTok;

impl Token<'_> for MinimalTok {
  type Kind = Kind;
  type Error = ();

  fn kind(&self) -> Kind {
    Kind::Other
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

impl LitToken<'_> for MinimalTok {}

#[test]
fn default_all_false() {
  let t = MinimalTok;
  assert!(!t.is_literal());
  assert!(!t.is_numeric_literal());
  assert!(!t.is_integer_literal());
  assert!(!t.is_decimal_literal());
  assert!(!t.is_hexadecimal_literal());
  assert!(!t.is_octal_literal());
  assert!(!t.is_binary_literal());
  assert!(!t.is_float_literal());
  assert!(!t.is_hex_float_literal());
  assert!(!t.is_string_literal());
  assert!(!t.is_inline_string_literal());
  assert!(!t.is_multiline_string_literal());
  assert!(!t.is_raw_string_literal());
  assert!(!t.is_char_literal());
  assert!(!t.is_byte_literal());
  assert!(!t.is_byte_string_literal());
  assert!(!t.is_boolean_literal());
  assert!(!t.is_true_literal());
  assert!(!t.is_false_literal());
  assert!(!t.is_null_literal());
}
