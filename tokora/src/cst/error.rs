pub use node_mismatch::*;
pub use token_mismatch::*;

use derive_more::{IsVariant, TryUnwrap, Unwrap};
use rowan::Language;

use crate::{cst::Node, error::IncompleteSyntax};

mod node_mismatch;
mod token_mismatch;

/// Error type for CST node casting operations.
///
/// This error can occur when attempting to cast an untyped `SyntaxNode` to a typed
/// [`Node`]. It encompasses two kinds of failures:
///
/// - **Mismatch**: The node's kind doesn't match the expected type
/// - **IncompleteSyntax**: The node is missing required child components
#[derive(Debug, Clone, PartialEq, Eq, Hash, IsVariant, Unwrap, TryUnwrap, thiserror::Error)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum SyntaxError<E: Node<Lang>, Lang: Language> {
  /// The syntax node's kind doesn't match the expected CST node type.
  #[error(transparent)]
  NodeMismatch(#[from] NodeMismatch<E, Lang>),
  /// The syntax token kind doesn't match the expected CST token type.
  #[error(transparent)]
  TokenMismatch(#[from] TokenMismatch<E, Lang>),
  /// The syntax node is incomplete and missing required child components.
  #[error(transparent)]
  Incomplete(#[from] IncompleteSyntax<E>),
}
