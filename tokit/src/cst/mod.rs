//! Concrete Syntax Tree (CST) utilities built on top of [rowan](https://docs.rs/rowan).
//!
//! This module provides infrastructure for building and working with typed concrete syntax trees.
//! Unlike Abstract Syntax Trees (ASTs), CSTs preserve all source information including whitespace,
//! comments, and exact token positions, making them ideal for:
//!
//! - **Code formatters**: Preserve exact formatting and whitespace
//! - **Linters**: Access complete source information for analysis
//! - **Language servers**: Provide accurate position information for IDE features
//! - **Refactoring tools**: Transform code while preserving formatting
//! - **Documentation generators**: Extract and preserve comments
//!
//! # Architecture
//!
//! The CST infrastructure has several key components:
//!
//! 1. **[`SyntaxTreeBuilder`](crate::cst::SyntaxTreeBuilder)**: Constructs CSTs from tokens using rowan's green tree builder
//! 2. **[`CstElement`](crate::cst::CstElement)**: Base trait for all typed CST elements (nodes and tokens)
//! 3. **[`CstNode`](crate::cst::CstNode)**: Trait for typed CST nodes with zero-cost conversions
//! 4. **[`CstToken`](crate::cst::CstToken)**: Trait for typed CST tokens (terminal elements)
//! 5. **[`cast`](crate::cst::cast)**: Utility functions for working with CST nodes and tokens
//! 6. **[`error`](crate::cst::error)**: Error types for CST operations
//!
//! # Design Philosophy
//!
//! - **Zero-cost abstractions**: Typed CST nodes are just pointers, no runtime overhead
//! - **Lossless**: All source information is preserved in the tree
//! - **Immutable**: Trees are immutable by default (use `clone_for_update()` for mutations)
//! - **Type-safe**: Compile-time guarantees about node types and relationships
//!
//! # See Also
//!
//! - [rowan documentation](https://docs.rs/rowan) - The underlying CST library

use core::{cell::RefCell, marker::PhantomData};

use derive_more::{From, Into};
use rowan::{GreenNodeBuilder, Language, SyntaxNode, SyntaxToken};

use crate::syntax::Syntax;

/// A builder for constructing concrete syntax trees.
///
/// `SyntaxTreeBuilder` wraps rowan's [`GreenNodeBuilder`] and provides a convenient
/// interface for building syntax trees from tokens during parsing. The builder uses
/// interior mutability ([`RefCell`]) to allow sharing across parser combinators.
///
/// # Type Parameters
///
/// - `Lang`: The [`Language`] type that defines the syntax kinds for your language
///
/// # Usage Pattern
///
/// The typical usage pattern is:
///
/// 1. Create a builder with [`new()`](Self::new)
/// 2. Pass it to your parser implementation
/// 3. The parser calls [`start_node()`](Self::start_node), [`token()`](Self::token),
///    and [`finish_node()`](Self::finish_node) to build the tree
/// 4. Call [`finish()`](Self::finish) to get the final [`rowan::GreenNode`]
///
/// # Examples
///
/// ```rust,ignore
/// use tokit::cst::SyntaxTreeBuilder;
///
/// let builder = SyntaxTreeBuilder::<MyLanguage>::new();
///
/// // Build a simple tree: Root(Identifier("hello"))
/// builder.start_node(SyntaxKind::Root);
/// builder.token(SyntaxKind::Identifier, "hello");
/// builder.finish_node();
///
/// let green_node = builder.finish();
/// ```
///
/// ## Using Checkpoints for Lookahead
///
/// Checkpoints allow you to start nodes retroactively, which is useful for
/// handling left-recursive or ambiguous grammars:
///
/// ```rust,ignore
/// let builder = SyntaxTreeBuilder::<MyLanguage>::new();
/// let checkpoint = builder.checkpoint();
///
/// builder.token(SyntaxKind::Number, "42");
///
/// // Decide to wrap the number in a UnaryExpression
/// builder.start_node_at(checkpoint, SyntaxKind::UnaryExpression);
/// builder.token(SyntaxKind::Plus, "+");
/// builder.finish_node();
/// ```
///
/// # Interior Mutability
///
/// The builder uses [`RefCell`] internally, which means:
/// - It can be shared immutably across parser combinators
/// - Mutations are checked at runtime (will panic if you violate borrow rules)
/// - Typically safe in single-threaded parsing contexts
#[derive(Debug)]
pub struct SyntaxTreeBuilder<Lang> {
  builder: RefCell<GreenNodeBuilder<'static>>,
  _marker: PhantomData<Lang>,
}

