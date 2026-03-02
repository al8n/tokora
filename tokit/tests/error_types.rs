use tokit::SimpleSpan;
/// Tests for all error types in the tokit error module.
/// Exercises constructors, methods, Display/Debug impls, and transformations.
use tokit::error::*;
use tokit::utils::{PositionedChar, knowledge::*};

// ── UnexpectedEnd ──────────────────────────────────────────────────────────────

#[test]
fn unexpected_end_eof() {
  let e = UnexpectedEnd::eof(100usize);
  assert_eq!(e.offset(), 100);
  assert_eq!(e.name(), Some("file"));
  assert_eq!(e.to_string(), "unexpected end of file, expected byte");
}

#[test]
fn unexpected_end_eot() {
  let e = UnexpectedEnd::eot(50usize);
  assert_eq!(e.offset(), 50);
  assert_eq!(e.name(), Some("token stream"));
  assert_eq!(
    e.to_string(),
    "unexpected end of token stream, expected token"
  );
}

#[test]
fn unexpected_end_eos() {
  let e = UnexpectedEnd::eos(25usize);
  assert_eq!(e.offset(), 25);
  assert_eq!(e.name(), Some("string"));
  assert_eq!(
    e.to_string(),
    "unexpected end of string, expected character"
  );
}

#[test]
fn unexpected_end_eorhs() {
  let e = UnexpectedEnd::eorhs(100usize);
  assert_eq!(e.offset(), 100);
  assert_eq!(e.name(), Some("expression (right hand side)"));
  assert_eq!(
    e.to_string(),
    "unexpected end of expression (right hand side), expected either an infix or a postfix"
  );
}

#[test]
fn unexpected_end_eolhs() {
  let e = UnexpectedEnd::eolhs(100usize);
  assert_eq!(e.offset(), 100);
  assert_eq!(e.name(), Some("expression (left hand side)"));
  assert_eq!(
    e.to_string(),
    "unexpected end of expression (left hand side), expected one of an operand, an infix or a postfix"
  );
}

#[test]
fn unexpected_end_new_no_name() {
  let e = UnexpectedEnd::new(10usize, FileHint);
  assert_eq!(e.name(), None);
  assert_eq!(e.offset(), 10);
  assert_eq!(e.to_string(), "unexpected end, expected byte");
}

#[test]
fn unexpected_end_with_hint() {
  let e = UnexpectedEnd::with_hint(15usize, TokenHint);
  assert_eq!(e.name(), None);
  assert_eq!(e.offset(), 15);
}

#[test]
fn unexpected_end_set_and_clear_name() {
  let mut e = UnexpectedEnd::new(10usize, FileHint);
  e.set_name("expression");
  assert_eq!(e.name(), Some("expression"));
  e.clear_name();
  assert_eq!(e.name(), None);
}

#[test]
fn unexpected_end_update_name() {
  let mut e = UnexpectedEnd::eof(10usize);
  e.update_name(Some("block"));
  assert_eq!(e.name(), Some("block"));
  e.update_name(None::<String>);
  assert_eq!(e.name(), None);
}

#[test]
fn unexpected_end_bump() {
  let mut e = UnexpectedEnd::eof(10usize);
  e.bump(&5);
  assert_eq!(e.offset(), 15);
}

#[test]
fn unexpected_end_map_hint() {
  let e = UnexpectedEnd::eof(100usize);
  let e2 = e.map_hint(|_| TokenHint);
  assert_eq!(e2.name(), Some("file"));
  assert_eq!(e2.offset(), 100);
}

#[test]
fn unexpected_end_reconstruct() {
  let e = UnexpectedEnd::eof(100usize);
  let e2 = e.reconstruct(Some("block"), |_| TokenHint);
  assert_eq!(e2.name(), Some("block"));
  assert_eq!(e2.offset(), 100);
}

#[test]
fn unexpected_end_reconstruct_with_name() {
  let e = UnexpectedEnd::eof(100usize);
  let e2 = e.reconstruct_with_name("expression", |_| TokenHint);
  assert_eq!(e2.name(), Some("expression"));
}

