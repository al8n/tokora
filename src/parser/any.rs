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

impl<'inp, L, E, C> ParseInput<'inp, L, Result<L::Token, E::Error>, E, C> for Any
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  E::Error: From<UnexpectedEot<L::Offset>> + From<<L::Token as Token<'inp>>::Error>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, inp: &mut InputRef<'inp, '_, L, E, C>) -> Result<L::Token, E::Error> {
    match inp.next() {
      Some(Spanned { data: tok, .. }) => match tok {
        Lexed::Token(tok) => Ok(tok),
        Lexed::Error(err) => Err(err.into()),
      },
      None => Err(UnexpectedEot::eot(inp.span().end()).into()),
    }
  }
}

/// Creates a parser that accepts any token.
#[cfg_attr(not(tarpaulin), inline(always))]
pub const fn any<'inp, L, Error>() -> With<Any, Parser<(), L, Result<L::Token, Error>, Error>>
where
  L: Lexer<'inp>,
  Error: From<UnexpectedEot<L::Offset>> + From<<L::Token as Token<'inp>>::Error>,
{
  Parser::with(Any::new())
}

#[cfg(test)]
mod tests {
  use crate::{DummyLexer, DummyToken};

  use super::*;

  const fn assert_any_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Result<DummyToken, ()>, ()>
  {
    any()
  }

  fn assert_any_parse_with_cache_impl<'inp>()
  -> impl Parse<'inp, DummyLexer, Result<DummyToken, ()>, ()> {
    any().with_cache::<()>(())
  }

  fn assert_any_parse_with_emitter_impl<'inp>()
  -> impl Parse<'inp, DummyLexer, Result<DummyToken, ()>, ()> {
    any()
      .with_emitter::<Fatal<()>>(Fatal::new())
      .with_cache::<()>(())
  }

  fn assert_any_parse_full_impl<'inp>() -> impl Parse<'inp, DummyLexer, Result<DummyToken, ()>, ()>
  {
    any()
      .with_emitter::<Fatal<()>>(Fatal::new())
      .with_cache::<()>(())
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_any_parse_impl();
    let _ = assert_any_parse_with_cache_impl();
    let _ = assert_any_parse_with_emitter_impl();
    let _ = assert_any_parse_full_impl();
  }
}
