use super::*;
use std::format;

struct TestNode(&'static str);

impl DisplaySyntaxTree for TestNode {
  fn fmt(
    &self,
    level: usize,
    indent: usize,
    f: &mut core::fmt::Formatter<'_>,
  ) -> core::fmt::Result {
    for _ in 0..level * indent {
      write!(f, " ")?;
    }
    write!(f, "{}", self.0)
  }
}

#[test]
fn display_syntax_tree_basic() {
  let node = TestNode("hello");
  let d = node.display(0, 2);
  assert_eq!(format!("{}", d), "hello");
}

#[test]
fn display_syntax_tree_indented() {
  let node = TestNode("child");
  let d = node.display(2, 4);
  assert_eq!(format!("{}", d), "        child");
}

#[test]
fn display_syntax_tree_ref() {
  let node = TestNode("ref");
  let r: &TestNode = &node;
  let d = r.display(1, 2);
  assert_eq!(format!("{}", d), "  ref");
}
