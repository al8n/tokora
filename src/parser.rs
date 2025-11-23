// use crate::{Emitter, Lexer, Token};

// /// A trait for parsers
// pub trait Parser<'source, T, O, E> {
//   /// Parses input and produces an output or an error.
//   fn parse<L, Em>(input: &'source T::Source) -> Result<O, E>
//   where
//     L: Lexer<'source, T>,
//     T: Token<'source>,
//     Em: Emitter<'source, T>;
// }

// impl<'source, T, O, E, F> Parser<'source, T, O, E> for F
// where
//   F: FnMut(&'source T::Source) -> Result<O, E>,
//   T: Token<'source>,
// {
//   fn parse<L, Em>(input: &'source <T>::Source) -> Result<O, E>
//   where
//     L: Lexer<'source, T>,
//     T: Token<'source>,
//     Em: Emitter<'source, T>,
//   {
//     todo!()
//   }
// }
