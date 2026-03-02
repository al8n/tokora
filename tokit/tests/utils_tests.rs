use tokit::SimpleSpan;
/// Tests for `utils` module traits: `IsAsciiChar`, `CharLen`, and `Lexeme` methods.
use tokit::utils::{IsAsciiChar, Lexeme, PositionedChar};

// ── IsAsciiChar for char ──────────────────────────────────────────────────────

#[test]
fn char_is_ascii_char_match() {
  use ascii::AsciiChar;
  let c = 'a';
  assert!(c.is_ascii_char(AsciiChar::a));
}

#[test]
fn char_is_ascii_char_no_match() {
  use ascii::AsciiChar;
  let c = 'b';
  assert!(!c.is_ascii_char(AsciiChar::a));
}

#[test]
fn char_is_ascii_char_non_ascii() {
  use ascii::AsciiChar;
  let c = 'é';
  assert!(!c.is_ascii_char(AsciiChar::a));
}

#[test]
fn char_is_ascii_digit_true() {
  let c = '7';
  assert!(c.is_ascii_digit());
}

#[test]
fn char_is_ascii_digit_false() {
  let c = 'a';
  assert!(!c.is_ascii_digit());
}

#[test]
fn char_one_of_match() {
  use ascii::AsciiChar;
  let c = 'b';
  assert!(c.one_of(&[AsciiChar::a, AsciiChar::b, AsciiChar::c]));
}

#[test]
fn char_one_of_no_match() {
  use ascii::AsciiChar;
  let c = 'z';
  assert!(!c.one_of(&[AsciiChar::a, AsciiChar::b, AsciiChar::c]));
}

// ── IsAsciiChar for u8 ────────────────────────────────────────────────────────

#[test]
fn u8_is_ascii_char_match() {
  use ascii::AsciiChar;
  let b: u8 = b'x';
  assert!(b.is_ascii_char(AsciiChar::x));
}

#[test]
fn u8_is_ascii_char_no_match() {
  use ascii::AsciiChar;
  let b: u8 = b'y';
  assert!(!b.is_ascii_char(AsciiChar::x));
}

#[test]
fn u8_is_ascii_digit_true() {
  let b: u8 = b'5';
  assert!(b.is_ascii_digit());
}

#[test]
fn u8_is_ascii_digit_false() {
  let b: u8 = b'Z';
  assert!(!b.is_ascii_digit());
}

// ── IsAsciiChar for str ───────────────────────────────────────────────────────

#[test]
fn str_is_ascii_char_single_match() {
  use ascii::AsciiChar;
  let s: &str = "x";
  assert!(s.is_ascii_char(AsciiChar::x));
}

#[test]
fn str_is_ascii_char_single_no_match() {
  use ascii::AsciiChar;
  let s: &str = "y";
  assert!(!s.is_ascii_char(AsciiChar::x));
}

#[test]
fn str_is_ascii_char_multi_char() {
  use ascii::AsciiChar;
  let s: &str = "ab";
  assert!(!s.is_ascii_char(AsciiChar::a));
}

#[test]
fn str_is_ascii_digit_single() {
  let s: &str = "3";
  assert!(s.is_ascii_digit());
}

#[test]
fn str_is_ascii_digit_multi() {
  let s: &str = "12";
  assert!(!s.is_ascii_digit());
}

// ── IsAsciiChar for [u8] ──────────────────────────────────────────────────────

#[test]
fn bytes_is_ascii_char_single_match() {
  use ascii::AsciiChar;
  let b: &[u8] = b"x";
  assert!(b.is_ascii_char(AsciiChar::x));
}

#[test]
fn bytes_is_ascii_char_single_no_match() {
  use ascii::AsciiChar;
  let b: &[u8] = b"y";
  assert!(!b.is_ascii_char(AsciiChar::x));
}

#[test]
fn bytes_is_ascii_char_multi() {
  use ascii::AsciiChar;
  let b: &[u8] = b"ab";
  assert!(!b.is_ascii_char(AsciiChar::a));
}

#[test]
fn bytes_is_ascii_digit_single() {
  let b: &[u8] = b"9";
  assert!(b.is_ascii_digit());
}

#[test]
fn bytes_is_ascii_digit_multi() {
  let b: &[u8] = b"99";
  assert!(!b.is_ascii_digit());
}

// ── IsAsciiChar via &T delegation ─────────────────────────────────────────────

#[test]
fn ref_char_is_ascii_char() {
  use ascii::AsciiChar;
  let c = 'a';
  let r = &c;
  assert!(r.is_ascii_char(AsciiChar::a));
}

#[test]
fn ref_char_is_ascii_digit() {
  let c = '5';
  let r = &c;
  assert!(r.is_ascii_digit());
}

#[test]
#[allow(clippy::needless_borrow)]
fn ref_char_one_of() {
  use ascii::AsciiChar;
  let c = 'b';
  assert!((&c).one_of(&[AsciiChar::a, AsciiChar::b]));
}

#[test]
fn ref_u8_is_ascii_char() {
  use ascii::AsciiChar;
  let b: u8 = b'z';
  let r = &b;
  assert!(r.is_ascii_char(AsciiChar::z));
}

