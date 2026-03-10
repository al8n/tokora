use super::super::{CstElement, Language};
use derive_more::{From, Into};
use rowan::SyntaxToken;

/// An error indicating a mismatch between expected and actual syntax token kinds.
///
/// This error occurs when attempting to cast a [`SyntaxToken`] to a typed [`CstToken`](crate::cst::CstToken)
/// type, but the token's kind doesn't match the expected kind for that type. This is the
/// token-equivalent of [`CstNodeMismatch`](super::CstNodeMismatch).
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct CstTokenMismatch<N, Lang: Language> {
  found: SyntaxToken<Lang>,
  _m: core::marker::PhantomData<N>,
}

impl<N, Lang: Language> core::fmt::Display for CstTokenMismatch<N, Lang>
where
  N: CstElement<Lang>,
  Lang::Kind: core::fmt::Display,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "syntax token mismatch: expected syntax token of kind {}, but found syntax token of kind {}",
      N::KIND,
      self.found.kind()
    )
  }
}

impl<N, Lang: Language> core::error::Error for CstTokenMismatch<N, Lang>
where
  N: CstElement<Lang> + core::fmt::Debug,
  Lang::Kind: core::fmt::Display,
{
}

impl<N, Lang: Language> CstTokenMismatch<N, Lang> {
  /// Creates a new syntax token mismatch error.
  ///
  /// This constructor is typically called by [`CstToken::try_cast_token()`](crate::cst::CstToken::try_cast_token)
  /// implementations when a cast fails. You rarely need to call this directly.
  #[inline]
  pub const fn new(found: SyntaxToken<Lang>) -> Self {
    Self {
      found,
      _m: core::marker::PhantomData,
    }
  }

  /// Returns the expected syntax token kind.
  ///
  /// This is the kind that was expected when attempting to cast to type `N`.
  /// For simple tokens, this is typically `N::KIND`. For enum tokens, this
  /// may be a marker kind representing the enum itself.
  #[inline]
  pub const fn expected(&self) -> Lang::Kind
  where
    N: CstElement<Lang>,
  {
    N::KIND
  }

  /// Returns a reference to the syntax token that was found.
  ///
  /// This provides access to the original token that failed to cast,
  /// allowing you to inspect its kind, text, position, and other properties.
  #[inline]
  pub const fn found(&self) -> &SyntaxToken<Lang> {
    &self.found
  }

  /// Consumes the error and returns the expected kind and found token.
  ///
  /// This is useful for recovering the original syntax token after a failed cast,
  /// allowing you to try casting to a different type or perform other operations
  /// on the token.
  ///
  /// # Returns
  ///
  /// A tuple of `(expected_kind, found_token)`:
  /// - `expected_kind`: The kind that was expected for type `N`
  /// - `found_token`: The original token that failed to cast
  #[inline]
  pub fn into_components(self) -> (Lang::Kind, SyntaxToken<Lang>)
  where
    N: CstElement<Lang>,
  {
    (N::KIND, self.found)
  }
}

#[cfg(test)]
#[allow(warnings)]
mod tests {
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
}
