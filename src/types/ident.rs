//! Identifier types for language syntax trees.
//!
//! This module provides generic identifier types that can be used across different
//! programming languages and string representations. Identifiers are fundamental
//! building blocks in most languages, representing names for variables, functions,
//! types, and other named entities.
//!
//! # Design Philosophy
//!
//! The [`Ident`] type is generic over both the source string type (`S`) and the
//! language marker (`Lang`). This design provides maximum flexibility:
//!
//! - **String type flexibility**: Use `&str` for zero-copy parsing, `String` for
//!   owned data, or custom interned string types for memory efficiency
//! - **Language safety**: The `Lang` parameter ensures identifiers from different
//!   languages don't mix accidentally
//! - **SimpleSpan tracking**: All identifiers carry their source location for diagnostics
//!
//! # Common Usage Patterns
//!
//! ## Zero-Copy Parsing
//!
//! ```rust,ignore
//! use tokit::types::Ident;
//! use tokit::utils::SimpleSimpleSpan;
//!
//! // Parse identifiers without allocating
//! type YulIdent<'a> = Ident<&'a str, YulLang>;
//!
//! let ident = YulIdent::new(SimpleSpan::new(0, 3), "foo");
//! assert_eq!(ident.source_ref(), &"foo");
//! ```
//!
//! ## Owned Identifiers
//!
//! ```rust,ignore
//! // Store identifiers in AST nodes that outlive the source
//! type OwnedIdent = Ident<String, MyLang>;
//!
//! let ident = OwnedIdent::new(span, source_str.to_string());
//! ```
//!
//! ## String Interning
//!
//! ```rust,ignore
//! // Use interned strings for memory efficiency
//! type InternedIdent = Ident<Symbol, MyLang>;
//!
//! let ident = InternedIdent::new(span, interner.intern("identifier"));
//! ```
//!
//! # Error Recovery
//!
//! `Ident` implements [`ErrorNode`] when the source type `S` also implements it,
//! allowing creation of placeholder identifiers during error recovery:
//!
//! ```rust,ignore
//! use tokit::error::ErrorNode;
//!
//! // Create placeholder for malformed identifier
//! let bad_ident = Ident::<String, YulLang>::error(span);
//!
//! // Create placeholder for missing identifier
//! let missing_ident = Ident::<String, YulLang>::missing(span);
//! ```

use core::marker::PhantomData;

use crate::{
  error::ErrorNode,
  utils::{AsSpan, IntoComponents, SimpleSpan},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
#[non_exhaustive]
enum Status {
  Valid,
  Error,
  Missing,
}

/// A language identifier with span tracking.
///
/// Identifiers are names used in source code to refer to variables, functions,
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
/// - **Large codebases**: Use interned strings to deduplicate common identifiers
///
/// ## Why Language Marker?
///
/// The `Lang` parameter prevents mixing identifiers from different languages:
/// ```rust,ignore
/// let yul_ident: Ident<&str, YulLang> = ...;
/// let sol_ident: Ident<&str, SolidityLang> = ...;
///
/// // Compile error: type mismatch
/// // let mixed = vec![yul_ident, sol_ident];
/// ```
///
/// # Examples
///
/// ## Creating Identifiers
///
/// ```rust
/// use tokit::types::Ident;
/// use tokit::utils::SimpleSimpleSpan;
/// # struct MyLang;
///
/// // Zero-copy identifier
/// let span = SimpleSpan::new(5, 11);
/// let ident = Ident::<&str, MyLang>::new(span, "my_var");
///
/// assert_eq!(ident.span(), span);
/// assert_eq!(ident.source_ref(), &"my_var");
/// ```
///
/// ## Extracting Components
///
/// ```rust
/// # use tokit::types::Ident;
/// # use tokit::utils::{SimpleSpan, IntoComponents};
/// # struct MyLang;
/// # let span = SimpleSpan::new(0, 3);
/// let ident = Ident::<&str, MyLang>::new(span, "foo");
///
/// // Destructure into span and source
/// let (span, source) = ident.into_components();
/// assert_eq!(source, "foo");
/// ```
///
/// ## Mutable Access
///
/// ```rust
/// # use tokit::types::Ident;
/// # use tokit::utils::SimpleSimpleSpan;
/// # struct MyLang;
/// # let span = SimpleSpan::new(0, 3);
/// let mut ident = Ident::<String, MyLang>::new(span, "original".to_string());
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
pub struct Ident<S, Lang> {
  span: SimpleSpan,
  ident: S,
  status: Status,
  _lang: PhantomData<Lang>,
}

impl<S, Lang> AsSpan<SimpleSpan> for Ident<S, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_span(&self) -> &SimpleSpan {
    self.span_ref()
  }
}