#[test]
#[allow(clippy::needless_borrow)]
fn ref_u8_is_ascii_digit() {
  let b: u8 = b'3';

  assert!((&b).is_ascii_digit());
}

// ── IsAsciiChar via &mut T delegation ────────────────────────────────────────

#[test]
fn ref_mut_char_is_ascii_char() {
  use ascii::AsciiChar;
  let mut c = 'a';
  let r = &mut c;
  assert!(r.is_ascii_char(AsciiChar::a));
}

#[test]
fn ref_mut_char_is_ascii_digit() {
  let mut c = '5';
  let r = &mut c;
  assert!(r.is_ascii_digit());
}

#[test]
#[allow(clippy::unnecessary_mut_passed)]
fn ref_mut_char_one_of() {
  use ascii::AsciiChar;
  let mut c = 'b';
  assert!((&mut c).one_of(&[AsciiChar::a, AsciiChar::b]));
}

// ── Lexeme methods ────────────────────────────────────────────────────────────

#[test]
fn lexeme_from_char_constructor() {
  let l: Lexeme<char, usize> = Lexeme::from_char(5, 'x');
  assert!(l.is_char());
  assert_eq!(l.start(), 5);
}

#[test]
fn lexeme_from_range_constructor() {
  let l: Lexeme<char, usize> = Lexeme::from_range(5..10);
  assert!(l.is_range());
  assert_eq!(l.start(), 5);
  assert_eq!(l.end(), 10);
}

#[test]
fn lexeme_from_range_const_constructor() {
  let l: Lexeme<char, usize> = Lexeme::from_range_const(SimpleSpan::new(5, 10));
  assert!(l.is_range());
  assert_eq!(l.start(), 5);
}

#[test]
fn lexeme_start_ref_char() {
  let l: Lexeme<char, usize> = Lexeme::from(PositionedChar::with_position('x', 5usize));
  assert_eq!(l.start_ref(), &5usize);
}

#[test]
fn lexeme_start_ref_range() {
  let l: Lexeme<char, usize> = Lexeme::from(SimpleSpan::new(10usize, 15usize));
  assert_eq!(l.start_ref(), &10usize);
}

#[test]
fn lexeme_end_char() {
  let l: Lexeme<char, usize> = Lexeme::from(PositionedChar::with_position('x', 5usize));
  assert_eq!(l.end(), 6); // 'x' is 1 byte
}

#[test]
fn lexeme_end_char_multibyte() {
  // '€' is 3 bytes in UTF-8
  let l: Lexeme<char, usize> = Lexeme::from(PositionedChar::with_position('€', 10usize));
  assert_eq!(l.end(), 13);
}

#[test]
fn lexeme_map_char_variant() {
  let l: Lexeme<char, usize> = Lexeme::from(PositionedChar::with_position('a', 5usize));
  let upper = l.map(|c| c.to_ascii_uppercase());
  assert_eq!(upper.unwrap_char().char(), 'A');
}

#[test]
fn lexeme_map_range_variant_unchanged() {
  let l: Lexeme<char, usize> = Lexeme::from(SimpleSpan::new(5usize, 10usize));
  let mapped = l.map(|c: char| c.to_ascii_uppercase());
  assert!(mapped.is_range());
}

#[test]
fn lexeme_span_with_char() {
  let l: Lexeme<char, usize> = Lexeme::from(PositionedChar::with_position('€', 10usize));
  let span = l.span_with(|c: &char| c.len_utf8());
  assert_eq!(span.start(), 10);
  assert_eq!(span.end(), 13);
}

#[test]
fn lexeme_span_with_range() {
  let l: Lexeme<char, usize> = Lexeme::from(SimpleSpan::new(5usize, 10usize));
  let span = l.span_with(|_: &char| 1);
  assert_eq!(span, SimpleSpan::new(5, 10));
}

#[test]
fn lexeme_span_char() {
  let l: Lexeme<char, usize> = Lexeme::from(PositionedChar::with_position('x', 5usize));
  assert_eq!(l.span(), SimpleSpan::new(5, 6));
}

#[test]
fn lexeme_span_range() {
  let l: Lexeme<char, usize> = Lexeme::from(SimpleSpan::new(10usize, 15usize));
  assert_eq!(l.span(), SimpleSpan::new(10, 15));
}

#[test]
fn lexeme_display_char() {
  let l: Lexeme<char, usize> = Lexeme::from(PositionedChar::with_position('n', 11usize));
  let s = format!("{l}");
  assert!(s.contains('n'));
}

#[test]
fn lexeme_display_range() {
  let l: Lexeme<char, usize> = Lexeme::from(SimpleSpan::new(5usize, 9usize));
  let s = format!("{l}");
  assert!(!s.is_empty());
}

#[test]
fn lexeme_bump_char() {
  let mut l: Lexeme<char, usize> = Lexeme::from(PositionedChar::with_position('n', 5usize));
  l.bump(&10usize);
  assert_eq!(l.start(), 15);
}

#[test]
fn lexeme_bump_range() {
  let mut l: Lexeme<char, usize> = Lexeme::from(SimpleSpan::new(5usize, 9usize));
  l.bump(&10usize);
  assert_eq!(l.start(), 15);
  assert_eq!(l.end(), 19);
}
