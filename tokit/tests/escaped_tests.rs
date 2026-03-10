/// Tests for `utils::{SingleCharEscape, MultiCharEscape, EscapedLexeme}`.
use tokit::{
  SimpleSpan,
  utils::{EscapedLexeme, Lexeme, MultiCharEscape, PositionedChar, SingleCharEscape},
};

// ── SingleCharEscape ──────────────────────────────────────────────────────

#[test]
fn single_from_positioned_char() {
  let pc = PositionedChar::with_position('n', 11usize);
  let e = SingleCharEscape::from_positioned_char(SimpleSpan::new(10, 12), pc);
  assert_eq!(e.char(), 'n');
  assert_eq!(e.position(), 11);
  assert_eq!(e.span(), SimpleSpan::new(10, 12));
}

#[test]
fn single_from_char() {
  let e = SingleCharEscape::from_char(SimpleSpan::new(14, 16), 15usize, 'r');
  assert_eq!(e.char(), 'r');
  assert_eq!(e.position(), 15);
  assert_eq!(e.span(), SimpleSpan::new(14, 16));
}

#[test]
fn single_char_ref() {
  let e = SingleCharEscape::from_char(SimpleSpan::new(0, 2), 1usize, 't');
  assert_eq!(*e.char_ref(), 't');
}

#[test]
fn single_char_mut() {
  let mut e = SingleCharEscape::from_char(SimpleSpan::new(0, 2), 1usize, 't');
  *e.char_mut() = 'n';
  assert_eq!(e.char(), 'n');
}

#[test]
fn single_position_ref() {
  let e = SingleCharEscape::from_char(SimpleSpan::new(10, 12), 11usize, 'n');
  assert_eq!(*e.position_ref(), 11usize);
}

#[test]
fn single_span_ref() {
  let e = SingleCharEscape::from_char(SimpleSpan::new(4, 6), 5usize, 'r');
  assert_eq!(e.span_ref(), SimpleSpan::new(&4usize, &6usize));
}

#[test]
fn single_span_mut() {
  let mut e = SingleCharEscape::from_char(SimpleSpan::new(0, 2), 1usize, 'n');
  **e.span_mut().start_mut() = 5;
  assert_eq!(e.span().start(), 5);
}

#[test]
fn single_bump() {
  let mut e = SingleCharEscape::from_char(SimpleSpan::new(10, 12), 11usize, 'n');
  e.bump(&5usize);
  assert_eq!(e.position(), 16);
  assert_eq!(e.span(), SimpleSpan::new(15, 17));
}

#[test]
fn single_display() {
  let e = SingleCharEscape::from_char(SimpleSpan::new(10, 12), 11usize, 'n');
  let s = format!("{e}");
  assert!(s.contains('n'));
}

#[test]
fn single_debug() {
  let e = SingleCharEscape::from_char(SimpleSpan::new(0, 2), 1usize, 't');
  let s = format!("{e:?}");
  assert!(s.contains("SingleCharEscape"));
}

// ── MultiCharEscape ────────────────────────────────────────────────────────

#[test]
fn multi_new_and_getters() {
  let e = MultiCharEscape::new(SimpleSpan::new(6, 9), SimpleSpan::new(5, 9));
  assert_eq!(e.content(), SimpleSpan::new(6, 9));
  assert_eq!(e.span(), SimpleSpan::new(5, 9));
}

#[test]
fn multi_content_ref() {
  let e = MultiCharEscape::new(SimpleSpan::new(6, 9), SimpleSpan::new(5, 9));
  assert_eq!(e.content_ref(), &SimpleSpan::new(6, 9));
}

#[test]
fn multi_content_mut() {
  let mut e = MultiCharEscape::new(SimpleSpan::new(6, 9), SimpleSpan::new(5, 9));
  e.content_mut().set_end(10);
  assert_eq!(e.content().end(), 10);
}

#[test]
fn multi_span_ref() {
  let e = MultiCharEscape::new(SimpleSpan::new(6, 9), SimpleSpan::new(5, 9));
  assert_eq!(e.span_ref(), &SimpleSpan::new(5, 9));
}

#[test]
fn multi_span_mut() {
  let mut e = MultiCharEscape::new(SimpleSpan::new(6, 9), SimpleSpan::new(5, 9));
  e.span_mut().set_start(0);
  assert_eq!(e.span().start(), 0);
}

#[test]
fn multi_bump() {
  let mut e = MultiCharEscape::new(SimpleSpan::new(6, 9), SimpleSpan::new(5, 9));
  e.bump(&10usize);
  assert_eq!(e.content(), SimpleSpan::new(16, 19));
  assert_eq!(e.span(), SimpleSpan::new(15, 19));
}

