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