#[test]
fn unexpected_end_reconstruct_without_name() {
  use tokit::utils::CowStr;
  let e = UnexpectedEnd::with_name(10usize, CowStr::from_static("file"), FileHint);
  let e2 = e.reconstruct_without_name(|_| TokenHint);
  assert_eq!(e2.name(), None);
}

#[test]
fn unexpected_end_replace_hint() {
  let mut e = UnexpectedEnd::eof(100usize);
  let _old = e.replace_hint(FileHint);
}

#[test]
fn unexpected_end_into_components() {
  let e = UnexpectedEnd::eof(100usize);
  let (offset, name, _hint) = e.into_components();
  assert_eq!(offset, 100);
  assert!(name.is_some());
}

#[test]
fn unexpected_end_offset_ref() {
  let e = UnexpectedEnd::eof(100usize);
  assert_eq!(*e.offset_ref(), 100);
}

#[test]
fn unexpected_end_hint_ref() {
  let e = UnexpectedEnd::eof(100usize);
  let _ = e.hint();
}

#[test]
fn unexpected_end_debug() {
  let e = UnexpectedEnd::eof(100usize);
  let s = format!("{e:?}");
  assert!(s.contains("UnexpectedEnd") || s.contains("offset"));
}

#[test]
fn unexpected_end_eorhs_of() {
  let e = UnexpectedEnd::<PrattRhsHint, usize>::eorhs_of(42);
  assert_eq!(e.offset(), 42);
}

#[test]
fn unexpected_end_eolhs_of() {
  let e = UnexpectedEnd::<PrattLhsHint, usize>::eolhs_of(42);
  assert_eq!(e.offset(), 42);
}

#[test]
fn unexpected_end_eof_of() {
  let e = UnexpectedEnd::<FileHint, usize>::eof_of(42);
  assert_eq!(e.offset(), 42);
}

#[test]
fn unexpected_end_eot_of() {
  let e = UnexpectedEnd::<TokenHint, usize>::eot_of(42);
  assert_eq!(e.offset(), 42);
}

#[test]
fn unexpected_end_eos_of() {
  let e = UnexpectedEnd::<CharacterHint, usize>::eos_of(42);
  assert_eq!(e.offset(), 42);
}

#[test]
fn unexpected_end_maybe_name() {
  use tokit::utils::CowStr;
  let e = UnexpectedEnd::maybe_name(10usize, Some(CowStr::from_static("string")), FileHint);
  assert_eq!(e.name(), Some("string"));
  assert_eq!(e.offset(), 10);
}

#[test]
fn unexpected_end_with_name() {
  use tokit::utils::CowStr;
  let e = UnexpectedEnd::with_name(20usize, CowStr::from_static("block"), FileHint);
  assert_eq!(e.name(), Some("block"));
}

#[test]
fn hint_displays() {
  assert_eq!(FileHint.to_string(), "byte");
  assert_eq!(TokenHint.to_string(), "token");
  assert_eq!(CharacterHint.to_string(), "character");
  assert_eq!(PrattRhsHint.to_string(), "either an infix or a postfix");
  assert_eq!(
    PrattLhsHint.to_string(),
    "one of an operand, an infix or a postfix"
  );
}

#[test]
fn unexpected_end_from_tuple() {
  let e: UnexpectedEof = (100usize, FileHint).into();
  assert_eq!(e.offset(), 100);
}

// ── Malformed ─────────────────────────────────────────────────────────────────

#[test]
fn malformed_int() {
  let e = Malformed::int(SimpleSpan::new(10, 16));
  assert_eq!(
    e.to_string(),
    "malformed at 10..16, did you mean int literal?"
  );
  assert_eq!(e.span(), SimpleSpan::new(10, 16));
}

#[test]
fn malformed_float() {
  let e = Malformed::float(SimpleSpan::new(5, 13));
  assert_eq!(
    e.to_string(),
    "malformed at 5..13, did you mean float literal?"
  );
}

#[test]
fn malformed_hex() {
  let e = Malformed::hex(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "malformed at 0..4, did you mean hex literal?"
  );
}