impl<Lang> Default for SyntaxTreeBuilder<Lang>
where
  Lang: Language,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<Lang> SyntaxTreeBuilder<Lang>
where
  Lang: Language,
{
  /// Creates a new empty syntax tree builder.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::SyntaxTreeBuilder;
  ///
  /// let builder = SyntaxTreeBuilder::<MyLanguage>::new();
  /// ```
  #[inline]
  pub fn new() -> Self {
    Self {
      builder: RefCell::new(GreenNodeBuilder::new()),
      _marker: PhantomData,
    }
  }

  /// Creates a checkpoint representing the current position in the tree.
  ///
  /// Checkpoints can be used with [`start_node_at()`](Self::start_node_at) to
  /// retroactively wrap already-added children in a new parent node. This is
  /// useful for handling left-recursive or ambiguous grammars.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::SyntaxTreeBuilder;
  ///
  /// let builder = SyntaxTreeBuilder::<MyLanguage>::new();
  /// let checkpoint = builder.checkpoint();
  ///
  /// builder.token(SyntaxKind::Number, "42");
  ///
  /// // Wrap the number in an expression node
  /// builder.start_node_at(checkpoint, SyntaxKind::Expression);
  /// builder.finish_node();
  /// ```
  ///
  /// See also: [`rowan::GreenNodeBuilder::checkpoint`]
  #[inline]
  #[must_use]
  pub fn checkpoint(&self) -> rowan::Checkpoint {
    self.builder.borrow().checkpoint()
  }

  /// Starts a new node with the given syntax kind.
  ///
  /// Must be paired with a corresponding [`finish_node()`](Self::finish_node) call.
  /// All tokens and child nodes added between `start_node()` and `finish_node()`
  /// will be children of this node.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::SyntaxTreeBuilder;
  ///
  /// let builder = SyntaxTreeBuilder::<MyLanguage>::new();
  ///
  /// builder.start_node(SyntaxKind::BinaryExpression);
  /// builder.token(SyntaxKind::Number, "1");
  /// builder.token(SyntaxKind::Plus, "+");
  /// builder.token(SyntaxKind::Number, "2");
  /// builder.finish_node();
  /// ```
  ///
  /// See also: [`rowan::GreenNodeBuilder::start_node`]
  #[inline]
  pub fn start_node(&self, kind: Lang::Kind) {
    self
      .builder
      .borrow_mut()
      .start_node(Lang::kind_to_raw(kind));
  }

  /// Starts a new node at a previously created checkpoint.
  ///
  /// This allows you to retroactively wrap children that were added after the
  /// checkpoint was created. Useful for handling operator precedence and
  /// left-recursive grammars.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::SyntaxTreeBuilder;
  ///
  /// let builder = SyntaxTreeBuilder::<MyLanguage>::new();
  /// let checkpoint = builder.checkpoint();
  ///
  /// // Add a number
  /// builder.token(SyntaxKind::Number, "42");
  ///
  /// // Later, decide to wrap it in a unary expression
  /// builder.start_node_at(checkpoint, SyntaxKind::UnaryExpression);
  /// builder.token(SyntaxKind::Minus, "-");
  /// builder.finish_node();
  /// // Result: UnaryExpression(Number("42"), Minus("-"))
  /// ```
  ///
  /// See also: [`rowan::GreenNodeBuilder::start_node_at`]
  #[inline]
  pub fn start_node_at(&self, checkpoint: rowan::Checkpoint, kind: Lang::Kind) {
    self
      .builder
      .borrow_mut()
      .start_node_at(checkpoint, Lang::kind_to_raw(kind));
  }

  /// Adds a token with the given kind and text to the current node.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::SyntaxTreeBuilder;
  ///
  /// let builder = SyntaxTreeBuilder::<MyLanguage>::new();
  ///
  /// builder.start_node(SyntaxKind::Identifier);
  /// builder.token(SyntaxKind::IdentifierToken, "my_variable");
  /// builder.finish_node();
  /// ```
  ///
  /// See also: [`rowan::GreenNodeBuilder::token`]
  #[inline]
  pub fn token(&self, kind: Lang::Kind, text: &str) {
    self
      .builder
      .borrow_mut()
      .token(Lang::kind_to_raw(kind), text);
  }

  /// Finishes the current node started with [`start_node()`](Self::start_node)
  /// or [`start_node_at()`](Self::start_node_at).
  ///
  /// # Panics
  ///
  /// Panics if there is no node to finish (i.e., `finish_node()` was called
  /// more times than `start_node()`).
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::SyntaxTreeBuilder;
  ///
  /// let builder = SyntaxTreeBuilder::<MyLanguage>::new();
  ///
  /// builder.start_node(SyntaxKind::Root);
  /// builder.token(SyntaxKind::Identifier, "foo");
  /// builder.finish_node(); // Finishes the Root node
  /// ```
  ///
  /// See also: [`rowan::GreenNodeBuilder::finish_node`]
  #[inline]
  pub fn finish_node(&self) {
    self.builder.borrow_mut().finish_node();
  }

  /// Completes the tree building and returns the final green node.
  ///
  /// This consumes the builder and returns the root [`rowan::GreenNode`],
  /// which can be converted to a [`rowan::SyntaxNode`] for traversal.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::SyntaxTreeBuilder;
  /// use rowan::SyntaxNode;
  ///
  /// let builder = SyntaxTreeBuilder::<MyLanguage>::new();
  ///
  /// builder.start_node(SyntaxKind::Root);
  /// builder.token(SyntaxKind::Identifier, "foo");
  /// builder.finish_node();
  ///
  /// let green = builder.finish();
  /// let root = SyntaxNode::new_root(green);
  /// ```
  ///
  /// See also: [`rowan::GreenNodeBuilder::finish`]
  #[inline]
  pub fn finish(self) -> rowan::GreenNode {
    self.builder.into_inner().finish()
  }
}

