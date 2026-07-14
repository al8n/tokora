use super::*;
use crate::error::InvalidHexDigits;
use crate::span::SimpleSpan;
use crate::utils::PositionedChar;

// ── IncompleteHexEscape ────────────────────────────────────────────

#[test]
fn incomplete_new_and_span() {
  let err = IncompleteHexEscape::new(SimpleSpan::new(10, 13));
  assert_eq!(err.span(), SimpleSpan::new(10, 13));
}

#[test]
fn incomplete_span_ref() {
  let err = IncompleteHexEscape::new(SimpleSpan::new(10, 13));
  assert_eq!(err.span_ref(), SimpleSpan::new(&10, &13));
}

#[test]
fn incomplete_bump() {
  let mut err = IncompleteHexEscape::new(SimpleSpan::new(10, 12));
  err.bump(&5);
  assert_eq!(err.span(), SimpleSpan::new(15, 17));
}

#[test]
fn incomplete_display() {
  let err = IncompleteHexEscape::new(SimpleSpan::new(10, 13));
  let s = std::format!("{}", err);
  assert!(s.contains("incomplete hexadecimal escape"));
}

#[test]
fn incomplete_error_trait() {
  let err = IncompleteHexEscape::new(SimpleSpan::new(10, 13));
  let e: &dyn core::error::Error = &err;
  assert!(e.source().is_none());
}

// ── MalformedHexEscape ─────────────────────────────────────────────

#[test]
fn malformed_new_span_digits() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let err = MalformedHexEscape::new(digits, SimpleSpan::new(10, 14));
  assert_eq!(err.span(), SimpleSpan::new(10, 14));
  assert_eq!(err.digits().len(), 1);
}

#[test]
fn malformed_digits_ref() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let err = MalformedHexEscape::new(digits, SimpleSpan::new(10, 14));
  assert_eq!(err.digits_ref().len(), 1);
}

#[test]
fn malformed_digits_mut() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let mut err = MalformedHexEscape::new(digits, SimpleSpan::new(10, 14));
  let dm = err.digits_mut();
  assert_eq!(dm.len(), 1);
}

#[test]
fn malformed_span_ref() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let err = MalformedHexEscape::new(digits, SimpleSpan::new(10, 14));
  assert_eq!(err.span_ref(), SimpleSpan::new(&10, &14));
}

#[test]
fn malformed_span_mut() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let mut err = MalformedHexEscape::new(digits, SimpleSpan::new(10, 14));
  let mut sm = err.span_mut();
  **sm.start_mut() = 20;
}

#[test]
fn malformed_bump() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let mut err = MalformedHexEscape::new(digits, SimpleSpan::new(10, 14));
  err.bump(&5);
  assert_eq!(err.span(), SimpleSpan::new(15, 19));
}

#[test]
fn malformed_display() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let err = MalformedHexEscape::new(digits, SimpleSpan::new(10, 14));
  let s = std::format!("{}", err);
  assert!(s.contains("malformed hexadecimal escape"));
}

#[test]
fn malformed_error_trait() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let err = MalformedHexEscape::new(digits, SimpleSpan::new(10, 14));
  let e: &dyn core::error::Error = &err;
  assert!(e.source().is_none());
}

// ── HexEscapeError ─────────────────────────────────────────────────

#[test]
fn hex_error_incomplete() {
  let err = HexEscapeError::<char>::incomplete(SimpleSpan::new(10, 13));
  assert!(err.is_incomplete());
  assert!(!err.is_malformed());
  assert_eq!(err.span(), SimpleSpan::new(10, 13));
}

#[test]
fn hex_error_malformed() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let err = HexEscapeError::malformed(digits, SimpleSpan::new(10, 14));
  assert!(!err.is_incomplete());
  assert!(err.is_malformed());
  assert_eq!(err.span(), SimpleSpan::new(10, 14));
}

#[test]
fn hex_error_bump_incomplete() {
  let mut err = HexEscapeError::<char>::incomplete(SimpleSpan::new(10, 12));
  err.bump(&5);
  assert_eq!(err.span(), SimpleSpan::new(15, 17));
}

#[test]
fn hex_error_bump_malformed() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let mut err = HexEscapeError::malformed(digits, SimpleSpan::new(10, 14));
  err.bump(&5);
  assert_eq!(err.span(), SimpleSpan::new(15, 19));
}

#[test]
fn hex_error_display_incomplete() {
  let err = HexEscapeError::<char>::incomplete(SimpleSpan::new(10, 13));
  let s = std::format!("{}", err);
  assert!(s.contains("incomplete hexadecimal escape"));
}

#[test]
fn hex_error_display_malformed() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let err = HexEscapeError::malformed(digits, SimpleSpan::new(10, 14));
  let s = std::format!("{}", err);
  assert!(s.contains("malformed hexadecimal escape"));
}

#[test]
fn hex_error_source_incomplete() {
  let err = HexEscapeError::<char>::incomplete(SimpleSpan::new(10, 13));
  let e: &dyn core::error::Error = &err;
  assert!(e.source().is_some());
}

#[test]
fn hex_error_source_malformed() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let err = HexEscapeError::malformed(digits, SimpleSpan::new(10, 14));
  let e: &dyn core::error::Error = &err;
  assert!(e.source().is_some());
}

#[test]
fn hex_error_from_incomplete() {
  let incomplete = IncompleteHexEscape::new(SimpleSpan::new(10, 13));
  let err: HexEscapeError<char> = incomplete.into();
  assert!(err.is_incomplete());
}

#[test]
fn hex_error_from_malformed() {
  let digits: InvalidHexDigits<char, 2> =
    InvalidHexDigits::from_positioned_char(PositionedChar::with_position('G', 12));
  let malformed = MalformedHexEscape::new(digits, SimpleSpan::new(10, 14));
  let err: HexEscapeError<char> = malformed.into();
  assert!(err.is_malformed());
}
