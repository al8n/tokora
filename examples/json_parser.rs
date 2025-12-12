// use deranged::RangedU8;
// use derive_more::{Display, Unwrap};
// use generic_arraydeque::typenum::U1;
// use logos::Logos;
// use tokit::{
//   Emitter, Lexed, Lexer, Parse, ParseChoice, ParseContext, ParseInput, Parser, Token as TokenT,
//   emitter::{DelimiterEmitter, Fatal, SeparatedByEmitter},
//   lexer::{Peeked, PunctuatorToken},
//   parser::{Action, Expect, SeparatedBy},
//   punct::Comma,
//   utils::{Expected, delimiter::Delimiter},
// };

// #[derive(Debug, Logos, Clone, Unwrap)]
// #[logos(skip r"[ \t\r\n\f]+")]
// enum Token {
//   #[token("false", |_| false)]
//   #[token("true", |_| true)]
//   Bool(bool),

//   #[token("{")]
//   BraceOpen,

//   #[token("}")]
//   BraceClose,

//   #[token("[")]
//   BracketOpen,

//   #[token("]")]
//   BracketClose,

//   #[token(":")]
//   Colon,

//   #[token(",")]
//   Comma,

//   #[token("null")]
//   Null,

//   #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
//   Number(f64),

//   #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
//   String(String),
// }

// impl Token {
//   #[inline]
//   fn is_value_start(&self) -> bool {
//     matches!(
//       self,
//       Token::Bool(_)
//         | Token::Null
//         | Token::Number(_)
//         | Token::String(_)
//         | Token::BraceOpen
//         | Token::BracketOpen
//     )
//   }
// }

// impl PunctuatorToken<'_> for Token {
//   #[inline]
//   fn is_comma(&self) -> bool {
//     matches!(self, Token::Comma)
//   }

//   #[inline]
//   fn is_colon(&self) -> bool {
//     matches!(self, Token::Colon)
//   }

//   #[inline]
//   fn is_open_brace(&self) -> bool {
//     matches!(self, Token::BraceOpen)
//   }

//   #[inline]
//   fn is_close_brace(&self) -> bool {
//     matches!(self, Token::BraceClose)
//   }

//   #[inline]
//   fn is_open_bracket(&self) -> bool {
//     matches!(self, Token::BracketOpen)
//   }

//   #[inline]
//   fn is_close_bracket(&self) -> bool {
//     matches!(self, Token::BracketClose)
//   }
// }

// #[derive(Debug, Display, PartialEq, Eq, Clone, Copy, Hash)]
// enum TokenKind {
//   #[display("bool")]
//   Bool,

//   #[display("{{")]
//   BraceOpen,
//   #[display("}}")]
//   BraceClose,
//   #[display("[")]
//   BracketOpen,
//   #[display("]")]
//   BracketClose,
//   #[display(":")]
//   Colon,
//   #[display(",")]
//   Comma,
//   #[display("null")]
//   Null,
//   #[display("number")]
//   Number,
//   #[display("string")]
//   String,
// }

// impl From<&Token> for TokenKind {
//   fn from(token: &Token) -> Self {
//     match token {
//       Token::Bool(_) => TokenKind::Bool,
//       Token::BraceOpen => TokenKind::BraceOpen,
//       Token::BraceClose => TokenKind::BraceClose,
//       Token::BracketOpen => TokenKind::BracketOpen,
//       Token::BracketClose => TokenKind::BracketClose,
//       Token::Colon => TokenKind::Colon,
//       Token::Comma => TokenKind::Comma,
//       Token::Null => TokenKind::Null,
//       Token::Number(_) => TokenKind::Number,
//       Token::String(_) => TokenKind::String,
//     }
//   }
// }

// impl TokenT<'_> for Token {
//   type Kind = TokenKind;

//   type Error = ();

//   #[inline]
//   fn kind(&self) -> Self::Kind {
//     TokenKind::from(self)
//   }

