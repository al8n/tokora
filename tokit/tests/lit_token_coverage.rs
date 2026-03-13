/// Tests that explicitly exercise the `impl<'a, T> LitToken<'a> for &'a T` blanket impl.
///
/// Each test uses UFCS syntax `<&Tok as LitToken>::method(&&tok)` so that the call goes
/// through the blanket impl rather than auto-derefing to the concrete type's impl.
use core::fmt;
use tokit::{Token, token::LitToken};

// ── Token infrastructure ──────────────────────────────────────────────────────

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

// ── Tests through blanket &T impl using UFCS ─────────────────────────────────
//
// `<&Tok as LitToken>::method(&&tok)` forces dispatch through
// `impl<'a, T> LitToken<'a> for &'a T` rather than auto-derefing to `Tok`.

#[test]
fn blanket_ref_is_literal() {
  let tok = Tok::new(Kind::Decimal);
  assert!(<&Tok as LitToken>::is_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_literal() {
  let tok = Tok::new(Kind::Other);
  assert!(!<&Tok as LitToken>::is_literal(&&tok));
}

#[test]
fn blanket_ref_is_numeric_literal() {
  let tok = Tok::new(Kind::Decimal);
  assert!(<&Tok as LitToken>::is_numeric_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_numeric_literal() {
  let tok = Tok::new(Kind::Other);
  assert!(!<&Tok as LitToken>::is_numeric_literal(&&tok));
}

#[test]
fn blanket_ref_is_integer_literal() {
  let tok = Tok::new(Kind::Binary);
  assert!(<&Tok as LitToken>::is_integer_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_integer_literal() {
  let tok = Tok::new(Kind::Float);
  assert!(!<&Tok as LitToken>::is_integer_literal(&&tok));
}

#[test]
fn blanket_ref_is_float_literal() {
  let tok = Tok::new(Kind::Float);
  assert!(<&Tok as LitToken>::is_float_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_float_literal() {
  let tok = Tok::new(Kind::Decimal);
  assert!(!<&Tok as LitToken>::is_float_literal(&&tok));
}

#[test]
fn blanket_ref_is_decimal_literal() {
  let tok = Tok::new(Kind::Decimal);
  assert!(<&Tok as LitToken>::is_decimal_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_decimal_literal() {
  let tok = Tok::new(Kind::Hex);
  assert!(!<&Tok as LitToken>::is_decimal_literal(&&tok));
}

#[test]
fn blanket_ref_is_hexadecimal_literal() {
  let tok = Tok::new(Kind::Hex);
  assert!(<&Tok as LitToken>::is_hexadecimal_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_hexadecimal_literal() {
  let tok = Tok::new(Kind::Decimal);
  assert!(!<&Tok as LitToken>::is_hexadecimal_literal(&&tok));
}

#[test]
fn blanket_ref_is_octal_literal() {
  let tok = Tok::new(Kind::Octal);
  assert!(<&Tok as LitToken>::is_octal_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_octal_literal() {
  let tok = Tok::new(Kind::Decimal);
  assert!(!<&Tok as LitToken>::is_octal_literal(&&tok));
}

#[test]
fn blanket_ref_is_binary_literal() {
  let tok = Tok::new(Kind::Binary);
  assert!(<&Tok as LitToken>::is_binary_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_binary_literal() {
  let tok = Tok::new(Kind::Decimal);
  assert!(!<&Tok as LitToken>::is_binary_literal(&&tok));
}

#[test]
fn blanket_ref_is_hex_float_literal() {
  let tok = Tok::new(Kind::HexFloat);
  assert!(<&Tok as LitToken>::is_hex_float_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_hex_float_literal() {
  let tok = Tok::new(Kind::Float);
  assert!(!<&Tok as LitToken>::is_hex_float_literal(&&tok));
}

#[test]
fn blanket_ref_is_inline_string_literal() {
  let tok = Tok::new(Kind::InlineStr);
  assert!(<&Tok as LitToken>::is_inline_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_inline_string_literal() {
  let tok = Tok::new(Kind::MultilineStr);
  assert!(!<&Tok as LitToken>::is_inline_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_multiline_string_literal() {
  let tok = Tok::new(Kind::MultilineStr);
  assert!(<&Tok as LitToken>::is_multiline_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_multiline_string_literal() {
  let tok = Tok::new(Kind::InlineStr);
  assert!(!<&Tok as LitToken>::is_multiline_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_raw_string_literal() {
  let tok = Tok::new(Kind::RawStr);
  assert!(<&Tok as LitToken>::is_raw_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_raw_string_literal() {
  let tok = Tok::new(Kind::InlineStr);
  assert!(!<&Tok as LitToken>::is_raw_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_string_literal() {
  let tok = Tok::new(Kind::InlineStr);
  assert!(<&Tok as LitToken>::is_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_string_literal() {
  let tok = Tok::new(Kind::RawStr);
  assert!(!<&Tok as LitToken>::is_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_char_literal() {
  let tok = Tok::new(Kind::Char);
  assert!(<&Tok as LitToken>::is_char_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_char_literal() {
  let tok = Tok::new(Kind::Byte);
  assert!(!<&Tok as LitToken>::is_char_literal(&&tok));
}

#[test]
fn blanket_ref_is_byte_literal() {
  let tok = Tok::new(Kind::Byte);
  assert!(<&Tok as LitToken>::is_byte_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_byte_literal() {
  let tok = Tok::new(Kind::Char);
  assert!(!<&Tok as LitToken>::is_byte_literal(&&tok));
}

#[test]
fn blanket_ref_is_byte_string_literal() {
  let tok = Tok::new(Kind::ByteStr);
  assert!(<&Tok as LitToken>::is_byte_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_byte_string_literal() {
  let tok = Tok::new(Kind::InlineStr);
  assert!(!<&Tok as LitToken>::is_byte_string_literal(&&tok));
}

#[test]
fn blanket_ref_is_true_literal() {
  let tok = Tok::new(Kind::True);
  assert!(<&Tok as LitToken>::is_true_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_true_literal() {
  let tok = Tok::new(Kind::False);
  assert!(!<&Tok as LitToken>::is_true_literal(&&tok));
}

#[test]
fn blanket_ref_is_false_literal() {
  let tok = Tok::new(Kind::False);
  assert!(<&Tok as LitToken>::is_false_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_false_literal() {
  let tok = Tok::new(Kind::True);
  assert!(!<&Tok as LitToken>::is_false_literal(&&tok));
}

#[test]
fn blanket_ref_is_boolean_literal_true() {
  let tok = Tok::new(Kind::True);
  assert!(<&Tok as LitToken>::is_boolean_literal(&&tok));
}

#[test]
fn blanket_ref_is_boolean_literal_false() {
  let tok = Tok::new(Kind::False);
  assert!(<&Tok as LitToken>::is_boolean_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_boolean_literal() {
  let tok = Tok::new(Kind::Other);
  assert!(!<&Tok as LitToken>::is_boolean_literal(&&tok));
}

#[test]
fn blanket_ref_is_null_literal() {
  let tok = Tok::new(Kind::Null);
  assert!(<&Tok as LitToken>::is_null_literal(&&tok));
}

#[test]
fn blanket_ref_is_not_null_literal() {
  let tok = Tok::new(Kind::Other);
  assert!(!<&Tok as LitToken>::is_null_literal(&&tok));
}
