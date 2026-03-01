/// Shared test infrastructure: lexer, token types, and trait impls.
///
/// All integration tests import this module via `mod common;`.
use tokit::{
  Token as TokenT,
  logos::{self, Logos},
  punct::{
    CloseBrace, CloseBracket, CloseParen, Comma, OpenBrace, OpenBracket, OpenParen, Semicolon,
  },
  token::PunctuatorToken,
};

// ── Token ─────────────────────────────────────────────────────────────────────

/// Test token with logos, default error type `()`.\
#[derive(Debug, Clone, Logos, PartialEq)]
#[logos(crate = logos, skip r"[ \t\r\n]+")]
pub enum Token {
  #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().unwrap_or(0))]
  Num(i64),
  #[token(",")]
  Comma,
  #[token(";")]
  Semi,
  #[token("+")]
  Plus,
  #[token("-")]
  Minus,
  #[token("*")]
  Star,
  #[token("/")]
  Slash,
  #[token("(")]
  LParen,
  #[token(")")]
  RParen,
  #[token("[")]
  LBracket,
  #[token("]")]
  RBracket,
  #[token("{")]
  LBrace,
  #[token("}")]
  RBrace,
  #[token("=")]
  Eq,
  #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
  Ident,
}

// ── TokenKind ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
  Num,
  Comma,
  Semi,
  Plus,
  Minus,
  Star,
  Slash,
  LParen,
  RParen,
  LBracket,
  RBracket,
  LBrace,
  RBrace,
  Eq,
  Ident,
}

impl core::fmt::Display for TokenKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      TokenKind::Num => write!(f, "number"),
      TokenKind::Comma => write!(f, ","),
      TokenKind::Semi => write!(f, ";"),
      TokenKind::Plus => write!(f, "+"),
      TokenKind::Minus => write!(f, "-"),
      TokenKind::Star => write!(f, "*"),
      TokenKind::Slash => write!(f, "/"),
      TokenKind::LParen => write!(f, "("),
      TokenKind::RParen => write!(f, ")"),
      TokenKind::LBracket => write!(f, "["),
      TokenKind::RBracket => write!(f, "]"),
      TokenKind::LBrace => write!(f, "{{"),
      TokenKind::RBrace => write!(f, "}}"),
      TokenKind::Eq => write!(f, "="),
      TokenKind::Ident => write!(f, "identifier"),
    }
  }
}

impl core::fmt::Display for Token {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Token::Num(n) => write!(f, "{n}"),
      Token::Comma => write!(f, ","),
      Token::Semi => write!(f, ";"),
      Token::Plus => write!(f, "+"),
      Token::Minus => write!(f, "-"),
      Token::Star => write!(f, "*"),
      Token::Slash => write!(f, "/"),
      Token::LParen => write!(f, "("),
      Token::RParen => write!(f, ")"),
      Token::LBracket => write!(f, "["),
      Token::RBracket => write!(f, "]"),
      Token::LBrace => write!(f, "{{"),
      Token::RBrace => write!(f, "}}"),
      Token::Eq => write!(f, "="),
      Token::Ident => write!(f, "identifier"),
    }
  }
}

impl From<&Token> for TokenKind {
  fn from(t: &Token) -> Self {
    match t {
      Token::Num(_) => TokenKind::Num,
      Token::Comma => TokenKind::Comma,
      Token::Semi => TokenKind::Semi,
      Token::Plus => TokenKind::Plus,
      Token::Minus => TokenKind::Minus,
      Token::Star => TokenKind::Star,
      Token::Slash => TokenKind::Slash,
      Token::LParen => TokenKind::LParen,
      Token::RParen => TokenKind::RParen,
      Token::LBracket => TokenKind::LBracket,
      Token::RBracket => TokenKind::RBracket,
      Token::LBrace => TokenKind::LBrace,
      Token::RBrace => TokenKind::RBrace,
      Token::Eq => TokenKind::Eq,
      Token::Ident => TokenKind::Ident,
    }
  }
}

// ── Token<'_> trait ───────────────────────────────────────────────────────────

impl TokenT<'_> for Token {
  type Kind = TokenKind;
  type Error = ();

  fn kind(&self) -> TokenKind {
    TokenKind::from(self)
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

// ── PunctuatorToken<'_> ───────────────────────────────────────────────────────

impl PunctuatorToken<'_> for Token {
  fn comma() -> Option<Self::Kind> {
    Some(TokenKind::Comma)
  }

  fn semicolon() -> Option<Self::Kind> {
    Some(TokenKind::Semi)
  }

  fn open_paren() -> Option<Self::Kind> {
    Some(TokenKind::LParen)
  }

  fn close_paren() -> Option<Self::Kind> {
    Some(TokenKind::RParen)
  }

  fn open_bracket() -> Option<Self::Kind> {
    Some(TokenKind::LBracket)
  }

  fn close_bracket() -> Option<Self::Kind> {
    Some(TokenKind::RBracket)
  }

  fn open_brace() -> Option<Self::Kind> {
    Some(TokenKind::LBrace)
  }

  fn close_brace() -> Option<Self::Kind> {
    Some(TokenKind::RBrace)
  }
}

// ── From<Punct> for TokenKind ─────────────────────────────────────────────────
//
// Required by `Punctuator<'inp, L>` impl in parser/punct.rs which needs
// `<L::Token as Token<'inp>>::Kind: From<$name<(), (), ()>>`.

impl From<Comma<(), (), ()>> for TokenKind {
  fn from(_: Comma<(), (), ()>) -> Self {
    TokenKind::Comma
  }
}

impl From<Semicolon<(), (), ()>> for TokenKind {
  fn from(_: Semicolon<(), (), ()>) -> Self {
    TokenKind::Semi
  }
}

impl From<OpenBracket<(), (), ()>> for TokenKind {
  fn from(_: OpenBracket<(), (), ()>) -> Self {
    TokenKind::LBracket
  }
}

impl From<CloseBracket<(), (), ()>> for TokenKind {
  fn from(_: CloseBracket<(), (), ()>) -> Self {
    TokenKind::RBracket
  }
}

impl From<OpenBrace<(), (), ()>> for TokenKind {
  fn from(_: OpenBrace<(), (), ()>) -> Self {
    TokenKind::LBrace
  }
}

impl From<CloseBrace<(), (), ()>> for TokenKind {
  fn from(_: CloseBrace<(), (), ()>) -> Self {
    TokenKind::RBrace
  }
}

impl From<OpenParen<(), (), ()>> for TokenKind {
  fn from(_: OpenParen<(), (), ()>) -> Self {
    TokenKind::LParen
  }
}

impl From<CloseParen<(), (), ()>> for TokenKind {
  fn from(_: CloseParen<(), (), ()>) -> Self {
    TokenKind::RParen
  }
}

// ── TestLexer ─────────────────────────────────────────────────────────────────

pub type TestLexer<'a> = tokit::lexer::LogosLexer<'a, Token>;