/// Base trait for all typed CST elements (nodes and tokens).
///
/// `CstElement` provides the common interface shared by both CST nodes
/// ([`CstNode`]) and CST tokens ([`CstToken`]). It enables:
/// - **Type checking**: Verify if an untyped element can be cast to a specific type
/// - **Type identity**: Associate elements with their syntax kind
/// - **Polymorphism**: Write generic code that works with both nodes and tokens
///
/// # Design
///
/// This trait serves as the foundation of the typed CST hierarchy:
/// ```text
/// CstElement (base)
///     ├── CstNode (for interior nodes)
///     └── CstToken (for leaf tokens)
/// ```
///
/// # Type Parameters
///
/// - `Lang`: The rowan [`Language`] type defining syntax kinds
pub trait CstElement<Lang: Language>: core::fmt::Debug {
  /// The syntax kind of this CST element.
  ///
  /// For enum elements representing multiple variants, this can be a marker value
  /// that is not directly used for casting but serves as documentation.
  const KIND: Lang::Kind;

  /// Returns `true` if the given kind can be cast to this CST element.
  ///
  /// This method determines whether an untyped rowan element with a specific
  /// syntax kind can be safely converted to this typed element.
  ///
  /// # Implementation Guidelines
  ///
  /// - **Single variant**: Return `kind == Self::KIND`
  /// - **Multiple variants**: Use pattern matching to check all valid kinds
  /// - **Performance**: This method is often called frequently, keep it fast
  ///
  /// # Examples
  ///
  /// ## Simple Element (Single Kind)
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstElement;
  ///
  /// impl CstElement for Comma {
  ///     const KIND: SyntaxKind = SyntaxKind::Comma;
  ///
  ///     fn castable(kind: SyntaxKind) -> bool {
  ///         kind == SyntaxKind::Comma
  ///     }
  /// }
  /// ```
  ///
  /// ## Enum Element (Multiple Kinds)
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstElement;
  ///
  /// impl CstElement for BinaryOperator {
  ///     const KIND: SyntaxKind = SyntaxKind::BinaryOp; // Marker
  ///
  ///     fn castable(kind: SyntaxKind) -> bool {
  ///         matches!(
  ///             kind,
  ///             SyntaxKind::Plus
  ///             | SyntaxKind::Minus
  ///             | SyntaxKind::Star
  ///             | SyntaxKind::Slash
  ///         )
  ///     }
  /// }
  /// ```
  ///
  /// ## Usage in Type Checking
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstElement;
  ///
  /// // Check before casting
  /// if Comma::castable(token.kind()) {
  ///     let comma = Comma::try_cast_node(token).unwrap();
  /// }
  /// ```
  fn castable(kind: Lang::Kind) -> bool
  where
    Self: Sized;
}

