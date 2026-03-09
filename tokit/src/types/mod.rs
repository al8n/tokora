//! Common types for building language-specific ASTs.
//!
//! This module provides generic, reusable building blocks for creating Abstract
//! Syntax Trees (ASTs) across different programming languages. These types are
//! designed to be language-agnostic while maintaining type safety through generic
//! parameters.
//!
//! # Available Types
//!
//! ## Identifiers
//!
//! - `Ident`: Generic identifier with span tracking and language marker
//!
//! ## Generic Literal
//!
//! - `Lit`: Generic literal (any literal type)
//!
//! ## Numeric Literals
//!
//! - `LitDecimal`: Decimal integer (e.g., `42`, `1_000`)
//! - `LitHex`: Hexadecimal integer (e.g., `0xFF`, `0x1A2B`)
//! - `LitOctal`: Octal integer (e.g., `0o77`, `0o644`)
//! - `LitBinary`: Binary integer (e.g., `0b1010`)
//! - `LitFloat`: Floating-point (e.g., `3.14`, `1.0e-5`)
//! - `LitHexFloat`: Hexadecimal float (e.g., `0x1.8p3`)
//!
//! ## String Literals
//!
//! - `LitString`: Single-line string (e.g., `"hello"`)
//! - `LitMultilineString`: Multi-line string (e.g., `"""..."""`)
//! - `LitRawString`: Raw string (e.g., `r"C:\path"`)
//!
//! ## Character/Byte Literals
//!
//! - `LitChar`: Character literal (e.g., `'a'`)
//! - `LitByte`: Byte literal (e.g., `b'a'`)
//! - `LitByteString`: Byte string (e.g., `b"bytes"`)
//!
//! ## Boolean and Null
//!
//! - `LitBool`: Boolean literal (`true`/`false`)
//! - `LitNull`: Null/nil literal
//!
//! # Design Principles
//!
//! All types in this module follow these principles:
//!
//! 1. **Generic over string representation**: Support zero-copy (`&str`), owned
//!    (`String`), and interned strings
//! 2. **Span tracking**: Every node carries its source location for diagnostics
//! 3. **Language safety**: Generic `Lang` parameter prevents mixing ASTs from
//!    different languages
//! 4. **Error recovery**: Implement [`ErrorNode`](crate::error::ErrorNode) when
//!    appropriate for creating placeholders during parsing errors
//!
//! # Example: Building a Simple Expression AST
//!
//! ```rust,ignore
//! use tokit::types::Ident;
//! use tokit::utils::SimpleSpan;
//!
//! // Define your language marker
//! struct MyLang;
//!
//! // Define expression type using Ident
//! enum Expr<'a> {
//!     Variable(Ident<&'a str, MyLang>),
//!     Number(i64, SimpleSpan),
//!     Add(Box<Expr<'a>>, Box<Expr<'a>>, SimpleSpan),
//! }
//!
//! // Create an expression
//! let var = Ident::new(SimpleSpan::new(0, 1), "x");
//! let expr = Expr::Variable(var);
//! ```

use derive_more::{IsVariant, TryUnwrap, Unwrap};

use crate::{
  error::ErrorNode,
  span::{AsSpan, SimpleSpan},
  syntax::{Language, Syntax},
};

pub use ident::*;
pub use ident_list::*;
pub use keyword::*;
pub use lit::*;

mod ident;
mod ident_list;
mod keyword;
mod lit;

/// A type representing a recoverable parse node, which can be a valid node,
/// an error node with span, or a missing node with span.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, IsVariant, TryUnwrap, Unwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Recoverable<T, S = SimpleSpan> {
  /// A valid parse node.
  Node(T),
  /// An error node with associated span.
  Error(S),
  /// A missing node with associated span.
  Missing(S),
}

impl<T, S> AsSpan<S> for Recoverable<T, S>
where
  T: AsSpan<S>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_span(&self) -> &S {
    match self {
      Self::Node(node) => node.as_span(),
      Self::Error(span) | Self::Missing(span) => span,
    }
  }
}

impl<T, S> Syntax for Recoverable<T, S>
where
  T: Syntax,
{
  type Lang = T::Lang;
  const KIND: <Self::Lang as Language>::SyntaxKind = T::KIND;

  type Component = T::Component;

  type COMPONENTS = T::COMPONENTS;

  type REQUIRED = T::REQUIRED;

  fn possible_components()
  -> &'static generic_arraydeque::GenericArrayDeque<Self::Component, Self::COMPONENTS> {
    T::possible_components()
  }

  fn required_components()
  -> &'static generic_arraydeque::GenericArrayDeque<Self::Component, Self::REQUIRED> {
    T::required_components()
  }
}

impl<T> ErrorNode for Recoverable<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn error(span: SimpleSpan) -> Self {
    Self::Error(span)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn missing(span: SimpleSpan) -> Self {
    Self::Missing(span)
  }
}

