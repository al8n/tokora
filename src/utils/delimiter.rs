use derive_more::{Display, IsVariant};

use crate::lexer::DelimiterToken;

/// Common delimiters used in lexing and parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Display)]
pub enum Delimiter {
  /// `{` and `}` delimiters.
  #[display("{{}}")]
  Brace,
  /// `(` and `)` delimiters.
  #[display("()")]
  Paren,
  /// `[` and `]` delimiters.
  #[display("[]")]
  Bracket,
  /// `<` and `>` delimiters.
  #[display("<>")]
  Angle,
}

impl Delimiter {
  /// Returns `true` if the given token is an opening delimiter of this kind.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_open<'a, T: DelimiterToken<'a>>(&self, token: &T) -> bool {
    match self {
      Delimiter::Brace => token.is_open_brace(),
      Delimiter::Paren => token.is_open_paren(),
      Delimiter::Bracket => token.is_open_bracket(),
      Delimiter::Angle => token.is_open_angle(),
    }
  }

  /// Returns `true` if the given token is a closing delimiter of this kind.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_close<'a, T: DelimiterToken<'a>>(&self, token: &T) -> bool {
    match self {
      Delimiter::Brace => token.is_close_brace(),
      Delimiter::Paren => token.is_close_paren(),
      Delimiter::Bracket => token.is_close_bracket(),
      Delimiter::Angle => token.is_close_angle(),
    }
  }
}
