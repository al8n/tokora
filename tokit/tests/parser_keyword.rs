#![cfg(all(feature = "std", feature = "logos"))]

//! Tests for parser/keyword.rs -- Keyword::try_parse, try_parse_sliced,
//! try_parse_exact, try_parse_exact_sliced and their _of variants.

use tokit::{
  Emitter, InputRef, Parse, ParseContext, ParseInput, Parser, Token as TokenT, TryParseInput,
  error::UnexpectedEot,
  logos::{self, Logos},
  token::{KeywordToken, PunctuatorToken},
  try_parse_input::ParseAttempt,
  types::Keyword,
  utils::IntoComponents,
};

// ── Token with keywords ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Logos, PartialEq)]
#[logos(crate = logos, skip r"[ \t\r\n]+")]
pub enum KwToken {
  #[token("let")]
  KwLet,
  #[token("if")]
  KwIf,
  #[token("return")]
  KwReturn,
  #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
  Ident,
  #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().unwrap_or(0))]
  Num(i64),
  #[token("+")]
  Plus,
  #[token(",")]
  Comma,
  #[token(";")]
  Semi,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KwTokenKind {
  KwLet,
  KwIf,
  KwReturn,
  Ident,
  Num,
  Plus,
  Comma,
  Semi,
  LParen,
  RParen,
  LBracket,
  RBracket,
  LBrace,
  RBrace,
}

impl core::fmt::Display for KwTokenKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      KwTokenKind::KwLet => write!(f, "let"),
      KwTokenKind::KwIf => write!(f, "if"),
      KwTokenKind::KwReturn => write!(f, "return"),
      KwTokenKind::Ident => write!(f, "identifier"),
      KwTokenKind::Num => write!(f, "number"),
      KwTokenKind::Plus => write!(f, "+"),
      KwTokenKind::Comma => write!(f, ","),
      KwTokenKind::Semi => write!(f, ";"),
      KwTokenKind::LParen => write!(f, "("),
      KwTokenKind::RParen => write!(f, ")"),
      KwTokenKind::LBracket => write!(f, "["),
      KwTokenKind::RBracket => write!(f, "]"),
      KwTokenKind::LBrace => write!(f, "{{"),
      KwTokenKind::RBrace => write!(f, "}}"),
    }
  }
}

impl core::fmt::Display for KwToken {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      KwToken::KwLet => write!(f, "let"),
      KwToken::KwIf => write!(f, "if"),
      KwToken::KwReturn => write!(f, "return"),
      KwToken::Ident => write!(f, "identifier"),
      KwToken::Num(n) => write!(f, "{n}"),
      KwToken::Plus => write!(f, "+"),
      KwToken::Comma => write!(f, ","),
      KwToken::Semi => write!(f, ";"),
      KwToken::LParen => write!(f, "("),
      KwToken::RParen => write!(f, ")"),
      KwToken::LBracket => write!(f, "["),
      KwToken::RBracket => write!(f, "]"),
      KwToken::LBrace => write!(f, "{{"),
      KwToken::RBrace => write!(f, "}}"),
    }
  }
}

impl From<&KwToken> for KwTokenKind {
  fn from(t: &KwToken) -> Self {
    match t {
      KwToken::KwLet => KwTokenKind::KwLet,
      KwToken::KwIf => KwTokenKind::KwIf,
      KwToken::KwReturn => KwTokenKind::KwReturn,
      KwToken::Ident => KwTokenKind::Ident,
      KwToken::Num(_) => KwTokenKind::Num,
      KwToken::Plus => KwTokenKind::Plus,
      KwToken::Comma => KwTokenKind::Comma,
      KwToken::Semi => KwTokenKind::Semi,
      KwToken::LParen => KwTokenKind::LParen,
      KwToken::RParen => KwTokenKind::RParen,
      KwToken::LBracket => KwTokenKind::LBracket,
      KwToken::RBracket => KwTokenKind::RBracket,
      KwToken::LBrace => KwTokenKind::LBrace,
      KwToken::RBrace => KwTokenKind::RBrace,
    }
  }
}

