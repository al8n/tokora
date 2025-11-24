// #![cfg(feature = "chumsky")]

// //! A simple calculator that demonstrates basic usage of logosky.
// //!
// //! This example shows how to:
// //! - Define tokens using Logos
// //! - Create a Token implementation
// //! - Build a recursive descent parser with Chumsky using Rich error reporting
// //! - Evaluate arithmetic expressions
// //! - Handle parse errors with detailed error messages
// //!
// //! Run with: cargo run --example simple_calculator --features chumsky

// use chumsky::prelude::*;
// use logos::Logos;
// use logosky::{Lexed, Token, Tokenizer, chumsky::LogoStream, utils::Spanned};

// // Step 1: Define the tokens using Logos
// #[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
// #[logos(skip r"[ \t\n\r]+")]
// enum CalcTokens<'a> {
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

//   #[regex(r"[0-9]+", |lex| lex.slice())]
//   Number(&'a str),
// }

// // Step 2: Define token kinds
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

// // Step 3: Create a Token wrapper
// #[derive(Debug, Clone, PartialEq)]
// struct CalcToken<'a> {
//   kind: CalcTokenKind,
//   logos: CalcTokens<'a>,
// }

// impl<'a> Token<'a> for CalcToken<'a> {
//   type Char = char;
//   type Kind = CalcTokenKind;
//   type Logos = CalcTokens<'a>;

//   fn kind(&self) -> Self::Kind {
//     self.kind
//   }
// }

// impl<'a> From<CalcTokens<'a>> for CalcToken<'a> {
//   fn from(logos: CalcTokens<'a>) -> Self {
//     let kind = match logos {
//       CalcTokens::Plus => CalcTokenKind::Plus,
//       CalcTokens::Minus => CalcTokenKind::Minus,
//       CalcTokens::Multiply => CalcTokenKind::Multiply,
//       CalcTokens::Divide => CalcTokenKind::Divide,
//       CalcTokens::LParen => CalcTokenKind::LParen,
//       CalcTokens::RParen => CalcTokenKind::RParen,
//       CalcTokens::Number(_) => CalcTokenKind::Number,
//     };
//     CalcToken { kind, logos }
//   }
// }

// impl std::fmt::Display for CalcToken<'_> {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     match self.logos {
//       CalcTokens::Plus => write!(f, "+"),
//       CalcTokens::Minus => write!(f, "-"),
//       CalcTokens::Multiply => write!(f, "*"),
//       CalcTokens::Divide => write!(f, "/"),
//       CalcTokens::LParen => write!(f, "("),
//       CalcTokens::RParen => write!(f, ")"),
//       CalcTokens::Number(n) => write!(f, "{}", n),
//     }
//   }
// }

// // Step 3.5: Define a custom error type with rich error reporting
// #[derive(Debug, Clone, PartialEq)]
// enum CalcError {
//   UnexpectedToken {
//     span: logosky::utils::Span,
//     found: Option<CalcTokenKind>,
//     expected: Vec<CalcTokenKind>,
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

// impl std::fmt::Display for CalcError {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     match self {
//       CalcError::UnexpectedToken {
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
//       CalcError::InvalidNumber {
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
//       CalcError::Custom { span, message } => {
//         write!(f, "at {}..{}: {}", span.start(), span.end(), message)
//       }
//     }
//   }
// }

// impl std::error::Error for CalcError {}

// impl<'a, I, L> chumsky::error::LabelError<'a, I, L> for CalcError
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
//     CalcError::Custom {
//       span,
//       message: "parse error".to_string(),
//     }
//   }

//   fn label_with(&mut self, _label: L) {}
// }

// impl<'a, I> chumsky::error::Error<'a, I> for CalcError where
//   I: chumsky::input::Input<'a, Span = logosky::utils::Span>
// {
// }

// // Step 4: Define the AST
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

// impl std::fmt::Display for BinaryOp {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     match self {
//       BinaryOp::Add => write!(f, "+"),
//       BinaryOp::Sub => write!(f, "-"),
//       BinaryOp::Mul => write!(f, "*"),
//       BinaryOp::Div => write!(f, "/"),
//     }
//   }
// }

// impl Expr {
//   // Evaluate the expression
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

//   // Pretty print the expression
//   fn display(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
//     match self {
//       Expr::Number(n) => write!(f, "{}", n),
//       Expr::BinaryOp { left, op, right } => {
//         write!(f, "({} ", op)?;
//         left.data.display(f, indent + 2)?;
//         write!(f, " ")?;
//         right.data.display(f, indent + 2)?;
//         write!(f, ")")
//       }
//     }
//   }
// }

// impl std::fmt::Display for Expr {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     self.display(f, 0)
//   }
// }

// // Step 5: Build the parser
// fn calc_parser<'a, I, E>() -> impl Parser<'a, I, Spanned<Expr>, E> + Clone
// where
//   I: LogoStream<'a, CalcToken<'a>, Slice = &'a str> + 'a,
//   E: extra::ParserExtra<'a, I, Error = CalcError> + 'a,
// {
//   recursive(|expr| {
//     // Parse numbers
//     let number = any()
//       .try_map(|tok: Lexed<'_, CalcToken<'_>>, span| match tok {
//         Lexed::Token(t) => {
//           if let CalcTokens::Number(n) = t.logos {
//             n.parse()
//               .map(|num| Spanned::new(span, Expr::Number(num)))
//               .map_err(|e| CalcError::InvalidNumber {
//                 span,
//                 value: n.to_string(),
//                 message: e.to_string(),
//               })
//           } else {
//             Err(CalcError::UnexpectedToken {
//               span,
//               found: Some(t.kind()),
//               expected: vec![CalcTokenKind::Number],
//             })
//           }
//         }
//         _ => Err(CalcError::UnexpectedToken {
//           span,
//           found: None,
//           expected: vec![CalcTokenKind::Number],
//         }),
//       })
//       .boxed();

