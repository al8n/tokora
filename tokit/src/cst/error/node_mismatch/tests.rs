use super::*;
use rowan::{GreenNodeBuilder, Language as _, SyntaxKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TK {
  Root,
  Ident,
}

impl core::fmt::Display for TK {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      TK::Root => write!(f, "Root"),
      TK::Ident => write!(f, "Ident"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TLang {}

impl Language for TLang {
  type Kind = TK;

  fn kind_from_raw(raw: SyntaxKind) -> TK {
    match raw.0 {
      0 => TK::Root,
      1 => TK::Ident,
      _ => panic!("unknown"),
    }
  }

  fn kind_to_raw(kind: TK) -> SyntaxKind {
    match kind {
      TK::Root => SyntaxKind(0),
      TK::Ident => SyntaxKind(1),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IdentNode;

impl CstElement<TLang> for IdentNode {
  const KIND: TK = TK::Ident;
  fn castable(kind: TK) -> bool {
    kind == TK::Ident
  }
}

fn make_root_node() -> SyntaxNode<TLang> {
  let mut builder = GreenNodeBuilder::new();
  builder.start_node(TLang::kind_to_raw(TK::Root));
  builder.token(TLang::kind_to_raw(TK::Ident), "x");
  builder.finish_node();
  SyntaxNode::new_root(builder.finish())
}

#[test]
fn new_and_found() {
  let node = make_root_node();
  let err = CstNodeMismatch::<IdentNode, TLang>::new(node.clone());
  assert_eq!(err.found().kind(), TK::Root);
}

#[test]
fn expected_kind() {
  let node = make_root_node();
  let err = CstNodeMismatch::<IdentNode, TLang>::new(node);
  assert_eq!(err.expected(), TK::Ident);
}

#[test]
fn into_components_returns_kind_and_node() {
  let node = make_root_node();
  let err = CstNodeMismatch::<IdentNode, TLang>::new(node.clone());
  let (kind, found) = err.into_components();
  assert_eq!(kind, TK::Ident);
  assert_eq!(found.kind(), TK::Root);
}

#[test]
fn display_impl() {
  let node = make_root_node();
  let err = CstNodeMismatch::<IdentNode, TLang>::new(node);
  let msg = std::format!("{}", err);
  assert!(msg.contains("Ident"));
  assert!(msg.contains("Root"));
}

#[test]
fn debug_clone_eq() {
  let node = make_root_node();
  let err = CstNodeMismatch::<IdentNode, TLang>::new(node);
  let err2 = err.clone();
  assert_eq!(err, err2);
  let _ = std::format!("{:?}", err);
}

#[test]
fn error_impl() {
  let node = make_root_node();
  let err = CstNodeMismatch::<IdentNode, TLang>::new(node);
  let _: &dyn core::error::Error = &err;
}
