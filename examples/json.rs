use std::num::ParseFloatError;

use derive_more::{Display, From, Unwrap};
use generic_arraydeque::typenum::U1;
use logos::Logos;
use tokit::{
  Accumulator, Branch, Emitter, InputRef, Lexer, Parse, ParseChoice, ParseContext, ParseInput,
  Parser, Token as TokenT, TryParseInput,
  cache::Peeked,
  emitter::{
    SeparatedEmitter, UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  error::{
    UnexpectedEot,
    syntax::MissingSyntaxOf,
    token::{MissingSeparatorOf, UnexpectedLeadingComma, UnexpectedToken, UnexpectedTrailingComma},
  },
  parser::{Action, expect},
  punct::{Brace, Bracket, CloseBrace, CloseBracket, Colon, Comma, OpenBrace, OpenBracket},
  span::Spanned,
  token::PunctuatorToken,
  try_parse_input::ParseAttempt,
  utils::Expected,
};

#[derive(Clone, Debug, From, PartialEq, Eq)]
enum JsonLexerError {
  ParseFloat(Spanned<ParseFloatError>),
  Other(&'static str),
}

impl Default for JsonLexerError {
  fn default() -> Self {
    JsonLexerError::Other("unknown lexer error")
  }
}

impl From<JsonLexerError> for JsonError<'_> {
  fn from(err: JsonLexerError) -> Self {
    match err {
      JsonLexerError::ParseFloat(e) => JsonError::Parse(e),
      JsonLexerError::Other(msg) => JsonError::Other(msg),
    }
  }
}

impl From<()> for JsonLexerError {
  fn from(_: ()) -> Self {
    JsonLexerError::Other("unknown lexer error")
  }
}

#[derive(Clone, From, Unwrap)]
enum JsonError<'a> {
  Parse(Spanned<ParseFloatError>),
  UnexpectedTrailingComma(UnexpectedTrailingComma<'a, JsonLexer<'a>>),
  UnexpectedLeadingComma(UnexpectedLeadingComma<'a, JsonLexer<'a>>),
  MissingComma(MissingSeparatorOf<'a, Comma, JsonLexer<'a>>),
  MissingColon(MissingSeparatorOf<'a, Colon, JsonLexer<'a>>),
  MissingElement(MissingSyntaxOf<'a, JsonLexer<'a>>),
  UnexpectedToken(
    UnexpectedToken<
      'a,
      <JsonLexer<'a> as Lexer<'a>>::Token,
      TokenKind,
      <JsonLexer<'a> as Lexer<'a>>::Span,
    >,
  ),
  Eot(UnexpectedEot),
  Other(&'a str),
}

impl Default for JsonError<'_> {
  fn default() -> Self {
    JsonError::Other("unknown error")
  }
}

impl From<Option<Spanned<ParseFloatError>>> for JsonError<'_> {
  fn from(opt: Option<Spanned<ParseFloatError>>) -> Self {
    match opt {
      Some(err) => JsonError::Parse(err),
      None => JsonError::Other("unknown parse float error"),
    }
  }
}

impl From<()> for JsonError<'_> {
  fn from(_: ()) -> Self {
    JsonError::Other("unknown error")
  }
}

impl core::fmt::Debug for JsonError<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Parse(err) => write!(f, "{err:?}"),
      Self::UnexpectedToken(err) => err.debug_fmt(f),
      Self::Eot(err) => write!(f, "{err:?}"),
      Self::UnexpectedLeadingComma(err) => err.debug_fmt(f),
      Self::UnexpectedTrailingComma(err) => err.debug_fmt(f),
      Self::MissingComma(err) => err.debug_fmt(f),
      Self::MissingColon(err) => err.debug_fmt(f),
      Self::MissingElement(err) => err.debug_fmt(f),
      Self::Other(msg) => write!(f, "{}", msg),
    }
  }
}

