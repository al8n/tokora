use crate::{Check, emitter::SeparatedByEmitter, error::token::UnexpectedToken};

use super::*;

// impl<'inp, F, T: Token<'inp>, Lang> Check<T, SeqSepHint<'inp, T::Kind>> for Comma<(), F, Lang>
// where
//   F: Fn(&T) -> 
// {
//   fn check(&self, target: &T) -> SeqSepHint<'inp, T::Kind> {
//     todo!()
//   }
// }

// /// A parser that accepts any input, returning the next token if available.
// ///
// /// Returns `None` if the input is exhausted.
// #[cfg_attr(not(tarpaulin), inline(always))]
// pub const fn comma_seq<'inp, O, P, L, Container>(
//   parser: P,
// ) -> Parser<SeqSep<P, Comma, O, Container>, L, Container, SeqSepOptions>
// where
//   L: Lexer<'inp>,
//   P: ParseInput<'inp, L, ParseResult<'inp, O, L, E>, E, C>,
// {
//   Parser::with(SeqSep::new(parser, Comma::PHANTOM))
// }

impl Parser<(), (), (), ()> {
  /// A parser that accepts an empty comma-separated sequence.
  pub const fn separated_by<'inp, F, H, L, O, Container, Error>(
    parser: F,
    hint: H,
  ) -> Parser<
    SeqSep<F, H, O, Container>,
    L,
    ParseResult<'inp, Container, L, Noop<Error>>,
    Error,
  >
  where
    F: ParseInput<'inp, L, ParseResult<'inp, O, L, Noop<Error>>, Noop<Error>, DefaultCache<'inp, L>>,
    H: Check<L::Token, SeqSepHint<'inp, <L::Token as Token<'inp>>::Kind>>,
    L: Lexer<'inp>,
    Error: From<<L::Token as Token<'inp>>::Error> + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>,
    Noop<Error>: SeparatedByEmitter<'inp, O, H, L>,
  {
    Parser::with(SeqSep::new(parser, hint))
  }
}
