use super::*;
use rowan::{GreenNodeBuilder, Language as _, SyntaxKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TK {
  Root,
  Ident,
  Plus,
}

impl core::fmt::Display for TK {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      TK::Root => write!(f, "Root"),
      TK::Ident => write!(f, "Ident"),
      TK::Plus => write!(f, "Plus"),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PlusToken;

impl CstElement<TLang> for PlusToken {
  const KIND: TK = TK::Plus;
  fn castable(kind: TK) -> bool {
    kind == TK::Plus
  }
}

fn make_ident_token() -> SyntaxToken<TLang> {
  let mut builder = GreenNodeBuilder::new();
  builder.start_node(TLang::kind_to_raw(TK::Root));
  builder.token(TLang::kind_to_raw(TK::Ident), "x");
  builder.finish_node();
  let root = rowan::SyntaxNode::<TLang>::new_root(builder.finish());
  root
    .children_with_tokens()
    .filter_map(|c| c.into_token())
    .next()
    .unwrap()
}

#[test]
fn new_and_found() {
  let tok = make_ident_token();
  let err = CstTokenMismatch::<PlusToken, TLang>::new(tok.clone());
  assert_eq!(err.found().kind(), TK::Ident);
}

#[test]
fn expected_kind() {
  let tok = make_ident_token();
  let err = CstTokenMismatch::<PlusToken, TLang>::new(tok);
  assert_eq!(err.expected(), TK::Plus);
}

#[test]
fn into_components_returns_kind_and_token() {
  let tok = make_ident_token();
  let err = CstTokenMismatch::<PlusToken, TLang>::new(tok.clone());
  let (kind, found) = err.into_components();
  assert_eq!(kind, TK::Plus);
  assert_eq!(found.kind(), TK::Ident);
}

#[test]
fn display_impl() {
  let tok = make_ident_token();
  let err = CstTokenMismatch::<PlusToken, TLang>::new(tok);
  let msg = std::format!("{}", err);
  assert!(msg.contains("Plus"));
  assert!(msg.contains("Ident"));
}

#[test]
fn debug_clone_eq() {
  let tok = make_ident_token();
  let err = CstTokenMismatch::<PlusToken, TLang>::new(tok);
  let err2 = err.clone();
  assert_eq!(err, err2);
  let _ = std::format!("{:?}", err);
}

#[test]
fn error_impl() {
  let tok = make_ident_token();
  let err = CstTokenMismatch::<PlusToken, TLang>::new(tok);
  let _: &dyn core::error::Error = &err;
}
