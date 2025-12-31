//! Keywordifier types for language syntax trees.
//!
//! This module provides generic identifier types that can be used across different
//! programming languages and string representations. Keywordifiers are fundamental
//! building blocks in most languages, representing names for variables, functions,
//! types, and other named entities.
//!
//! # Design Philosophy
//!
//! The [`Keyword`] type is generic over both the source string type (`S`) and the
//! language marker (`Lang`). This design provides maximum flexibility:
//!
//! - **String type flexibility**: Use `&str` for zero-copy parsing, `String` for
//!   owned data, or custom interned string types for memory efficiency
//! - **Language safety**: The `Lang` parameter ensures keywords from different
//!   languages don't mix accidentally
//! - **SimpleSpan tracking**: All keywords carry their source location for diagnostics
//!
//! # Common Usage Patterns
//!
//! ## Zero-Copy Parsing
//!
//! ```rust,ignore
//! use tokit::types::Keyword;
//! use tokit::utils::SimpleSimpleSpan;
//!
//! // Parse keywords without allocating
//! type YulKeyword<'a> = Keyword<&'a str, YulLang>;
//!
//! let ident = YulKeyword::new(SimpleSpan::new(0, 3), "foo");
//! assert_eq!(ident.source_ref(), &"foo");
//! ```
//!
//! ## Owned Keywordifiers
//!
//! ```rust,ignore
//! // Store keywords in AST nodes that outlive the source
//! type OwnedKeyword = Keyword<String, MyLang>;
//!
//! let ident = OwnedKeyword::new(span, source_str.to_string());
//! ```
//!
//! ## String Interning
//!
//! ```rust,ignore
//! // Use interned strings for memory efficiency
//! type InternedKeyword = Keyword<Symbol, MyLang>;
//!
//! let ident = InternedKeyword::new(span, interner.intern("identifier"));
//! ```
//!
//! # Error Recovery
//!
//! `Keyword` implements [`ErrorNode`] when the source type `S` also implements it,
//! allowing creation of placeholder keywords during error recovery:
//!
//! ```rust,ignore
//! use tokit::error::ErrorNode;
//!
//! // Create placeholder for malformed identifier
//! let bad_ident = Keyword::<String, YulLang>::error(span);
//!
//! // Create placeholder for missing identifier
//! let missing_ident = Keyword::<String, YulLang>::missing(span);
//! ```

use core::marker::PhantomData;

use crate::{
  error::ErrorNode,
  utils::{AsSpan, IntoComponents, SimpleSpan},
};

/// A language identifier with span tracking.
///
/// Keywordifiers are names used in source code to refer to variables, functions,
/// types, and other named entities. This type wraps a source string representation
/// with position information and a language marker.
///
/// # Type Parameters
///
/// - `S`: The source string type (`&str`, `String`, interned string, etc.)
/// - `Lang`: Language marker type for type safety (e.g., `YulLang`, `SolidityLang`)
///
/// # Design Notes
///
/// ## Why Generic Over String Type?
///
/// Different use cases require different string representations:
/// - **Parsing**: Use `&str` for zero-copy efficiency
/// - **AST storage**: Use `String` when the AST outlives the source
/// - **Large codebases**: Use interned strings to deduplicate common keywords
///
/// ## Why Language Marker?
///
/// The `Lang` parameter prevents mixing keywords from different languages:
/// ```rust,ignore
/// let yul_ident: Keyword<&str, YulLang> = ...;
/// let sol_ident: Keyword<&str, SolidityLang> = ...;
///
/// // Compile error: type mismatch
/// // let mixed = vec![yul_ident, sol_ident];
/// ```
///
/// # Examples
///
/// ## Creating Keywordifiers
///
/// ```rust
/// use tokit::types::Keyword;
/// use tokit::utils::SimpleSimpleSpan;
/// # struct MyLang;
///
/// // Zero-copy identifier
/// let span = SimpleSpan::new(5, 11);
/// let ident = Keyword::<&str, MyLang>::new(span, "my_var");
///
/// assert_eq!(ident.span(), span);
/// assert_eq!(ident.source_ref(), &"my_var");
/// ```
///
/// ## Extracting Components
///
/// ```rust
/// # use tokit::types::Keyword;
/// # use tokit::utils::{SimpleSpan, IntoComponents};
/// # struct MyLang;
/// # let span = SimpleSpan::new(0, 3);
/// let ident = Keyword::<&str, MyLang>::new(span, "foo");
///
/// // Destructure into span and source
/// let (span, source) = ident.into_components();
/// assert_eq!(source, "foo");
/// ```
///
/// ## Mutable Access
///
/// ```rust
/// # use tokit::types::Keyword;
/// # use tokit::utils::SimpleSimpleSpan;
/// # struct MyLang;
/// # let span = SimpleSpan::new(0, 3);
/// let mut ident = Keyword::<String, MyLang>::new(span, "original".to_string());
///
/// // Update the source string
/// *ident.source_mut() = "modified".to_string();
/// assert_eq!(ident.source_ref(), "modified");
///
/// // Update the span
/// *ident.span_mut() = SimpleSpan::new(10, 18);
/// assert_eq!(ident.span(), SimpleSpan::new(10, 18));
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Keyword<S, Span = SimpleSpan, Lang = ()> {
  span: Span,
  ident: S,
  _lang: PhantomData<Lang>,
}