impl TokenT<'_> for KwToken {
  type Kind = KwTokenKind;
  type Error = ();

  fn kind(&self) -> KwTokenKind {
    KwTokenKind::from(self)
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

impl PunctuatorToken<'_> for KwToken {
  fn comma() -> Option<Self::Kind> {
    Some(KwTokenKind::Comma)
  }
  fn semicolon() -> Option<Self::Kind> {
    Some(KwTokenKind::Semi)
  }
  fn open_paren() -> Option<Self::Kind> {
    Some(KwTokenKind::LParen)
  }
  fn close_paren() -> Option<Self::Kind> {
    Some(KwTokenKind::RParen)
  }
  fn open_bracket() -> Option<Self::Kind> {
    Some(KwTokenKind::LBracket)
  }
  fn close_bracket() -> Option<Self::Kind> {
    Some(KwTokenKind::RBracket)
  }
  fn open_brace() -> Option<Self::Kind> {
    Some(KwTokenKind::LBrace)
  }
  fn close_brace() -> Option<Self::Kind> {
    Some(KwTokenKind::RBrace)
  }
}

impl KeywordToken<'_> for KwToken {
  fn keyword(&self) -> Option<&'static str> {
    match self {
      KwToken::KwLet => Some("let"),
      KwToken::KwIf => Some("if"),
      KwToken::KwReturn => Some("return"),
      _ => None,
    }
  }
}

// ── Punct From impls ────────────────────────────────────────────────────────

use tokit::punct::{
  CloseBrace, CloseBracket, CloseParen, Comma, OpenBrace, OpenBracket, OpenParen, Semicolon,
};

impl From<Comma<(), (), ()>> for KwTokenKind {
  fn from(_: Comma<(), (), ()>) -> Self {
    KwTokenKind::Comma
  }
}
impl From<Semicolon<(), (), ()>> for KwTokenKind {
  fn from(_: Semicolon<(), (), ()>) -> Self {
    KwTokenKind::Semi
  }
}
impl From<OpenParen<(), (), ()>> for KwTokenKind {
  fn from(_: OpenParen<(), (), ()>) -> Self {
    KwTokenKind::LParen
  }
}
impl From<CloseParen<(), (), ()>> for KwTokenKind {
  fn from(_: CloseParen<(), (), ()>) -> Self {
    KwTokenKind::RParen
  }
}
impl From<OpenBracket<(), (), ()>> for KwTokenKind {
  fn from(_: OpenBracket<(), (), ()>) -> Self {
    KwTokenKind::LBracket
  }
}
impl From<CloseBracket<(), (), ()>> for KwTokenKind {
  fn from(_: CloseBracket<(), (), ()>) -> Self {
    KwTokenKind::RBracket
  }
}
impl From<OpenBrace<(), (), ()>> for KwTokenKind {
  fn from(_: OpenBrace<(), (), ()>) -> Self {
    KwTokenKind::LBrace
  }
}
impl From<CloseBrace<(), (), ()>> for KwTokenKind {
  fn from(_: CloseBrace<(), (), ()>) -> Self {
    KwTokenKind::RBrace
  }
}

// ── Error type ──────────────────────────────────────────────────────────────

impl<S, Lang: ?Sized> From<UnexpectedEot<S, Lang>> for KwTestError {
  fn from(_: UnexpectedEot<S, Lang>) -> Self {
    KwTestError
  }
}

use tokit::error::token::UnexpectedToken;

#[derive(Debug)]
struct KwTestError;

impl From<()> for KwTestError {
  fn from(_: ()) -> Self {
    KwTestError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>>
  for KwTestError
{
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    KwTestError
  }
}

// ── Aliases ─────────────────────────────────────────────────────────────────

type KwLexer<'a> = tokit::lexer::LogosLexer<'a, KwToken>;

// ── Tests: Keyword::try_parse ───────────────────────────────────────────────

#[test]
fn keyword_try_parse_accept() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>) -> Result<bool, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse(inp)?;
    Ok(matches!(result, ParseAttempt::Accept(_)))
  }

  let r: Result<bool, KwTestError> = Parser::new().apply(parse).parse_str("let");
  assert!(r.unwrap());
}

#[test]
fn keyword_try_parse_decline_on_non_keyword() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>) -> Result<bool, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }

  let r: Result<bool, KwTestError> = Parser::new().apply(parse).parse_str("foo");
  assert!(r.unwrap());
}

#[test]
fn keyword_try_parse_returns_token() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>,
  ) -> Result<Option<KwToken>, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse(inp)?;
    Ok(match result {
      ParseAttempt::Accept(kw) => {
        let (_span, tok) = kw.into_components();
        Some(tok)
      }
      ParseAttempt::Decline => None,
    })
  }

  let r = Parser::new().apply(parse).parse_str("if").unwrap();
  assert_eq!(r, Some(KwToken::KwIf));
}

// ── Tests: Keyword::try_parse_sliced ────────────────────────────────────────