#[test]
fn malformed_binary() {
  let e = Malformed::binary(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "malformed at 0..4, did you mean binary literal?"
  );
}

#[test]
fn malformed_octal() {
  let e = Malformed::octal(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "malformed at 0..4, did you mean octal literal?"
  );
}

#[test]
fn malformed_decimal() {
  let e = Malformed::decimal(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "malformed at 0..4, did you mean decimal literal?"
  );
}

#[test]
fn malformed_string() {
  let e = Malformed::string(SimpleSpan::new(0, 5));
  assert_eq!(
    e.to_string(),
    "malformed at 0..5, did you mean string literal?"
  );
}

#[test]
fn malformed_boolean() {
  let e = Malformed::boolean(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "malformed at 0..4, did you mean boolean literal?"
  );
}

#[test]
fn malformed_null() {
  let e = Malformed::null(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "malformed at 0..4, did you mean null literal?"
  );
}

#[test]
fn malformed_hex_float() {
  let e = Malformed::hex_float(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "malformed at 0..4, did you mean hex float literal?"
  );
}

#[test]
fn malformed_new_no_knowledge() {
  let e: Malformed<()> = Malformed::new(SimpleSpan::new(20, 25));
  assert_eq!(e.to_string(), "malformed at 20..25");
  assert!(e.knowledge().is_none());
}

#[test]
fn malformed_with_knowledge() {
  let e = Malformed::with_knowledge(SimpleSpan::new(15, 17), HexLiteral::default());
  assert!(e.knowledge().is_some());
}

#[test]
fn malformed_span_ref() {
  let e = Malformed::int(SimpleSpan::new(1, 5));
  assert_eq!(*e.span_ref(), SimpleSpan::new(1, 5));
}

#[test]
fn malformed_into_components() {
  let e = Malformed::int(SimpleSpan::new(1, 5));
  let (span, _knowledge) = e.into_components();
  assert_eq!(span, SimpleSpan::new(1, 5));
}

#[test]
fn malformed_bump() {
  let mut e = Malformed::int(SimpleSpan::new(5, 10));
  e.bump(&3usize);
  assert_eq!(e.span(), SimpleSpan::new(8, 13));
}

#[test]
fn malformed_enum_literal() {
  let e = Malformed::enumeration(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "malformed at 0..4, did you mean enum literal?"
  );
}

#[test]
fn malformed_enum_value_literal() {
  let e = Malformed::enum_value(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "malformed at 0..4, did you mean enum value literal?"
  );
}

#[test]
fn malformed_debug() {
  let e = Malformed::int(SimpleSpan::new(1, 5));
  let s = format!("{e:?}");
  assert!(!s.is_empty());
}

// ── Invalid ────────────────────────────────────────────────────────────────────

#[test]
fn invalid_int() {
  let e = Invalid::int(SimpleSpan::new(10, 16));
  assert_eq!(
    e.to_string(),
    "invalid at 10..16, did you mean int literal?"
  );
}

#[test]
fn invalid_float() {
  let e = Invalid::float(SimpleSpan::new(5, 13));
  assert_eq!(
    e.to_string(),
    "invalid at 5..13, did you mean float literal?"
  );
}

#[test]
fn invalid_hex() {
  let e = Invalid::hex(SimpleSpan::new(0, 4));
  assert_eq!(e.to_string(), "invalid at 0..4, did you mean hex literal?");
}

#[test]
fn invalid_string() {
  let e = Invalid::string(SimpleSpan::new(0, 5));
  assert_eq!(
    e.to_string(),
    "invalid at 0..5, did you mean string literal?"
  );
}

#[test]
fn invalid_boolean() {
  let e = Invalid::boolean(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "invalid at 0..4, did you mean boolean literal?"
  );
}

#[test]
fn invalid_null() {
  let e = Invalid::null(SimpleSpan::new(0, 4));
  assert_eq!(e.to_string(), "invalid at 0..4, did you mean null literal?");
}

#[test]
fn invalid_hex_float() {
  let e = Invalid::hex_float(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "invalid at 0..4, did you mean hex float literal?"
  );
}

