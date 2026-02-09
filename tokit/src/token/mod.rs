#[cfg(feature = "logos")]
#[cfg_attr(docsrs, doc(cfg(feature = "logos")))]
pub use logos::Logos;

pub use lit::*;
pub use punct::*;

mod lit;
mod punct;

/// The core trait for token types used with Tokit.
///
/// `Token` defines the interface that all token types must implement to work with
/// Tokit's [`Lexer`](crate::Lexer) trait. It bridges the gap between lexical analysis and the
/// structured token representation needed for parsing.
///
/// # Design
///
/// The `Token` trait separates the Logos enum (the raw lexer output) from the structured
/// token type that's used in parsing. This separation allows you to:
///
/// - Add custom data or behavior to tokens beyond what Logos provides
/// - Normalize different Logos variants into a unified token type
/// - Implement additional traits and methods specific to your language
/// - Keep parsing logic separate from lexing logic
///
/// # Required Associated Types
///
/// - **`Kind`**: An enum representing token categories (e.g., `Identifier`, `Number`, `Plus`)
/// - **`Error`**: The error type produced by the lexer for invalid tokens
///
/// # Required Traits
///
/// Implementors must also implement:
/// - `Clone`: Tokens need to be cloneable for backtracking in parsers
/// - `Debug`: For debugging and error messages
///
/// ## Examples
///
/// ## Basic Implementation
///
/// ```rust,ignore
/// use tokit::Token;
/// use logos::Logos;
///
/// // The Logos enum (raw lexer output)
/// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
/// enum MyTokens {
///     #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
///     Identifier,
///
///     #[regex(r"[0-9]+")]
///     Number,
///
///     #[token("+")]
///     Plus,
/// }
///
/// // Token kinds (semantic categories)
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum TokenKind {
///     Identifier,
///     Number,
///     Plus,
/// }
///
/// // The structured token type
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyToken {
///     kind: TokenKind,
///     // You can add extra fields here
///     // text: String,
///     // value: Option<i64>,
/// }
///
/// impl Token<'_> for MyToken {
///     type Char = char;
///     type Kind = TokenKind;
///     type Logos = MyTokens;
///
///     fn kind(&self) -> Self::Kind {
///         self.kind
///     }
/// }
///
/// impl From<MyTokens> for MyToken {
///     fn from(logos: MyTokens) -> Self {
///         let kind = match logos {
///             MyTokens::Identifier => TokenKind::Identifier,
///             MyTokens::Number => TokenKind::Number,
///             MyTokens::Plus => TokenKind::Plus,
///         };
///         MyToken { kind }
///     }
/// }
/// ```
///
/// ## Advanced: Storing Token Data
///
/// ```rust,ignore
/// // Token that stores the matched text
/// #[derive(Debug, Clone, PartialEq)]
/// struct RichToken<'a> {
///     kind: TokenKind,
///     text: &'a str,
/// }
///
/// impl<'a> Token<'a> for RichToken<'a> {
///     type Char = char;
///     type Kind = TokenKind;
///     type Logos = MyTokens;
///
///     fn kind(&self) -> Self::Kind {
///         self.kind
///     }
/// }
///
/// // Note: From<Logos> implementation would need access to the lexer
/// // to get the matched text, which typically happens in the Tokenizer
/// ```
///
/// ## Working with Bytes
///
/// ```rust,ignore
/// use tokit::Token;
/// use logos::Logos;
///
/// #[derive(Logos, Debug, Clone, Copy)]
/// #[logos(source = [u8])]
/// enum ByteTokens {
///     #[regex(br"[0-9]+")]
///     Number,
/// }
///
/// #[derive(Debug, Clone)]
/// struct ByteToken {
///     kind: ByteTokenKind,
/// }
///
/// impl Token<'_> for ByteToken {
///     type Char = u8;  // Using u8 for byte-based lexing
///     type Kind = ByteTokenKind;
///     type Logos = ByteTokens;
///
///     fn kind(&self) -> Self::Kind {
///         self.kind
///     }
/// }
/// ```
pub trait Token<'a>: Clone + core::fmt::Debug + 'a {
  /// The token kind discriminant used to categorize tokens.
  ///
  /// This is typically an enum that represents the semantic category of each token
  /// (e.g., `Identifier`, `Number`, `Operator`). It's separate from the Logos enum
  /// to allow for additional processing or normalization.
  ///
  /// # Requirements
  ///
  /// - Must be `Copy` for efficient passing
  /// - Must be `Debug` for error messages
  /// - Must be `PartialEq` and `Eq` for comparisons in parsers
  /// - Must be `Hash` for use in hash-based collections
  type Kind: Copy + core::fmt::Debug + core::fmt::Display + PartialEq + Eq + core::hash::Hash;

  /// The error type of this token.
  type Error: Clone + core::fmt::Debug;

  /// Returns the kind (category) of this token.
  ///
  /// This method is used extensively by parsers to determine what kind of token
  /// they're looking at without having to inspect the full token structure.
  ///
  /// ## Example
  ///
  /// ```rust,ignore
  /// let token = MyToken::from(logos_token);
  /// match token.kind() {
  ///     TokenKind::Identifier => handle_identifier(token),
  ///     TokenKind::Number => handle_number(token),
  ///     _ => handle_other(token),
  /// }
  /// ```
  fn kind(&self) -> Self::Kind;

  /// Returns `true` if this token represents trivia (whitespace, comments, etc.).
  ///
  /// Trivia tokens are lexical elements that don't affect the semantic meaning of code
  /// but are important for formatting, documentation, and code presentation.
  ///
  /// # Common Trivia Types
  ///
  /// - Whitespace: spaces, tabs, newlines, carriage returns
  /// - Comments: line comments, block comments, documentation comments
  /// - Language-specific formatting tokens
  ///
  /// ## Example
  ///
  /// ```rust
  /// use tokit::Token;
  /// use core::fmt;
  ///
  /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  /// enum TokenKind {
  ///     Whitespace,
  ///     Comment,
  ///     Number,
  /// }
  ///
  /// impl fmt::Display for TokenKind {
  ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  ///         let name = match self {
  ///             Self::Whitespace => "whitespace",
  ///             Self::Comment => "comment",
  ///             Self::Number => "number",
  ///         };
  ///         f.write_str(name)
  ///     }
  /// }
  ///
  /// #[derive(Debug, Clone, PartialEq)]
  /// struct MyToken {
  ///     kind: TokenKind,
  /// }
  ///
  /// impl Token<'_> for MyToken {
  ///     type Kind = TokenKind;
  ///     type Error = ();
  ///
  ///     fn kind(&self) -> Self::Kind {
  ///         self.kind
  ///     }
  ///
  ///     fn is_trivia(&self) -> bool {
  ///         matches!(self.kind, TokenKind::Whitespace | TokenKind::Comment)
  ///     }
  /// }
  ///
  /// let token = MyToken { kind: TokenKind::Whitespace };
  /// assert!(token.is_trivia());
  /// ```
  fn is_trivia(&self) -> bool;
}

impl<'a, T: Token<'a>> Token<'a> for &'a T {
  type Kind = T::Kind;
  type Error = T::Error;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn kind(&self) -> Self::Kind {
    (*self).kind()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_trivia(&self) -> bool {
    (*self).is_trivia()
  }
}

/// A trait for tokens that carry user-defined identifiers.
///
/// [`IdentifierToken`] focuses exclusively on identifier storage/matching. Keyword-related helpers live
/// in [`KeywordToken`], letting languages opt into only the capabilities they need.
///
/// ## Example
///
/// ```rust
/// use tokit::{Token, token::IdentifierToken};
/// use core::fmt;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum MyTokenKind {
///     Identifier,
///     KeywordIf,
///     KeywordElse,
/// }
///
/// impl fmt::Display for MyTokenKind {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         let name = match self {
///             Self::Identifier => "identifier",
///             Self::KeywordIf => "if",
///             Self::KeywordElse => "else",
///         };
///         f.write_str(name)
///     }
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// enum MyToken<'a> {
///     Identifier(&'a str),
///     If,
///     Else,
/// }
///
/// impl<'a> Token<'a> for MyToken<'a> {
///     type Kind = MyTokenKind;
///     type Error = ();
///
///     fn kind(&self) -> Self::Kind {
///         match self {
///            MyToken::Identifier(_) => MyTokenKind::Identifier,
///            MyToken::If => MyTokenKind::KeywordIf,
///            MyToken::Else => MyTokenKind::KeywordElse,
///         }
///     }
///
///     fn is_trivia(&self) -> bool {
///         false
///     }
/// }
///
/// impl<'a> IdentifierToken<'a> for MyToken<'a> {
///     fn is_identifier(&self) -> bool {
///         matches!(self, MyToken::Identifier(_))
///     }
/// }
///
/// let token = MyToken::Identifier("name");
/// assert!(token.is_identifier());
/// ```
pub trait IdentifierToken<'a>: Token<'a> {
  /// Returns `true` when the token is an identifier (user-defined name).
  fn is_identifier(&self) -> bool;
}

impl<'a, T: IdentifierToken<'a>> IdentifierToken<'a> for &'a T {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_identifier(&self) -> bool {
    T::is_identifier(self)
  }
}

/// A trait for tokens that represent keywords.
///
/// ## Example
///
/// ```rust
/// use tokit::{Token, token::KeywordToken};
/// use core::fmt;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum MyTokenKind {
///     Identifier,
///     KeywordIf,
///     KeywordElse,
/// }
///
/// impl fmt::Display for MyTokenKind {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         let name = match self {
///             Self::Identifier => "identifier",
///             Self::KeywordIf => "if",
///             Self::KeywordElse => "else",
///         };
///         f.write_str(name)
///     }
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// enum MyToken<'a> {
///     Identifier(&'a str),
///     If,
///     Else,
/// }
///
/// impl<'a> Token<'a> for MyToken<'a> {
///     type Kind = MyTokenKind;
///     type Error = ();
///
///     fn kind(&self) -> Self::Kind {
///         match self {
///            MyToken::Identifier(_) => MyTokenKind::Identifier,
///            MyToken::If => MyTokenKind::KeywordIf,
///            MyToken::Else => MyTokenKind::KeywordElse,
///         }
///     }
///
///     fn is_trivia(&self) -> bool {
///         false
///     }
/// }
///
/// impl<'a> KeywordToken<'a> for MyToken<'a> {
///     fn keyword(&self) -> Option<&'static str> {
///         match self.kind() {
///             MyTokenKind::KeywordIf => Some("if"),
///             MyTokenKind::KeywordElse => Some("else"),
///             _ => None,
///         }
///     }
/// }
///
/// let token = MyToken::If;
/// assert!(token.is_keyword());
/// assert_eq!(token.keyword(), Some("if"));
/// ```
pub trait KeywordToken<'a>: Token<'a> {
  /// Returns `true` when the token is any reserved keyword.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_keyword(&self) -> bool {
    self.keyword().is_some()
  }

  /// Returns the canonical spelling of the keyword, if this token represents one.
  fn keyword(&self) -> Option<&'static str>;
}

impl<'a, T: KeywordToken<'a>> KeywordToken<'a> for &'a T {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_keyword(&self) -> bool {
    T::is_keyword(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn keyword(&self) -> Option<&'static str> {
    T::keyword(self)
  }
}

/// A trait for tokens that classify operators (arithmetic, logical, comparison, assignment, etc.).
///
/// [`OperatorToken`] complements [`Token`] by centralizing operator knowledge so parsers and tooling
/// can branch on semantic operator categories without matching on the underlying token kind. This
/// covers both single-character and multi-character operators such as `+=`, `>>=`, `&&`, or `=>`.
///
/// # Usage
///
/// - Aggregation helpers (`is`, `is_math`, `is_assignment`, etc.) combine
///   the more granular predicates.
/// - Every predicate **returns `false` by default**; override only what your language emits.
/// - Consider implementing these methods alongside [`PunctuatorToken`] so punctuation and operator
///   classification stay in sync.
///
/// # Covered Operator Families
///
/// - Arithmetic: `is_math`, `is_plus`, `is_minus`, `is_increment`, etc.
/// - Assignment: `is_assignment`, `is_eq_assign`, `is_colon_eq_assign`,
///   `is_add_assign`, `is_shl_assign`, etc.
/// - Logical: `is_logical`, `is_logical_and`, `is_logical_or`, `is_logical_xor`, `is_logical_not`
/// - Comparison: `is_comparison`, `is_eq`, `is_strict_eq`, `is_ne`, `is_strict_ne`, `is_le`, etc.
/// - Shift / bitwise: `is_shift`, `is_shl`, `is_shr`, plus bitwise forms
/// - Miscellaneous: `pow`, `is_arrow`, `is_fat_arrow`, `is_backslash`
///
/// ## Example
///
/// ```rust
/// use tokit::{Token, token::OperatorToken};
/// use core::fmt;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum MyTokenKind {
///     Plus,
///     Increment,
///     PlusAssign,
/// }
///
/// impl fmt::Display for MyTokenKind {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         let name = match self {
///             Self::Plus => "+",
///             Self::Increment => "++",
///             Self::PlusAssign => "+=",
///         };
///         f.write_str(name)
///     }
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyToken {
///     kind: MyTokenKind,
/// }
///
/// impl Token<'_> for MyToken {
///     type Kind = MyTokenKind;
///     type Error = ();
///
///     fn kind(&self) -> Self::Kind {
///         self.kind
///     }
///
///     fn is_trivia(&self) -> bool {
///         false
///     }
/// }
///
/// impl OperatorToken<'_> for MyToken {
///     fn is_add(&self) -> bool {
///         matches!(self.kind, MyTokenKind::Plus)
///     }
///
///     fn is_increment(&self) -> bool {
///         matches!(self.kind, MyTokenKind::Increment)
///     }
///
///     fn is_add_assign(&self) -> bool {
///         matches!(self.kind, MyTokenKind::PlusAssign)
///     }
/// }
///
/// let token = MyToken { kind: MyTokenKind::PlusAssign };
/// assert!(token.is_add_assign());
/// ```
pub trait OperatorToken<'a>: Token<'a> {
  /// Returns `true` when the token is the simple assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_eq_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the colon-assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_colon_eq_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the addition operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_add(&self) -> bool {
    false
  }

  /// Returns `true` for the subtraction operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_sub(&self) -> bool {
    false
  }

  /// Returns `true` for the multiplication operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_mul(&self) -> bool {
    false
  }

  /// Returns `true` for the division operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_div(&self) -> bool {
    false
  }

  /// Returns `true` for the modulo operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_mod(&self) -> bool {
    false
  }

  /// Returns `true` for the exponentiation operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_pow(&self) -> bool {
    false
  }

  /// Returns `true` for the power-assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_pow_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the increment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_increment(&self) -> bool {
    false
  }

  /// Returns `true` for the decrement operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_decrement(&self) -> bool {
    false
  }

  /// Returns `true` for the plus-assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_add_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the minus-assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_sub_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the multiply-assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_mul_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the divide-assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_div_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the modulo-assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_mod_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the bitwise AND operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_bitand(&self) -> bool {
    false
  }

  /// Returns `true` for the bitwise OR operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_bitor(&self) -> bool {
    false
  }

  /// Returns `true` for the bitwise XOR operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_bitxor(&self) -> bool {
    false
  }

  /// Returns `true` for the bitwise AND assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_bitand_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the bitwise OR assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_bitor_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the bitwise XOR assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_bitxor_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the logical XOR operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_logical_xor(&self) -> bool {
    false
  }

  /// Returns `true` for the logical AND operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_logical_and(&self) -> bool {
    false
  }

  /// Returns `true` for the logical OR operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_logical_or(&self) -> bool {
    false
  }

  /// Returns `true` for the logical NOT operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_logical_not(&self) -> bool {
    false
  }

  /// Returns `true` for the equality comparison operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_eq(&self) -> bool {
    false
  }

  /// Returns `true` for strict equality operators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_strict_eq(&self) -> bool {
    false
  }

  /// Returns `true` for inequality operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ne(&self) -> bool {
    false
  }

  /// Returns `true` for strict inequality operators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_strict_ne(&self) -> bool {
    false
  }

  /// Returns `true` for the less-than operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_lt(&self) -> bool {
    false
  }

  /// Returns `true` for the strict less-than operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_strict_lt(&self) -> bool {
    false
  }

  /// Returns `true` for the less-than-or-equal operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_le(&self) -> bool {
    false
  }

  /// Returns `true` for the strict less-than-or-equal operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_strict_le(&self) -> bool {
    false
  }

  /// Returns `true` for the greater-than operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_gt(&self) -> bool {
    false
  }

  /// Returns `true` for the strict greater-than operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_strict_gt(&self) -> bool {
    false
  }

  /// Returns `true` for the greater-than-or-equal operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ge(&self) -> bool {
    false
  }

  /// Returns `true` for the strict greater-than-or-equal operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_strict_ge(&self) -> bool {
    false
  }

  /// Returns `true` for the left-shift operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_shl(&self) -> bool {
    false
  }

  /// Returns `true` for the right-shift operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_shr(&self) -> bool {
    false
  }

  /// Returns `true` for the SAR operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_sar(&self) -> bool {
    false
  }

  /// Returns `true` for the left-shift assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_shl_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the right-shift assignment operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_shr_assign(&self) -> bool {
    false
  }

  /// Returns `true` for the arrow operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_arrow(&self) -> bool {
    false
  }

  /// Returns `true` for the fat-arrow operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_fat_arrow(&self) -> bool {
    false
  }

  /// Returns `true` for the double-colon operator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_double_colon(&self) -> bool {
    false
  }

  /// Returns `true` for backslash-assignment operators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_backslash_assign(&self) -> bool {
    false
  }
}
