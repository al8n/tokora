use crate::{
  TryParseInput, error::UnexpectedEot, token::KeywordToken, try_parse_input::ParseAttempt,
  types::Keyword, utils::cmp::Equivalent,
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

  /// A parser that parses a specific keyword and returns a `Keyword` instance if it matches.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not the expected keyword,
  /// and promises no valid token is consumed.
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

  /// A parser that parses a specific keyword and returns a `Keyword` instance if it matches.
  ///
  /// If the function returns `Ok(ParseAttempt::Decline)`, it means the next token is not the expected keyword,
  /// and promises no valid token is consumed.
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
}
