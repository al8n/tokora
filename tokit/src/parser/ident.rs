use crate::{
  error::{UnexpectedEot, token::UnexpectedToken},
  span::Span,
  token::IdentifierToken,
  try_parse_input::ParseAttempt,
  types::Ident,
};

use super::*;

impl Ident<(), ()> {
  /// A parser that parses a token and returns an `Ident` instance if it matches.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not an identifier,
  /// and promises no valid token is consumed.
  pub fn try_parse<'inp, L, Ctx>(
    inp: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<
    ParseAttempt<Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: IdentifierToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
  {
    Self::try_parse_of(inp)
  }

  /// A parser that parses a token and returns an `Ident` instance if it matches for a specific language.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not an identifier,
  /// and promises no valid token is consumed.
  pub fn try_parse_of<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    ParseAttempt<Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: IdentifierToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    inp.try_expect(|t| t.data.is_identifier()).map(|res| {
      res
        .map(|tok| Ident::new(tok.into_span(), inp.slice()))
        .into()
    })
  }

  /// A parser that parses an identifier, erroring when the next token is not an
  /// identifier.
  ///
  /// Unlike [`try_parse`](Self::try_parse), a non-identifier token is converted
  /// into an [`UnexpectedToken`] error carrying the found token, and end of
  /// input into an [`UnexpectedEot`] error.
  pub fn parse<'inp, L, Ctx>(
    inp: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<
    Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: IdentifierToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>,
  {
    Self::parse_of(inp)
  }

  /// A parser that parses an identifier for a specific language, erroring when
  /// the next token is not an identifier.
  ///
  /// Unlike [`try_parse_of`](Self::try_parse_of), a non-identifier token is
  /// converted into an [`UnexpectedToken`] error carrying the found token, and
  /// end of input into an [`UnexpectedEot`] error.
  pub fn parse_of<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Ident<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: IdentifierToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
  {
    match inp.next()? {
      Some(spanned) => {
        if spanned.data().is_identifier() {
          Ok(Ident::new(spanned.into_span(), inp.slice()))
        } else {
          let (span, tok) = spanned.into_components();
          Err(UnexpectedToken::of(span).with_found(tok).into())
        }
      }
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }
}

#[cfg(all(test, feature = "std", feature = "logos"))]
mod tests {
  use super::*;

  use crate::{
    ParserContext, SimpleSpan,
    error::token::{UnexpectedToken, UnexpectedTokenOf},
    input::Cursor,
    lexer::LogosLexer,
    logos::{self, Logos},
    span::Spanned,
    token::Token as TokenTrait,
  };

  #[derive(Debug, Clone, Logos, PartialEq)]
  #[logos(crate = logos, skip r"[ \t\r\n]+")]
  enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
    #[regex(r"[0-9]+")]
    Num,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum TokenKind {
    Ident,
    Num,
  }

  impl core::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        TokenKind::Ident => write!(f, "identifier"),
        TokenKind::Num => write!(f, "number"),
      }
    }
  }

  impl TokenTrait<'_> for Token {
    type Kind = TokenKind;
    type Error = ();

    fn kind(&self) -> TokenKind {
      match self {
        Token::Ident => TokenKind::Ident,
        Token::Num => TokenKind::Num,
      }
    }

    fn is_trivia(&self) -> bool {
      false
    }
  }

  impl IdentifierToken<'_> for Token {
    fn is_identifier(&self) -> bool {
      matches!(self, Token::Ident)
    }
  }

  type TestLexer<'a> = LogosLexer<'a, Token>;

  #[derive(Debug, PartialEq)]
  enum E {
    Lex,
    Eot,
    Unexpected { found: Option<TokenKind> },
  }

  impl From<()> for E {
    fn from(_: ()) -> Self {
      E::Lex
    }
  }

  impl<O, Lang: ?Sized> From<UnexpectedEot<O, Lang>> for E {
    fn from(_: UnexpectedEot<O, Lang>) -> Self {
      E::Eot
    }
  }

  impl<'a, S, Lang: ?Sized> From<UnexpectedToken<'a, Token, TokenKind, S, Lang>> for E {
    fn from(err: UnexpectedToken<'a, Token, TokenKind, S, Lang>) -> Self {
      let (_span, found, _expected) = err.into_components();
      E::Unexpected {
        found: found.map(|t| t.kind()),
      }
    }
  }

  struct TestEm;

  impl<'inp> Emitter<'inp, TestLexer<'inp>> for TestEm {
    type Error = E;

    fn emit_lexer_error(
      &mut self,
      _: Spanned<
        <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
        <TestLexer<'inp> as Lexer<'inp>>::Span,
      >,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(E::Lex)
    }

    fn emit_unexpected_token(
      &mut self,
      _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(E::Lex)
    }

    fn emit_error(
      &mut self,
      err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>,
    ) -> Result<(), E>
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
      Err(err.into_data())
    }

    fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>)
    where
      TestLexer<'inp>: Lexer<'inp>,
    {
    }
  }

  fn ctx() -> ParserContext<'static, TestLexer<'static>, TestEm> {
    ParserContext::new(TestEm)
  }

  // Parsing two identifiers in a row must yield each identifier's own text.
  // `slice()` returns the current token, not the accumulated consumed prefix.
  #[test]
  fn try_parse_twice_slices_each_current_ident() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<(&'inp str, &'inp str), E> {
      let first = Ident::try_parse(inp)?.unwrap_accept();
      let second = Ident::try_parse(inp)?.unwrap_accept();
      Ok((first.source(), second.source()))
    }
    let r = Parser::with_context(ctx())
      .apply(parse)
      .parse_str("foo bar");
    assert_eq!(r.unwrap(), ("foo", "bar"));
  }

  #[test]
  fn parse_of_accepts_identifier() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Ident<&'inp str, SimpleSpan>, E> {
      Ident::parse_of(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("foo");
    let ident = r.unwrap();
    assert_eq!(ident.source(), "foo");
    assert_eq!(ident.span(), SimpleSpan::new(0, 3));
  }

  #[test]
  fn parse_of_errors_on_non_identifier() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Ident<&'inp str, SimpleSpan>, E> {
      Ident::parse_of(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("123");
    assert_eq!(
      r.unwrap_err(),
      E::Unexpected {
        found: Some(TokenKind::Num)
      }
    );
  }

  #[test]
  fn parse_of_errors_on_empty_input() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Ident<&'inp str, SimpleSpan>, E> {
      Ident::parse_of(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("");
    assert_eq!(r.unwrap_err(), E::Eot);
  }

  #[test]
  fn parse_accepts_identifier() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Ident<&'inp str, SimpleSpan>, E> {
      Ident::parse(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("bar");
    assert_eq!(r.unwrap().source(), "bar");
  }
}
