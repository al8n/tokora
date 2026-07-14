use super::*;
use crate::SimpleSpan;
use crate::utils::PositionedChar;

use std::format;

// --- InvalidUnicodeScalarKind ---

#[test]
fn invalid_unicode_scalar_kind_debug_clone() {
  let k = InvalidUnicodeScalarKind::Surrogate;
  let k2 = k;
  assert_eq!(k, k2);
  assert_eq!(format!("{:?}", k), "Surrogate");
  assert_eq!(
    format!("{:?}", InvalidUnicodeScalarKind::Overflow),
    "Overflow"
  );
}

// --- InvalidUnicodeScalarValue ---

#[test]
fn invalid_unicode_scalar_value_surrogate() {
  let e = InvalidUnicodeScalarValue::new(
    0xD800,
    SimpleSpan::new(10, 18),
    InvalidUnicodeScalarKind::Surrogate,
  );
  assert_eq!(e.codepoint(), 0xD800);
  assert_eq!(e.span(), SimpleSpan::new(10, 18));
  assert_eq!(e.kind(), InvalidUnicodeScalarKind::Surrogate);
  let display = format!("{}", e);
  assert!(display.contains("surrogate"));
  assert!(display.contains("D800"));
}

#[test]
fn invalid_unicode_scalar_value_overflow() {
  let e = InvalidUnicodeScalarValue::new(
    0x110000,
    SimpleSpan::new(20, 30),
    InvalidUnicodeScalarKind::Overflow,
  );
  assert_eq!(e.codepoint(), 0x110000);
  assert_eq!(e.kind(), InvalidUnicodeScalarKind::Overflow);
  let display = format!("{}", e);
  assert!(display.contains("out of range"));
}

#[test]
fn invalid_unicode_scalar_value_bump() {
  let mut e = InvalidUnicodeScalarValue::new(
    0xD800,
    SimpleSpan::new(10, 18),
    InvalidUnicodeScalarKind::Surrogate,
  );
  e.bump(&5);
  assert_eq!(e.span(), SimpleSpan::new(15, 23));
}

#[test]
fn invalid_unicode_scalar_value_span_ref_mut() {
  let mut e = InvalidUnicodeScalarValue::new(
    0xD800,
    SimpleSpan::new(10, 18),
    InvalidUnicodeScalarKind::Surrogate,
  );
  assert_eq!(*e.span_ref().start_ref(), &10);
  let span_mut = e.span_mut();
  assert_eq!(**span_mut.start_ref(), 10);
}

#[test]
fn invalid_unicode_scalar_value_error_trait() {
  let e = InvalidUnicodeScalarValue::new(
    0xD800,
    SimpleSpan::new(10, 18),
    InvalidUnicodeScalarKind::Surrogate,
  );
  let err: &dyn core::error::Error = &e;
  assert!(err.source().is_none());
}

// --- EmptyVariableUnicodeEscape ---

#[test]
fn empty_variable_unicode_escape() {
  let e = EmptyVariableUnicodeEscape::new(SimpleSpan::new(10, 14));
  assert_eq!(e.span(), SimpleSpan::new(10, 14));
  assert_eq!(
    format!("{}", e),
    "empty variable-length unicode escape at 10..14"
  );
}

#[test]
fn empty_variable_unicode_escape_bump() {
  let mut e = EmptyVariableUnicodeEscape::new(SimpleSpan::new(10, 14));
  e.bump(&5);
  assert_eq!(e.span(), SimpleSpan::new(15, 19));
}

#[test]
fn empty_variable_unicode_escape_span_ref_mut() {
  let mut e = EmptyVariableUnicodeEscape::new(SimpleSpan::new(10, 14));
  assert_eq!(*e.span_ref().start_ref(), &10);
  let span_mut = e.span_mut();
  assert_eq!(**span_mut.start_ref(), 10);
}

// --- TooManyDigitsInVariableUnicodeEscape ---

#[test]
fn too_many_digits() {
  let e = TooManyDigitsInVariableUnicodeEscape::new(SimpleSpan::new(10, 21), 7);
  assert_eq!(e.count(), 7);
  assert_eq!(e.span(), SimpleSpan::new(10, 21));
  assert_eq!(
    format!("{}", e),
    "too many digits (7) in variable-length unicode escape at 10..21"
  );
}

