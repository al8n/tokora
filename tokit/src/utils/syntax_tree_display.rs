/// A trait for displaying in a SyntaxTree style.
pub trait DisplaySyntaxTree {
  /// Formats the value in a SyntaxTree style.
  ///
  /// - `level` is the current indentation level.
  /// - `indent` is the number of spaces to indent per level.
  fn fmt(&self, level: usize, indent: usize, f: &mut core::fmt::Formatter<'_>)
  -> core::fmt::Result;

  /// Returns a wrapper which implement `Display`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn display(&self, level: usize, indent: usize) -> SyntaxTreeDisplay<'_, Self> {
    SyntaxTreeDisplay {
      t: self,
      indent,
      level,
    }
  }
}

impl<T: DisplaySyntaxTree + ?Sized> DisplaySyntaxTree for &T {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(
    &self,
    level: usize,
    indent: usize,
    f: &mut core::fmt::Formatter<'_>,
  ) -> core::fmt::Result {
    (*self).fmt(level, indent, f)
  }
}

/// A helper struct for displaying in a SyntaxTree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxTreeDisplay<'a, T: ?Sized> {
  t: &'a T,
  indent: usize,
  level: usize,
}

impl<T: DisplaySyntaxTree + ?Sized> core::fmt::Display for SyntaxTreeDisplay<'_, T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.t.fmt(self.level, self.indent, f)
  }
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod tests {
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
}