#[test]
fn keyword_try_parse_sliced_accept() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>,
  ) -> Result<Option<&'inp str>, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse_sliced(inp)?;
    Ok(match result {
      ParseAttempt::Accept(kw) => Some({
        let (_span, src) = kw.into_components();
        src
      }),
      ParseAttempt::Decline => None,
    })
  }

  let r = Parser::new().apply(parse).parse_str("return").unwrap();
  assert_eq!(r, Some("return"));
}

#[test]
fn keyword_try_parse_sliced_decline() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>) -> Result<bool, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse_sliced(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }

  let r: Result<bool, KwTestError> = Parser::new().apply(parse).parse_str("42");
  assert!(r.unwrap());
}

// ── Tests: Keyword::try_parse_exact ─────────────────────────────────────────

#[test]
fn keyword_try_parse_exact_accept() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>) -> Result<bool, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse_exact(&"let").try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Accept(_)))
  }

  let r: Result<bool, KwTestError> = Parser::new().apply(parse).parse_str("let");
  assert!(r.unwrap());
}

#[test]
fn keyword_try_parse_exact_decline_wrong_keyword() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>) -> Result<bool, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse_exact(&"let").try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }

  // "if" is a keyword, but not "let"
  let r: Result<bool, KwTestError> = Parser::new().apply(parse).parse_str("if");
  assert!(r.unwrap());
}

#[test]
fn keyword_try_parse_exact_decline_non_keyword() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>) -> Result<bool, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse_exact(&"let").try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }

  // "foo" is not a keyword
  let r: Result<bool, KwTestError> = Parser::new().apply(parse).parse_str("foo");
  assert!(r.unwrap());
}

#[test]
fn keyword_try_parse_exact_returns_token() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>,
  ) -> Result<Option<KwToken>, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse_exact(&"return").try_parse_input(inp)?;
    Ok(match result {
      ParseAttempt::Accept(kw) => Some({
        let (_span, src) = kw.into_components();
        src
      }),
      ParseAttempt::Decline => None,
    })
  }

  let r = Parser::new().apply(parse).parse_str("return").unwrap();
  assert_eq!(r, Some(KwToken::KwReturn));
}

// ── Tests: Keyword::try_parse_exact_sliced ──────────────────────────────────

#[test]
fn keyword_try_parse_exact_sliced_accept() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>,
  ) -> Result<Option<&'inp str>, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse_exact_sliced(&"if").try_parse_input(inp)?;
    Ok(match result {
      ParseAttempt::Accept(kw) => Some({
        let (_span, src) = kw.into_components();
        src
      }),
      ParseAttempt::Decline => None,
    })
  }

  let r = Parser::new().apply(parse).parse_str("if").unwrap();
  assert_eq!(r, Some("if"));
}

#[test]
fn keyword_try_parse_exact_sliced_decline() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>) -> Result<bool, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse_exact_sliced(&"if").try_parse_input(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }

  let r: Result<bool, KwTestError> = Parser::new().apply(parse).parse_str("let");
  assert!(r.unwrap());
}

// ── Tests: Keyword::try_parse on empty input ────────────────────────────────

#[test]
fn keyword_try_parse_on_empty_input() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>) -> Result<bool, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }

  let r: Result<bool, KwTestError> = Parser::new().apply(parse).parse_str("");
  assert!(r.unwrap());
}

#[test]
fn keyword_try_parse_sliced_on_empty_input() {
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>) -> Result<bool, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let result = Keyword::try_parse_sliced(inp)?;
    Ok(matches!(result, ParseAttempt::Decline))
  }

  let r: Result<bool, KwTestError> = Parser::new().apply(parse).parse_str("");
  assert!(r.unwrap());
}

// ── Tests: multiple keywords in sequence ────────────────────────────────────

#[test]
fn keyword_try_parse_multiple() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, KwLexer<'inp>, Ctx>,
  ) -> Result<Vec<KwToken>, KwTestError>
  where
    Ctx: ParseContext<'inp, KwLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, KwLexer<'inp>, Error = KwTestError>,
  {
    let mut keywords = Vec::new();
    loop {
      let result = Keyword::try_parse(inp)?;
      match result {
        ParseAttempt::Accept(kw) => keywords.push({
          let (_span, src) = kw.into_components();
          src
        }),
        ParseAttempt::Decline => break,
      }
    }
    Ok(keywords)
  }

  let r = Parser::new()
    .apply(parse)
    .parse_str("let if return foo")
    .unwrap();
  assert_eq!(r, vec![KwToken::KwLet, KwToken::KwIf, KwToken::KwReturn]);
}