impl<T> From<T> for Recoverable<T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(node: T) -> Self {
    Self::Node(node)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // --- Recoverable tests ---

  #[test]
  fn recoverable_node() {
    let r = Recoverable::<i32>::Node(42);
    assert!(r.is_node());
    assert!(!r.is_error());
    assert!(!r.is_missing());
  }

  #[test]
  fn recoverable_error() {
    let r = Recoverable::<i32>::Error(SimpleSpan::new(0, 5));
    assert!(!r.is_node());
    assert!(r.is_error());
    assert!(!r.is_missing());
  }

  #[test]
  fn recoverable_missing() {
    let r = Recoverable::<i32>::Missing(SimpleSpan::new(0, 5));
    assert!(!r.is_node());
    assert!(!r.is_error());
    assert!(r.is_missing());
  }

  #[test]
  fn recoverable_from_value() {
    let r: Recoverable<i32> = 42.into();
    assert!(r.is_node());
    assert_eq!(r.try_unwrap_node(), Ok(42));
  }

  #[test]
  fn recoverable_error_node_impl() {
    let err = Recoverable::<i32>::error(SimpleSpan::new(0, 5));
    assert!(err.is_error());

    let missing = Recoverable::<i32>::missing(SimpleSpan::new(0, 5));
    assert!(missing.is_missing());
  }

  // --- Ident tests ---

  #[test]
  fn ident_new_and_accessors() {
    struct MyLang;
    let ident = Ident::<&str, SimpleSpan, MyLang>::new(SimpleSpan::new(0, 3), "foo");
    assert_eq!(ident.span(), SimpleSpan::new(0, 3));
    assert_eq!(ident.source(), "foo");
    assert_eq!(ident.source_ref(), &"foo");
    assert!(ident.is_valid());
    assert!(!ident.is_error());
    assert!(!ident.is_missing());
  }

  #[test]
  fn ident_span_mut() {
    let mut ident = Ident::<&str>::new(SimpleSpan::new(0, 3), "foo");
    *ident.span_mut() = SimpleSpan::new(10, 13);
    assert_eq!(ident.span(), SimpleSpan::new(10, 13));
  }

  #[test]
  fn ident_source_mut() {
    let mut ident = Ident::<String>::new(SimpleSpan::new(0, 3), "foo".to_string());
    *ident.source_mut() = "bar".to_string();
    assert_eq!(ident.source_ref(), "bar");
  }

  #[test]
  fn ident_map() {
    let ident = Ident::<&str>::new(SimpleSpan::new(0, 3), "foo");
    let mapped = ident.map(|s| s.to_uppercase());
    assert_eq!(mapped.source_ref(), "FOO");
    assert_eq!(mapped.span(), SimpleSpan::new(0, 3));
  }

  #[test]
  fn ident_into_components() {
    use crate::utils::IntoComponents;
    let ident = Ident::<&str>::new(SimpleSpan::new(0, 3), "foo");
    let (span, source) = ident.into_components();
    assert_eq!(span, SimpleSpan::new(0, 3));
    assert_eq!(source, "foo");
  }

  #[test]
  fn ident_error_node() {
    let err = Ident::<&str>::error(SimpleSpan::new(0, 5));
    assert!(err.is_error());
    assert_eq!(err.source(), "<error>");
  }

  #[test]
  fn ident_missing_node() {
    let missing = Ident::<&str>::missing(SimpleSpan::new(0, 5));
    assert!(missing.is_missing());
    assert_eq!(missing.source(), "<missing>");
  }

  // --- Keyword tests ---

  #[test]
  fn keyword_new_and_accessors() {
    let kw = Keyword::<&str>::new(SimpleSpan::new(5, 11), "return");
    assert_eq!(kw.span(), SimpleSpan::new(5, 11));
    assert_eq!(kw.source(), "return");
    assert_eq!(kw.source_ref(), &"return");
  }

  #[test]
  fn keyword_span_mut() {
    let mut kw = Keyword::<&str>::new(SimpleSpan::new(0, 3), "let");
    *kw.span_mut() = SimpleSpan::new(10, 13);
    assert_eq!(kw.span(), SimpleSpan::new(10, 13));
  }

  #[test]
  fn keyword_source_mut() {
    let mut kw = Keyword::<String>::new(SimpleSpan::new(0, 3), "let".to_string());
    *kw.source_mut() = "var".to_string();
    assert_eq!(kw.source_ref(), "var");
  }

  #[test]
  fn keyword_map() {
    let kw = Keyword::<&str>::new(SimpleSpan::new(0, 3), "let");
    let mapped = kw.map(|s| s.to_uppercase());
    assert_eq!(mapped.source_ref(), "LET");
  }

  #[test]
  fn keyword_into_components() {
    let kw = Keyword::<&str>::new(SimpleSpan::new(0, 3), "let");
    let (span, source) = kw.into_components();
    assert_eq!(span, SimpleSpan::new(0, 3));
    assert_eq!(source, "let");
  }

