// #![allow(warnings)]
// #![cfg(all(feature = "chumsky", any(feature = "std", feature = "alloc")))]

// use chumsky::prelude::*;
// use logos::Logos;
// use logosky::{
//   Token, TokenExt, TriviaToken as _,
//   chumsky::LogoStream,
//   utils::{Span, Spanned},
// };

// type Tokenizer<'a> = logosky::Tokenizer<'a, CalcToken>;

// // A more complete example: a simple calculator language
// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
// #[logos(skip r"[ \t\n\r]+")]
// enum CalcTokens {
//   #[token("+")]
//   Plus,

//   #[token("-")]
//   Minus,

//   #[token("*")]
//   Multiply,

//   #[token("/")]
//   Divide,

//   #[token("(")]
//   LParen,

//   #[token(")")]
//   RParen,

//   #[regex(r"[0-9]+")]
//   Number,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum CalcTokenKind {
//   Plus,
//   Minus,
//   Multiply,
//   Divide,
//   LParen,
//   RParen,
//   Number,
// }

// #[derive(Debug, Clone, PartialEq)]
// struct CalcToken {
//   kind: CalcTokenKind,
//   logos: CalcTokens,
// }

// impl Token<'_> for CalcToken {
//   type Char = char;
//   type Kind = CalcTokenKind;
//   type Logos = CalcTokens;

//   fn kind(&self) -> Self::Kind {
//     self.kind
//   }
// }

// impl From<CalcTokens> for CalcToken {
//   fn from(logos: CalcTokens) -> Self {
//     let kind = match logos {
//       CalcTokens::Plus => CalcTokenKind::Plus,
//       CalcTokens::Minus => CalcTokenKind::Minus,
//       CalcTokens::Multiply => CalcTokenKind::Multiply,
//       CalcTokens::Divide => CalcTokenKind::Divide,
//       CalcTokens::LParen => CalcTokenKind::LParen,
//       CalcTokens::RParen => CalcTokenKind::RParen,
//       CalcTokens::Number => CalcTokenKind::Number,
//     };
//     CalcToken { kind, logos }
//   }
// }

