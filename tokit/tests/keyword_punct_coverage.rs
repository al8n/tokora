#![allow(warnings)]

//! Integration tests covering all generated methods from the `keyword!` and `punctuator!` macros.

use std::borrow::Borrow;
use std::fmt::Write as _;

use tokit::__private::span::{AsSpan, IntoSpan};
use tokit::__private::utils::IntoComponents;
use tokit::__private::utils::human_display::DisplayHuman;
use tokit::__private::utils::sdl_display::{DisplayCompact, DisplayPretty};
use tokit::span::SimpleSpan;

// ── Define a keyword and punctuator for testing ──────────────────────────────

tokit::keyword! {
  (TestKw, "TEST_KW", "test_keyword"),
  (AnotherKw, "ANOTHER_KW", "another"),
}

tokit::punctuator! {
  (TestPunct, "TEST_PUNCT", "@@"),
  (AnotherPunct, "ANOTHER_PUNCT", "##"),
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Keyword tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn keyword_new() {
  let span = SimpleSpan::new(0, 5);
  let kw = TestKw::new(span);
  assert_eq!(*kw.span(), span);
}

#[test]
fn keyword_with_content() {
  let span = SimpleSpan::new(1, 10);
  let kw = TestKw::with_content(span, "some_content");
  assert_eq!(*kw.span(), span);
  assert_eq!(*kw.content(), "some_content");
}

#[test]
fn keyword_raw() {
  assert_eq!(TestKw::<SimpleSpan>::raw(), "test_keyword");
  assert_eq!(AnotherKw::<SimpleSpan>::raw(), "another");
}

#[test]
fn keyword_as_ref() {
  let kw = TestKw::new(SimpleSpan::new(0, 1));
  let s: &str = kw.as_ref();
  assert_eq!(s, "test_keyword");
}

#[test]
fn keyword_borrow() {
  let kw = TestKw::new(SimpleSpan::new(0, 1));
  let s: &str = kw.borrow();
  assert_eq!(s, "test_keyword");
}

#[test]
fn keyword_display() {
  let kw = TestKw::new(SimpleSpan::new(0, 1));
  let s = format!("{}", kw);
  assert_eq!(s, "test_keyword");
}

#[test]
fn keyword_debug() {
  let kw = TestKw::new(SimpleSpan::new(0, 1));
  let dbg = format!("{:?}", kw);
  assert!(dbg.contains("TestKw"));
}

#[test]
fn keyword_clone_copy() {
  let kw = TestKw::new(SimpleSpan::new(0, 1));
  let kw2 = kw;
  let kw3 = kw;
  assert_eq!(kw2, kw3);
}

#[test]
fn keyword_eq_hash() {
  use std::collections::HashSet;
  let kw1 = TestKw::new(SimpleSpan::new(0, 1));
  let kw2 = TestKw::new(SimpleSpan::new(0, 1));
  assert_eq!(kw1, kw2);
  let mut set = HashSet::new();
  set.insert(kw1);
  assert!(set.contains(&kw2));
}

#[test]
fn keyword_as_span() {
  let span = SimpleSpan::new(5, 15);
  let kw = TestKw::new(span);
  assert_eq!(*AsSpan::as_span(&kw), span);
}

#[test]
fn keyword_into_span() {
  let span = SimpleSpan::new(5, 15);
  let kw = TestKw::new(span);
  let s: SimpleSpan = IntoSpan::into_span(kw);
  assert_eq!(s, span);
}

#[test]
fn keyword_into_components() {
  let span = SimpleSpan::new(2, 8);
  let kw = TestKw::with_content(span, 42u32);
  let (s, c) = kw.into_components();
  assert_eq!(s, span);
  assert_eq!(c, 42u32);
}

#[test]
fn keyword_display_human() {
  let kw = TestKw::new(SimpleSpan::new(0, 1));
  let mut buf = String::new();
  // Use the DisplayHuman trait via a manual Formatter invocation
  // The simplest way is through format_args + write
  struct HumanWrapper<'a, T: DisplayHuman>(&'a T);
  impl<T: DisplayHuman> std::fmt::Display for HumanWrapper<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      DisplayHuman::fmt(self.0, f)
    }
  }
  write!(buf, "{}", HumanWrapper(&kw)).unwrap();
  assert_eq!(buf, "test_keyword");
}

