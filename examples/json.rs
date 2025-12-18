use std::num::ParseFloatError;

use deranged::RangedU8;
use derive_more::{Display, From, Unwrap};
use generic_arraydeque::typenum::U1;
use logos::Logos;
use tokit::{
  Emitter, Lexed, Lexer, Parse, ParseChoice, ParseContext, ParseInput, Parser, Token as TokenT,
  emitter::{DelimitedEmitter, SeparatedEmitter},
  error::{
    UnclosedBrace, UnclosedBracket, Undelimited, UnexpectedEot, UnopenedBrace, UnopenedBracket,
    syntax::{FullContainer, MissingSyntaxOf, TooFew, TooMany},
    token::{
      MissingLeadingComma, MissingSeparatorOf, MissingTrailingComma, UnexpectedLeadingComma,
      UnexpectedToken, UnexpectedTrailingComma,
    },
  },
  lexer::{InputRef, Peeked, PunctuatorToken},
  parser::{Action, Expect},
  punct::{Brace, Bracket, Comma},
  utils::{Expected, Spanned},
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
  MissingValue(MissingSyntaxOf<'a, JsonValue<'a>, JsonLexer<'a>>),
  MissingField(MissingSyntaxOf<'a, (&'a str, JsonValue<'a>), JsonLexer<'a>>),
  MissingLeadingComma(MissingLeadingComma<'a, JsonLexer<'a>>),
  MissingTrailingComma(MissingTrailingComma<'a, JsonLexer<'a>>),
  UnopenedBrace(UnopenedBrace<<JsonLexer<'a> as Lexer<'a>>::Span>),
  UnclosedBrace(UnclosedBrace<<JsonLexer<'a> as Lexer<'a>>::Span>),
  UnopenedBracket(UnopenedBracket<<JsonLexer<'a> as Lexer<'a>>::Span>),
  UnclosedBracket(UnclosedBracket<<JsonLexer<'a> as Lexer<'a>>::Span>),
  UndelimitedBracket(Undelimited<Bracket, <JsonLexer<'a> as Lexer<'a>>::Span>),
  UndelimitedBrace(Undelimited<Brace, <JsonLexer<'a> as Lexer<'a>>::Span>),
  UnexpectedToken(
    UnexpectedToken<
      'a,
      <JsonLexer<'a> as Lexer<'a>>::Token,
      TokenKind,
      <JsonLexer<'a> as Lexer<'a>>::Span,
    >,
  ),
  TooMany(TooMany<JsonValue<'a>, <JsonLexer<'a> as Lexer<'a>>::Span>),
  TooFew(TooFew<JsonValue<'a>, <JsonLexer<'a> as Lexer<'a>>::Span>),
  FullContainer(FullContainer<JsonValue<'a>, <JsonLexer<'a> as Lexer<'a>>::Span>),
  TooManyField(TooMany<(&'a str, JsonValue<'a>), <JsonLexer<'a> as Lexer<'a>>::Span>),
  TooFewField(TooFew<(&'a str, JsonValue<'a>), <JsonLexer<'a> as Lexer<'a>>::Span>),
  FullFieldContainer(FullContainer<(&'a str, JsonValue<'a>), <JsonLexer<'a> as Lexer<'a>>::Span>),
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
      Self::UndelimitedBracket(err) => write!(f, "{err:?}"),
      Self::UndelimitedBrace(err) => write!(f, "{err:?}"),
      Self::UnopenedBrace(err) => write!(f, "{err:?}"),
      Self::UnclosedBrace(err) => write!(f, "{err:?}"),
      Self::UnopenedBracket(err) => write!(f, "{err:?}"),
      Self::UnclosedBracket(err) => write!(f, "{err:?}"),
      Self::UnexpectedLeadingComma(err) => err.debug_fmt(f),
      Self::UnexpectedTrailingComma(err) => err.debug_fmt(f),
      Self::MissingComma(err) => err.debug_fmt(f),
      Self::MissingValue(err) => err.debug_fmt(f),
      Self::MissingLeadingComma(err) => err.debug_fmt(f),
      Self::MissingTrailingComma(err) => err.debug_fmt(f),
      Self::TooMany(err) => write!(f, "{err:?}"),
      Self::TooFew(err) => write!(f, "{err:?}"),
      Self::FullContainer(err) => write!(f, "{err:?}"),
      Self::MissingField(err) => err.debug_fmt(f),
      Self::Other(msg) => write!(f, "{}", msg),
      Self::TooFewField(err) => write!(f, "{err:?}"),
      Self::TooManyField(err) => write!(f, "{err:?}"),
      Self::FullFieldContainer(err) => write!(f, "{err:?}"),
    }
  }
}

impl core::fmt::Display for JsonError<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Parse(err) => write!(f, "parse float error: {}", err),
      Self::UnexpectedToken(err) => err.display_fmt(f),
      Self::Eot(err) => write!(f, "{}", err),
      Self::UndelimitedBrace(err) => write!(f, "{}", err),
      Self::UndelimitedBracket(err) => write!(f, "{}", err),
      Self::UnopenedBrace(err) => write!(f, "{}", err),
      Self::UnclosedBrace(err) => write!(f, "{}", err),
      Self::UnopenedBracket(err) => write!(f, "{}", err),
      Self::UnclosedBracket(err) => write!(f, "{}", err),
      Self::UnexpectedLeadingComma(err) => err.display_fmt(f),
      Self::UnexpectedTrailingComma(err) => err.display_fmt(f),
      Self::MissingComma(err) => err.display_fmt(f),
      Self::MissingValue(err) => err.display_fmt(f),
      Self::MissingLeadingComma(err) => err.display_fmt(f),
      Self::MissingTrailingComma(err) => err.display_fmt(f),
      Self::TooMany(err) => err.display_fmt(f),
      Self::TooFew(err) => err.display_fmt(f),
      Self::FullContainer(err) => err.display_fmt(f),
      Self::MissingField(err) => err.display_fmt(f),
      Self::Other(msg) => write!(f, "{}", msg),
      Self::TooFewField(err) => err.display_fmt(f),
      Self::TooManyField(err) => err.display_fmt(f),
      Self::FullFieldContainer(err) => err.display_fmt(f),
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
  fn is_comma(&self) -> bool {
    matches!(self, Token::Comma)
  }

  #[inline]
  fn is_colon(&self) -> bool {
    matches!(self, Token::Colon)
  }

  #[inline]
  fn is_open_brace(&self) -> bool {
    matches!(self, Token::BraceOpen)
  }

  #[inline]
  fn is_close_brace(&self) -> bool {
    matches!(self, Token::BraceClose)
  }

  #[inline]
  fn is_open_bracket(&self) -> bool {
    matches!(self, Token::BracketOpen)
  }

  #[inline]
  fn is_close_bracket(&self) -> bool {
    matches!(self, Token::BracketClose)
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

type JsonLexer<'a> = tokit::lexer::LogosLexer<'a, Token<'a>, Token<'a>>;

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
        match tok.data() {
          Lexed::Token(tok) if tok.is_value_start() => Action::Continue,
          _ => Action::Stop,
        }
      }
    })
  }
}