//   #[inline]
//   fn is_trivia(&self) -> bool {
//     false
//   }
// }

// type JsonLexer<'a> = tokit::lexer::LogosLexer<'a, Token, Token>;

// // Example of using map combinator to extract token values
// #[derive(Debug, Clone)]
// enum JsonValue {
//   Null,
//   Bool(bool),
//   Number(f64),
//   String(String),
//   List(Vec<JsonValue>),
//   Object(Vec<(String, JsonValue)>),
// }

// impl JsonValue {
//   fn decide<'inp, E>(
//     mut peeked: Peeked<'_, 'inp, JsonLexer<'inp>, U1>,
//     _: &mut E,
//   ) -> Result<Action, E::Error>
//   where
//     E: Emitter<'inp, JsonLexer<'inp>>,
//   {
//     Ok(match peeked.pop_front() {
//       None => Action::Stop,
//       Some(tok) => {
//         let tok = tok
//           .as_maybe_ref()
//           .map(|t| t.token().copied(), |t| t.token())
//           .into_inner();
//         match tok.data() {
//           Lexed::Token(tok) if tok.is_value_start() => Action::Continue,
//           _ => Action::Stop,
//         }
//       }
//     })
//   }
// }

// fn open_brace<'inp>(t: &Token) -> Result<(), TokenKind> {
//   if matches!(t, Token::BraceOpen) {
//     Ok(())
//   } else {
//     Err(TokenKind::BraceOpen)
//   }
// }

// fn open_bracket<'inp>(t: &Token) -> Result<(), TokenKind> {
//   if matches!(t, Token::BracketOpen) {
//     Ok(())
//   } else {
//     Err(TokenKind::BracketOpen)
//   }
// }

// fn close_brace<'inp>(t: &Token) -> Result<(), TokenKind> {
//   if matches!(t, Token::BraceClose) {
//     Ok(())
//   } else {
//     Err(TokenKind::BraceClose)
//   }
// }

// fn close_bracket<'inp>(t: &Token) -> Result<(), TokenKind> {
//   if matches!(t, Token::BracketClose) {
//     Ok(())
//   } else {
//     Err(TokenKind::BracketClose)
//   }
// }

// fn expect_colon<'inp>(t: &Token) -> Result<(), Expected<'inp, TokenKind>> {
//   if matches!(t, Token::Colon) {
//     Ok(())
//   } else {
//     Err(Expected::one(TokenKind::Colon))
//   }
// }

// fn boolean_parser<'inp, Ctx>() -> impl ParseInput<'inp, JsonLexer<'inp>, bool, Ctx>
// where
//   Ctx: ParseContext<'inp, JsonLexer<'inp>>,
//   Ctx::Emitter: Emitter<'inp, JsonLexer<'inp>, Error = ()>,
// {
//   Expect::new(|t: &Token| {
//     if matches!(t, Token::Bool(_)) {
//       Ok(())
//     } else {
//       Err(Expected::one(TokenKind::Bool))
//     }
//   })
//   .map(Token::unwrap_bool)
// }

// fn null_parser<'inp, Ctx>() -> impl ParseInput<'inp, JsonLexer<'inp>, (), Ctx>
// where
//   Ctx: ParseContext<'inp, JsonLexer<'inp>>,
//   Ctx::Emitter: Emitter<'inp, JsonLexer<'inp>, Error = ()>,
// {
//   Expect::new(|t: &Token| {
//     if matches!(t, Token::Null) {
//       Ok(())
//     } else {
//       Err(Expected::one(TokenKind::Null))
//     }
//   })
//   .ignored()
// }

// fn number_parser<'inp, Ctx>() -> impl ParseInput<'inp, JsonLexer<'inp>, f64, Ctx>
// where
//   Ctx: ParseContext<'inp, JsonLexer<'inp>>,
//   Ctx::Emitter: Emitter<'inp, JsonLexer<'inp>, Error = ()>,
// {
//   Expect::new(|t: &Token| {
//     if matches!(t, Token::Number(_)) {
//       Ok(())
//     } else {
//       Err(Expected::one(TokenKind::Number))
//     }
//   })
//   .map(Token::unwrap_number)
// }

