pub use node_mismatch::*;
use rowan::Language;
pub use token_mismatch::*;

use crate::{cst::CstNode, error::IncompleteSyntax};

use derive_more::{IsVariant, TryUnwrap, Unwrap};

mod node_mismatch;
mod token_mismatch;

/// Error type for CST node casting operations.
///
/// This error can occur when attempting to cast an untyped `SyntaxNode` to a typed
/// [`CstNode`](crate::cst::CstNode). It encompasses two kinds of failures:
///
/// - **Mismatch**: The node's kind doesn't match the expected type
/// - **IncompleteSyntax**: The node is missing required child components
///
/// # Type Parameters
///
/// - `E`: The CST element type being cast to
/// - `N`: A `typenum` unsigned integer representing the number of missing components
///
/// # Examples
///
/// ```rust,ignore
/// use tokit::cst::{CstNode, error::SyntaxError};
/// use typenum::U2;
///
/// match MyNode::try_cast_node(syntax_node) {
///     Ok(node) => { /* success */ }
///     Err(SyntaxError::<MyNode, U2>::Mismatch(e)) => {
///         eprintln!("Wrong node type: {}", e);
///     }
///     Err(SyntaxError::<MyNode, U2>::IncompleteSyntax(e)) => {
///         eprintln!("Missing components: {}", e);
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, IsVariant, Unwrap, TryUnwrap, thiserror::Error)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum SyntaxError<E: CstNode<Lang>, Lang: Language> {
  /// The syntax node's kind doesn't match the expected CST node type.
  #[error(transparent)]
  NodeMismatch(#[from] CstNodeMismatch<E, Lang>),
  /// The syntax token kind doesn't match the expected CST token type.
  #[error(transparent)]
  TokenMismatch(#[from] CstTokenMismatch<E, Lang>),
  /// The syntax node is incomplete and missing required child components.
  #[error(transparent)]
  Incomplete(#[from] IncompleteSyntax<E>),
}
