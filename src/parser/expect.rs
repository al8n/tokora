use crate::{
  Check,
  error::{UnexpectedEot, token::UnexpectedToken},
  lexer::Span,
};

use super::*;

/// A parser that expects a specific token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Expect<Classifier, Ctx, Lang: ?Sized = ()> {
  is: Classifier,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<Classifier, Ctx> Expect<Classifier, Ctx> {
  /// Creates a parser that accepts a specific token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(classifier: Classifier) -> Self {
    Self::of(classifier)
  }

  /// Creates a parser that yields specific token with its span
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn spanned(classifier: Classifier) -> With<Self, PhantomSpan> {
    Self::spanned_of(classifier)
  }

  /// Creates a parser that yields specific token with its source
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn sliced(classifier: Classifier) -> With<Self, PhantomSliced> {
    Self::sliced_of(classifier)
  }

  /// Creates a parser that yields specific token without its source and span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn located(classifier: Classifier) -> With<Self, PhantomLocated> {
    Self::located_of(classifier)
  }
}

impl<Classifier, Ctx, Lang> Expect<Classifier, Ctx, Lang> {
  /// Creates a parser that accepts a specific token of a specific language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(classifier: Classifier) -> Self {
    Self {
      is: classifier,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Creates a parser that yields specific token with its span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn spanned_of(classifier: Classifier) -> With<Self, PhantomSpan> {
    With::new(Self::of(classifier), PhantomSpan::PHANTOM)
  }

  /// Creates a parser that yields specific token with its source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn sliced_of(classifier: Classifier) -> With<Self, PhantomSliced> {
    With::new(Self::of(classifier), PhantomSliced::PHANTOM)
  }

  /// Creates a parser that yields specific token without its source and span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn located_of(classifier: Classifier) -> With<Self, PhantomLocated> {
    With::new(Self::of(classifier), PhantomLocated::PHANTOM)
  }
}

impl<'inp, L, Ctx, Lang, Classifier> ParseInput<'inp, L, L::Token, Ctx, Lang>
  for Expect<Classifier, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<L::Token, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    match inp.next() {
      Some(Spanned { span, data: tok }) => match tok {
        Lexed::Token(tok) => match self.is.check(&tok) {
          Ok(()) => Ok(tok),
          Err(expected) => Err(
            UnexpectedToken::with_expected_of(span, expected)
              .with_found(tok)
              .into(),
          ),
        },
        Lexed::Error(err) => Err(From::from(err)),
      },
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

impl<'inp, L, Ctx, Lang, Classifier> ParseInput<'inp, L, Spanned<L::Token, L::Span>, Ctx, Lang>
  for With<Expect<Classifier, Ctx, Lang>, PhantomSpan>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    match inp.next() {
      Some(Spanned { span, data: tok }) => match tok {
        Lexed::Token(tok) => match self.primary.is.check(&tok) {
          Ok(()) => Ok(Spanned::new(span, tok)),
          Err(expected) => Err(
            UnexpectedToken::with_expected_of(span, expected)
              .with_found(tok)
              .into(),
          ),
        },
        Lexed::Error(err) => Err(From::from(err)),
      },
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

impl<'inp, L, Ctx, Lang, Classifier>
  ParseInput<'inp, L, Sliced<L::Token, <L::Source as Source<L::Offset>>::Slice<'inp>>, Ctx, Lang>
  for With<Expect<Classifier, Ctx, Lang>, PhantomSliced>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Sliced<L::Token, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    match inp.next() {
      Some(Spanned { span, data: tok }) => match tok {
        Lexed::Token(tok) => match self.primary.is.check(&tok) {
          Ok(()) => Ok(Sliced::new(
            inp
              .slice()
              .expect("lexer gurantees there must be a valid slice to yield a token"),
            tok,
          )),
          Err(expected) => Err(
            UnexpectedToken::with_expected_of(span, expected)
              .with_found(tok)
              .into(),
          ),
        },
        Lexed::Error(err) => Err(From::from(err)),
      },
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

impl<'inp, L, Ctx, Lang, Classifier>
  ParseInput<
    'inp,
    L,
    Located<L::Token, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    Ctx,
    Lang,
  > for With<Expect<Classifier, Ctx, Lang>, PhantomLocated>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>
    + From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedEot<L::Offset, Lang>>,
  Classifier: Check<L::Token, Result<(), Expected<'inp, <L::Token as Token<'inp>>::Kind>>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Located<L::Token, L::Span, <L::Source as Source<L::Offset>>::Slice<'inp>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    match inp.next() {
      Some(Spanned { span, data: tok }) => match tok {
        Lexed::Token(tok) => match self.primary.is.check(&tok) {
          Ok(()) => Ok(Located::new(
            inp
              .slice()
              .expect("lexer gurantees there must be a valid slice to yield a token"),
            span,
            tok,
          )),
          Err(expected) => Err(
            UnexpectedToken::with_expected_of(span, expected)
              .with_found(tok)
              .into(),
          ),
        },
        Lexed::Error(err) => Err(From::from(err)),
      },
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_expect_parse_impl_with_ctx<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::with_context(()).apply(Expect::new(|_tok: &DummyToken| Ok(())))
  }

  fn assert_expect_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Expect::new(|_tok: &DummyToken| Ok(())))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_expect_parse_impl();
    let _ = assert_expect_parse_impl_with_ctx();
  }
}
