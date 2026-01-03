#[cfg(feature = "logos")]
#[cfg_attr(docsrs, doc(cfg(feature = "logos")))]
pub use logos::Logos;

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
/// - **`Char`**: The character type used by the lexer (typically `char` for UTF-8 or `u8` for bytes)
/// - **`Kind`**: An enum representing token categories (e.g., `Identifier`, `Number`, `Plus`)
/// - **`Logos`**: The Logos enum that this token type wraps
///
/// # Required Traits
///
/// Implementors must also implement:
/// - `Clone`: Tokens need to be cloneable for backtracking in parsers
/// - `Debug`: For debugging and error messages
/// - `From<Self::Logos>`: Convert from the raw Logos token to the structured token
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
  /// use logos::Logos;
  ///
  /// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
  /// enum MyTokens {
  ///     #[regex(r"[ \t\n]+")]
  ///     Whitespace,
  ///     #[regex(r"//[^\n]*")]
  ///     Comment,
  ///     #[regex(r"[0-9]+")]
  ///     Number,
  /// }
  ///
  /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  /// enum TokenKind {
  ///     Whitespace,
  ///     Comment,
  ///     Number,
  /// }
  ///
  /// #[derive(Debug, Clone, PartialEq)]
  /// struct MyToken {
  ///     kind: TokenKind,
  /// }
  ///
  /// impl Token<'_> for MyToken {
  ///     type Char = char;
  ///     type Kind = TokenKind;
  ///     type Logos = MyTokens;
  ///
  ///     #[inline(always)]
  ///     fn kind(&self) -> Self::Kind {
  ///         self.kind
  ///     }
  ///
  ///     #[inline(always)]
  ///     fn is_trivia(&self) -> bool {
  ///         matches!(self.kind, TokenKind::Whitespace | TokenKind::Comment)
  ///     }
  /// }
  ///
  /// impl From<MyTokens> for MyToken {
  ///     fn from(logos: MyTokens) -> Self {
  ///         let kind = match logos {
  ///             MyTokens::Whitespace => TokenKind::Whitespace,
  ///             MyTokens::Comment => TokenKind::Comment,
  ///             MyTokens::Number => TokenKind::Number,
  ///         };
  ///         MyToken { kind }
  ///     }
  /// }
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