impl<S, Span, Lang> From<Keyword<S, Span, Lang>> for super::Ident<S, Span, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(keyword: Keyword<S, Span, Lang>) -> Self {
    Self::new(keyword.span, keyword.ident)
  }
}

impl<S, Span, Lang> AsSpan<Span> for Keyword<S, Span, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_span(&self) -> &Span {
    self.span_ref()
  }
}

impl<S, Span, Lang> IntoComponents for Keyword<S, Span, Lang> {
  type Components = (Span, S);

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_components(self) -> Self::Components {
    (self.span, self.ident)
  }
}

impl<S, Span, Lang> Keyword<S, Span, Lang> {
  /// Creates a new identifier with the given span and source string.
  ///
  /// # Parameters
  ///
  /// - `span`: The source location of this identifier
  /// - `source`: The identifier string (can be `&str`, `String`, or custom type)
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tokit::types::Keyword;
  /// use tokit::utils::SimpleSimpleSpan;
  /// # struct YulLang;
  ///
  /// let span = SimpleSpan::new(10, 15);
  /// let ident = Keyword::<&str, YulLang>::new(span, "count");
  ///
  /// assert_eq!(ident.span(), span);
  /// assert_eq!(ident.source_ref(), &"count");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: Span, source: S) -> Self {
    Self {
      span,
      ident: source,
      _lang: PhantomData,
    }
  }

  /// Returns the span (source location) of this identifier.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use tokit::types::Keyword;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let ident = Keyword::<&str, MyLang>::new(SimpleSpan::new(5, 10), "value");
  ///
  /// assert_eq!(ident.span(), SimpleSpan::new(5, 10));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> Span
  where
    Span: Copy,
  {
    self.span
  }

  /// Returns an immutable reference to the span.
  ///
  /// Use this when you need to borrow the span without copying it.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use tokit::types::Keyword;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let ident = Keyword::<&str, MyLang>::new(SimpleSpan::new(0, 3), "foo");
  ///
  /// let span_ref = ident.span_ref();
  /// assert_eq!(*span_ref, SimpleSpan::new(0, 3));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &Span {
    &self.span
  }

  /// Returns a mutable reference to the span.
  ///
  /// Use this to update the identifier's source location, for example during
  /// AST transformations or span adjustments.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use tokit::types::Keyword;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let mut ident = Keyword::<&str, MyLang>::new(SimpleSpan::new(0, 3), "foo");
  ///
  /// *ident.span_mut() = SimpleSpan::new(10, 13);
  /// assert_eq!(ident.span(), SimpleSpan::new(10, 13));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut Span {
    &mut self.span
  }

  /// Returns a mutable reference to the source string.
  ///
  /// Use this to modify the identifier's text, for example during AST
  /// transformations or name mangling.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use tokit::types::Keyword;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let mut ident = Keyword::<String, MyLang>::new(SimpleSpan::new(0, 3), "foo".to_string());
  ///
  /// *ident.source_mut() = "bar".to_string();
  /// assert_eq!(ident.source_ref(), "bar");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn source_mut(&mut self) -> &mut S {
    &mut self.ident
  }

  /// Returns an immutable reference to the source string.
  ///
  /// This is the most common way to access the identifier's text without
  /// consuming or copying it.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use tokit::types::Keyword;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let ident = Keyword::<&str, MyLang>::new(SimpleSpan::new(0, 8), "variable");
  ///
  /// assert_eq!(ident.source_ref(), &"variable");
  /// assert_eq!(ident.source_ref().len(), 8);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn source_ref(&self) -> &S {
    &self.ident
  }

  /// Returns a copy of the source string by value.
  ///
  /// This method is only available when the source type `S` implements [`Copy`].
  /// Useful for types like `&str` or interned string handles.
  ///
  /// For owned types like `String`, use [`source_ref`](Self::source_ref) to
  /// avoid cloning, or consume the identifier with
  /// [`into_components`](crate::utils::IntoComponents::into_components).
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use tokit::types::Keyword;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let ident = Keyword::<&str, MyLang>::new(SimpleSpan::new(0, 2), "id");
  ///
  /// let source: &str = ident.source(); // Copy
  /// assert_eq!(source, "id");
  /// // ident is still usable
  /// assert_eq!(ident.source_ref(), &"id");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn source(&self) -> S
  where
    S: Copy,
  {
    self.ident
  }

  /// Consumes the identifier and returns the span and source string.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (Span, S) {
    (self.span, self.ident)
  }

  /// Maps the source string to a new type, preserving the span and language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map<U>(self, f: impl FnOnce(S) -> U) -> Keyword<U, Span, Lang> {
    Keyword::new(self.span, f(self.ident))
  }
}