#[test]
fn invalid_decimal() {
  let e = Invalid::decimal(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "invalid at 0..4, did you mean decimal literal?"
  );
}

#[test]
fn invalid_binary() {
  let e = Invalid::binary(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "invalid at 0..4, did you mean binary literal?"
  );
}

#[test]
fn invalid_octal() {
  let e = Invalid::octal(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "invalid at 0..4, did you mean octal literal?"
  );
}

#[test]
fn invalid_new_no_knowledge() {
  let e: Invalid<()> = Invalid::new(SimpleSpan::new(20, 25));
  assert_eq!(e.to_string(), "invalid at 20..25");
}

#[test]
fn invalid_with_knowledge() {
  let e = Invalid::with_knowledge(SimpleSpan::new(15, 17), HexLiteral::default());
  assert!(e.knowledge().is_some());
}

#[test]
fn invalid_into_components() {
  let e = Invalid::int(SimpleSpan::new(1, 5));
  let (span, _k) = e.into_components();
  assert_eq!(span, SimpleSpan::new(1, 5));
}

#[test]
fn invalid_bump() {
  let mut e = Invalid::int(SimpleSpan::new(5, 10));
  e.bump(&3usize);
  assert_eq!(e.span(), SimpleSpan::new(8, 13));
}

#[test]
fn invalid_enumeration() {
  let e = Invalid::enumeration(SimpleSpan::new(0, 4));
  assert_eq!(e.to_string(), "invalid at 0..4, did you mean enum literal?");
}

#[test]
fn invalid_enum_value() {
  let e = Invalid::enum_value(SimpleSpan::new(0, 4));
  assert_eq!(
    e.to_string(),
    "invalid at 0..4, did you mean enum value literal?"
  );
}

#[test]
fn invalid_debug() {
  let e = Invalid::int(SimpleSpan::new(1, 5));
  let s = format!("{e:?}");
  assert!(!s.is_empty());
}

// ── InvalidHexDigits ──────────────────────────────────────────────────────────

#[test]
fn invalid_hex_digits_from_positioned_char() {
  let mut digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  assert_eq!(digits.len(), 1);
  digits.push(PositionedChar::with_position('H', 13));
  assert_eq!(digits.len(), 2);
}

#[test]
fn invalid_hex_digits_from_char() {
  let digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(10, 'G');
  assert_eq!(digits.len(), 1);
}

#[test]
fn invalid_hex_digits_from_array() {
  let digits: InvalidHexDigits<char, 4> = InvalidHexDigits::from_array([
    PositionedChar::with_position('G', 12),
    PositionedChar::with_position('H', 13),
    PositionedChar::with_position('I', 14),
    PositionedChar::with_position('J', 15),
  ]);
  assert_eq!(digits.len(), 4);
}

#[test]
fn invalid_hex_digits_is_full() {
  let digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_array([
    PositionedChar::with_position('G', 12),
    PositionedChar::with_position('H', 13),
  ]);
  assert!(digits.is_full());
}

#[test]
fn invalid_hex_digits_bump() {
  let mut digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(10, 'G');
  digits.bump(&5usize);
}

#[test]
fn invalid_hex_digits_debug() {
  let digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(10, 'G');
  let s = format!("{digits:?}");
  assert!(!s.is_empty());
}

// ── HexEscapeError ─────────────────────────────────────────────────────────────

#[test]
fn hex_escape_incomplete() {
  let e: HexEscapeError<char> = HexEscapeError::incomplete(SimpleSpan::new(5, 7));
  assert!(e.is_incomplete());
  assert!(!e.is_malformed());
}

#[test]
fn hex_escape_malformed() {
  let digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(10, 'G');
  let e: HexEscapeError<char> = HexEscapeError::malformed(digits, SimpleSpan::new(5, 7));
  assert!(!e.is_incomplete());
  assert!(e.is_malformed());
}