// // AST nodes
// #[derive(Debug, Clone, PartialEq)]
// enum Expr {
//   Number(i64),
//   BinaryOp {
//     left: Box<Spanned<Expr>>,
//     op: BinaryOp,
//     right: Box<Spanned<Expr>>,
//   },
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum BinaryOp {
//   Add,
//   Sub,
//   Mul,
//   Div,
// }

// impl Expr {
//   fn eval(&self) -> Result<i64, String> {
//     match self {
//       Expr::Number(n) => Ok(*n),
//       Expr::BinaryOp { left, op, right } => {
//         let l = left.data.eval()?;
//         let r = right.data.eval()?;
//         match op {
//           BinaryOp::Add => Ok(l + r),
//           BinaryOp::Sub => Ok(l - r),
//           BinaryOp::Mul => Ok(l * r),
//           BinaryOp::Div => {
//             if r == 0 {
//               Err("Division by zero".to_string())
//             } else {
//               Ok(l / r)
//             }
//           }
//         }
//       }
//     }
//   }
// }

// // Parser implementation
// fn calc_parser<'a, I, E>() -> impl Parser<'a, I, Spanned<Expr>, E> + Clone
// where
//   I: LogoStream<'a, CalcToken, Slice = &'a str> + 'a,
//   E: extra::ParserExtra<'a, I, Error = EmptyErr> + 'a,
// {
//   recursive(|expr| {
//     let number = any()
//       .try_map(|tok: logosky::Lexed<'_, CalcToken>, span| match tok {
//         logosky::Lexed::Token(t) if t.kind() == CalcTokenKind::Number => {
//           Ok(Spanned::new(span, Expr::Number(0)))
//         }
//         _ => Err(EmptyErr::default()),
//       })
//       .boxed();

//     let atom = number
//       .or(
//         any()
//           .try_map(|tok: logosky::Lexed<'_, CalcToken>, _| match tok {
//             logosky::Lexed::Token(t) if t.kind() == CalcTokenKind::LParen => Ok(()),
//             _ => Err(EmptyErr::default()),
//           })
//           .ignore_then(expr.clone())
//           .then_ignore(
//             any().try_map(|tok: logosky::Lexed<'_, CalcToken>, _| match tok {
//               logosky::Lexed::Token(t) if t.kind() == CalcTokenKind::RParen => Ok(()),
//               _ => Err(EmptyErr::default()),
//             }),
//           ),
//       )
//       .boxed();

//     let factor = atom.clone().foldl(
//       any()
//         .try_map(|tok: logosky::Lexed<'_, CalcToken>, _| match tok {
//           logosky::Lexed::Token(t) if t.kind() == CalcTokenKind::Multiply => Ok(BinaryOp::Mul),
//           logosky::Lexed::Token(t) if t.kind() == CalcTokenKind::Divide => Ok(BinaryOp::Div),
//           _ => Err(EmptyErr::default()),
//         })
//         .then(atom)
//         .repeated(),
//       |left, (op, right)| {
//         let span = Span::new(left.span.start(), right.span.end());
//         Spanned::new(
//           span,
//           Expr::BinaryOp {
//             left: Box::new(left),
//             op,
//             right: Box::new(right),
//           },
//         )
//       },
//     );

//     factor.clone().foldl(
//       any()
//         .try_map(|tok: logosky::Lexed<'_, CalcToken>, _| match tok {
//           logosky::Lexed::Token(t) if t.kind() == CalcTokenKind::Plus => Ok(BinaryOp::Add),
//           logosky::Lexed::Token(t) if t.kind() == CalcTokenKind::Minus => Ok(BinaryOp::Sub),
//           _ => Err(EmptyErr::default()),
//         })
//         .then(factor)
//         .repeated(),
//       |left, (op, right)| {
//         let span = Span::new(left.span.start(), right.span.end());
//         Spanned::new(
//           span,
//           Expr::BinaryOp {
//             left: Box::new(left),
//             op,
//             right: Box::new(right),
//           },
//         )
//       },
//     )
//   })
// }

// #[test]
// fn test_simple_number() {
//   let input = "42";
//   let stream = Tokenizer::new(input);
//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok());
// }

// #[test]
// fn test_simple_addition() {
//   let input = "1 + 2";
//   let stream = Tokenizer::new(input);
//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok());

//   if let Ok(expr) = result {
//     assert!(matches!(expr.data, Expr::BinaryOp { .. }));
//   }
// }

// #[test]
// fn test_operator_precedence() {
//   // Test that multiplication has higher precedence than addition
//   let input = "1 + 2 * 3";
//   let stream = Tokenizer::new(input);
//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok());
// }

// #[test]
// fn test_parentheses() {
//   let input = "(1 + 2) * 3";
//   let stream = Tokenizer::new(input);
//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok());
// }

// // JSON-like language test
// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
// #[logos(skip r"[ \t\n\r]+")]
// enum JsonTokens {
//   #[token("{")]
//   LBrace,

//   #[token("}")]
//   RBrace,

//   #[token("[")]
//   LBracket,

//   #[token("]")]
//   RBracket,

//   #[token(":")]
//   Colon,

//   #[token(",")]
//   Comma,

//   #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#)]
//   String,

//   #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
//   Number,

//   #[token("true")]
//   True,

//   #[token("false")]
//   False,

//   #[token("null")]
//   Null,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum JsonTokenKind {
//   LBrace,
//   RBrace,
//   LBracket,
//   RBracket,
//   Colon,
//   Comma,
//   String,
//   Number,
//   True,
//   False,
//   Null,
// }

// #[derive(Debug, Clone, PartialEq)]
// struct JsonToken {
//   kind: JsonTokenKind,
//   logos: JsonTokens,
// }

// impl<'a> Token<'a> for JsonToken {
//   type Char = char;
//   type Kind = JsonTokenKind;
//   type Logos = JsonTokens;

//   fn kind(&self) -> Self::Kind {
//     self.kind
//   }
// }

// impl From<JsonTokens> for JsonToken {
//   fn from(logos: JsonTokens) -> Self {
//     let kind = match logos {
//       JsonTokens::LBrace => JsonTokenKind::LBrace,
//       JsonTokens::RBrace => JsonTokenKind::RBrace,
//       JsonTokens::LBracket => JsonTokenKind::LBracket,
//       JsonTokens::RBracket => JsonTokenKind::RBracket,
//       JsonTokens::Colon => JsonTokenKind::Colon,
//       JsonTokens::Comma => JsonTokenKind::Comma,
//       JsonTokens::String => JsonTokenKind::String,
//       JsonTokens::Number => JsonTokenKind::Number,
//       JsonTokens::True => JsonTokenKind::True,
//       JsonTokens::False => JsonTokenKind::False,
//       JsonTokens::Null => JsonTokenKind::Null,
//     };
//     JsonToken { kind, logos }
//   }
// }

// #[test]
// fn test_json_tokenization() {
//   let input = r#"{"key": "value", "number": 42, "bool": true, "null": null}"#;
//   let mut lexer = logos::Lexer::<JsonTokens>::new(input);

//   let mut token_count = 0;
//   while let Some(token) = JsonToken::lex(&mut lexer) {
//     assert!(token.is_token() || token.is_error());
//     if token.is_token() {
//       token_count += 1;
//     }
//   }

//   // Should have tokenized: { key : value , number : 42 , bool : true , null : null }
//   assert!(token_count > 0);
// }

// #[test]
// fn test_json_array_tokenization() {
//   let input = r#"[1, 2, 3, "hello", true, false, null]"#;
//   let mut lexer = logos::Lexer::<JsonTokens>::new(input);

//   let mut token_count = 0;
//   while let Some(token) = JsonToken::lex(&mut lexer) {
//     assert!(token.is_token() || token.is_error());
//     if token.is_token() {
//       token_count += 1;
//     }
//   }

//   assert!(token_count > 0);
// }

// #[test]
// fn test_token_stream_clone() {
//   let input = "1 + 2";
//   let stream = Tokenizer::new(input);
//   let _stream2 = stream.clone();

//   // Both streams should be able to parse independently
//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();
//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok());
// }

// #[test]
// fn test_empty_input() {
//   let input = "";
//   let stream = Tokenizer::new(input);
//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   // Empty input should fail to parse as an expression
//   assert!(result.is_err());
// }

// #[test]
// fn test_whitespace_handling() {
//   let input1 = "1+2";
//   let input2 = "1 + 2";
//   let input3 = "  1   +   2  ";

//   let stream1 = Tokenizer::new(input1);
//   let stream2 = Tokenizer::new(input2);
//   let stream3 = Tokenizer::new(input3);

//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();

//   let result1 = parser.clone().parse(stream1).into_result();
//   let result2 = parser.clone().parse(stream2).into_result();
//   let result3 = parser.parse(stream3).into_result();

//   // All should parse successfully
//   assert!(result1.is_ok());
//   assert!(result2.is_ok());
//   assert!(result3.is_ok());
// }

// // Test error recovery and span tracking
// #[test]
// fn test_span_tracking() {
//   let input = "123";
//   let stream = Tokenizer::new(input);

//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();
//   let result = parser.parse(stream).into_result();

//   assert!(result.is_ok());
//   if let Ok(expr) = result {
//     // The span should cover the entire input
//     assert_eq!(expr.span.start(), 0);
//     assert_eq!(expr.span.end(), 3);
//   }
// }

// #[test]
// fn test_complex_expression() {
//   let input = "((1 + 2) * 3 - 4) / 5";
//   let stream = Tokenizer::new(input);
//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok());
// }

// // Test multiple source types
// #[test]
// fn test_str_source() {
//   let input: &str = "1 + 2";
//   let stream = Tokenizer::new(input);
//   let parser = calc_parser::<_, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok());
// }

// #[cfg(feature = "bytes")]
// #[test]
// fn test_bytes_source() {
//   use bytes::Bytes;
//   use logosky::source::CustomSource;

//   #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
//   #[logos(skip r"[ \t\n\r]+")]
//   #[logos(source = CustomSource<Bytes>)]
//   enum ByteTokens {
//     #[token("+")]
//     Plus,

//     #[regex(r"[0-9]+")]
//     Number,
//   }

//   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//   enum ByteTokenKind {
//     Plus,
//     Number,
//   }

//   #[derive(Debug, Clone, PartialEq)]
//   struct ByteToken {
//     kind: ByteTokenKind,
//     logos: ByteTokens,
//   }

//   impl Token<'_> for ByteToken {
//     type Char = u8;
//     type Kind = ByteTokenKind;
//     type Logos = ByteTokens;

//     fn kind(&self) -> Self::Kind {
//       self.kind
//     }
//   }

//   impl From<ByteTokens> for ByteToken {
//     fn from(logos: ByteTokens) -> Self {
//       let kind = match logos {
//         ByteTokens::Plus => ByteTokenKind::Plus,
//         ByteTokens::Number => ByteTokenKind::Number,
//       };
//       ByteToken { kind, logos }
//     }
//   }

//   let input: CustomSource<Bytes> = bytes::Bytes::from_static(b"1 + 2").into();
//   let mut lexer = logosky::Tokenizer::<ByteToken>::new(&input);
//   let mut token_count = 0;
//   while let Some(token) = lexer.iter().next() {
//     if token.is_token() {
//       token_count += 1;
//     }
//   }

//   assert_eq!(token_count, 3); // 1, +, 2
// }

// // Test stateful lexing
// #[derive(Default, Debug, Clone)]
// struct CounterState {
//   count: usize,
// }

// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
// #[logos(skip r"[ \t\n]+")]
// #[logos(extras = CounterState)]
// enum StatefulTokens {
//   #[regex(r"[a-z]+", |lex| { lex.extras.count += 1; })]
//   Word,

//   #[regex(r"[0-9]+")]
//   Number,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum StatefulTokenKind {
//   Word,
//   Number,
// }

// #[derive(Debug, Clone, PartialEq)]
// struct StatefulToken {
//   kind: StatefulTokenKind,
//   logos: StatefulTokens,
// }

// impl Token<'_> for StatefulToken {
//   type Char = char;
//   type Kind = StatefulTokenKind;
//   type Logos = StatefulTokens;

//   fn kind(&self) -> Self::Kind {
//     self.kind
//   }
// }

// impl From<StatefulTokens> for StatefulToken {
//   fn from(logos: StatefulTokens) -> Self {
//     let kind = match logos {
//       StatefulTokens::Word => StatefulTokenKind::Word,
//       StatefulTokens::Number => StatefulTokenKind::Number,
//     };
//     StatefulToken { kind, logos }
//   }
// }

// #[test]
// fn test_stateful_lexing() {
//   let input = "hello world foo bar";
//   let state = CounterState { count: 0 };
//   let stream = logosky::Tokenizer::<StatefulToken>::with_state(input, state);

//   // The stream should work with stateful lexers
//   assert_eq!(stream.input(), input);
// }

// // Tests for trivia token handling
// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
// enum TriviaTokens {
//   #[regex(r"[ \t\n\r]+")]
//   Whitespace,

//   #[regex(r"//[^\n]*")]
//   LineComment,

//   #[regex(r"/\*([^*]|\*[^/])*\*/")]
//   BlockComment,

//   #[token("+")]
//   Plus,

//   #[token("-")]
//   Minus,

//   #[regex(r"[0-9]+")]
//   Number,

//   #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
//   Identifier,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum TriviaTokenKind {
//   Whitespace,
//   LineComment,
//   BlockComment,
//   Plus,
//   Minus,
//   Number,
//   Identifier,
// }

// #[derive(Debug, Clone, PartialEq)]
// struct TriviaToken {
//   kind: TriviaTokenKind,
//   logos: TriviaTokens,
// }

// impl Token<'_> for TriviaToken {
//   type Char = char;
//   type Kind = TriviaTokenKind;
//   type Logos = TriviaTokens;

//   fn kind(&self) -> Self::Kind {
//     self.kind
//   }
// }

// impl logosky::TriviaToken<'_> for TriviaToken {
//   fn is_trivia(&self) -> bool {
//     matches!(
//       self.kind,
//       TriviaTokenKind::Whitespace | TriviaTokenKind::LineComment | TriviaTokenKind::BlockComment
//     )
//   }
// }

// impl From<TriviaTokens> for TriviaToken {
//   fn from(logos: TriviaTokens) -> Self {
//     let kind = match logos {
//       TriviaTokens::Whitespace => TriviaTokenKind::Whitespace,
//       TriviaTokens::LineComment => TriviaTokenKind::LineComment,
//       TriviaTokens::BlockComment => TriviaTokenKind::BlockComment,
//       TriviaTokens::Plus => TriviaTokenKind::Plus,
//       TriviaTokens::Minus => TriviaTokenKind::Minus,
//       TriviaTokens::Number => TriviaTokenKind::Number,
//       TriviaTokens::Identifier => TriviaTokenKind::Identifier,
//     };
//     TriviaToken { kind, logos }
//   }
// }

// type TriviaStream<'a> = logosky::Tokenizer<'a, TriviaToken>;

// #[test]
// fn test_skip_trivias_basic() {
//   use chumsky::prelude::*;

//   let input = "  \t\n  42";
//   let stream = TriviaStream::new(input);

//   // Create a parser that skips trivia then parses a number
//   let parser = TriviaStream::skip_trivias::<extra::Err<EmptyErr>>().ignore_then(any().try_map(
//     |tok: logosky::Lexed<'_, TriviaToken>, _| match tok {
//       logosky::Lexed::Token(t) if t.kind() == TriviaTokenKind::Number => Ok(()),
//       _ => Err(EmptyErr::default()),
//     },
//   ));

//   let result = parser.parse(stream).into_result();
//   assert!(
//     result.is_ok(),
//     "Should successfully skip whitespace and parse number"
//   );
// }

// #[test]
// fn test_skip_trivias_with_comments() {
//   use chumsky::prelude::*;

//   let input = "// This is a comment\n  /* block comment */  \t42";
//   let stream = TriviaStream::new(input);

//   let parser = TriviaStream::skip_trivias::<extra::Err<EmptyErr>>().ignore_then(any().try_map(
//     |tok: logosky::Lexed<'_, TriviaToken>, _| match tok {
//       logosky::Lexed::Token(t) if t.kind() == TriviaTokenKind::Number => Ok(()),
//       _ => Err(EmptyErr::default()),
//     },
//   ));

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok(), "Should skip comments and whitespace");
// }