/// Trait for typed CST tokens (leaf elements in the syntax tree).
///
/// `CstToken` provides a type-safe wrapper around rowan's untyped [`SyntaxToken`],
/// representing terminal elements in the concrete syntax tree. Tokens are the leaf nodes
/// that contain actual source text (keywords, identifiers, literals, punctuation, etc.).
///
/// # Design
///
/// Tokens differ from nodes ([`CstNode`]) in that:
/// - **Tokens are leaves**: They contain source text directly
/// - **Nodes are interior**: They have children and structure the tree
/// - **Zero-cost**: Token wrappers have the same memory layout as [`SyntaxToken`]
///
/// # Type Parameters
///
/// - `Lang`: The rowan [`Language`] type defining syntax kinds
pub trait CstToken<Lang: Language>: CstElement<Lang> {
  /// Attempts to cast the given syntax token to this typed token.
  ///
  /// Returns an error if the token's kind doesn't match this type.
  ///
  /// # Errors
  ///
  /// Returns [`CstTokenMismatch`](error::CstTokenMismatch) if:
  /// - The token's kind doesn't match the expected kind for this type
  /// - For enum tokens, the kind is not one of the valid variants
  fn try_cast_token(syntax: SyntaxToken<Lang>) -> Result<Self, error::CstTokenMismatch<Self, Lang>>
  where
    Self: Sized;

  /// Returns a reference to the underlying syntax token.
  ///
  /// This provides access to rowan's token APIs for inspecting position,
  /// text, and tree structure.
  fn syntax(&self) -> &SyntaxToken<Lang>;

  /// Returns the source text of this token.
  ///
  /// This is a convenience method that extracts the text from the underlying
  /// [`SyntaxToken`]. The text is always valid UTF-8.
  fn text(&self) -> &str
  where
    Lang: 'static,
  {
    self.syntax().text()
  }
}

/// The main trait for typed CST nodes with zero-cost conversions.
///
/// `CstNode` provides a type-safe wrapper around rowan's untyped [`SyntaxNode`], allowing
/// you to work with strongly-typed CST nodes. The conversion between typed and untyped
/// nodes has **zero runtime cost** - both representations have exactly the same memory
/// layout (a pointer to the tree root and a pointer to the node itself).
///
/// # Design
///
/// The `CstNode` trait enables:
/// - **Type safety**: Compile-time guarantees about node types
/// - **Zero-cost**: No runtime overhead for typed wrappers
/// - **Pattern matching**: Cast nodes to specific types
/// - **Tree traversal**: Navigate the CST with type information
///
/// # Type Parameters
///
/// - `Lang`: The rowan [`Language`] type defining syntax kinds
pub trait CstNode<Lang: Language>: CstElement<Lang> + Syntax {
  /// Attempts to cast the given syntax node to this CST node.
  ///
  /// Returns an error if the node's kind doesn't match this type.
  fn try_cast_node(syntax: SyntaxNode<Lang>) -> Result<Self, error::SyntaxError<Self, Lang>>
  where
    Self: Sized;

  /// Returns a reference to the underlying syntax node.
  ///
  /// This provides access to rowan's tree traversal APIs.
  fn syntax(&self) -> &SyntaxNode<Lang>;

  /// Returns the source string of this CST node.
  ///
  /// This includes all text spanned by this node, including whitespace and trivia.
  fn source_string(&self) -> String {
    self.syntax().to_string()
  }

  /// Clones this CST node for update operations.
  ///
  /// This creates a mutable copy of the node that can be modified using rowan's
  /// mutation APIs. The original tree remains unchanged.
  fn clone_for_update(&self) -> Self
  where
    Self: Sized,
  {
    Self::try_cast_node(self.syntax().clone_for_update()).unwrap()
  }

  /// Clones the subtree rooted at this CST node.
  ///
  /// This creates a deep copy of this node and all its descendants, detached
  /// from the original tree.
  fn clone_subtree(&self) -> Self
  where
    Self: Sized,
  {
    Self::try_cast_node(self.syntax().clone_subtree()).unwrap()
  }
}

/// An iterator over typed CST children of a particular node type.
///
/// `CstNodeChildren` filters and casts child nodes to a specific typed node type,
/// skipping any children that cannot be cast to the target type.
#[derive(Debug, From, Into)]
#[repr(transparent)]
pub struct CstNodeChildren<N, Lang: Language> {
  inner: rowan::SyntaxNodeChildren<Lang>,
  _m: PhantomData<N>,
}

impl<N, Lang: Language> Clone for CstNodeChildren<N, Lang> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      _m: PhantomData,
    }
  }
}

impl<N, Lang: Language> CstNodeChildren<N, Lang> {
  #[inline]
  fn new(parent: &SyntaxNode<Lang>) -> Self {
    Self {
      inner: parent.children(),
      _m: PhantomData,
    }
  }

  /// Returns an iterator over syntax node children matching a kind predicate.
  ///
  /// This allows further filtering of children based on their syntax kind,
  /// returning the underlying [`SyntaxNode`] instead of typed nodes.
  pub fn by_kind<F>(self, f: F) -> impl Iterator<Item = SyntaxNode<Lang>>
  where
    F: Fn(Lang::Kind) -> bool,
  {
    self.inner.by_kind(f)
  }
}