#[test]
fn keyword_display_compact() {
  let kw = TestKw::new(SimpleSpan::new(0, 1));
  let mut buf = String::new();
  struct CompactWrapper<'a, T: DisplayCompact<Options = ()>>(&'a T);
  impl<T: DisplayCompact<Options = ()>> std::fmt::Display for CompactWrapper<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      DisplayCompact::fmt(self.0, f, &())
    }
  }
  write!(buf, "{}", CompactWrapper(&kw)).unwrap();
  assert_eq!(buf, "test_keyword");
}

#[test]
fn keyword_display_pretty() {
  let kw = TestKw::new(SimpleSpan::new(0, 1));
  let mut buf = String::new();
  struct PrettyWrapper<'a, T: DisplayPretty<Options = ()>>(&'a T);
  impl<T: DisplayPretty<Options = ()>> std::fmt::Display for PrettyWrapper<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      DisplayPretty::fmt(self.0, f, &())
    }
  }
  write!(buf, "{}", PrettyWrapper(&kw)).unwrap();
  assert_eq!(buf, "test_keyword");
}

#[test]
fn keyword_another_variant() {
  let kw = AnotherKw::new(SimpleSpan::new(0, 7));
  assert_eq!(AnotherKw::<SimpleSpan>::raw(), "another");
  assert_eq!(format!("{}", kw), "another");
  let s: &str = kw.as_ref();
  assert_eq!(s, "another");
  let b: &str = kw.borrow();
  assert_eq!(b, "another");
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Punctuator tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn punct_unit() {
  let p = TestPunct::unit();
  assert_eq!(p.as_str(), "@@");
  assert_eq!(std::mem::size_of::<TestPunct<()>>(), 0);
}

#[test]
fn punct_unit_const() {
  let _p = TestPunct::UNIT;
  assert_eq!(TestPunct::UNIT.as_str(), "@@");
}

#[test]
fn punct_new() {
  let span = SimpleSpan::new(0, 2);
  let p = TestPunct::<SimpleSpan>::new(span);
  assert_eq!(*p.span(), span);
  assert_eq!(p.as_str(), "@@");
}

#[test]
fn punct_with_content() {
  let span = SimpleSpan::new(3, 5);
  let p = TestPunct::<SimpleSpan, &str>::with_content(span, "content");
  assert_eq!(*p.span(), span);
  assert_eq!(*p.content(), "content");
}

#[test]
fn punct_raw() {
  assert_eq!(TestPunct::raw(), "@@");
  assert_eq!(AnotherPunct::raw(), "##");
}

#[test]
fn punct_as_str() {
  let p = TestPunct::unit();
  assert_eq!(p.as_str(), "@@");
}

#[test]
fn punct_as_ref() {
  let p = TestPunct::unit();
  let s: &str = p.as_ref();
  assert_eq!(s, "@@");
}

#[test]
fn punct_borrow() {
  let p = TestPunct::unit();
  let s: &str = p.borrow();
  assert_eq!(s, "@@");
}

#[test]
fn punct_display() {
  let p = TestPunct::unit();
  assert_eq!(format!("{}", p), "@@");
}

#[test]
fn punct_debug() {
  let p = TestPunct::unit();
  let dbg = format!("{:?}", p);
  assert!(dbg.contains("TestPunct"));
}

#[test]
fn punct_clone_copy() {
  let p = TestPunct::unit();
  let p2 = p;
  let p3 = p;
  assert_eq!(p2, p3);
}

#[test]
fn punct_eq_hash() {
  use std::collections::HashSet;
  let p1 = TestPunct::unit();
  let p2 = TestPunct::unit();
  assert_eq!(p1, p2);
  let mut set = HashSet::new();
  set.insert(p1);
  assert!(set.contains(&p2));
}

#[test]
fn punct_partial_eq_str() {
  let p = TestPunct::unit();
  assert!(p == *"@@");
  assert!(!(p == *"##"));
}

#[test]
fn str_partial_eq_punct() {
  let p = TestPunct::unit();
  assert!(*"@@" == p);
  assert!(*"##" != p);
}

#[test]
fn punct_partial_ord_str() {
  let p = TestPunct::unit();
  assert_eq!(p.partial_cmp("@@"), Some(std::cmp::Ordering::Equal));
}

#[test]
fn str_partial_ord_punct() {
  let p = TestPunct::unit();
  assert_eq!("@@".partial_cmp(&p), Some(std::cmp::Ordering::Equal));
}

#[test]
fn punct_as_span() {
  let span = SimpleSpan::new(10, 20);
  let p = TestPunct::<SimpleSpan>::new(span);
  assert_eq!(*AsSpan::as_span(&p), span);
}

#[test]
fn punct_into_span() {
  let span = SimpleSpan::new(10, 20);
  let p = TestPunct::<SimpleSpan>::new(span);
  let s: SimpleSpan = IntoSpan::into_span(p);
  assert_eq!(s, span);
}

#[test]
fn punct_into_components() {
  let span = SimpleSpan::new(1, 3);
  let p = TestPunct::<SimpleSpan, i32>::with_content(span, 99);
  let (s, c) = p.into_components();
  assert_eq!(s, span);
  assert_eq!(c, 99);
}

#[test]
fn punct_change_language() {
  struct LangA;
  struct LangB;
  let p: TestPunct<(), (), LangA> = TestPunct::new(()).change_language();
  let p2: TestPunct<(), (), LangB> = p.change_language();
  assert_eq!(p2.as_str(), "@@");
}

#[test]
fn punct_change_language_const() {
  struct LangA;
  struct LangB;
  let p: TestPunct<(), (), LangA> = TestPunct::new(()).change_language_const();
  let p2: TestPunct<(), (), LangB> = p.change_language_const();
  assert_eq!(p2.as_str(), "@@");
}

#[test]
fn punct_display_human() {
  let p = TestPunct::unit();
  struct HumanWrapper<'a, T: DisplayHuman>(&'a T);
  impl<T: DisplayHuman> std::fmt::Display for HumanWrapper<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      DisplayHuman::fmt(self.0, f)
    }
  }
  let s = format!("{}", HumanWrapper(&p));
  assert_eq!(s, "@@");
}

#[test]
fn punct_display_compact() {
  let p = TestPunct::unit();
  struct CompactWrapper<'a, T: DisplayCompact<Options = ()>>(&'a T);
  impl<T: DisplayCompact<Options = ()>> std::fmt::Display for CompactWrapper<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      DisplayCompact::fmt(self.0, f, &())
    }
  }
  let s = format!("{}", CompactWrapper(&p));
  assert_eq!(s, "@@");
}

#[test]
fn punct_display_pretty() {
  let p = TestPunct::unit();
  struct PrettyWrapper<'a, T: DisplayPretty<Options = ()>>(&'a T);
  impl<T: DisplayPretty<Options = ()>> std::fmt::Display for PrettyWrapper<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      DisplayPretty::fmt(self.0, f, &())
    }
  }
  let s = format!("{}", PrettyWrapper(&p));
  assert_eq!(s, "@@");
}

#[test]
fn punct_another_variant() {
  let p = AnotherPunct::unit();
  assert_eq!(AnotherPunct::raw(), "##");
  assert_eq!(format!("{}", p), "##");
  let s: &str = p.as_ref();
  assert_eq!(s, "##");
  let b: &str = p.borrow();
  assert_eq!(b, "##");
}