  #[test]
  fn keyword_into_ident() {
    let kw = Keyword::<&str>::new(SimpleSpan::new(0, 3), "let");
    let ident: Ident<&str> = kw.into();
    assert_eq!(ident.source(), "let");
    assert_eq!(ident.span(), SimpleSpan::new(0, 3));
  }

  #[test]
  fn keyword_error_node() {
    let err = Keyword::<&str>::error(SimpleSpan::new(0, 5));
    assert_eq!(err.source(), "<error>");
  }

  #[test]
  fn keyword_missing_node() {
    let missing = Keyword::<&str>::missing(SimpleSpan::new(0, 5));
    assert_eq!(missing.source(), "<missing>");
  }

  // --- Literal types tests ---

  #[test]
  fn lit_decimal_new_and_accessors() {
    let lit = LitDecimal::<&str>::new(SimpleSpan::new(0, 2), "42");
    assert_eq!(lit.span(), SimpleSpan::new(0, 2));
    assert_eq!(lit.data(), "42");
    assert_eq!(lit.data_ref(), &"42");
  }

  #[test]
  fn lit_decimal_span_mut() {
    let mut lit = LitDecimal::<&str>::new(SimpleSpan::new(0, 2), "42");
    *lit.span_mut() = SimpleSpan::new(10, 12);
    assert_eq!(lit.span(), SimpleSpan::new(10, 12));
  }

  #[test]
  fn lit_decimal_data_mut() {
    let mut lit = LitDecimal::<String>::new(SimpleSpan::new(0, 2), "42".to_string());
    *lit.data_mut() = "99".to_string();
    assert_eq!(lit.data_ref(), "99");
  }

  #[test]
  fn lit_decimal_error_node() {
    let err = LitDecimal::<&str>::error(SimpleSpan::new(0, 5));
    assert_eq!(err.data(), "<error>");
  }

  #[test]
  fn lit_decimal_missing_node() {
    let missing = LitDecimal::<&str>::missing(SimpleSpan::new(0, 5));
    assert_eq!(missing.data(), "<missing>");
  }

  #[test]
  fn lit_bool_new() {
    let lit = LitBool::<bool>::new(SimpleSpan::new(0, 4), true);
    assert_eq!(lit.data(), true);
  }

  #[test]
  fn lit_null_new() {
    let lit = LitNull::<()>::new(SimpleSpan::new(0, 4), ());
    assert_eq!(lit.span(), SimpleSpan::new(0, 4));
  }

  #[test]
  fn lit_string_new() {
    let lit = LitString::<&str>::new(SimpleSpan::new(0, 7), "\"hello\"");
    assert_eq!(lit.data(), "\"hello\"");
  }

  #[test]
  fn lit_hex_new() {
    let lit = LitHex::<&str>::new(SimpleSpan::new(0, 4), "0xFF");
    assert_eq!(lit.data(), "0xFF");
  }

  #[test]
  fn lit_into_components() {
    use crate::utils::IntoComponents;
    let lit = LitDecimal::<&str>::new(SimpleSpan::new(0, 2), "42");
    let (span, data) = IntoComponents::into_components(lit);
    assert_eq!(span, SimpleSpan::new(0, 2));
    assert_eq!(data, "42");
  }

  // --- IdentList tests ---

  #[test]
  fn ident_list_new_and_accessors() {
    let idents = vec![
      Ident::<&str>::new(SimpleSpan::new(0, 3), "foo"),
      Ident::<&str>::new(SimpleSpan::new(4, 7), "bar"),
    ];
    let list = IdentList::<&str>::new(SimpleSpan::new(0, 7), idents);
    assert_eq!(list.span(), SimpleSpan::new(0, 7));
    assert_eq!(list.identifiers_slice().len(), 2);
    assert!(!list.is_empty());
    assert!(list.is_valid());
    assert!(!list.is_error());
    assert!(!list.is_missing());
  }

  #[test]
  fn ident_list_empty() {
    let list = IdentList::<&str>::new(SimpleSpan::new(0, 0), Vec::new());
    assert!(list.is_empty());
  }

  #[test]
  fn ident_list_with_error() {
    let idents = vec![
      Ident::<&str>::new(SimpleSpan::new(0, 3), "foo"),
      Ident::<&str>::error(SimpleSpan::new(4, 7)),
    ];
    let list = IdentList::<&str>::new(SimpleSpan::new(0, 7), idents);
    assert!(!list.is_valid());
    assert!(list.is_error());
  }

  #[test]
  fn ident_list_with_missing() {
    let idents = vec![Ident::<&str>::missing(SimpleSpan::new(0, 3))];
    let list = IdentList::<&str>::new(SimpleSpan::new(0, 3), idents);
    assert!(list.is_missing());
  }
}