impl core::fmt::Display for JsonError<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Parse(err) => write!(f, "parse float error: {}", err),
      Self::UnexpectedToken(err) => err.display_fmt(f),
      Self::Eot(err) => write!(f, "{}", err),
      Self::UnexpectedLeadingComma(err) => err.display_fmt(f),
      Self::UnexpectedTrailingComma(err) => err.display_fmt(f),
      Self::MissingComma(err) => err.display_fmt(f),
      Self::MissingColon(err) => err.display_fmt(f),
      Self::MissingElement(err) => err.display_fmt(f),
      Self::Other(msg) => write!(f, "{}", msg),
    }
  }
}

impl core::error::Error for JsonError<'_> {}

#[derive(Debug, Logos, Clone, Unwrap)]
#[logos(skip r"[ \t\r\n\f]+", error = JsonLexerError)]
enum Token<'a> {
  #[token("false", |_| false)]
  #[token("true", |_| true)]
  Bool(bool),

  #[token("{")]
  BraceOpen,

  #[token("}")]
  BraceClose,

  #[token("[")]
  BracketOpen,

  #[token("]")]
  BracketClose,

  #[token(":")]
  Colon,

  #[token(",")]
  Comma,

  #[token("null")]
  Null,

  #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().map_err(|e| JsonLexerError::ParseFloat(Spanned::new(lex.span().into(), e))))]
  Number(f64),

  #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice())]
  String(&'a str),
}

impl core::fmt::Display for Token<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Token::Bool(b) => write!(f, "{}", b),
      Token::BraceOpen => write!(f, "{{"),
      Token::BraceClose => write!(f, "}}"),
      Token::BracketOpen => write!(f, "["),
      Token::BracketClose => write!(f, "]"),
      Token::Colon => write!(f, ":"),
      Token::Comma => write!(f, ","),
      Token::Null => write!(f, "null"),
      Token::Number(n) => write!(f, "{}", n),
      Token::String(s) => write!(f, "\"{}\"", s),
    }
  }
}

impl Token<'_> {
  #[inline]
  fn is_value_start(&self) -> bool {
    matches!(
      self,
      Token::Bool(_)
        | Token::Null
        | Token::Number(_)
        | Token::String(_)
        | Token::BraceOpen
        | Token::BracketOpen
    )
  }
}

impl<'inp> PunctuatorToken<'inp> for Token<'inp> {
  #[inline]
  fn comma() -> Option<Self::Kind> {
    Some(TokenKind::Comma)
  }

  #[inline]
  fn colon() -> Option<Self::Kind> {
    Some(TokenKind::Colon)
  }

  #[inline]
  fn open_brace() -> Option<Self::Kind> {
    Some(TokenKind::BraceOpen)
  }

  #[inline]
  fn close_brace() -> Option<Self::Kind> {
    Some(TokenKind::BraceClose)
  }

  #[inline]
  fn open_bracket() -> Option<Self::Kind> {
    Some(TokenKind::BracketOpen)
  }

  #[inline]
  fn close_bracket() -> Option<Self::Kind> {
    Some(TokenKind::BracketClose)
  }
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Copy, Hash)]
enum TokenKind {
  #[display("bool")]
  Bool,

  #[display("{{")]
  BraceOpen,
  #[display("}}")]
  BraceClose,
  #[display("[")]
  BracketOpen,
  #[display("]")]
  BracketClose,
  #[display(":")]
  Colon,
  #[display(",")]
  Comma,
  #[display("null")]
  Null,
  #[display("number")]
  Number,
  #[display("string")]
  String,
}

impl From<&Token<'_>> for TokenKind {
  fn from(token: &Token<'_>) -> Self {
    match token {
      Token::Bool(_) => TokenKind::Bool,
      Token::BraceOpen => TokenKind::BraceOpen,
      Token::BraceClose => TokenKind::BraceClose,
      Token::BracketOpen => TokenKind::BracketOpen,
      Token::BracketClose => TokenKind::BracketClose,
      Token::Colon => TokenKind::Colon,
      Token::Comma => TokenKind::Comma,
      Token::Null => TokenKind::Null,
      Token::Number(_) => TokenKind::Number,
      Token::String(_) => TokenKind::String,
    }
  }
}

