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
//! 2. **[`Parseable`](crate::cst::Parseable)**: Trait for types that can produce CST parsers
//! 3. **[`CstElement`](crate::cst::CstElement)**: Base trait for all typed CST elements (nodes and tokens)
//! 4. **[`CstNode`](crate::cst::CstNode)**: Trait for typed CST nodes with zero-cost conversions
//! 5. **[`CstToken`](crate::cst::CstToken)**: Trait for typed CST tokens (terminal elements)
//! 6. **[`cast`](crate::cst::cast)**: Utility functions for working with CST nodes and tokens
//! 7. **[`error`](crate::cst::error)**: Error types for CST operations
//!
//! # Design Philosophy
//!
//! - **Zero-cost abstractions**: Typed CST nodes are just pointers, no runtime overhead
//! - **Lossless**: All source information is preserved in the tree
//! - **Immutable**: Trees are immutable by default (use `clone_for_update()` for mutations)
//! - **Type-safe**: Compile-time guarantees about node types and relationships
//!
//! # Basic Usage
//!
//! ```rust,ignore
//! use tokit::cst::{SyntaxTreeBuilder, Node, Parseable};
//! use rowan::Language;
//!
//! // 1. Define your language
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! enum SyntaxKind {
//!     Root,
//!     Identifier,
//!     Number,
//!     // ... other kinds
//! }
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! struct MyLanguage;
//!
//! impl Language for MyLanguage {
//!     type Kind = SyntaxKind;
//!
//!     fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
//!         // Convert from rowan's raw kind
//!         todo!()
//!     }
//!
//!     fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
//!         // Convert to rowan's raw kind
//!         todo!()
//!     }
//! }
//!
//! // 2. Create a builder
//! let builder = SyntaxTreeBuilder::<MyLanguage>::new();
//!
//! // 3. Build the tree (usually done by a parser)
//! builder.start_node(SyntaxKind::Root);
//! builder.token(SyntaxKind::Identifier, "foo");
//! builder.finish_node();
//!
//! let green = builder.finish();
//! let root = SyntaxNode::new_root(green);
//! ```
//!
//! # Integration with Parsers
//!
//! The [`Parseable`](crate::cst::Parseable) trait integrates CST building with Chumsky parsers:
//!
//! ```rust,ignore
//! use tokit::cst::{Parseable, SyntaxTreeBuilder};
//!
//! struct Expression;
//!
//! impl<'a, I, T, Error> Parseable<'a, I, T, Error> for Expression
//! where
//!     I: Tokenizer<'a, T>,
//!     T: TriviaToken<'a>,
//! {
//!     type Language = MyLanguage;
//!
//!     fn parser<E>(
//!         builder: &'a SyntaxTreeBuilder<Self::Language>,
//!     ) -> impl chumsky::Parser<'a, I, (), E> + Clone
//!     where
//!         E: chumsky::extra::ParserExtra<'a, I, Error = Error>,
//!     {
//!         // Return a parser that builds CST nodes
//!         todo!()
//!     }
//! }
//! ```
//!
//! # Working with Typed Nodes
//!
//! The [`Node`](crate::cst::Node) trait provides zero-cost typed wrappers around syntax nodes:
//!
//! ```rust,ignore
//! use tokit::cst::Node;
//!
//! #[derive(Debug)]
//! struct IdentifierNode {
//!     syntax: SyntaxNode<MyLanguage>,
//! }
//!
//! impl Node for IdentifierNode {
//!     type Language = MyLanguage;
//!     const KIND: SyntaxKind = SyntaxKind::Identifier;
//!
//!     fn can_cast(kind: SyntaxKind) -> bool {
//!         kind == Self::KIND
//!     }
//!
//!     fn try_cast_node(syntax: SyntaxNode<Self::Language>) -> Result<Self, error::CstNodeMismatch<Self>> {
//!         if Self::can_cast(syntax.kind()) {
//!             Ok(Self { syntax })
//!         } else {
//!             Err(error::CstNodeMismatch::new(Self::KIND, syntax))
//!         }
//!     }
//!
//!     fn syntax(&self) -> &SyntaxNode<Self::Language> {
//!         &self.syntax
//!     }
//! }
//! ```
//!
//! # See Also
//!
//! - [rowan documentation](https://docs.rs/rowan) - The underlying CST library
//! - [`TriviaToken`](crate::TriviaToken) - For handling trivia in CSTs

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
/// - `Language`: The rowan [`Language`] type defining syntax kinds
///
/// # Implementation
///
/// You typically don't implement this trait directly. Instead:
/// - For nodes: implement [`CstNode`] (which extends this trait)
/// - For tokens: implement [`CstToken`] (which extends this trait)
///
/// # Examples
///
/// ## Simple Token Implementation
///
/// ```rust,ignore
/// use tokit::cst::{CstElement, CstToken};
///
/// #[derive(Debug)]
/// struct Comma {
///     syntax: SyntaxToken<MyLanguage>,
/// }
///
/// impl CstElement for Comma {
///     type Language = MyLanguage;
///     const KIND: SyntaxKind = SyntaxKind::Comma;
///
///     fn can_cast(kind: SyntaxKind) -> bool {
///         kind == SyntaxKind::Comma
///     }
/// }
///
/// impl CstToken for Comma {
///     // ... implement token-specific methods
/// }
/// ```
///
/// ## Node with Multiple Variants
///
/// ```rust,ignore
/// use tokit::cst::{CstElement, CstNode};
///
/// #[derive(Debug)]
/// enum Literal {
///     Number(NumberLiteral),
///     String(StringLiteral),
///     Boolean(BooleanLiteral),
/// }
///
/// impl CstElement for Literal {
///     type Language = MyLanguage;
///     const KIND: SyntaxKind = SyntaxKind::Literal; // Marker
///
///     fn can_cast(kind: SyntaxKind) -> bool {
///         matches!(
///             kind,
///             SyntaxKind::NumberLiteral
///             | SyntaxKind::StringLiteral
///             | SyntaxKind::BooleanLiteral
///         )
///     }
/// }
///
/// impl CstNode for Literal {
///     // ... implement node-specific methods
/// }
/// ```
///
/// ## Generic Functions Using Elements
///
/// ```rust,ignore
/// use tokit::cst::CstElement;
///
/// fn element_kind<T: CstElement>(element: &T) -> String {
///     format!("{:?}", T::KIND)
/// }
///
/// // Works with both nodes and tokens
/// let comma: Comma = ...;
/// let expr: Expression = ...;
/// println!("Token kind: {}", element_kind(&comma));
/// println!("Node kind: {}", element_kind(&expr));
/// ```
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
  ///     type Language = MyLanguage;
  ///     const KIND: SyntaxKind = SyntaxKind::Comma;
  ///
  ///     fn can_cast(kind: SyntaxKind) -> bool {
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
  ///     type Language = MyLanguage;
  ///     const KIND: SyntaxKind = SyntaxKind::BinaryOp; // Marker
  ///
  ///     fn can_cast(kind: SyntaxKind) -> bool {
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
  /// if Comma::can_cast(token.kind()) {
  ///     let comma = Comma::try_cast_node(token).unwrap();
  /// }
  /// ```
  fn can_cast(kind: Lang::Kind) -> bool
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
/// - `Language`: The rowan [`Language`] type defining syntax kinds
///
/// # Implementation
///
/// To implement `CstToken`, you need to:
/// 1. Implement [`CstElement`] to define the token's kind and casting logic
/// 2. Implement [`try_cast_token()`](Self::try_cast_token) to convert from untyped tokens
/// 3. Implement [`syntax()`](Self::syntax) to access the underlying token
/// 4. Optionally override [`text()`](Self::text) if custom text extraction is needed
///
/// # Examples
///
/// ## Simple Token Implementation
///
/// ```rust,ignore
/// use tokit::cst::{CstElement, CstToken, error};
/// use rowan::SyntaxToken;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct Comma {
///     syntax: SyntaxToken<MyLanguage>,
/// }
///
/// impl CstElement for Comma {
///     type Language = MyLanguage;
///     const KIND: SyntaxKind = SyntaxKind::Comma;
///
///     fn can_cast(kind: SyntaxKind) -> bool {
///         kind == SyntaxKind::Comma
///     }
/// }
///
/// impl CstToken for Comma {
///     fn try_cast_token(
///         syntax: SyntaxToken<Self::Language>
///     ) -> Result<Self, error::CstTokenMismatch<Self>> {
///         if Self::can_cast(syntax.kind()) {
///             Ok(Self { syntax })
///         } else {
///             Err(error::CstTokenMismatch::new(Self::KIND, syntax))
///         }
///     }
///
///     fn syntax(&self) -> &SyntaxToken<Self::Language> {
///         &self.syntax
///     }
/// }
/// ```
///
/// ## Token with Multiple Variants (Enum)
///
/// ```rust,ignore
/// use tokit::cst::{CstElement, CstToken};
///
/// #[derive(Debug, Clone)]
/// enum BinaryOperator {
///     Plus(PlusToken),
///     Minus(MinusToken),
///     Star(StarToken),
///     Slash(SlashToken),
/// }
///
/// impl CstElement for BinaryOperator {
///     type Language = MyLanguage;
///     const KIND: SyntaxKind = SyntaxKind::BinaryOp; // Marker
///
///     fn can_cast(kind: SyntaxKind) -> bool {
///         matches!(
///             kind,
///             SyntaxKind::Plus | SyntaxKind::Minus
///             | SyntaxKind::Star | SyntaxKind::Slash
///         )
///     }
/// }
///
/// impl CstToken for BinaryOperator {
///     fn try_cast_token(
///         syntax: SyntaxToken<Self::Language>
///     ) -> Result<Self, error::CstTokenMismatch<Self>> {
///         match syntax.kind() {
///             SyntaxKind::Plus => Ok(BinaryOperator::Plus(PlusToken { syntax })),
///             SyntaxKind::Minus => Ok(BinaryOperator::Minus(MinusToken { syntax })),
///             SyntaxKind::Star => Ok(BinaryOperator::Star(StarToken { syntax })),
///             SyntaxKind::Slash => Ok(BinaryOperator::Slash(SlashToken { syntax })),
///             _ => Err(error::CstTokenMismatch::new(Self::KIND, syntax)),
///         }
///     }
///
///     fn syntax(&self) -> &SyntaxToken<Self::Language> {
///         match self {
///             BinaryOperator::Plus(t) => &t.syntax,
///             BinaryOperator::Minus(t) => &t.syntax,
///             BinaryOperator::Star(t) => &t.syntax,
///             BinaryOperator::Slash(t) => &t.syntax,
///         }
///     }
/// }
/// ```
///
/// ## Using Tokens
///
/// ```rust,ignore
/// use tokit::cst::{CstToken, cast};
///
/// // Cast from untyped token
/// let comma = Comma::try_cast_token(syntax_token)?;
///
/// // Access token text
/// assert_eq!(comma.text(), ",");
///
/// // Get underlying syntax token for rowan APIs
/// let parent = comma.syntax().parent();
/// ```
///
/// ## Finding Tokens in Nodes
///
/// ```rust,ignore
/// use tokit::cst::cast;
///
/// // Find a specific token in a node
/// let equals_token = cast::token(&assignment_node, &SyntaxKind::Equals);
///
/// // Check if a token exists
/// if let Some(async_kw) = cast::token(&function_node, &SyntaxKind::AsyncKeyword) {
///     println!("Function is async");
/// }
/// ```
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
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstToken;
  ///
  /// // Try to cast a token
  /// match Comma::try_cast_token(syntax_token) {
  ///     Ok(comma) => println!("Found comma at: {:?}", comma.syntax().text_range()),
  ///     Err(e) => eprintln!("Not a comma: {}", e),
  /// }
  ///
  /// // Unwrap if you're sure it's the right type
  /// let plus = PlusToken::try_cast_token(syntax_token).unwrap();
  /// ```
  fn try_cast_token(syntax: SyntaxToken<Lang>) -> Result<Self, error::CstTokenMismatch<Self, Lang>>
  where
    Self: Sized;

  /// Returns a reference to the underlying syntax token.
  ///
  /// This provides access to rowan's token APIs for inspecting position,
  /// text, and tree structure.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstToken;
  ///
  /// let comma: Comma = ...;
  ///
  /// // Get text range
  /// let range = comma.syntax().text_range();
  ///
  /// // Get parent node
  /// let parent = comma.syntax().parent();
  ///
  /// // Get next sibling
  /// let next = comma.syntax().next_sibling_or_token();
  /// ```
  fn syntax(&self) -> &SyntaxToken<Lang>;

  /// Returns the source text of this token.
  ///
  /// This is a convenience method that extracts the text from the underlying
  /// [`SyntaxToken`]. The text is always valid UTF-8.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::CstToken;
  ///
  /// let identifier: IdentifierToken = ...;
  /// assert_eq!(identifier.text(), "my_variable");
  ///
  /// let comma: Comma = ...;
  /// assert_eq!(comma.text(), ",");
  ///
  /// let number: NumberToken = ...;
  /// let value: i32 = number.text().parse()?;
  /// ```
  fn text(&self) -> &str
  where
    Lang: 'static,
  {
    self.syntax().text()
  }
}