impl<S, Span, Lang> ErrorNode<Span> for Keyword<S, Span, Lang>
where
  S: ErrorNode<Span>,
  Span: Clone,
{
  /// Creates a placeholder identifier for **malformed content**.
  ///
  /// Used during error recovery when the parser encounters invalid identifier
  /// syntax. The source string `S` will also be created as an error placeholder.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::types::Keyword;
  /// use tokit::error::ErrorNode;
  ///
  /// // Parser found "123abc" where an identifier was expected
  /// let bad_ident = Keyword::<String, YulLang>::error(span);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn error(span: Span) -> Self {
    Self::new(span.clone(), S::error(span))
  }

  /// Creates a placeholder identifier for **missing required content**.
  ///
  /// Used during error recovery when the parser expects an identifier but
  /// finds nothing at all. The source string `S` will also be created as
  /// a missing placeholder.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::types::Keyword;
  /// use tokit::error::ErrorNode;
  ///
  /// // Parser expected identifier after "let" but found "="
  /// // Correct: let name = 5;
  /// // Found:   let = 5;
  /// let missing_ident = Keyword::<String, YulLang>::missing(span);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn missing(span: Span) -> Self {
    Self::new(span.clone(), S::missing(span))
  }
}

// #[cfg(feature = "chumsky")]
// #[cfg_attr(docsrs, doc(cfg(feature = "chumsky")))]
// const _: () = {
//   use chumsky::{Parser, extra::ParserExtra, prelude::*};
//   use logos::{Logos, Source};

//   use crate::{
//     KeywordToken, Lexed, LogoStream, error::UnexpectedToken, syntax::Language, utils::SimpleSimpleSpanned,
//   };