#[test]
fn too_many_digits_bump() {
  let mut e = TooManyDigitsInVariableUnicodeEscape::new(SimpleSpan::new(10, 20), 7);
  e.bump(&5);
  assert_eq!(e.span(), SimpleSpan::new(15, 25));
}

#[test]
fn too_many_digits_span_ref_mut() {
  let mut e = TooManyDigitsInVariableUnicodeEscape::new(SimpleSpan::new(10, 20), 7);
  assert_eq!(*e.span_ref().start_ref(), &10);
  let span_mut = e.span_mut();
  assert_eq!(**span_mut.start_ref(), 10);
}

// --- MalformedVariableUnicodeSequence ---

#[test]
fn malformed_variable_from_char() {
  let e = MalformedVariableUnicodeSequence::<char>::from_char(12, 'G');
  assert_eq!(
    format!("{}", e),
    "invalid variable-length unicode escape character 'G' at position 12"
  );
  assert!(e.lexeme().is_char());
  assert_eq!(e.span(), SimpleSpan::new(12, 13));
}

#[test]
fn malformed_variable_from_range() {
  let e: MalformedVariableUnicodeSequence<char> =
    MalformedVariableUnicodeSequence::from_range(SimpleSpan::new(10, 15));
  let display = format!("{}", e);
  assert!(display.contains("malformed"));
  assert!(display.contains("10..15"));
}

#[test]
fn malformed_variable_from_positioned_char() {
  let e =
    MalformedVariableUnicodeSequence::from_positioned_char(PositionedChar::with_position('X', 42));
  assert_eq!(
    format!("{}", e),
    "invalid variable-length unicode escape character 'X' at position 42"
  );
}

#[test]
fn malformed_variable_bump() {
  let mut e = MalformedVariableUnicodeSequence::from_char(10, 'G');
  e.bump(&5);
  assert_eq!(e.span(), SimpleSpan::new(15, 16));
}

// --- MalformedFixedUnicodeEscape ---

#[test]
fn malformed_fixed_new() {
  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let e = MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16));
  assert_eq!(e.span(), SimpleSpan::new(10, 16));
  assert_eq!(e.digits_ref().len(), 1);
}

#[test]
fn malformed_fixed_display() {
  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let e = MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16));
  let display = format!("{}", e);
  assert!(display.contains("malformed"));
  assert!(display.contains("invalid digits"));
}

#[test]
fn malformed_fixed_bump() {
  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let mut e = MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16));
  e.bump(&5);
  assert_eq!(e.span(), SimpleSpan::new(15, 21));
}

#[test]
fn malformed_fixed_span_ref_mut() {
  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let mut e = MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16));
  assert_eq!(*e.span_ref().start_ref(), &10);
  let span_mut = e.span_mut();
  assert_eq!(**span_mut.start_ref(), 10);
}

#[test]
fn malformed_fixed_digits_clone() {
  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let e = MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16));
  let d = e.digits();
  assert_eq!(d.len(), 1);
}

#[test]
fn malformed_fixed_digits_mut() {
  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let mut e = MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16));
  let _d = e.digits_mut();
}

// --- IncompleteFixedUnicodeEscape ---

#[test]
fn incomplete_fixed() {
  let e = IncompleteFixedUnicodeEscape::new(SimpleSpan::new(10, 13));
  assert_eq!(e.span(), SimpleSpan::new(10, 13));
  let display = format!("{}", e);
  assert!(display.contains("incomplete"));
  assert!(display.contains("four hexadecimal digits"));
}

#[test]
fn incomplete_fixed_bump() {
  let mut e = IncompleteFixedUnicodeEscape::new(SimpleSpan::new(10, 12));
  e.bump(&5);
  assert_eq!(e.span(), SimpleSpan::new(15, 17));
}

#[test]
fn incomplete_fixed_span_ref_mut() {
  let mut e = IncompleteFixedUnicodeEscape::new(SimpleSpan::new(10, 13));
  assert_eq!(*e.span_ref().start_ref(), &10);
  let span_mut = e.span_mut();
  assert_eq!(**span_mut.start_ref(), 10);
}

// --- UnpairedSurrogateHint ---

#[test]
fn unpaired_surrogate_hint_display() {
  assert_eq!(format!("{}", UnpairedSurrogateHint::High), "high surrogate");
  assert_eq!(format!("{}", UnpairedSurrogateHint::Low), "low surrogate");
}