// #[test]
// fn test_skip_trivias_no_trivia() {
//   use chumsky::prelude::*;

//   let input = "42";
//   let stream = TriviaStream::new(input);

//   // Parser should work even when there's no trivia to skip
//   let parser = TriviaStream::skip_trivias::<extra::Err<EmptyErr>>().ignore_then(any().try_map(
//     |tok: logosky::Lexed<'_, TriviaToken>, _| match tok {
//       logosky::Lexed::Token(t) if t.kind() == TriviaTokenKind::Number => Ok(()),
//       _ => Err(EmptyErr::default()),
//     },
//   ));

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok(), "Should work even with no trivia present");
// }

// #[test]
// fn test_collect_trivias_basic() {
//   use chumsky::prelude::*;

//   let input = "  \t\n  42";
//   let stream = TriviaStream::new(input);

//   // Collect trivia tokens into a Vec
//   let parser = TriviaStream::collect_trivias::<Vec<Spanned<TriviaToken>>, extra::Err<EmptyErr>>()
//     .then_ignore(
//       any().try_map(|tok: logosky::Lexed<'_, TriviaToken>, _| match tok {
//         logosky::Lexed::Token(t) if t.kind() == TriviaTokenKind::Number => Ok(()),
//         _ => Err(EmptyErr::default()),
//       }),
//     );

