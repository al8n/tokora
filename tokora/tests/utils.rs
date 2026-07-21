use tokora::SimpleSpan;
/// Tests for `utils` module traits: `IsAsciiChar`, `CharLen`, and `Lexeme` methods.
use tokora::utils::{IsAsciiChar, Lexeme, PositionedChar};

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

// ── std-only tests (CowStr, OneOf) ────────────────────────────────────────────

#[cfg(feature = "std")]
mod std_utils {
  use std::borrow::Cow;
  use tokora::utils::{CowStr, OneOf};

  #[test]
  fn cowstr_from_static() {
    let s = CowStr::from_static("hello");
    assert_eq!(s.as_str(), "hello");
  }

  #[test]
  fn cowstr_from_string() {
    let s = CowStr::from_string(String::from("dynamic"));
    assert_eq!(s.as_str(), "dynamic");
  }

  #[test]
  fn cowstr_to_mut() {
    let mut s = CowStr::from_static("hello");
    let m = s.to_mut();
    m.push_str(" world");
    assert_eq!(s.as_str(), "hello world");
  }

  #[test]
  fn cowstr_into_inner() {
    let s = CowStr::from_static("test");
    let inner = s.into_inner();
    assert_eq!(&*inner, "test");
  }

  #[test]
  fn cowstr_as_inner() {
    let s = CowStr::from_static("test");
    let _ = s.as_inner();
  }

  #[test]
  fn cowstr_from_string_impl() {
    let s: CowStr = String::from("owned").into();
    assert_eq!(s.as_str(), "owned");
  }

  #[test]
  fn cowstr_from_cow() {
    let cow: Cow<'static, str> = Cow::Borrowed("borrowed");
    let s: CowStr = cow.into();
    assert_eq!(s.as_str(), "borrowed");
  }

  #[test]
  fn cowstr_into_cow() {
    let s = CowStr::from_static("test");
    let cow: Cow<'static, str> = s.into();
    assert_eq!(&*cow, "test");
  }

  #[test]
  fn cowstr_ref_into_cow() {
    let s = CowStr::from_static("test");
    let cow: Cow<'static, str> = (&s).into();
    assert_eq!(&*cow, "test");
  }

  #[test]
  fn cowstr_as_ref() {
    let s = CowStr::from_static("test");
    let r: &str = s.as_ref();
    assert_eq!(r, "test");
  }

  #[test]
  fn cowstr_borrow() {
    use std::borrow::Borrow;
    let s = CowStr::from_static("test");
    let r: &str = s.borrow();
    assert_eq!(r, "test");
  }

  #[test]
  fn cowstr_to_mut_from_static() {
    let mut s = CowStr::from_static("test");
    let m = s.to_mut();
    m.push('!');
    assert_eq!(s.as_str(), "test!");
  }

  #[test]
  fn cowstr_display() {
    let s = CowStr::from_static("hello");
    assert_eq!(format!("{s}"), "hello");
  }

  #[test]
  fn cowstr_debug() {
    let s = CowStr::from_static("hello");
    let _ = format!("{s:?}");
  }

  #[test]
  fn oneof_from_slice() {
    let items: &[i32] = &[1, 2, 3];
    let o = OneOf::from_slice(items);
    assert_eq!(o.as_slice(), &[1, 2, 3]);
  }

  #[test]
  fn oneof_from_vec() {
    let o = OneOf::from_vec(vec![1, 2, 3]);
    assert_eq!(o.as_slice(), &[1, 2, 3]);
  }

  #[test]
  fn oneof_to_mut() {
    let items: &[i32] = &[1, 2];
    let mut o = OneOf::from_slice(items);
    let m = o.to_mut();
    assert_eq!(m, &[1, 2]);
  }

  #[test]
  fn oneof_into_inner() {
    let o = OneOf::from_vec(vec![42]);
    let inner = o.into_inner();
    assert_eq!(&*inner, &[42]);
  }