//   impl<S, Lang> Keyword<S, Lang> {
//     /// Creates a Chumsky parser that parses identifier tokens into `Keyword`.
//     ///
//     /// This parser validates that the token is an identifier (not a keyword or other
//     /// token type) and converts it to an `Keyword` with proper span tracking.
//     ///
//     /// # Type Parameters
//     ///
//     /// - `'a`: Lifetime of the input source
//     /// - `I`: Token stream implementing [`LogoStream`]
//     /// - `T`: Token type implementing [`KeywordifierToken`]
//     /// - `Error`: Error type that can be constructed from lexer and parser errors
//     /// - `E`: Parser extra state carrying errors and metadata
//     ///
//     /// # Parameters
//     ///
//     /// - `ident_kind`: Function that returns the expected syntax kind for error
//     ///   reporting. Called when a non-identifier token is found.
//     ///
//     /// # Returns
//     ///
//     /// A Chumsky parser that produces `Keyword<S, Lang>` on success or emits an
//     /// [`UnexpectedToken`] error when a non-identifier is found.
//     ///
//     /// # Error Behavior
//     ///
//     /// The parser fails with an error in these cases:
//     /// - Token is not an identifier (e.g., keyword, operator, literal)
//     /// - Lexer error occurred while scanning the token
//     ///
//     /// # Examples
//     ///
//     /// ## Basic Usage
//     ///
//     /// ```rust,ignore
//     /// use tokit::types::Keyword;
//     /// use tokit::chumsky::Parser;
//     ///
//     /// // Parser for YUL keywords
//     /// let ident_parser = Keyword::<&str, YulLang>::parser(|| YulSyntaxKind::Keyword);
//     ///
//     /// // Parse "count" into Keyword
//     /// let result = ident_parser.parse(stream)?;
//     /// assert_eq!(result.source_ref(), &"count");
//     /// ```
//     ///
//     /// ## With Error Recovery
//     ///
//     /// ```rust,ignore
//     /// use tokit::types::Keyword;
//     /// use tokit::error::ErrorNode;
//     /// use tokit::chumsky::{Parser, prelude::*};
//     ///
//     /// // Parser with recovery for missing keywords
//     /// let ident_parser = Keyword::<String, YulLang>::parser(|| YulSyntaxKind::Keyword)
//     ///     .recover_with(via_parser(
//     ///         // Create placeholder on error
//     ///         empty().map_with(|_, exa| Keyword::missing(exa.span()))
//     ///     ));
//     ///
//     /// // Even with missing identifier, parsing continues
//     /// let result = ident_parser.parse(stream)?;
//     /// ```
//     ///
//     /// ## Custom String Type
//     ///
//     /// ```rust,ignore
//     /// // Use owned String for keywords
//     /// let parser = Keyword::<String, MyLang>::parser(|| MyKind::Keywordifier);
//     ///
//     /// // Use interned strings
//     /// let parser = Keyword::<Symbol, MyLang>::parser(|| MyKind::Keywordifier);
//     /// ```
//     ///
//     /// # See Also
//     ///
//     /// - [`KeywordToken`]: Trait for tokens that can be keywords
//     /// - [`UnexpectedToken`]: Error emitted when wrong token type is found
//     /// - [`ErrorNode`]: For creating placeholder keywords during recovery
//     #[cfg_attr(not(tarpaulin), inline(always))]
//     pub fn parser<'a, I, T, E>(
//       keyword_kind: impl Fn() -> Lang::SyntaxKind + Clone + 'a,
//     ) -> impl Parser<'a, I, Self, E> + Clone + 'a
//     where
//       I: LogoStream<'a, T>,
//       T: KeywordToken<'a>,
//       S: From<<<T::Logos as Logos<'a>>::Source as Source>::Slice<'a>> + 'a,
//       Lang: Language,
//       Lang::SyntaxKind: 'a,
//       E::Error: From<<T::Logos as Logos<'a>>::Error>
//         + From<<T::Logos as Logos<'a>>::Error>
//         + From<UnexpectedToken<'a, T, Lang::SyntaxKind>>,
//       E: ParserExtra<'a, I> + 'a,
//     {
//       any().try_map_with(move |tok: Lexed<'_, T>, exa| match tok {
//         Lexed::Token(SimpleSpanned { span, data: tok }) => match tok.is_keyword() {
//           true => Ok(Self::new(span, S::from(exa.slice()))),
//           false => Err(UnexpectedToken::expected_one_with_found(span, tok, keyword_kind()).into()),
//         },
//         Lexed::Error(e) => Err(E::Error::from(e)),
//       })
//     }
//   }
// };
