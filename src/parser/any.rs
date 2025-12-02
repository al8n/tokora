use crate::{Span, error::UnexpectedEot};

use super::*;

/// A parser that accepts any token.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Any<L, Ctx, Lang: ?Sized = ()> {
  _lxr: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<L, Ctx> Any<L, Ctx> {
  /// Creates a parser that accepts any token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self::of()
  }
}

impl<L, Ctx, Lang> Any<L, Ctx, Lang> {
  /// Creates a parser that accepts any token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of() -> Self {
    Any {
      _lxr: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> ParseInput<'inp, L, L::Token, Ctx, Lang> for Any<L, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error:
    From<UnexpectedEot<L::Offset, Lang>> + From<<L::Token as Token<'inp>>::Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<L::Token, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    Ctx: ParseContext<'inp, L, Lang>,
  {
    match inp.next() {
      Some(Spanned { data: tok, .. }) => match tok {
        Lexed::Token(tok) => Ok(tok),
        Lexed::Error(err) => Err(err.into()),
      },
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
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

  fn assert_any_parse_with_context_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::with_context(()).apply(Any::new())
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_any_parse_impl();
    let _ = assert_any_parse_with_context_impl();
  }
}
