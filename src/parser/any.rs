use super::*;

/// A parser that accepts any token.
pub struct Any;

impl<'inp, L, E, C> sealed::Sealed<'inp, L, Option<Spanned<Lexed<'inp, L::Token>, L::Span>>, E, C> for Any
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
}

impl<'inp, L, E, C> ParseInput<'inp, L, Option<Spanned<Lexed<'inp, L::Token>, L::Span>>, E, C> for Any
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, E, C>,
  ) -> Option<Spanned<Lexed<'inp, L::Token>, L::Span>> {
    inp.next()
  }
}

/// A parser that accepts any input, returning the next token if available.
/// 
/// Returns `None` if the input is exhausted.
#[cfg_attr(not(tarpaulin), inline(always))]
pub const fn any<'inp, L>() -> Parser<Any, L, Option<Spanned<Lexed<'inp, L::Token>, L::Span>>, ()>
where
  L: Lexer<'inp>,
{
  Parser::new(Any)
}
