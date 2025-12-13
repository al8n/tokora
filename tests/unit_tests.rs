// #![cfg(all(feature = "chumsky", any(feature = "std", feature = "alloc")))]

// use logos::Logos;
// use tokit::{
//   Token, TokenExt,
//   chumsky::Parseable,
//   utils::{AsSpan, IntoSpan, Span, Spanned},
// };

// type Tokenizer<'a> = tokit::Tokenizer<'a, SimpleToken>;

// // Define a simple token type for testing
// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
// #[logos(skip r"[ \t\n\f]+")]
// enum SimpleTokens {
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

//   #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
//   Identifier,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum SimpleTokenKind {
//   Plus,
//   Minus,
//   Multiply,
//   Divide,
//   LParen,
//   RParen,
//   Number,
//   Identifier,
// }

// #[derive(Debug, Clone, PartialEq)]
// struct SimpleToken {
//   kind: SimpleTokenKind,
//   logos: SimpleTokens,
// }

// impl Token<'_> for SimpleToken {
//   type Char = char;
//   type Kind = SimpleTokenKind;
//   type Logos = SimpleTokens;

//   fn kind(&self) -> Self::Kind {
//     self.kind
//   }
// }

// impl From<SimpleTokens> for SimpleToken {
//   fn from(logos: SimpleTokens) -> Self {
//     let kind = match logos {
//       SimpleTokens::Plus => SimpleTokenKind::Plus,
//       SimpleTokens::Minus => SimpleTokenKind::Minus,
//       SimpleTokens::Multiply => SimpleTokenKind::Multiply,
//       SimpleTokens::Divide => SimpleTokenKind::Divide,
//       SimpleTokens::LParen => SimpleTokenKind::LParen,
//       SimpleTokens::RParen => SimpleTokenKind::RParen,
//       SimpleTokens::Number => SimpleTokenKind::Number,
//       SimpleTokens::Identifier => SimpleTokenKind::Identifier,
//     };
//     SimpleToken { kind, logos }
//   }
// }

// mod span_tests {
//   use super::*;

//   #[test]
//   fn test_span_creation() {
//     let span = Span::new(0, 10);
//     assert_eq!(span.start(), 0);
//     assert_eq!(span.end(), 10);
//     assert_eq!(span.len(), 10);
//     assert!(!span.is_empty());
//   }

//   #[test]
//   fn test_span_empty() {
//     let span = Span::new(5, 5);
//     assert_eq!(span.len(), 0);
//     assert!(span.is_empty());
//   }

//   #[test]
//   #[should_panic(expected = "end must be greater than or equal to start")]
//   fn test_span_invalid() {
//     Span::new(10, 5);
//   }

//   #[test]
//   fn test_span_try_new() {
//     assert!(Span::try_new(0, 10).is_some());
//     assert!(Span::try_new(10, 5).is_none());
//     assert!(Span::try_new(5, 5).is_some());
//   }

//   #[test]
//   fn test_span_bump_start() {
//     let mut span = Span::new(0, 10);
//     span.bump_start(3);
//     assert_eq!(span.start(), 3);
//     assert_eq!(span.end(), 10);
//     assert_eq!(span.len(), 7);
//   }

//   #[test]
//   fn test_span_bump_end() {
//     let mut span = Span::new(0, 10);
//     span.bump_end(5);
//     assert_eq!(span.start(), 0);
//     assert_eq!(span.end(), 15);
//     assert_eq!(span.len(), 15);
//   }

//   #[test]
//   fn test_span_bump() {
//     let mut span = Span::new(0, 10);
//     span.bump(5);
//     assert_eq!(span.start(), 5);
//     assert_eq!(span.end(), 15);
//     assert_eq!(span.len(), 10);
//   }

//   #[test]
//   fn test_span_range_conversion() {
//     let range = 5..15;
//     let span: Span = range.clone().into();
//     assert_eq!(span.start(), 5);
//     assert_eq!(span.end(), 15);

