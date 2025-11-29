use crate::{Span, error::UnexpectedEot};

use super::*;

/// A parser that accepts any token.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Any(());

impl Any {
  /// Creates a new `Any` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self(())
  }
}

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
      Some(Spanned { span, data: tok }) => {
        match tok {
          Lexed::Token(tok) => Ok(Spanned { span, data: tok }),
          Lexed::Error(err) => Err(Spanned { span, data: err.into() }),
        }
      },
      None => {
        let end = inp.span().end();
        let span = L::Span::new(end.clone(), end);
        Err(Spanned::new(span.clone(), UnexpectedEot::eot(span).into()))
      }
    }
  }
}

/// Creates a parser that accepts any token.
#[cfg_attr(not(tarpaulin), inline(always))]
pub const fn any() -> Any {
  Any::new()
}
