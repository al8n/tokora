use super::*;
use core::borrow::Borrow;
use std::format;

#[test]
fn comma_unit_is_zero_sized() {
  assert_eq!(core::mem::size_of::<Comma>(), 0);
}

#[test]
fn comma_unit_returns_unit() {
  let c = Comma::unit();
  assert_eq!(c.as_str(), ",");
}

#[test]
fn comma_raw_returns_literal() {
  assert_eq!(Comma::raw(), ",");
}

#[test]
fn comma_new_with_span() {
  let c = Comma::<usize>::new(42);
  assert_eq!(*c.span(), 42);
  assert_eq!(c.as_str(), ",");
}

#[test]
fn comma_with_content() {
  let c = Comma::<usize, &str>::with_content(10, "hello");
  assert_eq!(*c.span(), 10);
  assert_eq!(*c.content(), "hello");
}

#[test]
fn punctuator_display() {
  let c = Comma::unit();
  assert_eq!(format!("{}", c), ",");

  let s = Semicolon::unit();
  assert_eq!(format!("{}", s), ";");

  let d = Dot::unit();
  assert_eq!(format!("{}", d), ".");
}

#[test]
fn punctuator_debug() {
  let c = Comma::unit();
  let dbg = format!("{:?}", c);
  assert!(dbg.contains("Comma"));
}

#[test]
fn punctuator_partial_eq_str() {
  let c = Comma::unit();
  assert!(c == *",");
  assert!(!(c == *";"));
}

#[test]
fn str_partial_eq_punctuator() {
  let c = Comma::unit();
  assert!(*"," == c);
  assert!((*";" != c));
}

#[test]
fn punctuator_partial_ord_str() {
  let c = Comma::unit();
  let ord = c.partial_cmp(",");
  assert_eq!(ord, Some(core::cmp::Ordering::Equal));
}

#[test]
fn str_partial_ord_punctuator() {
  let c = Comma::unit();
  let ord = ",".partial_cmp(&c);
  assert_eq!(ord, Some(core::cmp::Ordering::Equal));
}

#[test]
fn punctuator_borrow_str() {
  let c = Comma::unit();
  let s: &str = c.borrow();
  assert_eq!(s, ",");
}

#[test]
fn punctuator_as_ref_str() {
  let c = Comma::unit();
  let s: &str = c.as_ref();
  assert_eq!(s, ",");
}

#[test]
fn punctuator_clone_copy() {
  let c = Comma::unit();
  let c2 = c;
  let c3 = c;
  assert_eq!(c2.as_str(), c3.as_str());
}

#[test]
fn punctuator_eq_hash() {
  let c1 = Comma::unit();
  let c2 = Comma::unit();
  assert_eq!(c1, c2);
}

#[test]
fn change_language() {
  struct LangA;
  struct LangB;
  let c: Comma<(), (), LangA> = Comma {
    span: (),
    source: (),
    _lang: core::marker::PhantomData,
  };
  let c2: Comma<(), (), LangB> = c.change_language();
  assert_eq!(c2.as_str(), ",");
}

#[test]
fn change_language_const() {
  struct LangA;
  struct LangB;
  let c: Comma<(), (), LangA> = Comma {
    span: (),
    source: (),
    _lang: core::marker::PhantomData,
  };
  let c2: Comma<(), (), LangB> = c.change_language_const();
  assert_eq!(c2.as_str(), ",");
}

#[test]
fn into_components() {
  use crate::utils::IntoComponents;
  let c = Comma::<usize, &str>::with_content(42, "test");
  let (span, content) = c.into_components();
  assert_eq!(span, 42);
  assert_eq!(content, "test");
}

#[test]
fn into_span() {
  use crate::span::IntoSpan;
  let c = Comma::<usize>::new(99);
  let span = c.into_span();
  assert_eq!(span, 99);
}

#[test]
fn as_span() {
  use crate::span::AsSpan;
  let c = Comma::<usize>::new(77);
  assert_eq!(*c.as_span(), 77);
}

#[test]
fn various_punctuators_raw() {
  assert_eq!(OpenAngle::raw(), "<");
  assert_eq!(CloseAngle::raw(), ">");
  assert_eq!(OpenBrace::raw(), "{");
  assert_eq!(CloseBrace::raw(), "}");
  assert_eq!(OpenParen::raw(), "(");
  assert_eq!(CloseParen::raw(), ")");
  assert_eq!(OpenBracket::raw(), "[");
  assert_eq!(CloseBracket::raw(), "]");
  assert_eq!(At::raw(), "@");
  assert_eq!(Asterisk::raw(), "*");
  assert_eq!(Ampersand::raw(), "&");
  assert_eq!(Arrow::raw(), "->");
  assert_eq!(FatArrow::raw(), "=>");
  assert_eq!(Spread::raw(), "...");
  assert_eq!(DoubleColon::raw(), "::");
  assert_eq!(LogicalEqual::raw(), "==");
  assert_eq!(LogicalNotEqual::raw(), "!=");
  assert_eq!(Increment::raw(), "++");
  assert_eq!(Decrement::raw(), "--");
  assert_eq!(Exponentiation::raw(), "**");
  assert_eq!(LogicalAnd::raw(), "&&");
  assert_eq!(LogicalOr::raw(), "||");
  assert_eq!(NullCoalesce::raw(), "??");
  assert_eq!(OptionalChain::raw(), "?.");
  assert_eq!(Tab::raw(), "\t");
  assert_eq!(Newline::raw(), "\n");
  assert_eq!(Space::raw(), " ");
  assert_eq!(CarriageReturn::raw(), "\r");
  assert_eq!(CarriageReturnNewline::raw(), "\r\n");
}

#[test]
fn crnl_type_alias() {
  // Crnl is an alias for CarriageReturnNewline
  let c = Crnl::unit();
  assert_eq!(c.as_str(), "\r\n");
}

#[test]
fn punctuator_display_special_chars() {
  assert_eq!(format!("{}", Tab::unit()), "\t");
  assert_eq!(format!("{}", Newline::unit()), "\n");
  assert_eq!(format!("{}", Space::unit()), " ");
}