//   let result = parser.parse(stream).into_result();
//   match result {
//     Ok(trivias) => {
//       assert!(
//         !trivias.is_empty(),
//         "Should have collected some trivia tokens"
//       );
//       for trivia in &trivias {
//         assert!(trivia.is_trivia(), "All collected tokens should be trivia");
//       }
//     }
//     Err(e) => panic!("Failed to collect trivias: {:?}", e),
//   }
// }

// #[test]
// fn test_collect_trivias_with_comments() {
//   use chumsky::prelude::*;

//   let input = "// line comment\n  /* block */  ";
//   let stream = TriviaStream::new(input);

//   let parser = TriviaStream::collect_trivias::<Vec<Spanned<TriviaToken>>, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok(), "Should collect comments and whitespace");

//   if let Ok(trivias) = result {
//     assert!(!trivias.is_empty(), "Should have collected trivia");

//     // Verify we collected different types of trivia
//     let has_comment = trivias.iter().any(|t| {
//       matches!(
//         t.kind(),
//         TriviaTokenKind::LineComment | TriviaTokenKind::BlockComment
//       )
//     });
//     let has_whitespace = trivias
//       .iter()
//       .any(|t| t.kind() == TriviaTokenKind::Whitespace);

//     assert!(has_comment, "Should have collected at least one comment");
//     assert!(has_whitespace, "Should have collected whitespace");
//   }
// }

