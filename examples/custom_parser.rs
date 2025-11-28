// #![cfg(feature = "chumsky")]

// //! A custom parser example demonstrating the Parseable trait.
// //!
// //! This example shows how to:
// //! - Implement the Parseable trait for custom types
// //! - Compose parsers using the trait with Rich error reporting
// //! - Build reusable parser components
// //! - Parse structured data with nested elements
// //! - Handle parse errors with detailed error messages
// //!
// //! Run with: cargo run --example custom_parser --features chumsky

// use chumsky::prelude::*;
// use logos::Logos;
// use logosky::{
//   Lexed, Token,
//   chumsky::Parseable,
//   utils::{Span, Spanned},
// };

// type Tokenizer<'a> = logosky::Tokenizer<'a, ConfigToken<'a>>;

// // Define tokens for a simple configuration language
// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
// #[logos(skip r"[ \t\n\r]+")]
// enum ConfigToken<'a> {
//   #[token("=")]
//   Equals,

//   #[token("{")]
//   LBrace,

//   #[token("}")]
//   RBrace,

//   #[token(";")]
//   Semicolon,

//   #[regex(r#"[a-zA-Z_][a-zA-Z0-9_]*"#, |lex| lex.slice())]
//   Identifier(&'a str),

//   #[regex(r#""([^"\\]|\\["\\])*""#, |lex| lex.slice())]
//   String(&'a str),

