use mayber::{Owned, Ref};

use crate::{error::UnexpectedEot, lexer::KeywordToken, types::Keyword, utils::cmp::Equivalent};

use super::*;

impl Keyword<(), ()> {
  /// A parser that parses a token and returns an `Keyword` instance if matches.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not an keyword,
  /// and promise no valid token is consumed.
  pub fn try_parse<'inp, L, Ctx>(
    inp: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<Option<Keyword<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L>>::Error>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
  {
    Self::try_parse_of(inp)
  }

  /// A parser that parses a token and returns an `Keyword` instance if matches for a specific language.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not an keyword,
  /// and promise no valid token is consumed.
  pub fn try_parse_of<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Option<Keyword<L::Token, L::Span, Lang>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    let end = inp.cursor().as_inner().clone();
    let tok = inp.sync_until_token()?;

    match tok {
      None => Err(UnexpectedEot::eot_of(end).into()),
      Some(ct) => match ct {
        Owned(t) => {
          let (span, t) = t.into_token().into_components();
          if !t.is_keyword() {
            return Ok(None);
          }
          inp.skip_one();
          Ok(Some(Keyword::new(span.clone(), t)))
        }
        Ref(t) => {
          let t = t.into_token().into_data();
          if !t.is_keyword() {
            return Ok(None);
          }
          let Some(t) = inp.next_token()? else {
            panic!("Token was peeked but now missing");
          };
          let (span, t) = t.into_components();
          Ok(Some(Keyword::new(span, t)))
        }
      },
    }
  }

  /// A parser that parses a token and returns an `Keyword` instance if matches.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not an keyword,
  /// and promise no valid token is consumed.
  pub fn try_parse_sliced<'inp, L, Ctx>(
    inp: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<
    Option<Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>>,
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

  /// A parser that parses a token and returns an `Keyword` instance if matches for a specific language.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not an keyword,
  /// and promise no valid token is consumed.
  pub fn try_parse_sliced_of<'inp, L, Ctx, Lang: ?Sized>(
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<
    Option<Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    let end = inp.cursor().as_inner().clone();
    let tok = inp.sync_until_token()?;

    match tok {
      None => Err(UnexpectedEot::eot_of(end).into()),
      Some(ct) => {
        let (span, keyword) = ct
          .map(
            |t| {
              let (span, t) = t.into_token().into_components();
              (span.clone(), t.is_keyword())
            },
            |t| {
              let (span, t) = t.into_token().into_components();
              (span, t.is_keyword())
            },
          )
          .into_inner();

        if !keyword {
          return Ok(None);
        }

        inp.skip_one();
        Ok(Some(Keyword::new(span, inp.slice())))
      }
    }
  }

  /// A parser that parses a specific keyword and returns an `Keyword` instance if matches.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not the expected keyword,
  /// and promise no valid token is consumed.
  pub fn try_parse_exact<'inp, L, Ctx, Exp>(
    expected: &Exp,
  ) -> impl ParseInput<'inp, L, Option<Keyword<L::Token, L::Span>>, Ctx>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
    str: Equivalent<Exp>,
  {
    Self::try_parse_exact_of(expected)
  }

  /// A parser that parses a specific keyword and returns an `Keyword` instance if matches for a specific language.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not the expected keyword,
  /// and promise no valid token is consumed.
  pub fn try_parse_exact_of<'inp, L, Ctx, Exp, Lang: ?Sized>(
    expected: &Exp,
  ) -> impl ParseInput<'inp, L, Option<Keyword<L::Token, L::Span, Lang>>, Ctx, Lang>
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
    str: Equivalent<Exp>,
  {
    move |inp: &mut InputRef<'inp, '_, L, Ctx, Lang>| {
      let end = inp.cursor().as_inner().clone();
      let tok = inp.sync_until_token()?;

      match tok {
        None => Err(UnexpectedEot::eot_of(end).into()),
        Some(ct) => match ct {
          Owned(t) => {
            let (span, t) = t.into_token().into_components();
            match t.keyword() {
              Some(k) if k.equivalent(expected) => {}
              _ => return Ok(None),
            }
            inp.skip_one();

            Ok(Some(Keyword::new(span.clone(), t)))
          }
          Ref(t) => {
            let t = t.into_token().into_data();

            match t.keyword() {
              Some(k) if k.equivalent(expected) => {}
              _ => return Ok(None),
            }

            let Some(t) = inp.next_token()? else {
              panic!("Token was peeked but now missing");
            };
            let (span, t) = t.into_components();
            Ok(Some(Keyword::new(span, t)))
          }
        },
      }
    }
  }

  /// A parser that parses a specific keyword and returns an `Keyword` instance if matches.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not the expected keyword,
  /// and promise no valid token is consumed.
  pub fn try_parse_exact_sliced<'inp, L, Ctx, Exp>(
    expected: &Exp,
  ) -> impl ParseInput<
    'inp,
    L,
    Option<Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span>>,
    Ctx,
  >
  where
    L: Lexer<'inp>,
    L::Token: KeywordToken<'inp>,
    Ctx: ParseContext<'inp, L>,
    <Ctx::Emitter as Emitter<'inp, L>>::Error: From<UnexpectedEot<L::Offset>>,
    str: Equivalent<Exp>,
  {
    Self::try_parse_exact_sliced_of(expected)
  }

  /// A parser that parses a specific keyword and returns an `Keyword` instance if matches for a specific language.
  ///
  /// If the function returns `Ok(None)`, it means the next token is not the expected keyword,
  /// and promise no valid token is consumed.
  pub fn try_parse_exact_sliced_of<'inp, L, Ctx, Exp, Lang: ?Sized>(
    expected: &Exp,
  ) -> impl ParseInput<
    'inp,
    L,
    Option<Keyword<<L::Source as Source<L::Offset>>::Slice<'inp>, L::Span, Lang>>,
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
      let end = inp.cursor().as_inner().clone();
      let tok = inp.sync_until_token()?;

      match tok {
        None => Err(UnexpectedEot::eot_of(end).into()),
        Some(ct) => {
          let (span, keyword) = ct
            .map(
              |t| {
                let (span, t) = t.into_token().into_components();
                (
                  span.clone(),
                  t.keyword().is_some_and(|k| k.equivalent(expected)),
                )
              },
              |t| {
                let (span, t) = t.into_token().into_components();
                (span, t.keyword().is_some_and(|k| k.equivalent(expected)))
              },
            )
            .into_inner();

          if !keyword {
            return Ok(None);
          }

          inp.skip_one();
          Ok(Some(Keyword::new(span, inp.slice())))
        }
      }
    }
  }
}