impl<S, Lang> IntoComponents for Ident<S, Lang> {
  type Components = (SimpleSpan, S);

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_components(self) -> Self::Components {
    (self.span, self.ident)
  }
}

impl<S, Lang> Ident<S, Lang> {
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
  /// use tokit::types::Ident;
  /// use tokit::utils::SimpleSimpleSpan;
  /// # struct YulLang;
  ///
  /// let span = SimpleSpan::new(10, 15);
  /// let ident = Ident::<&str, YulLang>::new(span, "count");
  ///
  /// assert_eq!(ident.span(), span);
  /// assert_eq!(ident.source_ref(), &"count");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: SimpleSpan, source: S) -> Self {
    Self::with_status(span, source, Status::Valid)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn with_status(span: SimpleSpan, source: S, status: Status) -> Self {
    Self {
      span,
      ident: source,
      status,
      _lang: PhantomData,
    }
  }

  /// Returns the span (source location) of this identifier.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use tokit::types::Ident;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let ident = Ident::<&str, MyLang>::new(SimpleSpan::new(5, 10), "value");
  ///
  /// assert_eq!(ident.span(), SimpleSpan::new(5, 10));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> SimpleSpan {
    self.span
  }

  /// Returns an immutable reference to the span.
  ///
  /// Use this when you need to borrow the span without copying it.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use tokit::types::Ident;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let ident = Ident::<&str, MyLang>::new(SimpleSpan::new(0, 3), "foo");
  ///
  /// let span_ref = ident.span_ref();
  /// assert_eq!(*span_ref, SimpleSpan::new(0, 3));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &SimpleSpan {
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
  /// # use tokit::types::Ident;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let mut ident = Ident::<&str, MyLang>::new(SimpleSpan::new(0, 3), "foo");
  ///
  /// *ident.span_mut() = SimpleSpan::new(10, 13);
  /// assert_eq!(ident.span(), SimpleSpan::new(10, 13));
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut SimpleSpan {
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
  /// # use tokit::types::Ident;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let mut ident = Ident::<String, MyLang>::new(SimpleSpan::new(0, 3), "foo".to_string());
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
  /// # use tokit::types::Ident;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let ident = Ident::<&str, MyLang>::new(SimpleSpan::new(0, 8), "variable");
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
  /// # use tokit::types::Ident;
  /// # use tokit::utils::SimpleSimpleSpan;
  /// # struct MyLang;
  /// let ident = Ident::<&str, MyLang>::new(SimpleSpan::new(0, 2), "id");
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

  /// Returns `true` is this identifier represents an error identifier.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_error(&self) -> bool {
    matches!(self.status, Status::Error)
  }

  /// Returns `true` is this identifier represents a missing identifier.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_missing(&self) -> bool {
    matches!(self.status, Status::Missing)
  }

  /// Returns `true` is this identifier is valid (not error or missing).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_valid(&self) -> bool {
    matches!(self.status, Status::Valid)
  }

  /// Maps the source string to a new type, preserving the span and language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map<U>(self, f: impl FnOnce(S) -> U) -> Ident<U, Lang> {
    Ident::new(self.span, f(self.ident))
  }
}

impl<S, Lang> ErrorNode for Ident<S, Lang>
where
  S: ErrorNode,
{
  /// Creates a placeholder identifier for **malformed content**.
  ///
  /// Used during error recovery when the parser encounters invalid identifier
  /// syntax. The source string `S` will also be created as an error placeholder.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use tokit::types::Ident;
  /// use tokit::error::ErrorNode;
  ///
  /// // Parser found "123abc" where an identifier was expected
  /// let bad_ident = Ident::<String, YulLang>::error(span);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn error(span: SimpleSpan) -> Self {
    Self::with_status(span, S::error(span), Status::Error)
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
  /// use tokit::types::Ident;
  /// use tokit::error::ErrorNode;
  ///
  /// // Parser expected identifier after "let" but found "="
  /// // Correct: let name = 5;
  /// // Found:   let = 5;
  /// let missing_ident = Ident::<String, YulLang>::missing(span);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn missing(span: SimpleSpan) -> Self {
    Self::with_status(span, S::missing(span), Status::Missing)
  }
}