//   #[regex(r#"[0-9]+"#, |lex| lex.slice())]
//   Number(&'a str),
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum ConfigTokenKind {
//   Equals,
//   LBrace,
//   RBrace,
//   Semicolon,
//   Identifier,
//   String,
//   Number,
// }

// impl<'a> Token<'a> for ConfigToken<'a> {
//   type Char = char;
//   type Kind = ConfigTokenKind;
//   type Logos = Self;

//   fn kind(&self) -> Self::Kind {
//     match self {
//       Self::Equals => ConfigTokenKind::Equals,
//       Self::LBrace => ConfigTokenKind::LBrace,
//       Self::RBrace => ConfigTokenKind::RBrace,
//       Self::Semicolon => ConfigTokenKind::Semicolon,
//       Self::Identifier(_) => ConfigTokenKind::Identifier,
//       Self::String(_) => ConfigTokenKind::String,
//       Self::Number(_) => ConfigTokenKind::Number,
//     }
//   }
// }

// impl std::fmt::Display for ConfigToken<'_> {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     match self {
//       Self::Equals => write!(f, "="),
//       Self::LBrace => write!(f, "{{"),
//       Self::RBrace => write!(f, "}}"),
//       Self::Semicolon => write!(f, ";"),
//       Self::Identifier(s) => write!(f, "{}", s),
//       Self::String(s) => write!(f, "{}", s),
//       Self::Number(n) => write!(f, "{}", n),
//     }
//   }
// }

// // Define a custom error type with rich error reporting
// #[derive(Debug, Clone, PartialEq)]
// enum ConfigError {
//   UnexpectedToken {
//     span: logosky::utils::Span,
//     found: Option<ConfigTokenKind>,
//     expected: Vec<ConfigTokenKind>,
//   },
//   InvalidNumber {
//     span: logosky::utils::Span,
//     value: String,
//     message: String,
//   },
//   Custom {
//     span: logosky::utils::Span,
//     message: String,
//   },
// }

// impl std::fmt::Display for ConfigError {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     match self {
//       ConfigError::UnexpectedToken {
//         span,
//         found,
//         expected,
//       } => {
//         write!(f, "at {}..{}: ", span.start(), span.end())?;
//         match found {
//           Some(kind) => write!(f, "unexpected token {:?}", kind)?,
//           None => write!(f, "unexpected end of input")?,
//         }
//         if !expected.is_empty() {
//           write!(f, ", expected ")?;
//           if expected.len() == 1 {
//             write!(f, "{:?}", expected[0])?;
//           } else {
//             write!(f, "one of: ")?;
//             for (i, exp) in expected.iter().enumerate() {
//               if i > 0 {
//                 write!(f, ", ")?;
//               }
//               write!(f, "{:?}", exp)?;
//             }
//           }
//         }
//         Ok(())
//       }
//       ConfigError::InvalidNumber {
//         span,
//         value,
//         message,
//       } => {
//         write!(
//           f,
//           "at {}..{}: invalid number '{}': {}",
//           span.start(),
//           span.end(),
//           value,
//           message
//         )
//       }
//       ConfigError::Custom { span, message } => {
//         write!(f, "at {}..{}: {}", span.start(), span.end(), message)
//       }
//     }
//   }
// }

// impl std::error::Error for ConfigError {}

// impl<'a, I, L> chumsky::error::LabelError<'a, I, L> for ConfigError
// where
//   I: chumsky::input::Input<'a, Span = logosky::utils::Span>,
// {
//   fn expected_found<E>(
//     _expected: E,
//     _found: Option<chumsky::util::Maybe<I::Token, &'a I::Token>>,
//     span: I::Span,
//   ) -> Self
//   where
//     E: IntoIterator<Item = L>,
//   {
//     ConfigError::Custom {
//       span,
//       message: "parse error".to_string(),
//     }
//   }

//   fn label_with(&mut self, _label: L) {}
// }

// impl<'a, I> chumsky::error::Error<'a, I> for ConfigError where
//   I: chumsky::input::Input<'a, Span = logosky::utils::Span>
// {
// }

// // AST types
// #[derive(Debug, Clone, PartialEq)]
// enum Value {
//   String(String),
//   Number(i64),
//   Block(Vec<Spanned<Property>>),
// }

// #[derive(Debug, Clone, PartialEq)]
// struct Property {
//   key: String,
//   value: Value,
// }

// impl Property {
//   // Property contains a nested Value, so we need to pass a parser for Value to address recursion problems.
//   fn parser_with<'a, E, P>(vp: P) -> impl Parser<'a, Tokenizer<'a>, Self, E> + Clone
//   where
//     P: Parser<'a, Tokenizer<'a>, Value, E> + Clone + 'a,
//     E: chumsky::extra::ParserExtra<'a, Tokenizer<'a>, Error = ConfigError> + 'a,
//   {
//     // Parse identifier
//     let identifier = any()
//       .try_map(|tok: Lexed<'_, ConfigToken<'_>>, span: Span| match tok {
//         Lexed::Token(t) if t.kind() == ConfigTokenKind::Identifier => {
//           Ok(format!("id[{}..{}]", span.start(), span.end()))
//         }
//         Lexed::Token(t) => Err(ConfigError::UnexpectedToken {
//           span,
//           found: Some(t.kind()),
//           expected: vec![ConfigTokenKind::Identifier],
//         }),
//         _ => Err(ConfigError::UnexpectedToken {
//           span,
//           found: None,
//           expected: vec![ConfigTokenKind::Identifier],
//         }),
//       })
//       .boxed();

//     // key = value;
//     identifier
//       .then_ignore(
//         any().try_map(|tok: Lexed<'_, ConfigToken<'_>>, span| match tok {
//           Lexed::Token(t) if t.kind() == ConfigTokenKind::Equals => Ok(()),
//           Lexed::Token(t) => Err(ConfigError::UnexpectedToken {
//             span,
//             found: Some(t.kind()),
//             expected: vec![ConfigTokenKind::Equals],
//           }),
//           _ => Err(ConfigError::UnexpectedToken {
//             span,
//             found: None,
//             expected: vec![ConfigTokenKind::Equals],
//           }),
//         }),
//       )
//       .then(vp)
//       .then_ignore(
//         any().try_map(|tok: Lexed<'_, ConfigToken<'_>>, span| match tok {
//           Lexed::Token(t) if t.kind() == ConfigTokenKind::Semicolon => Ok(()),
//           Lexed::Token(t) => Err(ConfigError::UnexpectedToken {
//             span,
//             found: Some(t.kind()),
//             expected: vec![ConfigTokenKind::Semicolon],
//           }),
//           _ => Err(ConfigError::UnexpectedToken {
//             span,
//             found: None,
//             expected: vec![ConfigTokenKind::Semicolon],
//           }),
//         }),
//       )
//       .map(|(key, value)| Property { key, value })
//   }
// }

// // Implement Parseable for Value
// impl<'a> Parseable<'a, Tokenizer<'a>, ConfigToken<'a>, ConfigError> for Value {
//   fn parser<E>() -> impl chumsky::Parser<'a, Tokenizer<'a>, Self, E> + Clone
//   where
//     Self: Sized + 'a,
//     E: chumsky::extra::ParserExtra<'a, Tokenizer<'a>, Error = ConfigError> + 'a,
//   {
//     recursive(|value| {
//       // Parse string values
//       let string = any()
//         .try_map(|tok: Lexed<'_, ConfigToken<'_>>, span: Span| match tok {
//           Lexed::Token(t) if t.kind() == ConfigTokenKind::String => {
//             // Extract string content (remove quotes)
//             let s = span.start() + 1..span.end() - 1;
//             Ok(Value::String(format!("string[{}..{}]", s.start, s.end)))
//           }
//           Lexed::Token(t) => Err(ConfigError::UnexpectedToken {
//             span,
//             found: Some(t.kind()),
//             expected: vec![
//               ConfigTokenKind::String,
//               ConfigTokenKind::Number,
//               ConfigTokenKind::LBrace,
//             ],
//           }),
//           _ => Err(ConfigError::UnexpectedToken {
//             span,
//             found: None,
//             expected: vec![
//               ConfigTokenKind::String,
//               ConfigTokenKind::Number,
//               ConfigTokenKind::LBrace,
//             ],
//           }),
//         })
//         .boxed();

//       // Parse number values
//       let number = any()
//         .try_map(|tok: Lexed<'_, ConfigToken<'_>>, span| match tok {
//           Lexed::Token(t) => {
//             if let ConfigToken::Number(n) = t.data() {
//               n.parse::<i64>()
//                 .map(Value::Number)
//                 .map_err(|e| ConfigError::InvalidNumber {
//                   span,
//                   value: n.to_string(),
//                   message: e.to_string(),
//                 })
//             } else {
//               Err(ConfigError::UnexpectedToken {
//                 span,
//                 found: Some(t.kind()),
//                 expected: vec![
//                   ConfigTokenKind::String,
//                   ConfigTokenKind::Number,
//                   ConfigTokenKind::LBrace,
//                 ],
//               })
//             }
//           }
//           _ => Err(ConfigError::UnexpectedToken {
//             span,
//             found: None,
//             expected: vec![
//               ConfigTokenKind::String,
//               ConfigTokenKind::Number,
//               ConfigTokenKind::LBrace,
//             ],
//           }),
//         })
//         .boxed();

//       // Parse block values
//       let block = any()
//         .try_map(|tok: Lexed<'_, ConfigToken<'_>>, span| match tok {
//           Lexed::Token(t) if t.kind() == ConfigTokenKind::LBrace => Ok(()),
//           Lexed::Token(t) => Err(ConfigError::UnexpectedToken {
//             span,
//             found: Some(t.kind()),
//             expected: vec![
//               ConfigTokenKind::String,
//               ConfigTokenKind::Number,
//               ConfigTokenKind::LBrace,
//             ],
//           }),
//           _ => Err(ConfigError::UnexpectedToken {
//             span,
//             found: None,
//             expected: vec![
//               ConfigTokenKind::String,
//               ConfigTokenKind::Number,
//               ConfigTokenKind::LBrace,
//             ],
//           }),
//         })
//         .ignore_then(
//           Property::parser_with(value.clone())
//             .map_with(|prop, exa| Spanned::new(exa.span(), prop))
//             .repeated()
//             .collect::<Vec<_>>(),
//         )
//         .then_ignore(
//           any().try_map(|tok: Lexed<'_, ConfigToken<'_>>, span| match tok {
//             Lexed::Token(t) if t.kind() == ConfigTokenKind::RBrace => Ok(()),
//             Lexed::Token(t) => Err(ConfigError::UnexpectedToken {
//               span,
//               found: Some(t.kind()),
//               expected: vec![ConfigTokenKind::RBrace],
//             }),
//             _ => Err(ConfigError::UnexpectedToken {
//               span,
//               found: None,
//               expected: vec![ConfigTokenKind::RBrace],
//             }),
//           }),
//         )
//         .map(Value::Block)
//         .boxed();

//       choice((string, number, block))
//     })
//   }
// }

// impl<'a> Parseable<'a, Tokenizer<'a>, ConfigToken<'a>, ConfigError> for Property {
//   fn parser<E>() -> impl chumsky::Parser<'a, Tokenizer<'a>, Self, E> + Clone
//   where
//     Self: Sized + 'a,
//     E: chumsky::extra::ParserExtra<'a, Tokenizer<'a>, Error = ConfigError> + 'a,
//   {
//     Property::parser_with(Value::parser())
//   }
// }

// impl std::fmt::Display for Value {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     match self {
//       Value::String(s) => write!(f, "\"{}\"", s),
//       Value::Number(n) => write!(f, "{}", n),
//       Value::Block(props) => {
//         writeln!(f, "{{")?;
//         for prop in props {
//           writeln!(f, "  {}", prop.data)?;
//         }
//         write!(f, "}}")
//       }
//     }
//   }
// }

// impl std::fmt::Display for Property {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     write!(f, "{} = {};", self.key, self.value)
//   }
// }

// fn main() {
//   println!("Custom Parser Example\n");
//   println!("This demonstrates implementing the Parseable trait for custom types.\n");

//   // Example configuration files
//   let examples = vec![
//     ("Simple property", r#"name = "Alice";"#),
//     ("Number property", r#"age = 30;"#),
//     (
//       "Nested block",
//       r#"user = {
//         name = "Bob";
//         age = 25;
//       };"#,
//     ),
//     (
//       "Deep nesting",
//       r#"config = {
//         database = {
//           host = "localhost";
//           port = 5432;
//         };
//         timeout = 30;
//       };"#,
//     ),
//   ];

//   for (name, input) in examples {
//     println!("=== {} ===\n", name);
//     println!("Input:\n{}\n", input);

//     let stream = Tokenizer::<'_>::new(input);
//     let parser =
//       <Spanned<Property> as Parseable<Tokenizer<'_>, ConfigToken<'_>, ConfigError>>::parser::<
//         extra::Err<ConfigError>,
//       >();

//     match parser.parse(stream).into_result() {
//       Ok(property) => {
//         println!("Parsed successfully:");
//         println!("{}\n", property.data);
//       }
//       Err(errors) => {
//         for error in errors {
//           println!("Parse error: {}\n", error);
//         }
//       }
//     }

//     println!();
//   }
// }

fn main() {}