#[test]
fn hex_escape_incomplete_hex_escape_display() {
  let e = IncompleteHexEscape::new(SimpleSpan::new(5, 7));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn hex_escape_malformed_hex_escape_display() {
  let digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(10, 'G');
  let e = MalformedHexEscape::new(digits, SimpleSpan::new(5, 7));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn hex_escape_error_display_incomplete() {
  let e: HexEscapeError<char> = HexEscapeError::incomplete(SimpleSpan::new(5, 7));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn hex_escape_error_display_malformed() {
  let digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(10, 'G');
  let e: HexEscapeError<char> = HexEscapeError::malformed(digits, SimpleSpan::new(5, 7));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn hex_escape_incomplete_span() {
  let e = IncompleteHexEscape::new(SimpleSpan::new(5, 7));
  assert_eq!(e.span(), SimpleSpan::new(5, 7));
}

#[test]
fn hex_escape_malformed_span() {
  let digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(10, 'G');
  let e = MalformedHexEscape::new(digits.clone(), SimpleSpan::new(5, 7));
  assert_eq!(e.span(), SimpleSpan::new(5, 7));
  assert_eq!(e.digits_ref(), &digits);
}

#[test]
fn hex_escape_incomplete_bump() {
  let mut e = IncompleteHexEscape::new(SimpleSpan::new(5, 7));
  e.bump(&3);
  assert_eq!(e.span(), SimpleSpan::new(8, 10));
}

#[test]
fn hex_escape_malformed_bump() {
  let digits: InvalidHexDigits<char, 2> = InvalidHexDigits::from_char(10, 'G');
  let mut e = MalformedHexEscape::new(digits, SimpleSpan::new(5, 7));
  e.bump(&3);
  assert_eq!(e.span(), SimpleSpan::new(8, 10));
}

// ── UnknownLexeme ─────────────────────────────────────────────────────────────

#[test]
fn unknown_lexeme_from_char() {
  let e: UnknownLexeme<char, ()> = UnknownLexeme::from_char(10usize, 'x', ());
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unknown_lexeme_from_positioned_char() {
  let pc = PositionedChar::with_position('x', 10usize);
  let e: UnknownLexeme<char, ()> = UnknownLexeme::from_positioned_char(pc, ());
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unknown_lexeme_bump() {
  let mut e: UnknownLexeme<char, ()> = UnknownLexeme::from_char(5usize, 'x', ());
  e.bump(&3);
}

#[test]
fn unknown_lexeme_into_components() {
  let e: UnknownLexeme<char, ()> = UnknownLexeme::from_char(10usize, 'x', ());
  let (_lexeme, _knowledge) = e.into_components();
}

#[test]
fn unknown_lexeme_debug() {
  let e: UnknownLexeme<char, ()> = UnknownLexeme::from_char(10usize, 'x', ());
  let s = format!("{e:?}");
  assert!(!s.is_empty());
}

// ── UnexpectedLexeme ─────────────────────────────────────────────────────────

#[test]
fn unexpected_lexeme_from_char() {
  let e: UnexpectedLexeme<char, &str> = UnexpectedLexeme::from_char(10usize, 'x', "identifier");
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unexpected_lexeme_from_positioned_char() {
  let pc = PositionedChar::with_position('x', 10usize);
  let e: UnexpectedLexeme<char, &str> = UnexpectedLexeme::from_positioned_char(pc, "identifier");
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unexpected_lexeme_new_line() {
  let e: UnexpectedLineTerminator<char> = UnexpectedLineTerminator::new_line(10usize, '\n');
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unexpected_lexeme_carriage_return() {
  let e: UnexpectedLineTerminator<char> = UnexpectedLineTerminator::carriage_return(10usize, '\r');
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unexpected_lexeme_carriage_return_new_line() {
  let e: UnexpectedLineTerminator<char> =
    UnexpectedLineTerminator::carriage_return_new_line(SimpleSpan::new(10, 12));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unexpected_lexeme_bump() {
  let mut e: UnexpectedLexeme<char, &str> = UnexpectedLexeme::from_char(5usize, 'x', "id");
  e.bump(&3);
}

#[test]
fn unexpected_lexeme_into_components() {
  let e: UnexpectedLexeme<char, &str> = UnexpectedLexeme::from_char(10usize, 'x', "id");
  let (_lexeme, _hint) = e.into_components();
}

#[test]
fn unexpected_lexeme_debug() {
  let e: UnexpectedLexeme<char, &str> = UnexpectedLexeme::from_char(10usize, 'x', "id");
  let s = format!("{e:?}");
  assert!(!s.is_empty());
}

// ── UnexpectedPrefix ──────────────────────────────────────────────────────────

#[test]
fn unexpected_prefix_from_char() {
  // token span is 1..5, prefix char '0' at position 0 (prefix must end before token starts)
  let e: UnexpectedPrefix<char, ()> = UnexpectedPrefix::from_char(SimpleSpan::new(1, 5), 0, '0');
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unexpected_prefix_from_positioned_char() {
  let pc = PositionedChar::with_position('0', 0usize);
  let e: UnexpectedPrefix<char, ()> =
    UnexpectedPrefix::from_positioned_char(SimpleSpan::new(1, 5), pc);
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unexpected_prefix_bump() {
  let mut e: UnexpectedPrefix<char, ()> =
    UnexpectedPrefix::from_char(SimpleSpan::new(1, 5), 0, '0');
  e.bump(&3);
}

#[test]
fn unexpected_prefix_into_components() {
  let e: UnexpectedPrefix<char, ()> = UnexpectedPrefix::from_char(SimpleSpan::new(1, 5), 0, '0');
  let (_token_span, _prefix_lexeme) = e.into_components();
}

#[test]
fn unexpected_prefix_debug() {
  let e: UnexpectedPrefix<char, ()> = UnexpectedPrefix::from_char(SimpleSpan::new(1, 5), 0, '0');
  let s = format!("{e:?}");
  assert!(!s.is_empty());
}

// ── UnexpectedSuffix ──────────────────────────────────────────────────────────

#[test]
fn unexpected_suffix_from_char() {
  // token span is 0..5, suffix char 'x' at position 5 (suffix must start at or after token end)
  let e: UnexpectedSuffix<char, ()> = UnexpectedSuffix::from_char(SimpleSpan::new(0, 5), 5, 'x');
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unexpected_suffix_from_positioned_char() {
  let pc = PositionedChar::with_position('x', 5usize);
  let e: UnexpectedSuffix<char, ()> =
    UnexpectedSuffix::from_positioned_char(SimpleSpan::new(0, 5), pc);
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unexpected_suffix_bump() {
  let mut e: UnexpectedSuffix<char, ()> =
    UnexpectedSuffix::from_char(SimpleSpan::new(0, 5), 5, 'x');
  e.bump(&3);
}

#[test]
fn unexpected_suffix_into_components() {
  let e: UnexpectedSuffix<char, ()> = UnexpectedSuffix::from_char(SimpleSpan::new(0, 5), 5, 'x');
  let (_token_span, _suffix_lexeme) = e.into_components();
}

#[test]
fn unexpected_suffix_debug() {
  let e: UnexpectedSuffix<char, ()> = UnexpectedSuffix::from_char(SimpleSpan::new(0, 5), 5, 'x');
  let s = format!("{e:?}");
  assert!(!s.is_empty());
}

// ── UnicodeEscapeError ────────────────────────────────────────────────────────

#[test]
fn unicode_escape_malformed_fixed() {
  let digits = InvalidFixedUnicodeHexDigits::from_array([
    PositionedChar::with_position('G', 12),
    PositionedChar::with_position('H', 13),
    PositionedChar::with_position('I', 14),
    PositionedChar::with_position('J', 15),
  ]);
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::malformed_fixed_unicode_escape(digits, SimpleSpan::new(10, 16));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unicode_escape_incomplete_fixed() {
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::incomplete_fixed_unicode_escape(SimpleSpan::new(5, 9));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unicode_escape_unpaired_surrogate() {
  use tokit::utils::Lexeme;
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::unpaired_high_surrogate(Lexeme::Range(SimpleSpan::new(5, 11)));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unicode_escape_empty_variable() {
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::empty_variable_unicode_escape(SimpleSpan::new(5, 9));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unicode_escape_surrogate_variable() {
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::surrogate_variable_unicode_escape(SimpleSpan::new(10, 18), 0xD800);
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unicode_escape_overflow_variable() {
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::overflow_variable_unicode_escape(SimpleSpan::new(20, 30), 0x110000);
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unicode_escape_too_many_digits_variable() {
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::too_many_digits_in_variable_unicode_escape(SimpleSpan::new(5, 15), 7);
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unicode_escape_unclosed_variable() {
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::unclosed_variable_unicode_escape(SimpleSpan::new(0, 10));
  let s = e.to_string();
  assert!(!s.is_empty());
}

#[test]
fn unicode_escape_is_fixed_variable() {
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::incomplete_fixed_unicode_escape(SimpleSpan::new(5, 9));
  assert!(e.is_fixed());
  assert!(!e.is_variable());

  let e2: UnicodeEscapeError<char> =
    UnicodeEscapeError::empty_variable_unicode_escape(SimpleSpan::new(5, 9));
  assert!(!e2.is_fixed());
  assert!(e2.is_variable());
}

#[test]
fn unicode_escape_debug() {
  let e: UnicodeEscapeError<char> =
    UnicodeEscapeError::incomplete_fixed_unicode_escape(SimpleSpan::new(5, 9));
  let s = format!("{e:?}");
  assert!(!s.is_empty());
}

// ── PositionedChar ─────────────────────────────────────────────────────────────

#[test]
fn positioned_char_with_position() {
  let pc = PositionedChar::with_position('x', 10usize);
  assert_eq!(pc.char(), 'x');
  assert_eq!(pc.position(), 10);
}

#[test]
fn positioned_char_char_ref() {
  let pc = PositionedChar::with_position('x', 10usize);
  assert_eq!(pc.char_ref(), &'x');
}

#[test]
fn positioned_char_position_ref() {
  let pc = PositionedChar::with_position('x', 10usize);
  assert_eq!(pc.position_ref(), &10);
}

#[test]
fn positioned_char_span() {
  let pc = PositionedChar::with_position('a', 5usize);
  let span = pc.span();
  assert_eq!(span.start(), 5);
}

#[test]
fn positioned_char_set_position() {
  let mut pc = PositionedChar::with_position('x', 10usize);
  pc.set_position(20);
  assert_eq!(pc.position(), 20);
}

#[test]
fn positioned_char_map() {
  let pc = PositionedChar::with_position('x', 10usize);
  let pc2 = pc.map(|c| c.to_ascii_uppercase());
  assert_eq!(pc2.char(), 'X');
}

#[test]
fn positioned_char_into_components() {
  let pc = PositionedChar::with_position('x', 10usize);
  let (ch, pos) = pc.into_components();
  assert_eq!(ch, 'x');
  assert_eq!(pos, 10);
}

#[test]
fn positioned_char_debug() {
  let pc = PositionedChar::with_position('x', 10usize);
  let s = format!("{pc:?}");
  assert!(!s.is_empty());
}

// ── SimpleSpan ────────────────────────────────────────────────────────────────

#[test]
fn simple_span_new() {
  let s = SimpleSpan::new(5, 10);
  assert_eq!(s.start(), 5);
  assert_eq!(s.end(), 10);
}

#[test]
fn simple_span_len() {
  let s = SimpleSpan::new(5, 10);
  assert_eq!(s.len(), 5);
}

#[test]
fn simple_span_is_empty() {
  let s = SimpleSpan::new(5, 5);
  assert!(s.is_empty());
  let s2 = SimpleSpan::new(5, 10);
  assert!(!s2.is_empty());
}

#[test]
fn simple_span_display() {
  let s = SimpleSpan::new(5, 10);
  assert_eq!(s.to_string(), "5..10");
}

#[test]
fn simple_span_debug() {
  let s = SimpleSpan::new(5, 10);
  let dbg = format!("{s:?}");
  assert!(!dbg.is_empty());
}

#[test]
fn simple_span_ordering() {
  let a = SimpleSpan::new(1, 5);
  let b = SimpleSpan::new(5, 10);
  assert!(a < b);
  assert_eq!(a, a);
}