/// The main trait for typed CST nodes with zero-cost conversions.
///
/// `Node` provides a type-safe wrapper around rowan's untyped [`SyntaxNode`], allowing
/// you to work with strongly-typed CST nodes. The conversion between typed and untyped
/// nodes has **zero runtime cost** - both representations have exactly the same memory
/// layout (a pointer to the tree root and a pointer to the node itself).
///
/// # Design
///
/// The `Node` trait enables:
/// - **Type safety**: Compile-time guarantees about node types
/// - **Zero-cost**: No runtime overhead for typed wrappers
/// - **Pattern matching**: Cast nodes to specific types
/// - **Tree traversal**: Navigate the CST with type information
///
/// # Type Parameters
///
/// - `Language`: The rowan [`Language`] type defining syntax kinds
///
/// # Implementation
///
/// To implement `Node`, you need to:
/// 1. Define a struct wrapping [`SyntaxNode<Language>`](SyntaxNode)
/// 2. Specify the [`KIND`](Self::KIND) constant
/// 3. Implement [`can_cast()`](Self::can_cast) to check if a kind matches
/// 4. Implement [`try_cast_node()`](Self::try_cast_node) to convert from untyped nodes
/// 5. Implement [`syntax()`](Self::syntax) to access the underlying node
///
/// # Examples
///
/// ## Basic Node Implementation
///
/// ```rust,ignore
/// use tokit::cst::{Node, error};
/// use rowan::SyntaxNode;
///
/// #[derive(Debug, Clone)]
/// struct IdentifierNode {
///     syntax: SyntaxNode<MyLanguage>,
/// }
///
/// impl Node for IdentifierNode {
///     type Language = MyLanguage;
///     const KIND: SyntaxKind = SyntaxKind::Identifier;
///
///     fn can_cast(kind: SyntaxKind) -> bool {
///         kind == Self::KIND
///     }
///
///     fn try_cast_node(
///         syntax: SyntaxNode<Self::Language>
///     ) -> Result<Self, error::CstNodeMismatch<Self>> {
///         if Self::can_cast(syntax.kind()) {
///             Ok(Self { syntax })
///         } else {
///             Err(error::CstNodeMismatch::new(Self::KIND, syntax))
///         }
///     }
///
///     fn syntax(&self) -> &SyntaxNode<Self::Language> {
///         &self.syntax
///     }
/// }
/// ```
///
/// ## Using Nodes
///
/// ```rust,ignore
/// use tokit::cst::CstNode;
///
/// // Try to cast an untyped node
/// let identifier = IdentifierNode::try_cast_node(syntax_node)?;
///
/// // Access the source text
/// let text = identifier.source_string();
///
/// // Clone for mutation
/// let mutable = identifier.clone_for_update();
/// ```
///
/// ## Enum Nodes for Variants
///
/// ```rust,ignore
/// use tokit::cst::CstNode;
///
/// #[derive(Debug, Clone)]
/// enum Expression {
///     Binary(BinaryExpr),
///     Unary(UnaryExpr),
///     Literal(LiteralExpr),
/// }
///
/// impl CstNode for Expression {
///     type Language = MyLanguage;
///     const KIND: SyntaxKind = SyntaxKind::Expression; // Marker, not used
///
///     fn can_cast(kind: SyntaxKind) -> bool {
///         matches!(
///             kind,
///             SyntaxKind::BinaryExpr | SyntaxKind::UnaryExpr | SyntaxKind::Literal
///         )
///     }
///
///     fn try_cast_node(
///         syntax: SyntaxNode<Self::Language>
///     ) -> Result<Self, error::CstNodeMismatch<Self>> {
///         match syntax.kind() {
///             SyntaxKind::BinaryExpr => Ok(Expression::Binary(BinaryExpr { syntax })),
///             SyntaxKind::UnaryExpr => Ok(Expression::Unary(UnaryExpr { syntax })),
///             SyntaxKind::Literal => Ok(Expression::Literal(LiteralExpr { syntax })),
///             _ => Err(error::CstNodeMismatch::new(Self::KIND, syntax)),
///         }
///     }
///
///     fn syntax(&self) -> &SyntaxNode<Self::Language> {
///         match self {
///             Expression::Binary(e) => &e.syntax,
///             Expression::Unary(e) => &e.syntax,
///             Expression::Literal(e) => &e.syntax,
///         }
///     }
/// }
/// ```
pub trait CstNode<Lang: Language>: CstElement<Lang> + Syntax {
  /// Attempts to cast the given syntax node to this CST node.
  ///
  /// Returns an error if the node's kind doesn't match this type.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::Node;
  ///
  /// let identifier = IdentifierNode::try_cast_node(syntax_node)?;
  /// ```
  fn try_cast_node(syntax: SyntaxNode<Lang>) -> Result<Self, error::SyntaxError<Self, Lang>>
  where
    Self: Sized;