//     let back_to_range: std::ops::Range<usize> = span.into();
//     assert_eq!(back_to_range, range);
//   }

//   #[test]
//   fn test_span_with_methods() {
//     let span = Span::new(0, 10).with_start(5).with_end(20);
//     assert_eq!(span.start(), 5);
//     assert_eq!(span.end(), 20);
//   }
// }

// mod spanned_tests {
//   use super::*;

//   #[test]
//   fn test_spanned_creation() {
//     let span = Span::new(0, 5);
//     let spanned = Spanned::new(span, 42);

//     assert_eq!(spanned.span(), &span);
//     assert_eq!(spanned.data(), &42);
//     assert_eq!(*spanned, 42); // Test Deref
//   }

//   #[test]
//   fn test_spanned_as_span() {
//     let span = Span::new(10, 20);
//     let spanned = Spanned::new(span, "test");

//     assert_eq!(spanned.as_span(), &span);
//   }

//   #[test]
//   fn test_spanned_into_span() {
//     let span = Span::new(10, 20);
//     let spanned = Spanned::new(span, "test");

//     assert_eq!(spanned.into_span(), span);
//   }

//   #[test]
//   fn test_spanned_into_components() {
//     let span = Span::new(5, 15);
//     let spanned = Spanned::new(span, "data");

//     let (s, d) = spanned.into_components();
//     assert_eq!(s, span);
//     assert_eq!(d, "data");
//   }

//   #[test]
//   fn test_spanned_deref_mut() {
//     let span = Span::new(0, 1);
//     let mut spanned = Spanned::new(span, 10);

//     *spanned += 5;
//     assert_eq!(*spanned, 15);
//   }
// }

// mod token_stream_tests {
//   use super::*;

//   #[test]
//   fn test_token_stream_creation() {
//     let input = "1 + 2";
//     let _stream = Tokenizer::new(input);
//   }

//   #[test]
//   fn test_token_ext_lexer() {
//     let input = "42 + 13";
//     let _lexer = SimpleToken::lexer(input);
//   }

//   #[test]
//   fn test_token_stream_input() {
//     let input = "123";
//     let stream = Tokenizer::new(input);
//     assert_eq!(stream.input(), input);
//   }
// }

// mod lexer_tests {
//   use super::*;
//   use tokit::Lexed;

//   #[test]
//   fn test_simple_lexing() {
//     let input = "42";
//     let mut lexer = logos::Lexer::<SimpleTokens>::new(input);

//     let token = SimpleToken::lex(&mut lexer);
//     assert!(token.is_some());

//     let lexed = token.unwrap();
//     assert!(lexed.is_token());

//     if let Lexed::Token(spanned) = lexed {
//       assert_eq!(spanned.data().kind(), SimpleTokenKind::Number);
//       assert_eq!(spanned.span().start(), 0);
//       assert_eq!(spanned.span().end(), 2);
//     }
//   }

//   #[test]
//   fn test_operator_lexing() {
//     let input = "+ - * /";
//     let mut lexer = logos::Lexer::<SimpleTokens>::new(input);

//     let ops = vec![
//       SimpleTokenKind::Plus,
//       SimpleTokenKind::Minus,
//       SimpleTokenKind::Multiply,
//       SimpleTokenKind::Divide,
//     ];

//     for expected_kind in ops {
//       let token = SimpleToken::lex(&mut lexer);
//       assert!(token.is_some());

//       if let Some(Lexed::Token(spanned)) = token {
//         assert_eq!(spanned.data().kind(), expected_kind);
//       } else {
//         panic!("Expected token, got error or None");
//       }
//     }
//   }

//   #[test]
//   fn test_identifier_lexing() {
//     let input = "foo bar_123 _baz";
//     let mut lexer = logos::Lexer::<SimpleTokens>::new(input);

