use derive_more::Display;
use logos::*;
use logosky::{Token as TokenT, Any, Parse};

#[derive(Debug, Logos, Clone)]
#[logos(skip r"[ \t\r\n\f]+")]
enum Token {
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

  #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
  Number(f64),

  #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
  String(String),
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

impl From<&Token> for TokenKind {
  fn from(token: &Token) -> Self {
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

impl TokenT<'_> for Token {
  type Kind = TokenKind;

  type Error = ();

  fn kind(&self) -> Self::Kind {
    TokenKind::from(self)
  }
}

type JsonLexer<'a> = logosky::LogosLexer<'a, Token, Token>;

// Example of using map combinator to extract token values
#[derive(Debug, Clone)]
enum JsonValue {
  Null,
  Bool(bool),
  Number(f64),
  String(String),
}

fn main() {
  println!("Parser Combinator Examples\n");
  println!("===========================\n");

  // Example 1: Using map() to transform parser output
  // Parse any token and extract just its kind
  println!("Example 1: Using map() to extract token kind");
  let kind_parser = Any::parser::<'_, JsonLexer<'_>, ()>()
    .map(|result: Result<Token, ()>| result.map(|tok| tok.kind()));

  let src = "true";
  let result = kind_parser.parse(src);
  println!("  Input: \"{}\"", src);
  println!("  Result: {:?}\n", result);

  // Example 2: Using map_ok() to transform only successful results
  // Parse a number token and extract its value
  println!("Example 2: Using map_ok() to extract number value");
  let number_parser = Any::parser::<'_, JsonLexer<'_>, ()>()
    .map_ok(|tok: Token| match tok {
      Token::Number(n) => Some(n),
      _ => None,
    });

  let src = "42.5";
  let result = number_parser.parse(src);
  println!("  Input: \"{}\"", src);
  println!("  Result: {:?}\n", result);

  // Example 3: Chaining map operations
  // Parse any token and convert it to a JsonValue
  println!("Example 3: Using map_ok() to convert tokens to JsonValue");
  let value_parser = Any::parser::<'_, JsonLexer<'_>, ()>()
    .map_ok(|tok: Token| match tok {
      Token::Null => JsonValue::Null,
      Token::Bool(b) => JsonValue::Bool(b),
      Token::Number(n) => JsonValue::Number(n),
      Token::String(s) => JsonValue::String(s),
      _ => JsonValue::Null,
    });

  let src = r#""hello""#;
  let result = value_parser.parse(src);
  println!("  Input: {}", src);
  println!("  Result: {:?}\n", result);

  // Example 4: Chaining multiple map operations
  println!("Example 4: Chaining multiple transformations");
  let chained_parser = Any::parser::<'_, JsonLexer<'_>, ()>()
    .map_ok(|tok: Token| tok.kind())
    .map(|result: Result<TokenKind, ()>| {
      result.map(|kind| format!("Parsed: {}", kind))
    });

  let src = "null";
  let result = chained_parser.parse(src);
  println!("  Input: \"{}\"", src);
  println!("  Result: {:?}\n", result);

  println!("All examples completed successfully!");
}