#[test]
fn unpaired_surrogate_hint_is_variant() {
  assert!(UnpairedSurrogateHint::High.is_high());
  assert!(UnpairedSurrogateHint::Low.is_low());
  assert!(!UnpairedSurrogateHint::High.is_low());
}

// --- VariableUnicodeEscapeError ---

#[test]
fn variable_error_empty() {
  let e = VariableUnicodeEscapeError::<char>::empty(SimpleSpan::new(10, 14));
  assert!(e.is_empty());
  assert_eq!(e.span(), SimpleSpan::new(10, 14));
}

#[test]
fn variable_error_too_many_digits() {
  let e = VariableUnicodeEscapeError::<char>::too_many_digits(SimpleSpan::new(5, 16), 7);
  assert!(e.is_too_many_digits());
}

#[test]
fn variable_error_unclosed() {
  let e = VariableUnicodeEscapeError::<char>::unclosed(SimpleSpan::new(10, 15));
  assert!(e.is_unclosed());
  let display = format!("{}", e);
  assert!(display.contains("unclosed"));
}

#[test]
fn variable_error_overflow() {
  let e = VariableUnicodeEscapeError::<char>::overflow(SimpleSpan::new(10, 20), 0x110000);
  assert!(e.is_invalid_scalar());
}

#[test]
fn variable_error_surrogate() {
  let e = VariableUnicodeEscapeError::<char>::surrogate(SimpleSpan::new(10, 18), 0xD800);
  assert!(e.is_invalid_scalar());
}

#[test]
fn variable_error_bump() {
  let mut e = VariableUnicodeEscapeError::<char>::empty(SimpleSpan::new(10, 14));
  e.bump(&5);
  assert_eq!(e.span(), SimpleSpan::new(15, 19));

  let mut e2 = VariableUnicodeEscapeError::<char>::too_many_digits(SimpleSpan::new(5, 16), 7);
  e2.bump(&3);

  let mut e3 = VariableUnicodeEscapeError::<char>::unclosed(SimpleSpan::new(10, 15));
  e3.bump(&2);

  let mut e4 = VariableUnicodeEscapeError::<char>::overflow(SimpleSpan::new(10, 20), 0x110000);
  e4.bump(&1);

  let mut e5 =
    VariableUnicodeEscapeError::Malformed(MalformedVariableUnicodeSequence::from_char(10, 'G'));
  e5.bump(&3);
}

#[test]
fn variable_error_display_all_variants() {
  let e1 = VariableUnicodeEscapeError::<char>::empty(SimpleSpan::new(10, 14));
  assert!(format!("{}", e1).contains("empty"));

  let e2 = VariableUnicodeEscapeError::<char>::too_many_digits(SimpleSpan::new(5, 16), 7);
  assert!(format!("{}", e2).contains("too many digits"));

  let e3 = VariableUnicodeEscapeError::<char>::unclosed(SimpleSpan::new(10, 15));
  assert!(format!("{}", e3).contains("unclosed"));

  let e4 = VariableUnicodeEscapeError::<char>::overflow(SimpleSpan::new(10, 20), 0x110000);
  assert!(format!("{}", e4).contains("out of range"));

  let e5 = VariableUnicodeEscapeError::<char>::surrogate(SimpleSpan::new(10, 18), 0xD800);
  assert!(format!("{}", e5).contains("surrogate"));

  let e6 =
    VariableUnicodeEscapeError::Malformed(MalformedVariableUnicodeSequence::from_char(10, 'G'));
  assert!(format!("{}", e6).contains("invalid"));
}

#[test]
fn variable_error_source() {
  use core::error::Error;
  let e = VariableUnicodeEscapeError::<char>::empty(SimpleSpan::new(10, 14));
  assert!(e.source().is_some());

  let e2 = VariableUnicodeEscapeError::<char>::too_many_digits(SimpleSpan::new(5, 16), 7);
  assert!(e2.source().is_some());

  let e3 = VariableUnicodeEscapeError::<char>::unclosed(SimpleSpan::new(10, 15));
  assert!(e3.source().is_some());

  let e4 = VariableUnicodeEscapeError::<char>::overflow(SimpleSpan::new(10, 20), 0x110000);
  assert!(e4.source().is_some());

  let e5 =
    VariableUnicodeEscapeError::Malformed(MalformedVariableUnicodeSequence::from_char(10, 'G'));
  assert!(e5.source().is_some());
}

