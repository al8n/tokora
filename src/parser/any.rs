use crate::{Span, error::UnexpectedEot};

use super::*;

/// A parser that accepts any token.
pub struct Any;

impl<'inp, L, E, C> sealed::Sealed<'inp, L, ParseResult<'inp, L::Token, L, E>, E, C> for Any
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
}

impl<'inp, L, E, C> ParseInput<'inp, L, ParseResult<'inp, L::Token, L, E>, E, C> for Any
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, E, C>,
  ) -> ParseResult<'inp, L::Token, L, E> {
    match inp.next() {
      Some(_) => todo!(),
      None => {
        let end = inp.span().end();
        let span = L::Span::new(end.clone(), end);
        Err(Spanned::new(span.clone(), UnexpectedEot::eot(span).into()))
      }
    }
  }
}

// /// A parser that accepts any input, returning the next token if available.
// ///
// /// Returns `None` if the input is exhausted.
// #[cfg_attr(not(tarpaulin), inline(always))]
// pub const fn any<'inp, L>() -> Parser<Any, L, Option<Spanned<Lexed<'inp, L::Token>, L::Span>>, ()>
// where
//   L: Lexer<'inp>,
// {
//   Parser::with(Any)
// }

impl Parser<(), (), (), ()> {
  /// A parser that accepts any input, returning the next token if available.
  ///
  /// Returns `None` if the input is exhausted.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn any<'inp, L, Error>()
  -> Parser<Any, L, Option<Spanned<Lexed<'inp, L::Token>, L::Span>>, Error>
  where
    L: Lexer<'inp>,
  {
    Parser::with(Any)
  }
}
