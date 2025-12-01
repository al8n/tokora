use crate::{Span, error::UnexpectedEot};

use super::*;

/// A parser that accepts any token.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Any<Lang: ?Sized = ()>(PhantomData<Lang>);

impl Any {
  /// Creates a parser that accepts any token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self::of()
  }
}

impl<Lang> Any<Lang> {
  /// Creates a parser that accepts any token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of() -> Self {
    Any(PhantomData)
  }
}

impl<'inp, L, E, C, Lang> ParseInput<'inp, L, L::Token, E, C> for Any<Lang>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  E::Error: From<UnexpectedEot<L::Offset, Lang>> + From<<L::Token as Token<'inp>>::Error>,
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

#[cfg(test)]
mod tests {
  use crate::{DummyLexer, DummyToken};

  use super::*;

  fn assert_any_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Any::new())
  }

  fn assert_any_parse_with_cache_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().with_cache::<()>(()).apply(Any::new())
  }

  fn assert_any_parse_with_emitter_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new()
      .with_emitter::<Fatal<()>>(Fatal::new())
      .with_cache::<()>(())
      .apply(Any::new())
  }

  fn assert_any_parse_full_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new()
      .with_emitter::<Fatal<()>>(Fatal::new())
      .with_cache::<()>(())
      .apply(Any::new())
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_any_parse_impl();
    let _ = assert_any_parse_with_cache_impl();
    let _ = assert_any_parse_with_emitter_impl();
    let _ = assert_any_parse_full_impl();
  }
}