impl<N, Lang> Iterator for CstNodeChildren<N, Lang>
where
  N: CstNode<Lang>,
  Lang: Language,
{
  type Item = N;

  #[inline]
  fn next(&mut self) -> Option<N> {
    self.inner.find_map(|t| N::try_cast_node(t).ok())
  }
}

/// Utility functions for casting and accessing CST nodes.
///
/// This module provides convenient functions for working with typed CST nodes,
/// including finding children, accessing tokens, and casting between types.
pub mod cast;

/// Error types for CST operations.
pub mod error;

#[cfg(test)]
mod tests {
  use super::*;
  use rowan::{Language, SyntaxKind};

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  enum TestKind {
    Root,
    Ident,
    Plus,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  enum TestLang {}

  impl Language for TestLang {
    type Kind = TestKind;

    fn kind_from_raw(raw: SyntaxKind) -> TestKind {
      match raw.0 {
        0 => TestKind::Root,
        1 => TestKind::Ident,
        2 => TestKind::Plus,
        _ => panic!("unknown kind"),
      }
    }

    fn kind_to_raw(kind: TestKind) -> SyntaxKind {
      match kind {
        TestKind::Root => SyntaxKind(0),
        TestKind::Ident => SyntaxKind(1),
        TestKind::Plus => SyntaxKind(2),
      }
    }
  }

  #[test]
  fn builder_new_and_default() {
    let b1 = SyntaxTreeBuilder::<TestLang>::new();
    let b2 = SyntaxTreeBuilder::<TestLang>::default();
    // Just verify they can be created
    let _ = format!("{:?}", b1);
    let _ = format!("{:?}", b2);
  }

  #[test]
  fn builder_simple_tree() {
    let builder = SyntaxTreeBuilder::<TestLang>::new();
    builder.start_node(TestKind::Root);
    builder.token(TestKind::Ident, "hello");
    builder.finish_node();
    let green = builder.finish();

    let root = rowan::SyntaxNode::<TestLang>::new_root(green);
    assert_eq!(root.kind(), TestKind::Root);
    assert_eq!(root.to_string(), "hello");
  }

  #[test]
  fn builder_with_checkpoint() {
    let builder = SyntaxTreeBuilder::<TestLang>::new();
    builder.start_node(TestKind::Root);

    let checkpoint = builder.checkpoint();
    builder.token(TestKind::Ident, "foo");

    // Wrap the identifier in a new node retroactively
    builder.start_node_at(checkpoint, TestKind::Root);
    builder.finish_node();

    builder.finish_node();
    let green = builder.finish();
    let root = rowan::SyntaxNode::<TestLang>::new_root(green);
    assert_eq!(root.to_string(), "foo");
  }

  #[test]
  fn builder_multiple_tokens() {
    let builder = SyntaxTreeBuilder::<TestLang>::new();
    builder.start_node(TestKind::Root);
    builder.token(TestKind::Ident, "a");
    builder.token(TestKind::Plus, "+");
    builder.token(TestKind::Ident, "b");
    builder.finish_node();
    let green = builder.finish();

    let root = rowan::SyntaxNode::<TestLang>::new_root(green);
    assert_eq!(root.to_string(), "a+b");
  }

  #[test]
  fn cst_node_children_clone() {
    let builder = SyntaxTreeBuilder::<TestLang>::new();
    builder.start_node(TestKind::Root);
    builder.token(TestKind::Ident, "hello");
    builder.finish_node();
    let green = builder.finish();
    let root = rowan::SyntaxNode::<TestLang>::new_root(green);

    let children: CstNodeChildren<rowan::SyntaxNode<TestLang>, TestLang> =
      CstNodeChildren::new(&root);
    let _cloned = children.clone();
  }

  #[test]
  fn cst_node_children_by_kind() {
    let builder = SyntaxTreeBuilder::<TestLang>::new();
    builder.start_node(TestKind::Root);
    builder.start_node(TestKind::Root);
    builder.token(TestKind::Ident, "inner");
    builder.finish_node();
    builder.finish_node();
    let green = builder.finish();
    let root = rowan::SyntaxNode::<TestLang>::new_root(green);

    let children: CstNodeChildren<rowan::SyntaxNode<TestLang>, TestLang> =
      CstNodeChildren::new(&root);
    let matching: Vec<_> = children.by_kind(|k| k == TestKind::Root).collect();
    assert_eq!(matching.len(), 1);
  }
}
