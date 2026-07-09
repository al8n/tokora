use super::*;
use std::format;

struct TestCompact {
  value: i32,
}

impl DisplayCompact for TestCompact {
  type Options = ();

  fn fmt(&self, f: &mut core::fmt::Formatter<'_>, _: &()) -> core::fmt::Result {
    write!(f, "compact({})", self.value)
  }
}

struct TestPretty {
  value: i32,
}

impl DisplayPretty for TestPretty {
  type Options = usize;

  fn fmt(&self, f: &mut core::fmt::Formatter<'_>, indent: &usize) -> core::fmt::Result {
    write!(f, "{}pretty({})", " ".repeat(*indent), self.value)
  }
}

// --- DisplayCompact tests ---

#[test]
fn compact_display_fmt() {
  let t = TestCompact { value: 42 };
  let d = t.display(&());
  assert_eq!(format!("{}", d), "compact(42)");
}

#[test]
fn compact_display_ref() {
  let t = TestCompact { value: 7 };
  let r = &t;
  let d = DisplayCompact::display(r, &());
  assert_eq!(format!("{}", d), "compact(7)");
}

#[test]
fn compact_display_sdl_fmt() {
  let t = TestCompact { value: 10 };
  let compact = t.display(&());
  let sdl_display = DisplaySDL::display(&compact, &());
  assert_eq!(format!("{}", sdl_display), "compact(10)");
}

#[test]
fn compact_display_sdl_direct_fmt() {
  let t = TestCompact { value: 5 };
  let compact = t.display(&());
  let mut buf = std::string::String::new();
  use core::fmt::Write;
  write!(buf, "{}", compact).unwrap();
  assert_eq!(buf, "compact(5)");
}

// --- DisplayPretty tests ---

#[test]
fn pretty_display_fmt() {
  let t = TestPretty { value: 42 };
  let d = t.display(&2);
  assert_eq!(format!("{}", d), "  pretty(42)");
}

#[test]
fn pretty_display_ref() {
  let t = TestPretty { value: 7 };
  let r = &t;
  let d = DisplayPretty::display(r, &0);
  assert_eq!(format!("{}", d), "pretty(7)");
}

#[test]
fn pretty_display_sdl_fmt() {
  let t = TestPretty { value: 10 };
  let pretty = t.display(&1);
  let sdl_display = DisplaySDL::display(&pretty, &0);
  assert_eq!(format!("{}", sdl_display), "pretty(10)");
}

// --- DisplaySDL for &T ---

#[test]
fn display_sdl_ref_delegation() {
  let t = TestCompact { value: 99 };
  let compact = t.display(&());
  let r = &compact;
  let d = DisplaySDL::display(r, &());
  assert_eq!(format!("{}", d), "compact(99)");
}
