use crate::{
  error::UnexpectedEot, token::IdentifierToken, try_parse_input::ParseAttempt, types::Ident,
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
}

#[cfg(all(test, feature = "std", feature = "logos"))]
mod tests {
  use super::*;

  use crate::{
    ParserContext,
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
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum TokenKind {
    Ident,
  }

  impl core::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        TokenKind::Ident => write!(f, "identifier"),
      }
    }
  }

  impl TokenTrait<'_> for Token {
    type Kind = TokenKind;
    type Error = ();

    fn kind(&self) -> TokenKind {
      match self {
        Token::Ident => TokenKind::Ident,
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

  impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
    fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
      E::Lex
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
}