fn open_brace(t: &Token<'_>) -> Result<(), TokenKind> {
  if matches!(t, Token::BraceOpen) {
    Ok(())
  } else {
    Err(TokenKind::BraceOpen)
  }
}

fn open_bracket(t: &Token<'_>) -> Result<(), TokenKind> {
  if matches!(t, Token::BracketOpen) {
    Ok(())
  } else {
    Err(TokenKind::BracketOpen)
  }
}

fn close_brace(t: &Token<'_>) -> Result<(), TokenKind> {
  if matches!(t, Token::BraceClose) {
    Ok(())
  } else {
    Err(TokenKind::BraceClose)
  }
}

fn close_bracket(t: &Token<'_>) -> Result<(), TokenKind> {
  if matches!(t, Token::BracketClose) {
    Ok(())
  } else {
    Err(TokenKind::BracketClose)
  }
}

fn expect_colon<'inp>(t: &Token<'inp>) -> Result<(), Expected<'inp, TokenKind>> {
  if matches!(t, Token::Colon) {
    Ok(())
  } else {
    Err(Expected::one(TokenKind::Colon))
  }
}

fn boolean<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<bool, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, JsonLexer<'inp>, Error = JsonError<'inp>>,
{
  Expect::new(|t: &Token<'inp>| {
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
  Expect::new(|t: &Token<'inp>| {
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
  Expect::new(|t: &Token<'inp>| {
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
  Expect::new(|t: &Token<'inp>| {
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
  Ctx::Emitter: SeparatedEmitter<'inp, JsonValue<'inp>, Comma, JsonLexer<'inp>, Error = JsonError<'inp>>
    + SeparatedEmitter<
      'inp,
      (&'inp str, JsonValue<'inp>),
      Comma,
      JsonLexer<'inp>,
      Error = JsonError<'inp>,
    > + DelimitedEmitter<'inp, Bracket, JsonLexer<'inp>, Error = JsonError<'inp>>
    + DelimitedEmitter<'inp, Brace, JsonLexer<'inp>, Error = JsonError<'inp>>,
{
  json_value
    .separated_by_comma::<_, U1>(JsonValue::decide::<Ctx>)
    .delimited_by(open_bracket, close_bracket, Bracket::PHANTOM)
    .collect()
    .parse_input(inp)
}

fn field<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<(&'inp str, JsonValue<'inp>), JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: SeparatedEmitter<'inp, JsonValue<'inp>, Comma, JsonLexer<'inp>, Error = JsonError<'inp>>
    + SeparatedEmitter<
      'inp,
      (&'inp str, JsonValue<'inp>),
      Comma,
      JsonLexer<'inp>,
      Error = JsonError<'inp>,
    > + DelimitedEmitter<'inp, Bracket, JsonLexer<'inp>, Error = JsonError<'inp>>
    + DelimitedEmitter<'inp, Brace, JsonLexer<'inp>, Error = JsonError<'inp>>,
{
  string
    .then_ignore(Expect::new(expect_colon))
    .then(json_value::<Ctx>)
    .parse_input(inp)
}

fn object<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<Vec<(&'inp str, JsonValue<'inp>)>, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: SeparatedEmitter<'inp, JsonValue<'inp>, Comma, JsonLexer<'inp>, Error = JsonError<'inp>>
    + SeparatedEmitter<
      'inp,
      (&'inp str, JsonValue<'inp>),
      Comma,
      JsonLexer<'inp>,
      Error = JsonError<'inp>,
    > + DelimitedEmitter<'inp, Bracket, JsonLexer<'inp>, Error = JsonError<'inp>>
    + DelimitedEmitter<'inp, Brace, JsonLexer<'inp>, Error = JsonError<'inp>>,
{
  field
    .separated_by_comma::<_, U1>(JsonValue::decide::<Ctx>)
    .delimited_by(open_brace, close_brace, Brace::PHANTOM)
    .collect()
    .parse_input(inp)
}

fn json_value<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, JsonLexer<'inp>, Ctx>,
) -> Result<JsonValue<'inp>, JsonError<'inp>>
where
  Ctx: ParseContext<'inp, JsonLexer<'inp>>,
  Ctx::Emitter: SeparatedEmitter<'inp, JsonValue<'inp>, Comma, JsonLexer<'inp>, Error = JsonError<'inp>>
    + SeparatedEmitter<
      'inp,
      (&'inp str, JsonValue<'inp>),
      Comma,
      JsonLexer<'inp>,
      Error = JsonError<'inp>,
    > + DelimitedEmitter<'inp, Bracket, JsonLexer<'inp>, Error = JsonError<'inp>>
    + DelimitedEmitter<'inp, Brace, JsonLexer<'inp>, Error = JsonError<'inp>>,
{
  let end = inp.input().len();
  (
    boolean.map(JsonValue::Bool),
    null.map(|_| JsonValue::Null),
    number.map(JsonValue::Number),
    string.map(JsonValue::String),
    list.map(JsonValue::List),
    object.map(JsonValue::Object),
  )
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
            Lexed::Token(tok) => match tok {
              Token::Bool(_) => Ok(RangedU8::new(0).unwrap()),
              Token::Null => Ok(RangedU8::new(1).unwrap()),
              Token::Number(_) => Ok(RangedU8::new(2).unwrap()),
              Token::String(_) => Ok(RangedU8::new(3).unwrap()),
              Token::BracketOpen => Ok(RangedU8::new(4).unwrap()),
              Token::BraceOpen => Ok(RangedU8::new(5).unwrap()),
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
                .with_found(tok.clone()),
              )),
            },
            Lexed::Error(e) => Err(e.clone().into()),
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