#[test]
fn multi_display() {
  let e = MultiCharEscape::new(SimpleSpan::new(6, 9), SimpleSpan::new(5, 9));
  let s = format!("{e}");
  assert!(s.contains("escape sequence"));
}

#[test]
fn multi_debug() {
  let e = MultiCharEscape::new(SimpleSpan::new(6, 9), SimpleSpan::new(5, 9));
  let s = format!("{e:?}");
  assert!(s.contains("MultiCharEscape"));
}

// ── EscapedLexeme ──────────────────────────────────────────────────────────

#[test]
fn escaped_from_positioned_char() {
  let pc = PositionedChar::with_position('n', 11usize);
  let e = EscapedLexeme::from_positioned_char(SimpleSpan::new(10, 12), pc);
  assert_eq!(e.span(), SimpleSpan::new(10, 12));
  assert!(e.lexeme_ref().is_char());
}

#[test]
fn escaped_from_char() {
  let e: EscapedLexeme = EscapedLexeme::from_char(SimpleSpan::new(10, 12), 11usize, 't');
  assert!(e.lexeme_ref().is_char());
  assert_eq!(e.span(), SimpleSpan::new(10, 12));
}

#[test]
fn escaped_from_sequence() {
  let e: EscapedLexeme = EscapedLexeme::from_sequence(SimpleSpan::new(5, 9), SimpleSpan::new(6, 9));
  assert_eq!(e.span(), SimpleSpan::new(5, 9));
  assert!(e.lexeme_ref().is_range());
}

#[test]
fn escaped_new_with_lexeme() {
  let lexeme = Lexeme::from(PositionedChar::with_position('n', 11usize));
  let e = EscapedLexeme::new(SimpleSpan::new(10, 12), lexeme);
  assert!(e.lexeme_ref().is_char());
}

#[test]
fn escaped_span_ref() {
  let e: EscapedLexeme = EscapedLexeme::from_char(SimpleSpan::new(10, 12), 11usize, 'n');
  assert_eq!(e.span_ref(), SimpleSpan::new(&10usize, &12usize));
}

#[test]
fn escaped_span_mut() {
  let mut e: EscapedLexeme = EscapedLexeme::from_char(SimpleSpan::new(10, 12), 11usize, 'n');
  **e.span_mut().start_mut() = 5;
  assert_eq!(e.span().start(), 5);
}

#[test]
fn escaped_lexeme_copy() {
  let e: EscapedLexeme = EscapedLexeme::from_char(SimpleSpan::new(10, 12), 11usize, 'n');
  let lex = e.lexeme();
  assert!(lex.is_char());
}

#[test]
fn escaped_lexeme_ref() {
  let e: EscapedLexeme = EscapedLexeme::from_char(SimpleSpan::new(10, 12), 11usize, 'n');
  assert!(e.lexeme_ref().is_char());
}

#[test]
fn escaped_lexeme_mut() {
  let mut e: EscapedLexeme =
    EscapedLexeme::from_sequence(SimpleSpan::new(5, 9), SimpleSpan::new(6, 9));
  assert!(e.lexeme_mut().is_range());
}

#[test]
fn escaped_bump_char() {
  let mut e: EscapedLexeme = EscapedLexeme::from_char(SimpleSpan::new(10, 12), 11usize, 'n');
  e.bump(&5usize);
  assert_eq!(e.span(), SimpleSpan::new(15, 17));
}

#[test]
fn escaped_bump_sequence() {
  let mut e: EscapedLexeme =
    EscapedLexeme::from_sequence(SimpleSpan::new(5, 9), SimpleSpan::new(6, 9));
  e.bump(&10usize);
  assert_eq!(e.span(), SimpleSpan::new(15, 19));
}

#[test]
fn escaped_display_char() {
  let e: EscapedLexeme = EscapedLexeme::from_char(SimpleSpan::new(10, 12), 11usize, 'n');
  let s = format!("{e}");
  assert!(s.contains('n'));
}

#[test]
fn escaped_display_sequence() {
  let e: EscapedLexeme = EscapedLexeme::from_sequence(SimpleSpan::new(5, 9), SimpleSpan::new(6, 9));
  let s = format!("{e}");
  assert!(s.contains("escape sequence"));
}

#[test]
fn escaped_debug() {
  let e: EscapedLexeme = EscapedLexeme::from_char(SimpleSpan::new(0, 2), 1usize, 'n');
  let s = format!("{e:?}");
  assert!(s.contains("EscapedLexeme"));
}
