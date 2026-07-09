use super::*;

// --- IsAsciiChar for char ---

#[test]
fn char_is_ascii_char() {
  assert!(IsAsciiChar::is_ascii_char(&'a', ascii::AsciiChar::a));
  assert!(!IsAsciiChar::is_ascii_char(&'b', ascii::AsciiChar::a));
  assert!(!IsAsciiChar::is_ascii_char(
    &'\u{00E9}',
    ascii::AsciiChar::a
  ));
}

#[test]
fn char_is_ascii_digit() {
  assert!(IsAsciiChar::is_ascii_digit(&'0'));
  assert!(IsAsciiChar::is_ascii_digit(&'9'));
  assert!(!IsAsciiChar::is_ascii_digit(&'a'));
}

// --- IsAsciiChar for u8 ---

#[test]
fn u8_is_ascii_char() {
  assert!(IsAsciiChar::is_ascii_char(&b'a', ascii::AsciiChar::a));
  assert!(!IsAsciiChar::is_ascii_char(&b'b', ascii::AsciiChar::a));
}

#[test]
fn u8_is_ascii_digit() {
  assert!(IsAsciiChar::is_ascii_digit(&b'0'));
  assert!(IsAsciiChar::is_ascii_digit(&b'9'));
  assert!(!IsAsciiChar::is_ascii_digit(&b'a'));
}

// --- IsAsciiChar for str ---

#[test]
fn str_is_ascii_char() {
  assert!(IsAsciiChar::is_ascii_char("a", ascii::AsciiChar::a));
  assert!(!IsAsciiChar::is_ascii_char("b", ascii::AsciiChar::a));
  assert!(!IsAsciiChar::is_ascii_char("ab", ascii::AsciiChar::a));
  assert!(!IsAsciiChar::is_ascii_char("", ascii::AsciiChar::a));
}

#[test]
fn str_is_ascii_digit() {
  assert!(IsAsciiChar::is_ascii_digit("5"));
  assert!(!IsAsciiChar::is_ascii_digit("a"));
  assert!(!IsAsciiChar::is_ascii_digit("55"));
  assert!(!IsAsciiChar::is_ascii_digit(""));
}

// --- IsAsciiChar for [u8] ---

#[test]
fn slice_is_ascii_char() {
  assert!(IsAsciiChar::is_ascii_char(
    [b'a'].as_slice(),
    ascii::AsciiChar::a
  ));
  assert!(!IsAsciiChar::is_ascii_char(
    [b'a', b'b'].as_slice(),
    ascii::AsciiChar::a
  ));
  assert!(!IsAsciiChar::is_ascii_char(
    [].as_slice(),
    ascii::AsciiChar::a
  ));
}

#[test]
fn slice_is_ascii_digit() {
  assert!(IsAsciiChar::is_ascii_digit([b'5'].as_slice()));
  assert!(!IsAsciiChar::is_ascii_digit([b'a'].as_slice()));
  assert!(!IsAsciiChar::is_ascii_digit([b'5', b'6'].as_slice()));
  assert!(!IsAsciiChar::is_ascii_digit([].as_slice()));
}

// --- IsAsciiChar for references ---

#[test]
fn ref_is_ascii_char() {
  let ch = 'a';
  assert!(IsAsciiChar::is_ascii_char(&&ch, ascii::AsciiChar::a));
  assert!(IsAsciiChar::is_ascii_digit(&&'5'));
}

#[test]
fn mut_ref_is_ascii_char() {
  let mut ch = 'a';
  assert!(IsAsciiChar::is_ascii_char(&&mut ch, ascii::AsciiChar::a));
  assert!(IsAsciiChar::is_ascii_digit(&&mut '5'));
}

// --- one_of ---

#[test]
fn one_of_matches() {
  let choices = &[
    ascii::AsciiChar::a,
    ascii::AsciiChar::b,
    ascii::AsciiChar::c,
  ];
  assert!(IsAsciiChar::one_of(&'a', choices));
  assert!(IsAsciiChar::one_of(&'b', choices));
  assert!(!IsAsciiChar::one_of(&'d', choices));
  assert!(!IsAsciiChar::one_of(&'A', choices));
}

#[test]
fn one_of_ref() {
  let choices = &[ascii::AsciiChar::a];
  assert!(IsAsciiChar::one_of(&&'a', choices));
  let mut ch = 'a';
  assert!(IsAsciiChar::one_of(&&mut ch, choices));
}

// --- CharLen ---

#[test]
fn char_len_u8() {
  assert_eq!(CharLen::char_len(&42u8), 1);
  assert_eq!(CharLen::char_len(&0u8), 1);
  assert_eq!(CharLen::char_len(&255u8), 1);
}

#[test]
fn char_len_char() {
  assert_eq!(CharLen::char_len(&'a'), 1);
  assert_eq!(CharLen::char_len(&'\u{00E9}'), 2);
  assert_eq!(CharLen::char_len(&'\u{20AC}'), 3); // Euro sign
  assert_eq!(CharLen::char_len(&'\u{1F980}'), 4); // Crab emoji
}

#[test]
fn char_len_ref() {
  let ch = 'a';
  assert_eq!(CharLen::char_len(&&ch), 1);
}

#[test]
fn char_len_positioned_char() {
  let pc = PositionedChar::with_position('a', 0usize);
  assert_eq!(CharLen::char_len(&pc), 1);
  let pc2 = PositionedChar::with_position('\u{20AC}', 0usize);
  assert_eq!(CharLen::char_len(&pc2), 3);
}

// --- IntoComponents ---

#[test]
fn into_components_trait() {
  // Test via a punctuator which implements IntoComponents
  use crate::punct::Comma;
  let c = Comma::<usize, &str>::with_content(42, "test");
  let (span, content) = IntoComponents::into_components(c);
  assert_eq!(span, 42);
  assert_eq!(content, "test");
}

// --- Additional mut ref tests ---

#[test]
fn mut_ref_is_ascii_digit() {
  let mut ch = '5';
  assert!(IsAsciiChar::is_ascii_digit(&&mut ch));
  let mut ch2 = 'a';
  assert!(!IsAsciiChar::is_ascii_digit(&&mut ch2));
}

#[test]
fn mut_ref_one_of() {
  let choices = &[ascii::AsciiChar::a, ascii::AsciiChar::b];
  let mut ch = 'a';
  assert!(IsAsciiChar::one_of(&&mut ch, choices));
  let mut ch2 = 'z';
  assert!(!IsAsciiChar::one_of(&&mut ch2, choices));
}

#[test]
fn ref_one_of_empty() {
  let choices: &[ascii::AsciiChar] = &[];
  assert!(!IsAsciiChar::one_of(&'a', choices));
}

// --- CharLen for positioned char ref ---

#[test]
fn char_len_positioned_char_ref() {
  let pc = PositionedChar::with_position('a', 0usize);
  assert_eq!(CharLen::char_len(&&pc), 1);
}

// --- non-ASCII char tests ---

#[test]
fn char_non_ascii_is_not_ascii_char() {
  // Multi-byte char should not match any AsciiChar
  assert!(!IsAsciiChar::is_ascii_char(
    &'\u{1F600}',
    ascii::AsciiChar::a
  ));
}

#[test]
fn str_multibyte_not_digit() {
  // Multi-byte string should not be a digit
  assert!(!IsAsciiChar::is_ascii_digit("\u{00E9}"));
}

#[test]
fn slice_multibyte_not_digit() {
  assert!(!IsAsciiChar::is_ascii_digit([0xFF].as_slice()));
}
