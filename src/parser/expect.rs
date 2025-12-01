use crate::{
  Check, Span,
  error::{UnexpectedEot, token::UnexpectedToken},
};

use super::*;

/// A parser that expects a specific token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Expect<Classifier, Lang: ?Sized = ()> {
  is: Classifier,
  _lang: PhantomData<Lang>,
}

impl<Classifier> Expect<Classifier> {
  /// Creates a parser that accepts a specific token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(classifier: Classifier) -> Self {
    Self::of(classifier)
  }
}

impl<Classifier, Lang> Expect<Classifier, Lang> {
  /// Creates a parser that accepts a specific token of a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(classifier: Classifier) -> Self {
    Self {
      is: classifier,
      _lang: PhantomData,
    }
  }
}

impl<'inp, L, E, C, Lang, Classifier> ParseInput<'inp, L, L::Token, E, C>
  for Expect<Classifier, Lang>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  E::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, inp: &mut InputRef<'inp, '_, L, E, C>) -> Result<L::Token, E::Error> {
    match inp.next() {
      Some(Spanned { span, data: tok }) => match tok {
        Lexed::Token(tok) => match self.is.check(&tok) {
          Ok(()) => Ok(tok),
          Err(expected) => Err(
            UnexpectedToken::with_expected(span, expected)
              .with_found(tok)
              .into(),
          ),
        },
        Lexed::Error(err) => Err(E::Error::from(err)),
      },
      None => Err(UnexpectedEot::eot(inp.span().end()).into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{DummyLexer, DummyToken};

  use super::*;

  fn assert_expect_parse_impl_with_all<'inp>()
  -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new()
      .with_cache::<()>(())
      .with_emitter(Fatal::new())
      .apply(Expect::new(|_tok: &DummyToken| Ok(())))
  }

  fn assert_expect_parse_impl_with_emitter<'inp>()
  -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new()
      .with_emitter(Fatal::new())
      .apply(Expect::new(|_tok: &DummyToken| Ok(())))
  }

  fn assert_expect_parse_impl_with_cache<'inp>()
  -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new()
      .with_cache::<()>(())
      .apply(Expect::new(|_tok: &DummyToken| Ok(())))
  }

  fn assert_expect_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new()
      .apply(Expect::new(|_tok: &DummyToken| Ok(())))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_expect_parse_impl();
    let _ = assert_expect_parse_impl_with_all();
    let _ = assert_expect_parse_impl_with_emitter();
    let _ = assert_expect_parse_impl_with_cache();
  }
}