impl<'inp> TokenT<'inp> for Token<'inp> {
  type Kind = TokenKind;

  type Error = JsonLexerError;

  #[inline]
  fn kind(&self) -> Self::Kind {
    TokenKind::from(self)
  }

  #[inline]
  fn is_trivia(&self) -> bool {
    false
  }
}

impl From<Colon> for TokenKind {
  #[inline]
  fn from(_: Colon) -> Self {
    TokenKind::Colon
  }
}

impl From<OpenBrace> for TokenKind {
  #[inline]
  fn from(_: OpenBrace) -> Self {
    TokenKind::BraceOpen
  }
}

impl From<CloseBrace> for TokenKind {
  #[inline]
  fn from(_: CloseBrace) -> Self {
    TokenKind::BraceClose
  }
}

impl From<OpenBracket> for TokenKind {
  #[inline]
  fn from(_: OpenBracket) -> Self {
    TokenKind::BracketOpen
  }
}

impl From<CloseBracket> for TokenKind {
  #[inline]
  fn from(_: CloseBracket) -> Self {
    TokenKind::BracketClose
  }
}

type JsonLexer<'a> = tokit::lexer::LogosLexer<'a, Token<'a>>;

// Example of using map combinator to extract token values
#[derive(Debug, Clone)]
pub enum JsonValue<'a> {
  Null,
  Bool(bool),
  Number(f64),
  String(&'a str),
  List(Vec<JsonValue<'a>>),
  Object(Vec<(&'a str, JsonValue<'a>)>),
}

impl<'inp> JsonValue<'inp> {
  fn decide<Ctx>(
    mut peeked: Peeked<'_, 'inp, JsonLexer<'inp>, U1>,
    _: &mut Ctx::Emitter,
  ) -> Result<Action, <Ctx::Emitter as Emitter<'inp, JsonLexer<'inp>>>::Error>
  where
    Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  {
    Ok(match peeked.pop_front() {
      None => Action::Stop,
      Some(tok) => {
        let tok = tok
          .as_maybe_ref()
          .map(|t| t.token().copied(), |t| t.token())
          .into_inner();
        match tok.data().is_value_start() {
          true => Action::Continue,
          _ => Action::Stop,
        }
      }
    })
  }
}

fn boolean<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<bool, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, JsonLexer<'inp>, Error = JsonError<'inp>>,
{
  expect(|t: &Token<'inp>| {
    if matches!(t, Token::Bool(_)) {
      Ok(())
    } else {
      Err(Expected::one(TokenKind::Bool))
    }
  })
  .map(Token::unwrap_bool)
  .parse_input(inp)
}

fn null<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<(), JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, JsonLexer<'inp>, Error = JsonError<'inp>>,
{
  expect(|t: &Token<'inp>| {
    if matches!(t, Token::Null) {
      Ok(())
    } else {
      Err(Expected::one(TokenKind::Null))
    }
  })
  .ignored()
  .parse_input(inp)
}

fn number<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<f64, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, JsonLexer<'inp>, Error = JsonError<'inp>>,
{
  expect(|t: &Token<'inp>| {
    if matches!(t, Token::Number(_)) {
      Ok(())
    } else {
      Err(Expected::one(TokenKind::Number))
    }
  })
  .map(Token::unwrap_number)
  .parse_input(inp)
}

fn string<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<&'inp str, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, JsonLexer<'inp>, Error = JsonError<'inp>>,
{
  expect(|t: &Token<'inp>| {
    if matches!(t, Token::String(_)) {
      Ok(())
    } else {
      Err(Expected::one(TokenKind::String))
    }
  })
  .map(Token::unwrap_string)
  .parse_input(inp)
}

fn list<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<Vec<JsonValue<'inp>>, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: SeparatedEmitter<'inp, Comma, JsonLexer<'inp>, Error = JsonError<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>,
{
  try_json_value
    .separated_by_comma()
    .delimited::<Bracket>()
    .collect()
    .parse_input(inp)
}