  #[test]
  fn oneof_as_inner() {
    let o = OneOf::from_vec(vec![1]);
    let _ = o.as_inner();
  }

  #[test]
  fn oneof_from_vec_impl() {
    let o: OneOf<'_, i32> = vec![1, 2].into();
    assert_eq!(o.as_slice(), &[1, 2]);
  }

  #[test]
  fn oneof_from_cow() {
    let cow: Cow<'_, [i32]> = Cow::Borrowed(&[1, 2]);
    let o: OneOf<'_, i32> = cow.into();
    assert_eq!(o.as_slice(), &[1, 2]);
  }

  #[test]
  fn oneof_into_cow() {
    let o = OneOf::from_vec(vec![1]);
    let cow: Cow<'_, [i32]> = o.into();
    assert_eq!(&*cow, &[1]);
  }

  #[test]
  fn oneof_ref_into_cow() {
    let o = OneOf::from_vec(vec![1]);
    let cow: Cow<'_, [i32]> = (&o).into();
    assert_eq!(&*cow, &[1]);
  }

  #[test]
  fn oneof_as_ref() {
    let o = OneOf::from_vec(vec![1, 2]);
    let r: &[i32] = o.as_ref();
    assert_eq!(r, &[1, 2]);
  }

  #[test]
  fn oneof_borrow() {
    use std::borrow::Borrow;
    let o = OneOf::from_vec(vec![1, 2]);
    let r: &[i32] = o.borrow();
    assert_eq!(r, &[1, 2]);
  }

  #[test]
  fn oneof_display() {
    // OneOf's Display delegates to the inner Cow<[T]>; exercise via Debug which always works
    let o = OneOf::from_vec(vec![1, 2, 3]);
    let _ = format!("{o:?}");
  }

  #[test]
  fn oneof_debug() {
    let o = OneOf::from_vec(vec![1, 2]);
    let _ = format!("{o:?}");
  }
}

// ── hipstr `Equivalent` (both directions) ───────────────────────────────────