// #[test]
// fn test_collect_trivias_empty_source() {
//   use chumsky::prelude::*;

//   let input = "";
//   let stream = TriviaStream::new(input);

//   // When there's no trivia, should return empty container
//   let parser = TriviaStream::collect_trivias::<Vec<Spanned<TriviaToken>>, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok(), "Should succeed even with no trivia");

//   if let Ok(trivias) = result {
//     assert!(trivias.is_empty(), "Should have collected no trivia tokens");
//   }
// }

// #[test]
// fn test_collect_trivias_empty() {
//   use chumsky::prelude::*;

//   let input = "42";
//   let stream = TriviaStream::new(input);

//   // When there's no trivia, should return empty container
//   let parser = TriviaStream::collect_trivias::<Vec<Spanned<TriviaToken>>, extra::Err<EmptyErr>>()
//     .then_ignore(
//       any().try_map(|tok: logosky::Lexed<'_, TriviaToken>, _| match tok {
//         logosky::Lexed::Token(t) if t.kind() == TriviaTokenKind::Number => Ok(()),
//         _ => Err(EmptyErr::default()),
//       }),
//     );

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok(), "Should succeed even with no trivia");

//   if let Ok(trivias) = result {
//     assert!(trivias.is_empty(), "Should have collected no trivia tokens");
//   }
// }

