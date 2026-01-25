use super::super::{CstElement, Language};
use derive_more::{From, Into};
use rowan::SyntaxNode;

/// An error indicating a mismatch between expected and actual syntax node kinds.
///
/// This error occurs when attempting to cast a [`SyntaxNode`] to a typed [`CstNode`](crate::cst::CstNode)
/// type, but the node's kind doesn't match the expected kind for that type.
///
/// # Examples
///
/// ```rust,ignore
/// use tokit::cst::{CstNode, error};
///
/// let result = IdentifierNode::try_cast_node(syntax_node);
///
/// match result {
///     Ok(identifier) => {
///         // Successfully cast
///         println!("Identifier: {}", identifier.source_string());
///     }
///     Err(mismatch) => {
///         // Cast failed
///         eprintln!("Type mismatch: {}", mismatch);
///         eprintln!("Expected: {:?}", mismatch.expected());
///         eprintln!("Found: {:?}", mismatch.found().kind());
///     }
/// }
/// ```
///
/// ## Recovering from Errors
///
/// ```rust,ignore
/// use tokit::cst::error::CstNodeMismatch;
///
/// let result = Expression::try_cast_node(syntax_node);
///
/// let node = match result {
///     Ok(expr) => expr,
///     Err(mismatch) => {
///         // Recover the original syntax node
///         let (expected_kind, original_node) = mismatch.into_components();
///         // Try a different type
///         Statement::try_cast_node(original_node)?
///     }
/// };
/// ```
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
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::error::CstNodeMismatch;
  ///
  /// let error = CstNodeMismatch::new(
  ///     SyntaxKind::Identifier,
  ///     syntax_node
  /// );
  /// ```
  #[inline]
  pub const fn new(found: SyntaxNode<Lang>) -> Self {
    Self {
      found,
      _m: core::marker::PhantomData,
    }
  }

  /// Returns the expected syntax node kind.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::Node;
  ///
  /// if let Err(mismatch) = IdentifierNode::try_cast_node(node) {
  ///     println!("Expected kind: {:?}", mismatch.expected());
  /// }
  /// ```
  #[inline]
  pub const fn expected(&self) -> Lang::Kind
  where
    N: CstElement<Lang>,
  {
    N::KIND
  }

  /// Returns a reference to the syntax node that was found.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstNode;
  ///
  /// if let Err(mismatch) = IdentifierNode::try_cast_node(node) {
  ///     println!("Found kind: {:?}", mismatch.found().kind());
  ///     println!("Found text: {}", mismatch.found().text());
  /// }
  /// ```
  #[inline]
  pub const fn found(&self) -> &SyntaxNode<Lang> {
    &self.found
  }

  /// Consumes the error and returns the expected kind and found node.
  ///
  /// This is useful for recovering the original syntax node after a failed cast,
  /// allowing you to try casting to a different type.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstNode;
  ///
  /// let result = IdentifierNode::try_cast_node(syntax_node);
  ///
  /// if let Err(mismatch) = result {
  ///     let (expected, node) = mismatch.into_components();
  ///     // Try casting to a different type
  ///     let keyword = KeywordNode::try_cast_node(node)?;
  /// }
  /// ```
  #[inline]
  pub fn into_components(self) -> (Lang::Kind, SyntaxNode<Lang>)
  where
    N: CstElement<Lang>,
  {
    (N::KIND, self.found)
  }
}
