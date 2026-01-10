use super::Token;
use crate::span::Spanned;

macro_rules! is_punctuator {
  ($this:ident($punct:ident, $($follow:ident),+$(,)?)) => {{
    paste::paste! {
      $this.[< is_ $punct >]() $(|| $this.[< is_ $follow >]())*
    }
  }};
}

macro_rules! define_punctuator_token_traits {
  (
    $(
      $(#[$meta:meta])*
      $punct:ident: $punct_char:literal
    ), +$(,)?
  ) => {
    paste::paste! {
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
        $(
          #[doc = "Returns `Some(_)` when " $punct " (`" $punct_char "`) is one of kinds of the token."]
          $(#[$meta])*
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn $punct() -> Option<Self::Kind> {
            None
          }
        )*
      }


      /// Extension trait providing default implementations for punctuator token methods.
      pub trait PunctuatorTokenExt<'a>: PunctuatorToken<'a> {
        /// Returns `true` when the token is a punctuator.
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn is_punctuator(&self) -> bool {
          is_punctuator!(self($($punct), +))
        }

        #[doc = "Returns `true` when the token is the hyphen punctuator (`-`)."]
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn is_hyphen(&self) -> bool {
          self.is_minus()
        }

        #[doc = "Returns `true` when the token is the thin arrow punctuator (`->`)."]
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn is_thin_arrow(&self) -> bool {
          self.is_arrow()
        }

        $(
          #[doc = "Returns `true` when the token is the " $punct " punctuator (`" $punct_char "`)."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn [< is_ $punct >](&self) -> bool {
            Self::$punct().is_some_and(|k| self.kind().eq(&k))
          }
        )*
      }

      impl<'a, T> PunctuatorTokenExt<'a> for T where T: PunctuatorToken<'a> {}

      /// Extension trait providing default implementations for punctuator token methods.
      pub trait SpannedPunctuatorToken<'a, L: crate::Lexer<'a>, Lang: ?Sized = ()>: Sized
      where
        L::Token: PunctuatorToken<'a>,
      {

        #[doc = "Returns `Some(_)` when the token is hyphen (`-`)."]
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn expect_hyphen(self) -> Result<Spanned<L::Token, L::Span>, crate::error::token::UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>>
        {
          self.expect_minus()
        }

        #[doc = "Returns `Some(_)` when the token is thin arrow (`->`)."]
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn expect_thin_arrow(self) -> Result<Spanned<L::Token, L::Span>, crate::error::token::UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>>
        {
          self.expect_arrow()
        }

        $(
          #[doc = "Returns `Some(_)` when the token is " $punct " (`" $punct_char "`)."]
          fn [< expect_ $punct >](self) -> Result<Spanned<L::Token, L::Span>, crate::error::token::UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>>;
        )*
      }

      impl<'a, L, Lang: ?Sized> SpannedPunctuatorToken<'a, L, Lang> for crate::span::Spanned<L::Token, L::Span>
      where
        L: crate::Lexer<'a>,
        L::Token: PunctuatorToken<'a>,
      {
        $(
          #[doc = "Returns `Some(_)` when the token is " $punct " (`" $punct_char "`)."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          fn [< expect_ $punct >](self) -> Result<Spanned<L::Token, L::Span>, crate::error::token::UnexpectedToken<'a, L::Token, <L::Token as Token<'a>>::Kind, L::Span, Lang>> {
            if self.data().[< is_ $punct >]() {
              Ok(self)
            } else {
              let (span, tok) = self.into_components();
              Err(crate::error::token::UnexpectedToken::expected_one(
                span,
                <L::Token>::$punct()
                  .expect(concat!("`", stringify!($punct), "` should not be `None` if `is_", stringify!($punct), "` is `true`")))
                  .with_found(tok)
              )
            }
          }
        )*
      }
    }
  };
}

define_punctuator_token_traits!(
  open_angle: "<",
  close_angle: ">",
  open_brace: "{",
  close_brace: "}",
  open_paren: "(",
  close_paren: ")",
  open_bracket: "[",
  close_bracket: "]",
  comma: ",",
  semicolon: ";",
  colon: ":",
  dot: ".",
  tilde: "~",
  underscore: "_",
  equal: "=",
  minus: "-",
  #[doc(alias = "thin_arrow")]
  arrow: "->",
  fat_arrow: "=>",
  double_colon: "::",
  tab: "\t",
  newline: "\n",
  carriage_return: "\r",
  crlf: "\r\n",
  space: " ",
  pipe: "|",
  ampersand: "&",
  percent: "%",
  slash: "/",
  backslash: "\\",
  dollar: "$",
  hash: "#",
  at: "@",
  asterisk: "*",
  apostrophe: "'",
  double_quote: "\"",
  plus: "+",
  exclamation: "!",
  question: "?",
  backtick: "`",
  caret: "^",
);