fn field<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<(&'inp str, JsonValue<'inp>), JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: SeparatedEmitter<'inp, Comma, JsonLexer<'inp>, Error = JsonError<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>,
{
  string
    .then_ignore(Colon::parse_of)
    .then(json_value::<Ctx>)
    .parse_input(inp)
}

fn object<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<Vec<(&'inp str, JsonValue<'inp>)>, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: SeparatedEmitter<'inp, Comma, JsonLexer<'inp>, Error = JsonError<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>,
{
  field
    .separated_by_comma_while::<_, U1>(JsonValue::decide::<Ctx>)
    .delimited::<Brace>()
    .collect()
    .parse_input(inp)
}

fn try_json_value<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<JsonValue<'inp>>, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: SeparatedEmitter<'inp, Comma, JsonLexer<'inp>, Error = JsonError<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>,
{
  let end = inp.source().len();
  (
    boolean.map(JsonValue::Bool),
    null.map(|_| JsonValue::Null),
    number.map(JsonValue::Number),
    string.map(JsonValue::String),
    list.map(JsonValue::List),
    object.map(JsonValue::Object),
  )
    // Use `peek_then_try_choice` here as we want to return None if the next token is not a valid start of a JSON value
    .peek_then_try_choice::<_, U1>(
      |mut peeked: Peeked<'_, 'inp, JsonLexer<'inp>, U1>, _emitter| match peeked.pop_front() {
        None => Err(JsonError::Eot(UnexpectedEot::eot(end))),
        Some(tok) => {
          let tok = tok
            .as_maybe_ref()
            .map(|t| t.token().copied(), |t| t.token())
            .into_inner();

          Ok(Some(match tok.data() {
            Token::Bool(_) => Branch::B0,
            Token::Null => Branch::B1,
            Token::Number(_) => Branch::B2,
            Token::String(_) => Branch::B3,
            Token::BracketOpen => Branch::B4,
            Token::BraceOpen => Branch::B5,
            _ => return Ok(None),
          }))
        }
      },
    )
    .try_parse_input(inp)
}

fn json_value<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<JsonValue<'inp>, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: SeparatedEmitter<'inp, Comma, JsonLexer<'inp>, Error = JsonError<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, Comma, JsonLexer<'inp>>,
{
  let end = inp.source().len();
  (
    boolean.map(JsonValue::Bool),
    null.map(|_| JsonValue::Null),
    number.map(JsonValue::Number),
    string.map(JsonValue::String),
    list.map(JsonValue::List),
    object.map(JsonValue::Object),
  )
    // Use `peek_then_choice` here as we want to return an error if the next token is not a valid start of a JSON value
    .peek_then_choice::<_, U1>(
      |mut peeked: Peeked<'_, 'inp, JsonLexer<'inp>, U1>, _emitter| match peeked.pop_front() {
        None => Err(JsonError::Eot(UnexpectedEot::eot(end))),
        Some(tok) => {
          let tok = tok
            .as_maybe_ref()
            .map(|t| t.token().copied(), |t| t.token())
            .into_inner();
          let span = tok.span();
          match tok.data() {
            Token::Bool(_) => Ok(Branch::B0),
            Token::Null => Ok(Branch::B1),
            Token::Number(_) => Ok(Branch::B2),
            Token::String(_) => Ok(Branch::B3),
            Token::BracketOpen => Ok(Branch::B4),
            Token::BraceOpen => Ok(Branch::B5),
            tok => Err(JsonError::UnexpectedToken(
              UnexpectedToken::expected_one_of(
                *span,
                &[
                  TokenKind::Bool,
                  TokenKind::Null,
                  TokenKind::Number,
                  TokenKind::String,
                  TokenKind::BracketOpen,
                  TokenKind::BraceOpen,
                ],
              )
              .with_found((*tok).clone()),
            )),
          }
        }
      },
    )
    .parse_input(inp)
}

const SRC: &str = include_str!("sample.json");

fn main() {
  let output = Parser::new().apply(json_value).parse(SRC).unwrap();
  println!("{:#?}", output);
}