// #[cfg(feature = "chumsky")]
// #[cfg_attr(docsrs, doc(cfg(feature = "chumsky")))]
// const _: () = {
//   use chumsky::{Parser, extra::ParserExtra, prelude::*};
//   use logos::{Logos, Source};

//   use crate::{
//     IdentifierToken, Lexed, LogoStream, error::UnexpectedToken, syntax::Language, utils::SimpleSimpleSpanned,
//   };

//   impl<S, Lang> Ident<S, Lang> {
//     /// Creates a Chumsky parser that parses identifier tokens into `Ident`.
//     ///
//     /// This parser validates that the token is an identifier (not a keyword or other
//     /// token type) and converts it to an `Ident` with proper span tracking.
//     ///
//     /// # Type Parameters
//     ///
//     /// - `'a`: Lifetime of the input source
//     /// - `I`: Token stream implementing [`LogoStream`]
//     /// - `T`: Token type implementing [`IdentifierToken`]
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
//     /// A Chumsky parser that produces `Ident<S, Lang>` on success or emits an
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
//     /// use tokit::types::Ident;
//     /// use tokit::chumsky::Parser;
//     ///
//     /// // Parser for YUL identifiers
//     /// let ident_parser = Ident::<&str, YulLang>::parser(|| YulSyntaxKind::Ident);
//     ///
//     /// // Parse "count" into Ident
//     /// let result = ident_parser.parse(stream)?;
//     /// assert_eq!(result.source_ref(), &"count");
//     /// ```
//     ///
//     /// ## With Error Recovery
//     ///
//     /// ```rust,ignore
//     /// use tokit::types::Ident;
//     /// use tokit::error::ErrorNode;
//     /// use tokit::chumsky::{Parser, prelude::*};
//     ///
//     /// // Parser with recovery for missing identifiers
//     /// let ident_parser = Ident::<String, YulLang>::parser(|| YulSyntaxKind::Ident)
//     ///     .recover_with(via_parser(
//     ///         // Create placeholder on error
//     ///         empty().map_with(|_, exa| Ident::missing(exa.span()))
//     ///     ));
//     ///
//     /// // Even with missing identifier, parsing continues
//     /// let result = ident_parser.parse(stream)?;
//     /// ```
//     ///
//     /// ## Custom String Type
//     ///
//     /// ```rust,ignore
//     /// // Use owned String for identifiers
//     /// let parser = Ident::<String, MyLang>::parser(|| MyKind::Identifier);
//     ///
//     /// // Use interned strings
//     /// let parser = Ident::<Symbol, MyLang>::parser(|| MyKind::Identifier);
//     /// ```
//     ///
//     /// # See Also
//     ///
//     /// - [`IdentifierToken`]: Trait for tokens that can be identifiers
//     /// - [`UnexpectedToken`]: Error emitted when wrong token type is found
//     /// - [`ErrorNode`]: For creating placeholder identifiers during recovery
//     #[cfg_attr(not(tarpaulin), inline(always))]
//     pub fn parser<'a, I, T, E>(
//       ident_kind: impl Fn() -> Lang::SyntaxKind + Clone + 'a,
//     ) -> impl Parser<'a, I, Self, E> + Clone + 'a
//     where
//       I: LogoStream<'a, T>,
//       T: IdentifierToken<'a>,
//       S: From<<<<T>::Logos as Logos<'a>>::Source as Source>::Slice<'a>> + 'a,
//       Lang: Language,
//       Lang::SyntaxKind: 'a,
//       E::Error: From<<T::Logos as Logos<'a>>::Error>
//         + From<<T::Logos as Logos<'a>>::Error>
//         + From<UnexpectedToken<'a, T, Lang::SyntaxKind>>,
//       E: ParserExtra<'a, I> + 'a,
//     {
//       any().try_map(move |tok: Lexed<'_, T>, _| match tok {
//         Lexed::Token(SimpleSpanned { span, data: tok }) => match tok.try_into_identifier() {
//           Ok(ident) => Ok(Ident::new(span, ident.into())),
//           Err(tok) => Err(UnexpectedToken::expected_one_with_found(span, tok, ident_kind()).into()),
//         },
//         Lexed::Error(e) => Err(E::Error::from(e)),
//       })
//     }
//   }
// };