  /// Returns a reference to the underlying syntax node.
  ///
  /// This provides access to rowan's tree traversal APIs.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::Node;
  ///
  /// let node: IdentifierNode = ...;
  /// let parent = node.syntax().parent();
  /// let children = node.syntax().children();
  /// ```
  fn syntax(&self) -> &SyntaxNode<Lang>;

  /// Returns the source string of this CST node.
  ///
  /// This includes all text spanned by this node, including whitespace and trivia.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::Node;
  ///
  /// let identifier: IdentifierNode = ...;
  /// assert_eq!(identifier.source_string(), "my_variable");
  /// ```
  fn source_string(&self) -> String {
    self.syntax().to_string()
  }

  /// Clones this CST node for update operations.
  ///
  /// This creates a mutable copy of the node that can be modified using rowan's
  /// mutation APIs. The original tree remains unchanged.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::Node;
  ///
  /// let original: IdentifierNode = ...;
  /// let mut mutable = original.clone_for_update();
  ///
  /// // Modify the mutable copy
  /// // (using rowan's mutation APIs)
  /// ```
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
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::Node;
  ///
  /// let original: ExpressionNode = ...;
  /// let copy = original.clone_subtree();
  ///
  /// // copy is independent of original
  /// ```
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
///
/// # Type Parameters
///
/// - `N`: The typed [`Node`] type to iterate over
///
/// # Examples
///
/// ```rust,ignore
/// use tokit::cst::{Node, cast};
///
/// // Get all Identifier children of a function
/// let identifiers: Vec<IdentifierNode> = cast::children(&function_node.syntax())
///     .collect();
///
/// // Filter by kind within the same type
/// let params = cast::children::<Parameter>(&function_node.syntax())
///     .by_kind(|k| k == SyntaxKind::Parameter);
/// ```
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
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::cst::cast;
  ///
  /// // Get all expression children, filtered by specific kinds
  /// let binary_exprs = cast::children::<Expression>(&node)
  ///     .by_kind(|k| k == SyntaxKind::BinaryExpr);
  /// ```
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
///
/// # Examples
///
/// ```rust,ignore
/// use tokit::cst::cast;
///
/// // Get the first identifier child
/// let identifier = cast::child::<IdentifierNode>(&parent_node);
///
/// // Get all statement children
/// let statements: Vec<Statement> = cast::children(&function_node).collect();
///
/// // Get a specific token
/// let equals_token = cast::token(&assignment_node, &SyntaxKind::Equals);
/// ```
pub mod cast;

/// Error types for CST operations.
pub mod error;