//     for _ in 0..3 {
//       let token = SimpleToken::lex(&mut lexer);
//       assert!(token.is_some());

//       if let Some(Lexed::Token(spanned)) = token {
//         assert_eq!(spanned.data().kind(), SimpleTokenKind::Identifier);
//       } else {
//         panic!("Expected identifier token");
//       }
//     }
//   }

//   #[test]
//   fn test_mixed_tokens() {
//     let input = "foo + 123";
//     let mut lexer = logos::Lexer::<SimpleTokens>::new(input);

//     let expected = vec![
//       SimpleTokenKind::Identifier,
//       SimpleTokenKind::Plus,
//       SimpleTokenKind::Number,
//     ];

//     for expected_kind in expected {
//       let token = SimpleToken::lex(&mut lexer);
//       assert!(token.is_some());

//       if let Some(Lexed::Token(spanned)) = token {
//         assert_eq!(spanned.data().kind(), expected_kind);
//       } else {
//         panic!("Expected token");
//       }
//     }
//   }
// }

// mod is_ascii_char_tests {
//   use ascii::AsciiChar;
//   use tokit::utils::IsAsciiChar;

//   #[test]
//   fn test_char_is_ascii_char() {
//     assert!('a'.is_ascii_char(AsciiChar::a));
//     assert!(!'a'.is_ascii_char(AsciiChar::b));
//     assert!('0'.is_ascii_char(AsciiChar::_0));
//   }

//   #[test]
//   fn test_char_is_ascii_digit() {
//     assert!('0'.is_ascii_digit());
//     assert!('9'.is_ascii_digit());
//     assert!(!'a'.is_ascii_digit());
//   }

//   #[test]
//   fn test_u8_is_ascii_char() {
//     assert!(b'a'.is_ascii_char(AsciiChar::a));
//     assert!(!b'a'.is_ascii_char(AsciiChar::b));
//     assert!(b'0'.is_ascii_char(AsciiChar::_0));
//   }

//   #[test]
//   fn test_u8_is_ascii_digit() {
//     assert!(b'0'.is_ascii_digit());
//     assert!(b'9'.is_ascii_digit());
//     assert!(!b'a'.is_ascii_digit());
//   }

//   #[test]
//   fn test_str_is_ascii_char() {
//     assert!("a".is_ascii_char(AsciiChar::a));
//     assert!(!"a".is_ascii_char(AsciiChar::b));
//     assert!(!"ab".is_ascii_char(AsciiChar::a)); // Multi-char string
//   }

//   #[test]
//   fn test_str_is_ascii_digit() {
//     assert!("0".is_ascii_digit());
//     assert!(!"a".is_ascii_digit());
//     assert!(!"12".is_ascii_digit()); // Multi-char string
//   }

//   #[test]
//   fn test_byte_slice_is_ascii_char() {
//     assert!(b"a".as_slice().is_ascii_char(AsciiChar::a));
//     assert!(!b"b".as_slice().is_ascii_char(AsciiChar::a));
//     assert!(!b"ab".as_slice().is_ascii_char(AsciiChar::a)); // Multi-byte
//   }

//   #[test]
//   fn test_one_of() {
//     let choices = &[AsciiChar::a, AsciiChar::b, AsciiChar::c];
//     assert!('a'.one_of(choices));
//     assert!('b'.one_of(choices));
//     assert!(!'d'.one_of(choices));
//   }
// }

// mod parseable_tests {
//   use super::*;
//   use chumsky::prelude::*;

//   // Simple parseable type for testing
//   #[derive(Debug, Clone, PartialEq)]
//   struct Number(usize);

//   impl<'a, Error> Parseable<'a, Tokenizer<'a>, SimpleToken, Error> for Number
//   where
//     Error: 'a,
//   {
//     fn parser<E>() -> impl chumsky::Parser<'a, Tokenizer<'a>, Self, E> + Clone
//     where
//       Self: Sized + 'a,
//       E: chumsky::extra::ParserExtra<'a, Tokenizer<'a>, Error = Error> + 'a,
//     {
//       any()
//         .filter(|tok: &tokit::Lexed<'_, SimpleToken>| {
//           tok.is_token() && tok.unwrap_token_ref().kind() == SimpleTokenKind::Number
//         })
//         .map(|_| Number(0))
//     }
//   }

