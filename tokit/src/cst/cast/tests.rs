use super::*;
use rowan::{GreenNodeBuilder, Language as _, SyntaxKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TK {
  Root,
  Ident,
  Plus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TLang {}

impl Language for TLang {
  type Kind = TK;

  fn kind_from_raw(raw: SyntaxKind) -> TK {
    match raw.0 {
      0 => TK::Root,
      1 => TK::Ident,
      2 => TK::Plus,
      _ => panic!("unknown"),
    }
  }

  fn kind_to_raw(kind: TK) -> SyntaxKind {
    match kind {
      TK::Root => SyntaxKind(0),
      TK::Ident => SyntaxKind(1),
      TK::Plus => SyntaxKind(2),
    }
  }
}

fn make_tree() -> SyntaxNode<TLang> {
  let mut builder = GreenNodeBuilder::new();
  builder.start_node(TLang::kind_to_raw(TK::Root));
  builder.token(TLang::kind_to_raw(TK::Ident), "x");
  builder.token(TLang::kind_to_raw(TK::Plus), "+");
  builder.finish_node();
  SyntaxNode::new_root(builder.finish())
}

#[test]
fn token_finds_by_kind() {
  let root = make_tree();
  let plus = token(&root, &TK::Plus);
  assert!(plus.is_some());
  assert_eq!(plus.unwrap().text(), "+");
}

#[test]
fn token_finds_ident() {
  let root = make_tree();
  let ident = token(&root, &TK::Ident);
  assert!(ident.is_some());
  assert_eq!(ident.unwrap().text(), "x");
}

#[test]
fn token_returns_none_when_not_found() {
  let mut builder = GreenNodeBuilder::new();
  builder.start_node(TLang::kind_to_raw(TK::Root));
  builder.token(TLang::kind_to_raw(TK::Ident), "x");
  builder.finish_node();
  let root = SyntaxNode::<TLang>::new_root(builder.finish());
  let plus = token(&root, &TK::Plus);
  assert!(plus.is_none());
}