#[test]
fn variable_error_span_all_variants() {
  let e1 = VariableUnicodeEscapeError::<char>::empty(SimpleSpan::new(10, 14));
  assert_eq!(e1.span(), SimpleSpan::new(10, 14));

  let e2 = VariableUnicodeEscapeError::<char>::too_many_digits(SimpleSpan::new(5, 16), 7);
  assert_eq!(e2.span(), SimpleSpan::new(5, 16));

  let e4 = VariableUnicodeEscapeError::<char>::overflow(SimpleSpan::new(10, 20), 0x110000);
  assert_eq!(e4.span(), SimpleSpan::new(10, 20));

  let e5 =
    VariableUnicodeEscapeError::Malformed(MalformedVariableUnicodeSequence::from_char(10, 'G'));
  assert_eq!(e5.span(), SimpleSpan::new(10, 11));
}

// --- FixedUnicodeEscapeError ---

#[test]
fn fixed_error_incomplete() {
  let e: FixedUnicodeEscapeError =
    FixedUnicodeEscapeError::Incomplete(IncompleteFixedUnicodeEscape::new(SimpleSpan::new(10, 14)));
  assert!(e.is_incomplete());
  assert_eq!(e.span(), SimpleSpan::new(10, 14));
}

#[test]
fn fixed_error_unpaired_high() {
  use crate::utils::Lexeme;
  let e = FixedUnicodeEscapeError::<char>::unpaired_high_surrogate(Lexeme::Range(SimpleSpan::new(
    10, 16,
  )));
  assert!(e.is_unpaired_surrogate());
  let display = format!("{}", e);
  assert!(display.contains("unpaired high surrogate"));
}

#[test]
fn fixed_error_unpaired_low() {
  use crate::utils::Lexeme;
  let e =
    FixedUnicodeEscapeError::<char>::unpaired_low_surrogate(Lexeme::Range(SimpleSpan::new(10, 16)));
  assert!(e.is_unpaired_surrogate());
  let display = format!("{}", e);
  assert!(display.contains("unpaired low surrogate"));
}

#[test]
fn fixed_error_malformed() {
  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let e: FixedUnicodeEscapeError = FixedUnicodeEscapeError::Malformed(
    MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16)),
  );
  assert!(e.is_malformed());
  let display = format!("{}", e);
  assert!(display.contains("malformed"));
}

#[test]
fn fixed_error_bump() {
  let mut e: FixedUnicodeEscapeError =
    FixedUnicodeEscapeError::Incomplete(IncompleteFixedUnicodeEscape::new(SimpleSpan::new(10, 14)));
  e.bump(&5);
  assert_eq!(e.span(), SimpleSpan::new(15, 19));

  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let mut e2: FixedUnicodeEscapeError = FixedUnicodeEscapeError::Malformed(
    MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16)),
  );
  e2.bump(&3);

  use crate::utils::Lexeme;
  let mut e3 = FixedUnicodeEscapeError::<char>::unpaired_high_surrogate(Lexeme::Range(
    SimpleSpan::new(10, 16),
  ));
  e3.bump(&2);
}

#[test]
fn fixed_error_span_all_variants() {
  use crate::utils::Lexeme;

  let e1: FixedUnicodeEscapeError =
    FixedUnicodeEscapeError::Incomplete(IncompleteFixedUnicodeEscape::new(SimpleSpan::new(10, 14)));
  assert_eq!(e1.span(), SimpleSpan::new(10, 14));

  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let e2: FixedUnicodeEscapeError = FixedUnicodeEscapeError::Malformed(
    MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16)),
  );
  assert_eq!(e2.span(), SimpleSpan::new(10, 16));

  let e3 = FixedUnicodeEscapeError::<char>::unpaired_high_surrogate(Lexeme::Range(
    SimpleSpan::new(10, 16),
  ));
  assert_eq!(e3.span(), SimpleSpan::new(10, 16));
}

#[test]
fn fixed_error_source() {
  use crate::utils::Lexeme;
  use core::error::Error;

  let e1: FixedUnicodeEscapeError =
    FixedUnicodeEscapeError::Incomplete(IncompleteFixedUnicodeEscape::new(SimpleSpan::new(10, 14)));
  assert!(e1.source().is_none());

  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let e2: FixedUnicodeEscapeError = FixedUnicodeEscapeError::Malformed(
    MalformedFixedUnicodeEscape::new(digits, SimpleSpan::new(10, 16)),
  );
  assert!(e2.source().is_some());

  let e3 = FixedUnicodeEscapeError::<char>::unpaired_high_surrogate(Lexeme::Range(
    SimpleSpan::new(10, 16),
  ));
  assert!(e3.source().is_some());
}