// #[test]
// fn test_skip_trivias_between_tokens() {
//   use chumsky::prelude::*;

//   let input = "42  // comment\n  +  /* block */  13";
//   let stream = TriviaStream::new(input);

//   // Parse: number, skip trivia, plus, skip trivia, number
//   let number_parser = any().try_map(|tok: logosky::Lexed<'_, TriviaToken>, _| match tok {
//     logosky::Lexed::Token(t) if t.kind() == TriviaTokenKind::Number => Ok(()),
//     _ => Err(EmptyErr::default()),
//   });

//   let plus_parser = any().try_map(|tok: logosky::Lexed<'_, TriviaToken>, _| match tok {
//     logosky::Lexed::Token(t) if t.kind() == TriviaTokenKind::Plus => Ok(()),
//     _ => Err(EmptyErr::default()),
//   });

//   let parser = number_parser
//     .then_ignore(TriviaStream::skip_trivias::<extra::Err<EmptyErr>>())
//     .then_ignore(plus_parser)
//     .then_ignore(TriviaStream::skip_trivias::<extra::Err<EmptyErr>>())
//     .then_ignore(number_parser);

//   let result = parser.parse(stream).into_result();
//   assert!(
//     result.is_ok(),
//     "Should parse tokens with trivia between them"
//   );
// }

// #[test]
// fn test_collect_trivias_preserves_spans() {
//   use chumsky::prelude::*;

//   let input = "  \t  ";
//   let stream = TriviaStream::new(input);

//   let parser = TriviaStream::collect_trivias::<Vec<Spanned<TriviaToken>>, extra::Err<EmptyErr>>();

//   let result = parser.parse(stream).into_result();
//   assert!(result.is_ok(), "Should collect trivia");

//   if let Ok(trivias) = result {
//     for trivia in &trivias {
//       let span = trivia.span();
//       assert!(
//         span.start() < span.end(),
//         "Each trivia should have a valid span"
//       );
//       assert!(
//         span.end() <= input.len(),
//         "Span should be within input bounds"
//       );
//     }
//   }
// }
