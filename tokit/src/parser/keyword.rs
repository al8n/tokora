use crate::{
  ParseInput, TryParseInput,
  error::{UnexpectedEot, token::UnexpectedToken},
  span::Span,
  token::KeywordToken,
  try_parse_input::ParseAttempt,
  types::Keyword,
  utils::cmp::Equivalent,
};

use super::*;

impl Keyword<(), ()> {
  /// A parser that parses a token and returns a `Keyword` instance if it matches.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not a keyword,
  /// and promises no valid token is consumed.
  pub fn try_parse<'inp, L, Ctx>(
    inp: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<ParseAttempt<Keyword<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L>>::Error>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
  {
    Self::try_parse_of(inp)
  }

  /// A parser that parses a token and returns a `Keyword` instance if it matches for a specific language.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not a keyword,
  /// and promises no valid token is consumed.
  pub fn try_parse_of<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    ParseAttempt<Keyword<L::Token, L::Span, Lang>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    inp
      .try_expect(|t| t.into_data().is_keyword())
      .map(|opt_tok| {
        opt_tok
          .map(|tok| {
            let (span, t) = tok.into_components();
            Keyword::new(span, t)
          })
          .into()
      })
  }

  /// A parser that parses any keyword, erroring when the next token is not a
  /// keyword.
  ///
  /// Unlike [`try_parse`](Self::try_parse), a non-keyword token is converted
  /// into an [`UnexpectedToken`] error carrying the found token, and end of
  /// input into an [`UnexpectedEot`] error.
  pub fn parse<'inp, L, Ctx>(
    inp: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<Keyword<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L>>::Error>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>,
  {
    Self::parse_of(inp)
  }

  /// A parser that parses any keyword for a specific language, erroring when the
  /// next token is not a keyword.
  ///
  /// Unlike [`try_parse_of`](Self::try_parse_of), a non-keyword token is
  /// converted into an [`UnexpectedToken`] error carrying the found token, and
  /// end of input into an [`UnexpectedEot`] error.
  pub fn parse_of<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Keyword<L::Token, L::Span, Lang>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
  {
    match inp.next()? {
      Some(spanned) => {
        if spanned.data().is_keyword() {
          let (span, t) = spanned.into_components();
          Ok(Keyword::new(span, t))
        } else {
          let (span, tok) = spanned.into_components();
          Err(UnexpectedToken::of(span).with_found(tok).into())
        }
      }
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }

  /// A parser that parses a token and returns a `Keyword` instance if it matches.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not a keyword,
  /// and promises no valid token is consumed.
  pub fn try_parse_sliced<'inp, L, Ctx>(
    inp: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<
    ParseAttempt<Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
  {
    Self::try_parse_sliced_of(inp)
  }

  /// A parser that parses a token and returns a `Keyword` instance if it matches for a specific language.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not a keyword,
  /// and promises no valid token is consumed.
  pub fn try_parse_sliced_of<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    ParseAttempt<Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    inp
      .try_expect(|t| t.into_data().is_keyword())
      .map(|opt_tok| {
        opt_tok
          .map(|tok| Keyword::new(tok.into_span(), inp.slice()))
          .into()
      })
  }

  /// A parser that parses any keyword and returns its source slice, erroring
  /// when the next token is not a keyword.
  ///
  /// Unlike [`try_parse_sliced`](Self::try_parse_sliced), a non-keyword token is
  /// converted into an [`UnexpectedToken`] error carrying the found token, and
  /// end of input into an [`UnexpectedEot`] error.
  pub fn parse_sliced<'inp, L, Ctx>(
    inp: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<
    Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>,
  {
    Self::parse_sliced_of(inp)
  }

  /// A parser that parses any keyword for a specific language and returns its
  /// source slice, erroring when the next token is not a keyword.
  ///
  /// Unlike [`try_parse_sliced_of`](Self::try_parse_sliced_of), a non-keyword
  /// token is converted into an [`UnexpectedToken`] error carrying the found
  /// token, and end of input into an [`UnexpectedEot`] error.
  pub fn parse_sliced_of<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
  {
    match inp.next()? {
      Some(spanned) => {
        if spanned.data().is_keyword() {
          Ok(Keyword::new(spanned.into_span(), inp.slice()))
        } else {
          let (span, tok) = spanned.into_components();
          Err(UnexpectedToken::of(span).with_found(tok).into())
        }
      }
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }

  /// A parser that parses a specific keyword and returns a `Keyword` instance if it matches.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not the expected keyword,
  /// and promises no valid token is consumed.
  #[must_use]
  pub fn try_parse_exact<'inp, L, Ctx, Exp>(
    expected: &Exp,
  ) -> impl TryParseInput<'inp, L, Keyword<L::Token, L::Span>, Ctx>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
    str: Equivalent<Exp>,
  {
    Self::try_parse_exact_of(expected)
  }

  /// A parser that parses a specific keyword and returns a `Keyword` instance if it matches for a specific language.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not the expected keyword,
  /// and promises no valid token is consumed.
  #[must_use]
  pub fn try_parse_exact_of<'inp, L, Ctx, Exp, Lang: ?Sized>(
    expected: &Exp,
  ) -> impl TryParseInput<'inp, L, Keyword<L::Token, L::Span, Lang>, Ctx, Lang>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
    str: Equivalent<Exp>,
  {
    move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| {
      inp
        .try_expect(|t| {
          t.into_data()
            .keyword()
            .is_some_and(|k| k.equivalent(expected))
        })
        .map(|opt_tok| {
          opt_tok
            .map(|tok| {
              let (span, t) = tok.into_components();
              Keyword::new(span, t)
            })
            .into()
        })
    }
  }

  /// A parser that parses a specific keyword, erroring when the next token is
  /// not that keyword.
  ///
  /// Unlike [`try_parse_exact`](Self::try_parse_exact), an unexpected token is
  /// converted into an [`UnexpectedToken`] error carrying the found token, and
  /// end of input into an [`UnexpectedEot`] error.
  #[must_use]
  pub fn parse_exact<'inp, L, Ctx, Exp>(
    expected: &Exp,
  ) -> impl ParseInput<'inp, L, Keyword<L::Token, L::Span>, Ctx>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>,
    str: Equivalent<Exp>,
  {
    Self::parse_exact_of(expected)
  }

  /// A parser that parses a specific keyword, erroring when the next token is
  /// not that keyword, for a specific language.
  ///
  /// Unlike [`try_parse_exact_of`](Self::try_parse_exact_of), an unexpected
  /// token is converted into an [`UnexpectedToken`] error carrying the found
  /// token, and end of input into an [`UnexpectedEot`] error.
  #[must_use]
  pub fn parse_exact_of<'inp, L, Ctx, Exp, Lang: ?Sized>(
    expected: &Exp,
  ) -> impl ParseInput<'inp, L, Keyword<L::Token, L::Span, Lang>, Ctx, Lang>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
    str: Equivalent<Exp>,
  {
    move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| match inp.next()? {
      Some(spanned) => {
        if spanned
          .data()
          .keyword()
          .is_some_and(|k| k.equivalent(expected))
        {
          let (span, t) = spanned.into_components();
          Ok(Keyword::new(span, t))
        } else {
          let (span, tok) = spanned.into_components();
          Err(UnexpectedToken::of(span).with_found(tok).into())
        }
      }
      None => Err(UnexpectedEot::eot_of(inp.span().end()).into()),
    }
  }

  /// A parser that parses a specific keyword and returns a `Keyword` instance if it matches.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not the expected keyword,
  /// and promises no valid token is consumed.
  #[must_use]
  pub fn try_parse_exact_sliced<'inp, L, Ctx, Exp>(
    expected: &Exp,
  ) -> impl TryParseInput<'inp, L, Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>, Ctx>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
    str: Equivalent<Exp>,
  {
    Self::try_parse_exact_sliced_of(expected)
  }

  /// A parser that parses a specific keyword and returns a `Keyword` instance if it matches for a specific language.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not the expected keyword,
  /// and promises no valid token is consumed.
  #[must_use]
  pub fn try_parse_exact_sliced_of<'inp, L, Ctx, Exp, Lang: ?Sized>(
    expected: &Exp,
  ) -> impl TryParseInput<
    'inp,
    L,
    Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>,
    Ctx,
    Lang,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
    str: Equivalent<Exp>,
  {
    move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| {
      inp
        .try_expect(|t| {
          t.into_data()
            .keyword()
            .is_some_and(|k| k.equivalent(expected))
        })
        .map(|opt_tok| {
          opt_tok
            .map(|tok| Keyword::new(tok.into_span(), inp.slice()))
            .into()
        })
    }
  }

  /// A parser that parses a specific keyword and returns its source slice,
  /// erroring when the next token is not that keyword.
  ///
  /// Unlike [`try_parse_exact_sliced`](Self::try_parse_exact_sliced), an
  /// unexpected token is converted into an [`UnexpectedToken`] error carrying
  /// the found token, and end of input into an [`UnexpectedEot`] error.
  #[must_use]
  pub fn parse_exact_sliced<'inp, L, Ctx, Exp>(
    expected: &Exp,
  ) -> impl ParseInput<'inp, L, Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>, Ctx>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>,
    str: Equivalent<Exp>,
  {
    Self::parse_exact_sliced_of(expected)
  }

  /// A parser that parses a specific keyword for a specific language and returns
  /// its source slice, erroring when the next token is not that keyword.
  ///
  /// Unlike [`try_parse_exact_sliced_of`](Self::try_parse_exact_sliced_of), an
  /// unexpected token is converted into an [`UnexpectedToken`] error carrying
  /// the found token, and end of input into an [`UnexpectedEot`] error.
  #[must_use]
  pub fn parse_exact_sliced_of<'inp, L, Ctx, Exp, Lang: ?Sized>(
    expected: &Exp,
  ) -> impl ParseInput<
    'inp,
    L,
    Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>,
    Ctx,
    Lang,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
      + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
    str: Equivalent<Exp>,
  {
    move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| match inp.next()? {
      Some(spanned) => {
        if spanned
          .data()
          .keyword()
          .is_some_and(|k| k.equivalent(expected))
        {
          Ok(Keyword::new(spanned.into_span(), inp.slice()))
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
    ParseInput, ParserContext, SimpleSpan,
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
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum TokenKind {
    If,
    Else,
    Ident,
  }

  impl core::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        TokenKind::If => write!(f, "if"),
        TokenKind::Else => write!(f, "else"),
        TokenKind::Ident => write!(f, "identifier"),
      }
    }
  }

  impl TokenTrait<'_> for Token {
    type Kind = TokenKind;
    type Error = ();

    fn kind(&self) -> TokenKind {
      match self {
        Token::If => TokenKind::If,
        Token::Else => TokenKind::Else,
        Token::Ident => TokenKind::Ident,
      }
    }

    fn is_trivia(&self) -> bool {
      false
    }
  }

  impl KeywordToken<'_> for Token {
    fn keyword(&self) -> Option<&'static str> {
      match self {
        Token::If => Some("if"),
        Token::Else => Some("else"),
        Token::Ident => None,
      }
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
      Err(E::Unexpected { found: None })
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

  #[test]
  fn parse_exact_of_accepts_matching_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<Token, SimpleSpan>, E> {
      Keyword::parse_exact_of(&"if").parse_input(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("if");
    let (span, tok) = r.unwrap().into_components();
    assert_eq!(tok, Token::If);
    assert_eq!(span, SimpleSpan::new(0, 2));
  }

  #[test]
  fn parse_exact_of_errors_on_wrong_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<Token, SimpleSpan>, E> {
      Keyword::parse_exact_of(&"if").parse_input(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("else");
    assert_eq!(
      r.unwrap_err(),
      E::Unexpected {
        found: Some(TokenKind::Else)
      }
    );
  }

  #[test]
  fn parse_exact_of_errors_on_empty_input() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<Token, SimpleSpan>, E> {
      Keyword::parse_exact_of(&"if").parse_input(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("");
    assert_eq!(r.unwrap_err(), E::Eot);
  }

  #[test]
  fn parse_exact_accepts_matching_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<Token, SimpleSpan>, E> {
      Keyword::parse_exact(&"if").parse_input(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("if");
    let (span, tok) = r.unwrap().into_components();
    assert_eq!(tok, Token::If);
    assert_eq!(span, SimpleSpan::new(0, 2));
  }

  // The sliced keyword payload must be the current keyword's text, so parsing
  // two keywords in a row yields each keyword's own slice, not the prefix.
  #[test]
  fn try_parse_sliced_slices_each_current_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<(&'inp str, &'inp str), E> {
      let first = Keyword::try_parse_sliced(inp)?.unwrap_accept();
      let second = Keyword::try_parse_sliced(inp)?.unwrap_accept();
      Ok((first.source(), second.source()))
    }
    let r = Parser::with_context(ctx())
      .apply(parse)
      .parse_str("if else");
    assert_eq!(r.unwrap(), ("if", "else"));
  }

  #[test]
  fn parse_of_accepts_any_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<Token, SimpleSpan>, E> {
      Keyword::parse_of(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("else");
    let (span, tok) = r.unwrap().into_components();
    assert_eq!(tok, Token::Else);
    assert_eq!(span, SimpleSpan::new(0, 4));
  }

  #[test]
  fn parse_of_errors_on_non_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<Token, SimpleSpan>, E> {
      Keyword::parse_of(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("foo");
    assert_eq!(
      r.unwrap_err(),
      E::Unexpected {
        found: Some(TokenKind::Ident)
      }
    );
  }

  #[test]
  fn parse_of_errors_on_empty_input() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<Token, SimpleSpan>, E> {
      Keyword::parse_of(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("");
    assert_eq!(r.unwrap_err(), E::Eot);
  }

  #[test]
  fn parse_accepts_any_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<Token, SimpleSpan>, E> {
      Keyword::parse(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("if");
    let (span, tok) = r.unwrap().into_components();
    assert_eq!(tok, Token::If);
    assert_eq!(span, SimpleSpan::new(0, 2));
  }

  #[test]
  fn parse_sliced_of_accepts_any_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<&'inp str, SimpleSpan>, E> {
      Keyword::parse_sliced_of(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("else");
    let (span, source) = r.unwrap().into_components();
    assert_eq!(source, "else");
    assert_eq!(span, SimpleSpan::new(0, 4));
  }

  #[test]
  fn parse_sliced_of_errors_on_non_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<&'inp str, SimpleSpan>, E> {
      Keyword::parse_sliced_of(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("foo");
    assert_eq!(
      r.unwrap_err(),
      E::Unexpected {
        found: Some(TokenKind::Ident)
      }
    );
  }

  #[test]
  fn parse_sliced_of_errors_on_empty_input() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<&'inp str, SimpleSpan>, E> {
      Keyword::parse_sliced_of(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("");
    assert_eq!(r.unwrap_err(), E::Eot);
  }

  #[test]
  fn parse_sliced_accepts_any_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<&'inp str, SimpleSpan>, E> {
      Keyword::parse_sliced(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("if");
    assert_eq!(r.unwrap().source(), "if");
  }

  #[test]
  fn parse_exact_sliced_of_accepts_matching_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<&'inp str, SimpleSpan>, E> {
      Keyword::parse_exact_sliced_of(&"if").parse_input(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("if");
    let (span, source) = r.unwrap().into_components();
    assert_eq!(source, "if");
    assert_eq!(span, SimpleSpan::new(0, 2));
  }

  #[test]
  fn parse_exact_sliced_of_errors_on_wrong_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<&'inp str, SimpleSpan>, E> {
      Keyword::parse_exact_sliced_of(&"if").parse_input(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("else");
    assert_eq!(
      r.unwrap_err(),
      E::Unexpected {
        found: Some(TokenKind::Else)
      }
    );
  }

  #[test]
  fn parse_exact_sliced_of_errors_on_empty_input() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<&'inp str, SimpleSpan>, E> {
      Keyword::parse_exact_sliced_of(&"if").parse_input(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("");
    assert_eq!(r.unwrap_err(), E::Eot);
  }

  #[test]
  fn parse_exact_sliced_accepts_matching_keyword() {
    fn parse<'inp>(
      inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, TestEm>>,
    ) -> Result<Keyword<&'inp str, SimpleSpan>, E> {
      Keyword::parse_exact_sliced(&"if").parse_input(inp)
    }
    let r = Parser::with_context(ctx()).apply(parse).parse_str("if");
    assert_eq!(r.unwrap().source(), "if");
  }
}