// fn string_parser<'inp, Ctx>() -> impl ParseInput<'inp, JsonLexer<'inp>, String, Ctx>
// where
//   Ctx: ParseContext<'inp, JsonLexer<'inp>>,
//   Ctx::Emitter: Emitter<'inp, JsonLexer<'inp>, Error = ()>,
// {
//   Expect::new(|t: &Token| {
//     if matches!(t, Token::String(_)) {
//       Ok(())
//     } else {
//       Err(Expected::one(TokenKind::String))
//     }
//   })
//   .map(Token::unwrap_string)
// }

// fn field_parser<'inp, Ctx>() -> impl ParseInput<'inp, JsonLexer<'inp>, (String, JsonValue), Ctx>
// where
//   Ctx: ParseContext<'inp, JsonLexer<'inp>>,
//   Ctx::Emitter: SeparatedByEmitter<'inp, JsonValue, Comma, JsonLexer<'inp>, Error = ()>
//     + SeparatedByEmitter<'inp, (String, JsonValue), Comma, JsonLexer<'inp>, Error = ()>
//     + DelimiterEmitter<'inp, Delimiter, JsonLexer<'inp>, Error = ()>,
// {
//   string_parser()
//     .then_ignore(Expect::new(expect_colon))
//     .then(parser())
// }

// fn object_parser<'inp, Ctx>()
// -> impl ParseInput<'inp, JsonLexer<'inp>, Vec<(String, JsonValue)>, Ctx>
// where
//   Ctx: ParseContext<'inp, JsonLexer<'inp>>,
//   Ctx::Emitter: SeparatedByEmitter<'inp, JsonValue, Comma, JsonLexer<'inp>, Error = ()>
//     + SeparatedByEmitter<'inp, (String, JsonValue), Comma, JsonLexer<'inp>, Error = ()>
//     + DelimiterEmitter<'inp, Delimiter, JsonLexer<'inp>, Error = ()>,
// {
//   SeparatedBy::comma::<'inp, JsonLexer<'inp>, U1, Ctx>(
//     field_parser(),
//     JsonValue::decide::<Ctx::Emitter>,
//   )
//   .delimited_by(open_brace, close_brace, Delimiter::Brace)
//   .collect()
// }

// fn list_parser<'inp, Ctx>() -> impl ParseInput<'inp, JsonLexer<'inp>, Vec<JsonValue>, Ctx>
// where
//   Ctx: ParseContext<'inp, JsonLexer<'inp>>,
//   Ctx::Emitter: SeparatedByEmitter<'inp, JsonValue, Comma, JsonLexer<'inp>, Error = ()>
//     + DelimiterEmitter<'inp, Delimiter, JsonLexer<'inp>, Error = ()>,
// {
//   SeparatedBy::comma::<'inp, JsonLexer<'inp>, U1, Ctx>(
//     tokit::parser::Todo::<JsonValue>::new(),
//     JsonValue::decide::<Ctx::Emitter>,
//   )
//   .delimited_by(open_bracket, close_bracket, Delimiter::Bracket)
//   .collect()
// }