#[cfg(feature = "hipstr_0_8")]
#[test]
fn hipstr_equivalent_both_directions() {
  use hipstr_0_8::{HipByt, HipStr};
  use tokora::utils::cmp::Equivalent;

  // A generic helper whose bound `A: Equivalent<B>` forces the *impl* to
  // exist for the concrete `A`. Plain `hipstr.equivalent(..)` would deref-coerce
  // `HipStr`->`str` and pass even without a `HipStr: Equivalent<_>` impl, so it
  // cannot detect the one-directional-impl bug: method-call syntax would deref-coerce
  // and silently test the blanket impl.
  fn equiv<A, B>(a: &A, b: &B) -> bool
  where
    A: Equivalent<B> + ?Sized,
    B: ?Sized,
  {
    a.equivalent(b)
  }

  let hs = HipStr::from("hello");
  // HipStr on the left (the previously-missing direction).
  assert!(equiv::<HipStr<'_>, str>(&hs, "hello"));
  assert!(!equiv::<HipStr<'_>, str>(&hs, "world"));
  assert!(equiv::<HipStr<'_>, [u8]>(&hs, b"hello"));
  assert!(equiv::<HipStr<'_>, HipStr<'_>>(&hs, &HipStr::from("hello")));
  // str/[u8] on the left (existing blanket direction).
  assert!(equiv::<str, HipStr<'_>>("hello", &hs));
  assert!(equiv::<[u8], HipStr<'_>>(b"hello", &hs));

  let hb = HipByt::from(b"hello".as_slice());
  // HipByt on the left (the previously-missing direction).
  assert!(equiv::<HipByt<'_>, [u8]>(&hb, b"hello"));
  assert!(!equiv::<HipByt<'_>, [u8]>(&hb, b"world"));
  assert!(equiv::<HipByt<'_>, str>(&hb, "hello"));
  assert!(equiv::<HipByt<'_>, HipByt<'_>>(
    &hb,
    &HipByt::from(b"hello".as_slice())
  ));
  // [u8]/str on the left (existing blanket direction).
  assert!(equiv::<[u8], HipByt<'_>>(b"hello", &hb));
  assert!(equiv::<str, HipByt<'_>>("hello", &hb));
}

// ── smol-bytes `Equivalent` (both directions) ───────────────────────────────

#[cfg(feature = "smol_bytes_0_1")]
#[test]
fn smol_bytes_equivalent_both_directions() {
  use smol_bytes_0_1::{Utf8Bytes, compact, shared};
  use tokora::utils::cmp::Equivalent;

  // Same rationale as `hipstr_equivalent_both_directions` above: this helper
  // forces the *impl* to be resolved for the concrete `A`, so method-call
  // syntax can't silently deref-coerce past a missing `A: Equivalent<B>` impl.
  fn equiv<A, B>(a: &A, b: &B) -> bool
  where
    A: Equivalent<B> + ?Sized,
    B: ?Sized,
  {
    a.equivalent(b)
  }

  let sb = shared::Bytes::copy_from_slice(b"hello");
  // shared::Bytes on the left (the previously-missing direction).
  assert!(equiv::<shared::Bytes, str>(&sb, "hello"));
  assert!(!equiv::<shared::Bytes, str>(&sb, "world"));
  assert!(equiv::<shared::Bytes, [u8]>(&sb, b"hello"));
  assert!(!equiv::<shared::Bytes, [u8]>(&sb, b"world"));
  assert!(equiv::<shared::Bytes, shared::Bytes>(
    &sb,
    &shared::Bytes::copy_from_slice(b"hello")
  ));
  assert!(!equiv::<shared::Bytes, shared::Bytes>(
    &sb,
    &shared::Bytes::copy_from_slice(b"world")
  ));
  // str/[u8] on the left (existing blanket direction).
  assert!(equiv::<str, shared::Bytes>("hello", &sb));
  assert!(equiv::<[u8], shared::Bytes>(b"hello", &sb));

  let cb = compact::Bytes::copy_from_slice(b"hello");
  // compact::Bytes on the left (the previously-missing direction).
  assert!(equiv::<compact::Bytes, str>(&cb, "hello"));
  assert!(!equiv::<compact::Bytes, str>(&cb, "world"));
  assert!(equiv::<compact::Bytes, [u8]>(&cb, b"hello"));
  assert!(!equiv::<compact::Bytes, [u8]>(&cb, b"world"));
  assert!(equiv::<compact::Bytes, compact::Bytes>(
    &cb,
    &compact::Bytes::copy_from_slice(b"hello")
  ));
  assert!(!equiv::<compact::Bytes, compact::Bytes>(
    &cb,
    &compact::Bytes::copy_from_slice(b"world")
  ));
  // str/[u8] on the left (existing blanket direction).
  assert!(equiv::<str, compact::Bytes>("hello", &cb));
  assert!(equiv::<[u8], compact::Bytes>(b"hello", &cb));

  let ub = Utf8Bytes::from("hello");
  // Utf8Bytes on the left (the previously-missing direction).
  assert!(equiv::<Utf8Bytes, str>(&ub, "hello"));
  assert!(!equiv::<Utf8Bytes, str>(&ub, "world"));
  assert!(equiv::<Utf8Bytes, [u8]>(&ub, b"hello"));
  assert!(!equiv::<Utf8Bytes, [u8]>(&ub, b"world"));
  assert!(equiv::<Utf8Bytes, Utf8Bytes>(
    &ub,
    &Utf8Bytes::from("hello")
  ));
  assert!(!equiv::<Utf8Bytes, Utf8Bytes>(
    &ub,
    &Utf8Bytes::from("world")
  ));
  // str/[u8] on the left (existing blanket direction).
  assert!(equiv::<str, Utf8Bytes>("hello", &ub));
  assert!(equiv::<[u8], Utf8Bytes>(b"hello", &ub));
}