/// A trait for tokens that can classify punctuation without pattern matching on kinds.
///
/// [`PunctuatorToken`] builds on [`Token`] to provide ergonomic helpers for recognizing
/// common punctuation lexemes. This is useful when:
///
/// - Building parsers that frequently branch on punctuation and benefit from readable predicates
/// - Writing formatter or linter passes that need to treat punctuation uniformly regardless of kind names
/// - Exposing a stable surface for downstream users so token-kind refactors do not cascade outward
///
/// # Relationship to [`Token`]
///
/// The base [`Token`] trait exposes [`Token::kind`], leaving higher-level classification to
/// consumers. [`PunctuatorToken`] moves that logic into the token type itself, so downstream
/// code can remain agnostic of `Kind` enums or their discriminants.
///
/// # Covered ASCII Punctuation
///
/// Every method maps to a single ASCII character and **returns `false` by default**—override only
/// the ones that matter for your language, mapping them to your own token kinds. The provided
/// predicates are:
///
/// - Structural: `is_open_paren` `(`, `is_close_paren` `)`, `is_open_brace` `{`, `is_close_brace` `}`,
///   `is_open_bracket` `[`, `is_close_bracket` `]`
/// - Separators: `is_comma` `,`, `is_dot` `.`, `is_colon` `:`, `is_semicolon` `;`
/// - Quote markers: `is_double_quote` `"`, `is_apostrophe` `'`, `is_backtick` `` ` ``
/// - Math / operators: `is_plus` `+`, `is_minus` `-`, `is_asterisk` `*`, `is_slash` `/`,
///   `is_backslash` `\`, `is_percent` `%`, `is_ampersand` `&`, `is_pipe` `|`, `is_caret` `^`,
///   `is_tilde` `~`, `is_underscore` `_`
/// - Comparators: `is_lt` `<`, `is_gt` `>`, `is_equal` `=`
/// - Misc punctuation: `is_exclamation` `!`, `is_question` `?`, `is_hash` `#`, `is_dollar` `$`,
///   `is_at` `@`
///
/// ## Example
///
/// ```rust
/// use tokit::{Token, PunctuatorToken, utils::cmp::Equivalent};
/// use logos::Logos;
///
/// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
/// enum MyTokens {
///     #[token(".")]
///     Dot,
///     #[token(",")]
///     Comma,
///     #[token(":")]
///     Colon,
///     #[token(";")]
///     SemiColon,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum MyTokenKind {
///     Dot,
///     Comma,
///     Colon,
///     SemiColon,
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyToken {
///     kind: MyTokenKind,
/// }
///
/// impl Token<'_> for MyToken {
///     type Char = char;
///     type Kind = MyTokenKind;
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
///             MyTokens::Dot => MyTokenKind::Dot,
///             MyTokens::Comma => MyTokenKind::Comma,
///             MyTokens::Colon => MyTokenKind::Colon,
///             MyTokens::SemiColon => MyTokenKind::SemiColon,
///         };
///         Self { kind }
///     }
/// }
///
/// impl Equivalent<MyToken> for str {
///     fn equivalent(&self, other: &MyToken) -> bool {
///         match other.kind {
///             MyTokenKind::Dot => self == ".",
///             MyTokenKind::Comma => self == ",",
///             MyTokenKind::Colon => self == ":",
///             MyTokenKind::SemiColon => self == ";",
///         }
///     }
/// }
///
/// impl PunctuatorToken<'_> for MyToken {
///     fn is_dot(&self) -> bool {
///         matches!(self.kind, MyTokenKind::Dot)
///     }
///
///     fn is_comma(&self) -> bool {
///         matches!(self.kind, MyTokenKind::Comma)
///     }
///
///     fn is_colon(&self) -> bool {
///         matches!(self.kind, MyTokenKind::Colon)
///     }
///
///     fn is_semicolon(&self) -> bool {
///         matches!(self.kind, MyTokenKind::SemiColon)
///     }
///
///     // Unhandled punctuation can keep the default `false`.
/// }
/// ```
pub trait PunctuatorToken<'a>: Token<'a> {
  /// Returns `true` when the token is a punctuator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_punctuator(&self) -> bool {
    self.is_dot()
      || self.is_comma()
      || self.is_colon()
      || self.is_semicolon()
      || self.is_exclamation()
      || self.is_double_quote()
      || self.is_apostrophe()
      || self.is_hash()
      || self.is_dollar()
      || self.is_percent()
      || self.is_ampersand()
      || self.is_asterisk()
      || self.is_plus()
      || self.is_minus()
      || self.is_slash()
      || self.is_backslash()
      || self.is_open_angle()
      || self.is_equal()
      || self.is_close_angle()
      || self.is_question()
      || self.is_at()
      || self.is_open_bracket()
      || self.is_close_bracket()
      || self.is_open_brace()
      || self.is_close_brace()
      || self.is_open_paren()
      || self.is_close_paren()
      || self.is_backtick()
      || self.is_pipe()
      || self.is_caret()
      || self.is_underscore()
      || self.is_tilde()
      || self.is_space()
      || self.is_tab()
      || self.is_newline()
      || self.is_carriage_return()
      || self.is_crlf()
  }

  /// Returns `true` when the token is the dot punctuator (`.`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_dot(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the space punctuator (` `).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_space(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a tab punctuator (`\t`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_tab(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the newline punctuator (`\n`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_newline(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the carriage return punctuator (`\r`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_carriage_return(&self) -> bool {
    false
  }

  /// Returns `true` when the token is carriage return + newline punctuator (`\r\n`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_crlf(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the comma punctuator (`,`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_comma(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the colon punctuator (`:`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_colon(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the semicolon punctuator (`;`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_semicolon(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the exclamation punctuator (`!`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_exclamation(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the double-quote punctuator (`"`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_double_quote(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the apostrophe/single-quote punctuator (`'`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_apostrophe(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the hash punctuator (`#`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_hash(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the dollar punctuator (`$`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_dollar(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the percent punctuator (`%`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_percent(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the ampersand punctuator (`&`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_ampersand(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the asterisk punctuator (`*`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_asterisk(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the plus punctuator (`+`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_plus(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the minus punctuator (`-`).
  #[doc(alias = "is_dash")]
  #[doc(alias = "is_hyphen")]
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_minus(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the slash punctuator (`/`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_slash(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the backslash punctuator (`\`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_backslash(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the angle open punctuator (`<`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_angle(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the equal punctuator (`=`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_equal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the angle close punctuator (`>`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_angle(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the question punctuator (`?`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_question(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the at-sign punctuator (`@`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_at(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the bracket-open punctuator (`[`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_bracket(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the bracket-close punctuator (`]`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_bracket(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the brace-open punctuator (`{`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_brace(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the brace-close punctuator (`}`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_brace(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the paren-open punctuator (`(`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_paren(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the paren-close punctuator (`)`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_paren(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the backtick punctuator (`` ` ``).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_backtick(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the pipe punctuator (`|`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_pipe(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the caret punctuator (`^`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_caret(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the underscore punctuator (`_`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_underscore(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the tilde punctuator (`~`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_tilde(&self) -> bool {
    false
  }
}

impl<'a, T> DelimiterToken<'a> for T
where
  T: PunctuatorToken<'a>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_paren(&self) -> bool {
    PunctuatorToken::is_open_paren(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_paren(&self) -> bool {
    PunctuatorToken::is_close_paren(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_brace(&self) -> bool {
    PunctuatorToken::is_open_brace(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_brace(&self) -> bool {
    PunctuatorToken::is_close_brace(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_bracket(&self) -> bool {
    PunctuatorToken::is_open_bracket(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_bracket(&self) -> bool {
    PunctuatorToken::is_close_bracket(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_angle(&self) -> bool {
    PunctuatorToken::is_open_angle(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_angle(&self) -> bool {
    PunctuatorToken::is_close_angle(self)
  }
}

/// A trait for tokens that can classify literal tokens without exposing internal kinds.
///
/// [`LitToken`] augments [`Token`] with convenience predicates for common literal categories
/// (numbers, strings, booleans, etc.). This lets downstream code work with semantic literals
/// without matching on the token-kind enum directly.
///
/// # Usage
///
/// Every method **returns `false` by default**. Implementors override whichever literal kinds
/// their language supports, forwarding the checks to `self.kind()` or other internal data.
///
/// # Covered Literal Categories
///
/// - Numbers: `is_integer_literal`, `is_float_literal`, `is_decimal_literal`, `is_hexadecimal_literal`,
///   `is_octal_literal`, `is_binary_literal`, `is_hex_float_literal`
/// - Textual: `is_string_literal`, `is_inline_string_literal`, `is_multiline_string_literal`,
///   `is_raw_string_literal`, `is_char_literal`
/// - Byte-oriented: `is_byte_literal`, `is_byte_string_literal`
/// - Semantic markers: `is_boolean_literal`, `is_true_literal`, `is_false_literal`, `is_null_literal`
///
/// Override only what you need; everything else can keep the default `false`.
///
/// ## Example
///
/// ```rust
/// use tokit::{Token, LitToken};
/// use logos::Logos;
///
/// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
/// enum MyTokens {
///     #[regex(r"[0-9]+")]
///     Integer,
///     #[regex(r"[0-9]+\.[0-9]+")]
///     Float,
///     #[regex(r#""([^"\\]|\\.)*""#)]
///     String,
///     #[regex(r"true|false")]
///     Boolean,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum MyTokenKind {
///     Integer,
///     Float,
///     String,
///     Boolean,
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyToken {
///     kind: MyTokenKind,
/// }
///
/// impl Token<'_> for MyToken {
///     type Char = char;
///     type Kind = MyTokenKind;
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
///             MyTokens::Integer => MyTokenKind::Integer,
///             MyTokens::Float => MyTokenKind::Float,
///             MyTokens::String => MyTokenKind::String,
///             MyTokens::Boolean => MyTokenKind::Boolean,
///         };
///         Self { kind }
///     }
/// }
///
/// impl LitToken<'_> for MyToken {
///     fn is_integer_literal(&self) -> bool {
///         matches!(self.kind, MyTokenKind::Integer)
///     }
///
///     fn is_float_literal(&self) -> bool {
///         matches!(self.kind, MyTokenKind::Float)
///     }
///
///     fn is_string_literal(&self) -> bool {
///         matches!(self.kind, MyTokenKind::String)
///     }
///
///     fn is_boolean_literal(&self) -> bool {
///         matches!(self.kind, MyTokenKind::Boolean)
///     }
/// }
/// ```
pub trait LitToken<'a>: Token<'a> {
  /// Returns `true` if the token is any literal (number, string, boolean, etc.).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_literal(&self) -> bool {
    self.is_numeric_literal()
      || self.is_string_literal()
      || self.is_raw_string_literal()
      || self.is_char_literal()
      || self.is_byte_literal()
      || self.is_byte_string_literal()
      || self.is_boolean_literal()
      || self.is_null_literal()
  }

  /// Returns `true` when the token is any numeric literal.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_numeric_literal(&self) -> bool {
    self.is_integer_literal() || self.is_float_literal() || self.is_hex_float_literal()
  }

  /// Returns `true` when the token is an integer literal (e.g., binary, decimal, hex, octal).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_integer_literal(&self) -> bool {
    self.is_binary_literal()
      || self.is_decimal_literal()
      || self.is_hexadecimal_literal()
      || self.is_octal_literal()
  }

  /// Returns `true` when the token is a floating-point literal.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_float_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a base-10 integer literal.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_decimal_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a hexadecimal integer literal (e.g., `0xFF`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_hexadecimal_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is an octal integer literal (e.g., `0o77`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_octal_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a binary integer literal (e.g., `0b1010`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_binary_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a hexadecimal floating-point literal (e.g., `0x1.fp3`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_hex_float_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is any string literal (quoted text).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_string_literal(&self) -> bool {
    self.is_inline_string_literal() || self.is_multiline_string_literal()
  }

  /// Returns `true` when the token is a single-line/inline string literal.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_inline_string_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a multi-line string literal.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_multiline_string_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a raw string literal.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_raw_string_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a character literal (e.g., `'a'`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_char_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a byte literal (e.g., `b'a'`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_byte_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a byte-string literal (e.g., `b"..."`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_byte_string_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a boolean literal (`true`/`false`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_boolean_literal(&self) -> bool {
    self.is_true_literal() || self.is_false_literal()
  }

  /// Returns `true` when the token is the `true` literal.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_true_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is the `false` literal.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_false_literal(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a null/nil literal.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_null_literal(&self) -> bool {
    false
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
/// use tokit::{Token, IdentifierToken};
/// use logos::Logos;
///
/// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
/// enum MyTokens<'a> {
///     #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
///     Identifier(&'a str),
///     #[token("if")]
///     If,
///     #[token("else")]
///     Else,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum MyTokenKind {
///     Identifier,
///     KeywordIf,
///     KeywordElse,
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
///     type Char = char;
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
/// impl<'a> From<MyTokens<'a>> for MyToken<'a> {
///     fn from(logos: MyTokens<'a>) -> Self {
///         match logos {
///             MyTokens::Identifier(s) => MyToken::Identifier(s),
///             MyTokens::If => MyToken::If,
///             MyTokens::Else => MyToken::Else,
///         }
///     }
/// }
///
/// impl<'a> IdentifierToken<'a> for MyToken<'a> {
///     fn identifier(&self) -> Option<&&'a str> {
///         if let MyToken::Identifier(name) = self {
///             Some(name)
///         } else {
///             None
///         }
///     }
///
///     fn try_into_identifier(self) -> Result<&'a str, Self> {
///         match &self {
///             MyToken::Identifier(name) => Ok(*name),
///             _ => Err(self),
///         }
///     }
/// }
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
/// use tokit::{Token, KeywordToken, utils::cmp::Equivalent};
/// use logos::Logos;
///
/// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
/// enum MyTokens<'a> {
///     #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
///     Identifier(&'a str),
///     #[token("if")]
///     If,
///     #[token("else")]
///     Else,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum MyTokenKind {
///     Identifier,
///     KeywordIf,
///     KeywordElse,
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
///     type Char = char;
///     type Kind = MyTokenKind;
///     type Logos = MyTokens<'a>;
///
///     fn kind(&self) -> Self::Kind {
///         match self {
///            MyToken::Identifier(_) => MyTokenKind::Identifier,
///            MyToken::If => MyTokenKind::KeywordIf,
///            MyToken::Else => MyTokenKind::KeywordElse,
///         }
///     }
/// }
///
/// impl<'a> From<MyTokens<'a>> for MyToken<'a> {
///     fn from(logos: MyTokens<'a>) -> Self {
///         match logos {
///             MyTokens::Identifier(s) => MyToken::Identifier(s),
///             MyTokens::If => MyToken::If,
///             MyTokens::Else => MyToken::Else,
///         }
///     }
/// }
///
/// impl<'a> Equivalent<MyToken<'a>> for str {
///     fn equivalent(&self, other: &MyToken<'a>) -> bool {
///         other.keyword().is_some_and(|kw| kw == self)
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
/// ```
pub trait KeywordToken<'a>: Token<'a> {
  /// Returns `true` when the token is any reserved keyword.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_keyword(&self) -> bool {
    self.keyword().is_some()
  }

  /// Returns the canonical spelling of the keyword, if this token represents one.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn keyword(&self) -> Option<&'static str> {
    None
  }
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
/// - Miscellaneous: `pow`, `is_arrow`, `is_fat_arrow`, `is_pipe_forward`,
///   `is_backslash`
///
/// ## Example
///
/// ```rust
/// use tokit::{Token, OperatorToken, utils::cmp::Equivalent};
/// use logos::Logos;
///
/// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
/// enum MyTokens {
///     #[token("+")]
///     Plus,
///     #[token("++")]
///     Increment,
///     #[token("+=")]
///     PlusAssign,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum MyTokenKind {
///     Plus,
///     Increment,
///     PlusAssign,
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyToken {
///     kind: MyTokenKind,
/// }
///
/// impl Token<'_> for MyToken {
///     type Char = char;
///     type Kind = MyTokenKind;
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
///             MyTokens::Plus => MyTokenKind::Plus,
///             MyTokens::Increment => MyTokenKind::Increment,
///             MyTokens::PlusAssign => MyTokenKind::PlusAssign,
///         };
///         Self { kind }
///     }
/// }
///
/// impl Equivalent<MyToken> for str {
///     fn equivalent(&self, other: &MyToken) -> bool {
///         match other.kind {
///             MyTokenKind::Plus => self == "+",
///             MyTokenKind::Increment => self == "++",
///             MyTokenKind::PlusAssign => self == "+=",
///         }
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

  /// Returns `true` for pipe-forward operators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_pipe_forward(&self) -> bool {
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

/// A trait for tokens that represent paired delimiters (parentheses, braces, brackets, quotes).
///
/// [`DelimiterToken`] keeps delimiter semantics separate from raw punctuation, allowing parsers,
/// formatters, or syntax tree builders to reason about nesting and matching without pattern
/// matching on the token kind.
///
/// # Usage
///
/// - Implementors override whichever predicates apply to their language (all default to `false`).
/// - Aggregation helpers (`is_opening_delimiter`, `is_closing_delimiter`) combine the granular checks.
///
/// ## Example
///
/// ```rust
/// use tokit::{Token, DelimiterToken, utils::cmp::Equivalent};
/// use logos::Logos;
///
/// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
/// enum MyTokens {
///     #[token("(")]
///     LParen,
///     #[token(")")]
///     RParen,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum MyTokenKind {
///     ParenOpen,
///     ParenClose,
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyToken {
///     kind: MyTokenKind,
/// }
///
/// impl Token<'_> for MyToken {
///     type Char = char;
///     type Kind = MyTokenKind;
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
///             MyTokens::LParen => MyTokenKind::ParenOpen,
///             MyTokens::RParen => MyTokenKind::ParenClose,
///         };
///         Self { kind }
///     }
/// }
///
/// impl Equivalent<MyToken> for str {
///     fn equivalent(&self, other: &MyToken) -> bool {
///         match other.kind {
///             MyTokenKind::ParenOpen => self == "(",
///             MyTokenKind::ParenClose => self == ")",
///         }
///     }
/// }
///
/// impl DelimiterToken<'_> for MyToken {
///     fn is_open_paren(&self) -> bool {
///         matches!(self.kind, MyTokenKind::ParenOpen)
///     }
///
///     fn is_close_paren(&self) -> bool {
///         matches!(self.kind, MyTokenKind::ParenClose)
///     }
/// }
/// ```
pub trait DelimiterToken<'a>: Token<'a> {
  /// Returns `true` if the token is any delimiter (opening or closing).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_delimiter(&self) -> bool {
    self.is_opening_delimiter() || self.is_closing_delimiter()
  }

  /// Returns `true` when the token is any opening delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_opening_delimiter(&self) -> bool {
    self.is_open_paren() || self.is_open_brace() || self.is_open_bracket() || self.is_open_angle()
  }

  /// Returns `true` when the token is any closing delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_closing_delimiter(&self) -> bool {
    self.is_close_paren()
      || self.is_close_brace()
      || self.is_close_bracket()
      || self.is_close_angle()
  }

  /// Returns `true` when the token is a parenthesis opening delimiter (`(`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_paren(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a parenthesis closing delimiter (`)`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_paren(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a brace opening delimiter (`{`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_brace(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a brace closing delimiter (`}`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_brace(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a bracket opening delimiter (`[`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_bracket(&self) -> bool {
    false
  }

  /// Returns `true` when the token is a bracket closing delimiter (`]`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_bracket(&self) -> bool {
    false
  }

  /// Returns `true` when the token is an angle/chevron opening delimiter (`<`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_open_angle(&self) -> bool {
    false
  }

  /// Returns `true` when the token is an angle/chevron closing delimiter (`>`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_close_angle(&self) -> bool {
    false
  }
}
