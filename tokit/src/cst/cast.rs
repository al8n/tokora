use super::{CstNode, CstNodeChildren, Language, SyntaxNode};

/// Returns the first child of a specific typed node type.
///
/// Searches through the children of the parent node and returns the first child
/// that can be successfully cast to the specified node type `N`.
///
/// # Type Parameters
///
/// - `N`: The typed [`CstNode`] type to search for
///
/// # Examples
///
/// ```rust,ignore
/// use tokit::cst::cast;
///
/// // Find the first identifier child
/// if let Some(identifier) = cast::child::<IdentifierNode>(&function_syntax) {
///     println!("Function name: {}", identifier.source_string());
/// }
///
/// // Find the first expression in a statement
/// let expr = cast::child::<Expression>(&statement_syntax)?;
/// ```
#[inline]
pub fn child<N: CstNode<Lang>, Lang: Language>(parent: &SyntaxNode<Lang>) -> Option<N> {
  parent.children().find_map(|t| N::try_cast_node(t).ok())
}

/// Returns an iterator over all children of a specific typed node type.
///
/// Iterates through all children of the parent node, yielding only those that
/// can be successfully cast to the specified node type `N`.
///
/// # Type Parameters
///
/// - `N`: The typed [`Node`] type to iterate over
///
/// # Examples
///
/// ```rust,ignore
/// use tokit::cst::cast;
///
/// // Get all parameters of a function
/// let parameters: Vec<Parameter> = cast::children(&function_syntax).collect();
///
/// // Count the number of statements in a block
/// let statement_count = cast::children::<Statement>(&block_syntax).count();
///
/// // Find the first parameter with a specific name
/// let param = cast::children::<Parameter>(&function_syntax)
///     .find(|p| p.name() == "self");
/// ```
#[inline]
pub fn children<N: CstNode<Lang>, Lang: Language>(
  parent: &SyntaxNode<Lang>,
) -> CstNodeChildren<N, Lang> {
  CstNodeChildren::new(parent)
}

/// Returns the first token child with the specified syntax kind.
///
/// Searches through all tokens (not nodes) that are direct children of the parent
/// and returns the first one matching the specified kind.
///
/// # Type Parameters
///
/// - `L`: The [`Language`] type defining syntax kinds
///
/// # Examples
///
/// ```rust,ignore
/// use tokit::cst::cast;
///
/// // Get the equals token from an assignment
/// let equals = cast::token(&assignment_node, &SyntaxKind::Equals)?;
///
/// // Get the opening parenthesis of a function call
/// let lparen = cast::token(&call_node, &SyntaxKind::LeftParen)?;
///
/// // Check if a node has a specific keyword
/// if let Some(async_kw) = cast::token(&function_node, &SyntaxKind::AsyncKeyword) {
///     println!("Function is async");
/// }
/// ```
#[inline]
pub fn token<L: Language>(parent: &SyntaxNode<L>, kind: &L::Kind) -> Option<rowan::SyntaxToken<L>> {
  parent
    .children_with_tokens()
    .filter_map(|child| {
      child
        .into_token()
        .and_then(|t| t.kind().eq(kind).then_some(t))
    })
    .next()
}