//     // Parse atoms (numbers or parenthesized expressions)
//     let atom = number
//       .or(
//         any()
//           .try_map(|tok: Lexed<'_, CalcToken<'_>>, span| match tok {
//             Lexed::Token(t) if t.kind() == CalcTokenKind::LParen => Ok(()),
//             Lexed::Token(t) => Err(CalcError::UnexpectedToken {
//               span,
//               found: Some(t.kind()),
//               expected: vec![CalcTokenKind::LParen, CalcTokenKind::Number],
//             }),
//             _ => Err(CalcError::UnexpectedToken {
//               span,
//               found: None,
//               expected: vec![CalcTokenKind::LParen, CalcTokenKind::Number],
//             }),
//           })
//           .ignore_then(expr.clone())
//           .then_ignore(
//             any().try_map(|tok: Lexed<'_, CalcToken<'_>>, span| match tok {
//               Lexed::Token(t) if t.kind() == CalcTokenKind::RParen => Ok(()),
//               Lexed::Token(t) => Err(CalcError::UnexpectedToken {
//                 span,
//                 found: Some(t.kind()),
//                 expected: vec![CalcTokenKind::RParen],
//               }),
//               _ => Err(CalcError::UnexpectedToken {
//                 span,
//                 found: None,
//                 expected: vec![CalcTokenKind::RParen],
//               }),
//             }),
//           ),
//       )
//       .boxed();

//     // Parse multiplication and division (higher precedence)
//     let factor = atom.clone().foldl(
//       any()
//         .try_map(|tok: Lexed<'_, CalcToken<'_>>, span| match tok {
//           Lexed::Token(t) if t.kind() == CalcTokenKind::Multiply => Ok(BinaryOp::Mul),
//           Lexed::Token(t) if t.kind() == CalcTokenKind::Divide => Ok(BinaryOp::Div),
//           Lexed::Token(t) => Err(CalcError::UnexpectedToken {
//             span,
//             found: Some(t.kind()),
//             expected: vec![CalcTokenKind::Multiply, CalcTokenKind::Divide],
//           }),
//           _ => Err(CalcError::UnexpectedToken {
//             span,
//             found: None,
//             expected: vec![CalcTokenKind::Multiply, CalcTokenKind::Divide],
//           }),
//         })
//         .then(atom)
//         .repeated(),
//       |left, (op, right)| {
//         let span = logosky::utils::Span::new(left.span.start(), right.span.end());
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

//     // Parse addition and subtraction (lower precedence)
//     factor.clone().foldl(
//       any()
//         .try_map(|tok: Lexed<'_, CalcToken<'_>>, span| match tok {
//           Lexed::Token(t) if t.kind() == CalcTokenKind::Plus => Ok(BinaryOp::Add),
//           Lexed::Token(t) if t.kind() == CalcTokenKind::Minus => Ok(BinaryOp::Sub),
//           Lexed::Token(t) => Err(CalcError::UnexpectedToken {
//             span,
//             found: Some(t.kind()),
//             expected: vec![CalcTokenKind::Plus, CalcTokenKind::Minus],
//           }),
//           _ => Err(CalcError::UnexpectedToken {
//             span,
//             found: None,
//             expected: vec![CalcTokenKind::Plus, CalcTokenKind::Minus],
//           }),
//         })
//         .then(factor)
//         .repeated(),
//       |left, (op, right)| {
//         let span = logosky::utils::Span::new(left.span.start(), right.span.end());
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

// fn main() {
//   // Test expressions
//   let expressions = vec![
//     "42",
//     "1 + 2",
//     "3 * 4",
//     "10 - 5",
//     "20 / 4",
//     "2 + 3 * 4",
//     "(2 + 3) * 4",
//     "100 - 50 + 25",
//     "((1 + 2) * 3 - 4) / 5",
//   ];

//   println!("Simple Calculator Example\n");
//   println!("This demonstrates how to use logosky to build a parser.");
//   println!("We'll parse and evaluate some arithmetic expressions:\n");

//   for expr_str in expressions {
//     println!("Input:  {}", expr_str);

//     // Create a token stream from the input
//     let stream = Tokenizer::<CalcToken<'_>>::new(expr_str);

//     // Parse the expression with rich error reporting
//     let parser = calc_parser::<_, extra::Err<CalcError>>();
//     let result = parser.parse(stream).into_result();

//     match result {
//       Ok(expr) => {
//         println!("AST:    {}", expr.data);

//         // Evaluate the expression
//         match expr.data.eval() {
//           Ok(value) => println!("Result: {}\n", value),
//           Err(e) => println!("Error:  {}\n", e),
//         }
//       }
//       Err(errors) => {
//         for error in errors {
//           println!("Parse error: {}\n", error);
//         }
//       }
//     }
//   }
// }