//   #[test]
//   fn test_parseable_option() {
//     // Test that Option<T> implements Parseable when T does
//     let input = "42";
//     let stream = Tokenizer::new(input);

//     let parser = <Option<Number> as Parseable<Tokenizer<'_>, SimpleToken, _>>::parser::<
//       extra::Err<EmptyErr>,
//     >();

//     // The parser should successfully parse an optional number
//     let result = parser.parse(stream);
//     assert!(result.into_result().is_ok());
//   }

//   #[test]
//   fn test_parseable_vec() {
//     // Test that Vec<T> implements Parseable when T does
//     let input = "42 13";
//     let stream = Tokenizer::new(input);

//     let parser =
//       <Vec<Number> as Parseable<Tokenizer<'_>, SimpleToken, _>>::parser::<extra::Err<EmptyErr>>();

//     // The parser should successfully parse repeated numbers
//     let result = parser.parse(stream);
//     assert!(result.into_result().is_ok());
//   }

//   #[test]
//   fn test_parseable_spanned() {
//     // Test that Spanned<T> implements Parseable when T does
//     let input = "42";
//     let stream = Tokenizer::new(input);

//     let parser = <Spanned<Number> as Parseable<Tokenizer<'_>, SimpleToken, _>>::parser::<
//       extra::Err<EmptyErr>,
//     >();

//     let result = parser.parse(stream);
//     assert!(result.into_result().is_ok());
//   }
// }

// #[cfg(feature = "either")]
// mod either_tests {
//   use super::*;
//   use either::Either;

//   #[test]
//   fn test_either_left_as_span() {
//     let span = Span::new(0, 5);
//     let spanned = Spanned::new(span, 42);
//     let either: Either<Spanned<i32>, Spanned<String>> = Either::Left(spanned);

//     assert_eq!(either.as_span(), &span);
//   }

//   #[test]
//   fn test_either_right_as_span() {
//     let span = Span::new(10, 20);
//     let spanned = Spanned::new(span, "test".to_string());
//     let either: Either<Spanned<i32>, Spanned<String>> = Either::Right(spanned);

//     assert_eq!(either.as_span(), &span);
//   }

//   #[test]
//   fn test_either_into_span() {
//     let span = Span::new(5, 15);
//     let spanned = Spanned::new(span, 100);
//     let either: Either<Spanned<i32>, Spanned<String>> = Either::Left(spanned);

//     assert_eq!(either.into_span(), span);
//   }
// }

// #[cfg(feature = "among")]
// mod among_tests {
//   use super::*;
//   use among::Among;

//   #[test]
//   fn test_among_left_as_span() {
//     let span = Span::new(0, 5);
//     let spanned = Spanned::new(span, 42);
//     let among: Among<Spanned<i32>, Spanned<String>, Spanned<bool>> = Among::Left(spanned);

//     assert_eq!(among.as_span(), &span);
//   }

//   #[test]
//   fn test_among_middle_as_span() {
//     let span = Span::new(5, 10);
//     let spanned = Spanned::new(span, "middle".to_string());
//     let among: Among<Spanned<i32>, Spanned<String>, Spanned<bool>> = Among::Middle(spanned);

//     assert_eq!(among.as_span(), &span);
//   }

//   #[test]
//   fn test_among_right_as_span() {
//     let span = Span::new(10, 15);
//     let spanned = Spanned::new(span, true);
//     let among: Among<Spanned<i32>, Spanned<String>, Spanned<bool>> = Among::Right(spanned);

//     assert_eq!(among.as_span(), &span);
//   }
// }
