use super::*;
use rowan::{Language, SyntaxKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TestKind {
  Root,
  Ident,
  Plus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TestLang {}

impl Language for TestLang {
  type Kind = TestKind;

  fn kind_from_raw(raw: SyntaxKind) -> TestKind {
    match raw.0 {
      0 => TestKind::Root,
      1 => TestKind::Ident,
      2 => TestKind::Plus,
      _ => panic!("unknown kind"),
    }
  }

  fn kind_to_raw(kind: TestKind) -> SyntaxKind {
    match kind {
      TestKind::Root => SyntaxKind(0),
      TestKind::Ident => SyntaxKind(1),
      TestKind::Plus => SyntaxKind(2),
    }
  }
}

#[test]
fn builder_new_and_default() {
  let b1 = SyntaxTreeBuilder::<TestLang>::new();
  let b2 = SyntaxTreeBuilder::<TestLang>::default();
  // Just verify they can be created
  let _ = format!("{:?}", b1);
  let _ = format!("{:?}", b2);
}

#[test]
fn builder_simple_tree() {
  let builder = SyntaxTreeBuilder::<TestLang>::new();
  builder.start_node(TestKind::Root);
  builder.token(TestKind::Ident, "hello");
  builder.finish_node();
  let green = builder.finish();

  let root = rowan::SyntaxNode::<TestLang>::new_root(green);
  assert_eq!(root.kind(), TestKind::Root);
  assert_eq!(root.to_string(), "hello");
}

#[test]
fn builder_with_checkpoint() {
  let builder = SyntaxTreeBuilder::<TestLang>::new();
  builder.start_node(TestKind::Root);

  let checkpoint = builder.checkpoint();
  builder.token(TestKind::Ident, "foo");

  // Wrap the identifier in a new node retroactively
  builder.start_node_at(checkpoint, TestKind::Root);
  builder.finish_node();

  builder.finish_node();
  let green = builder.finish();
  let root = rowan::SyntaxNode::<TestLang>::new_root(green);
  assert_eq!(root.to_string(), "foo");
}

#[test]
fn builder_multiple_tokens() {
  let builder = SyntaxTreeBuilder::<TestLang>::new();
  builder.start_node(TestKind::Root);
  builder.token(TestKind::Ident, "a");
  builder.token(TestKind::Plus, "+");
  builder.token(TestKind::Ident, "b");
  builder.finish_node();
  let green = builder.finish();

  let root = rowan::SyntaxNode::<TestLang>::new_root(green);
  assert_eq!(root.to_string(), "a+b");
}

#[test]
fn cst_node_children_clone() {
  let builder = SyntaxTreeBuilder::<TestLang>::new();
  builder.start_node(TestKind::Root);
  builder.token(TestKind::Ident, "hello");
  builder.finish_node();
  let green = builder.finish();
  let root = rowan::SyntaxNode::<TestLang>::new_root(green);

  let children: CstNodeChildren<rowan::SyntaxNode<TestLang>, TestLang> =
    CstNodeChildren::new(&root);
  let _cloned = children.clone();
}

#[test]
fn cst_node_children_by_kind() {
  let builder = SyntaxTreeBuilder::<TestLang>::new();
  builder.start_node(TestKind::Root);
  builder.start_node(TestKind::Root);
  builder.token(TestKind::Ident, "inner");
  builder.finish_node();
  builder.finish_node();
  let green = builder.finish();
  let root = rowan::SyntaxNode::<TestLang>::new_root(green);

  let children: CstNodeChildren<rowan::SyntaxNode<TestLang>, TestLang> =
    CstNodeChildren::new(&root);
  let matching: Vec<_> = children.by_kind(|k| k == TestKind::Root).collect();
  assert_eq!(matching.len(), 1);
}