// fn parser<'inp, Ctx>(src: &'inp str) -> Result<JsonValue, ()>
// where
//   Ctx: ParseContext<'inp, JsonLexer<'inp>>,
//   Ctx::Emitter: SeparatedByEmitter<'inp, JsonValue, Comma, JsonLexer<'inp>, Error = ()>
//     + SeparatedByEmitter<'inp, (String, JsonValue), Comma, JsonLexer<'inp>, Error = ()>
//     + DelimiterEmitter<'inp, Delimiter, JsonLexer<'inp>, Error = ()>,
// {
//   let parser = (
//     boolean_parser().map(JsonValue::Bool),
//     null_parser().map(|_| JsonValue::Null),
//     number_parser().map(JsonValue::Number),
//     string_parser().map(JsonValue::String),
//     list_parser().map(JsonValue::List),
//     object_parser().map(JsonValue::Object),
//   )
//     .peek_then_choice::<_, U1>(
//       |mut peeked: Peeked<'_, 'inp, JsonLexer<'inp>, U1>, _emitter| match peeked.pop_front() {
//         None => Err(()),
//         Some(tok) => {
//           let tok = tok
//             .as_maybe_ref()
//             .map(|t| t.token().copied(), |t| t.token())
//             .into_inner();
//           match tok.data() {
//             Lexed::Token(tok) => match tok {
//               Token::Bool(_) => Ok(RangedU8::new(0).unwrap()),
//               Token::Null => Ok(RangedU8::new(1).unwrap()),
//               Token::Number(_) => Ok(RangedU8::new(2).unwrap()),
//               Token::String(_) => Ok(RangedU8::new(3).unwrap()),
//               Token::BracketOpen => Ok(RangedU8::new(4).unwrap()),
//               Token::BraceOpen => Ok(RangedU8::new(5).unwrap()),
//               _ => Err(()),
//             },
//             Lexed::Error(_) => Err(()),
//           }
//         }
//       },
//     );

//   Parser::new().apply(parser).parse(src)
// }

fn main() {
  // use tokit::parser::{FatalContext, Parser};

  // let src = r#"{"key": "value", "number": 42}"#;
  // let parser = Parser::with_parser_and_context(parser::<FatalContext<JsonLexer>>());
  // let result = parser.parse(src);
  // println!("{:?}", result);

  // println!("Parser Combinator Examples\n");
  // println!("===========================\n");

  // // Example 1: Using map() to transform parser output
  // // Parse any token and extract just its kind
  // println!("Example 1: Using map() to extract token kind");
  // let kind_parser = Any::parser::<'_, JsonLexer<'_>, ()>()
  //   .map(|result: Result<Token, ()>| result.map(|tok| tok.kind()));

  // let src = "true";
  // let result = kind_parser.parse(src);
  // println!("  Input: \"{}\"", src);
  // println!("  Result: {:?}\n", result);

  // // Example 2: Using map_ok() to transform only successful results
  // // Parse a number token and extract its value
  // println!("Example 2: Using map_ok() to extract number value");
  // let number_parser = Any::parser::<'_, JsonLexer<'_>, ()>().map_ok(|tok: Token| match tok {
  //   Token::Number(n) => Some(n),
  //   _ => None,
  // });

  // let src = "42.5";
  // let result = number_parser.parse(src);
  // println!("  Input: \"{}\"", src);
  // println!("  Result: {:?}\n", result);

  // // Example 3: Chaining map operations
  // // Parse any token and convert it to a JsonValue
  // println!("Example 3: Using map_ok() to convert tokens to JsonValue");
  // let value_parser = Any::parser::<'_, JsonLexer<'_>, ()>().map_ok(|tok: Token| match tok {
  //   Token::Null => JsonValue::Null,
  //   Token::Bool(b) => JsonValue::Bool(b),
  //   Token::Number(n) => JsonValue::Number(n),
  //   Token::String(s) => JsonValue::String(s),
  //   _ => JsonValue::Null,
  // });

  // let src = r#""hello""#;
  // let result = value_parser.parse(src);
  // println!("  Input: {}", src);
  // println!("  Result: {:?}\n", result);

  // // Example 4: Chaining multiple map operations
  // println!("Example 4: Chaining multiple transformations");
  // let chained_parser = Any::parser::<'_, JsonLexer<'_>, ()>()
  //   .map_ok(|tok: Token| tok.kind())
  //   .map(|result: Result<TokenKind, ()>| result.map(|kind| format!("Parsed: {}", kind)));

  // let src = "null";
  // let result = chained_parser.parse(src);
  // println!("  Input: \"{}\"", src);
  // println!("  Result: {:?}\n", result);

  // println!("All examples completed successfully!");
}