// --- UnicodeEscapeError ---

#[test]
fn unicode_error_constructors() {
  use crate::utils::Lexeme;

  let e1 = UnicodeEscapeError::<char>::incomplete_fixed_unicode_escape(SimpleSpan::new(10, 14));
  assert!(e1.is_fixed());

  let e2 = UnicodeEscapeError::<char>::empty_variable_unicode_escape(SimpleSpan::new(10, 14));
  assert!(e2.is_variable());

  let e3 = UnicodeEscapeError::<char>::too_many_digits_in_variable_unicode_escape(
    SimpleSpan::new(5, 16),
    7,
  );
  assert!(e3.is_variable());

  let e4 = UnicodeEscapeError::<char>::unclosed_variable_unicode_escape(SimpleSpan::new(10, 15));
  assert!(e4.is_variable());

  let e5 =
    UnicodeEscapeError::<char>::surrogate_variable_unicode_escape(SimpleSpan::new(10, 18), 0xD800);
  assert!(e5.is_variable());

  let e6 =
    UnicodeEscapeError::<char>::overflow_variable_unicode_escape(SimpleSpan::new(10, 20), 0x110000);
  assert!(e6.is_variable());

  let e7 = UnicodeEscapeError::<char>::invalid_variable_unicode_escape_char(12, 'G');
  assert!(e7.is_variable());

  let e8 =
    UnicodeEscapeError::<char>::invalid_variable_unicode_escape_sequence(SimpleSpan::new(10, 15));
  assert!(e8.is_variable());

  let e9 =
    UnicodeEscapeError::<char>::unpaired_high_surrogate(Lexeme::Range(SimpleSpan::new(10, 16)));
  assert!(e9.is_fixed());

  let e10 =
    UnicodeEscapeError::<char>::unpaired_low_surrogate(Lexeme::Range(SimpleSpan::new(10, 16)));
  assert!(e10.is_fixed());

  let digits = InvalidFixedUnicodeHexDigits::from(PositionedChar::with_position('G', 12));
  let e11 = UnicodeEscapeError::malformed_fixed_unicode_escape(digits, SimpleSpan::new(10, 16));
  assert!(e11.is_fixed());
}

#[test]
fn unicode_error_display() {
  let e1 = UnicodeEscapeError::<char>::incomplete_fixed_unicode_escape(SimpleSpan::new(10, 14));
  assert!(format!("{}", e1).contains("incomplete"));

  let e2 = UnicodeEscapeError::<char>::empty_variable_unicode_escape(SimpleSpan::new(10, 14));
  assert!(format!("{}", e2).contains("empty"));
}

#[test]
fn unicode_error_bump() {
  let mut e1 = UnicodeEscapeError::<char>::incomplete_fixed_unicode_escape(SimpleSpan::new(10, 14));
  e1.bump(&5);
  assert_eq!(e1.span(), SimpleSpan::new(15, 19));

  let mut e2 = UnicodeEscapeError::<char>::empty_variable_unicode_escape(SimpleSpan::new(10, 14));
  e2.bump(&3);
  assert_eq!(e2.span(), SimpleSpan::new(13, 17));
}

#[test]
fn unicode_error_span() {
  let e1 = UnicodeEscapeError::<char>::incomplete_fixed_unicode_escape(SimpleSpan::new(10, 14));
  assert_eq!(e1.span(), SimpleSpan::new(10, 14));

  let e2 = UnicodeEscapeError::<char>::empty_variable_unicode_escape(SimpleSpan::new(10, 14));
  assert_eq!(e2.span(), SimpleSpan::new(10, 14));
}

#[test]
fn unicode_error_source() {
  use core::error::Error;

  let e1 = UnicodeEscapeError::<char>::incomplete_fixed_unicode_escape(SimpleSpan::new(10, 14));
  assert!(e1.source().is_some());

  let e2 = UnicodeEscapeError::<char>::empty_variable_unicode_escape(SimpleSpan::new(10, 14));
  assert!(e2.source().is_some());
}
