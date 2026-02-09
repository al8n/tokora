use super::super::{CstElement, Language};
use derive_more::{From, Into};
use rowan::SyntaxNode;

/// An error indicating a mismatch between expected and actual syntax node kinds.
///
/// This error occurs when attempting to cast a [`SyntaxNode`] to a typed [`CstNode`](crate::cst::CstNode)
/// type, but the node's kind doesn't match the expected kind for that type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct CstNodeMismatch<N, Lang: Language> {
  found: SyntaxNode<Lang>,
  _m: core::marker::PhantomData<N>,
}

impl<N: CstElement<Lang>, Lang: Language> core::fmt::Display for CstNodeMismatch<N, Lang>
where
  Lang::Kind: core::fmt::Display,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "syntax node mismatch: expected syntax node of kind {}, but found syntax node of kind {}",
      N::KIND,
      self.found.kind()
    )
  }
}

impl<N: CstElement<Lang>, Lang: Language> core::error::Error for CstNodeMismatch<N, Lang>
where
  N: CstElement<Lang> + core::fmt::Debug,
  Lang::Kind: core::fmt::Display,
{
}

impl<N, Lang: Language> CstNodeMismatch<N, Lang> {
  /// Creates a new syntax node mismatch error.
  /// ```
  #[inline]
  pub const fn new(found: SyntaxNode<Lang>) -> Self {
    Self {
      found,
      _m: core::marker::PhantomData,
    }
  }

  /// Returns the expected syntax node kind.
  #[inline]
  pub const fn expected(&self) -> Lang::Kind
  where
    N: CstElement<Lang>,
  {
    N::KIND
  }

  /// Returns a reference to the syntax node that was found.
  #[inline]
  pub const fn found(&self) -> &SyntaxNode<Lang> {
    &self.found
  }

  /// Consumes the error and returns the expected kind and found node.
  ///
  /// This is useful for recovering the original syntax node after a failed cast,
  /// allowing you to try casting to a different type.
  #[inline]
  pub fn into_components(self) -> (Lang::Kind, SyntaxNode<Lang>)
  where
    N: CstElement<Lang>,
  {
    (N::KIND, self.found)
  }
}
