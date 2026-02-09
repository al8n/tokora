use super::*;

use crate::{
  error::UnexpectedEot,
  token::{PunctuatorToken, PunctuatorTokenExt, SpannedPunctuatorToken},
};

macro_rules! try_expect_punct {
  ($($punct:ident $(:$alias:ident)? :$punct_char:literal),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Tries to advance to the next valid token if it is " $punct " (" $punct_char "). Otherwise leaves the input unchanged."]
        #[cfg_attr(not(tarpaulin), inline(always))]
        pub fn [< try_expect_ $punct >](
          &mut self,
        ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
        where
          L::Token: crate::token::PunctuatorToken<'inp>,
        {
          self.try_expect(|t| t.data.[<is_ $punct>]())
        }

        #[doc = "Advances to the next valid token and expects it to be " $punct " (" $punct_char ")."]
        #[cfg_attr(not(tarpaulin), inline(always))]
        pub fn [< expect_ $punct >](
          &mut self,
        ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
        where
          L::Token: PunctuatorToken<'inp>,
          <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
            + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
        {
          match self.next()? {
            Some(spanned) => {
              <Spanned<L::Token, L::Span> as SpannedPunctuatorToken<'inp, L, Lang>>::[< expect_ $punct >](spanned).map_err(Into::into)
            },
            None => Err(UnexpectedEot::eot_of(self.span().end()).into()),
          }
        }

        $(
          #[doc = "Tries to advance to the next valid token if it is " $alias " (" $punct_char "). Otherwise leaves the input unchanged."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub fn [< try_expect_ $alias >](
            &mut self,
          ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
          where
            L::Token: PunctuatorToken<'inp>,
          {
            self.[< try_expect_ $punct >]()
          }

          #[doc = "Advances to the next valid token and expects it to be " $alias " (" $punct_char ")."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub fn [< expect_ $alias >](
            &mut self,
          ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
          where
            L::Token: PunctuatorToken<'inp>,
            <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>
              + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span, Lang>>,
          {
            self.[< expect_ $punct >]()
          }
        )?
      )*
    }
  };
}

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  try_expect_punct!(
    open_angle:"<",
    close_angle:">",
    open_brace:"{",
    close_brace:"}",
    open_paren:"(",
    close_paren:")",
    open_bracket:"[",
    close_bracket:"]",
    comma:",",
    semicolon:";",
    colon:":",
    dot:".",
    tilde:"~",
    underscore:"_",
    equal:"=",
    minus:hyphen:"-",
    arrow:thin_arrow:"->",
    fat_arrow:"=>",
    pipe_arrow:pipe_forward:"|>",
    double_colon:"::",
    colon_eq:colon_assign:":=",
    tab:"\t",
    newline:"\n",
    carriage_return:"\r",
    crlf:"\r\n",
    space:" ",
    pipe:"|",
    ampersand:"&",
    percent:"%",
    slash:"/",
    backslash:"\\",
    dollar:"$",
    hash:"#",
    at:"@",
    asterisk:"*",
    apostrophe:"'",
    double_quote:"\"",
    plus:"+",
    exclamation:bang:"!",
    question:"?",
    backtick:"`",
    caret:"^",
  );

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `true`, the token is consumed and returned.
  /// Otherwise, the token remains in the cache and `Ok(None)` is returned.
  pub fn try_expect<F>(
    &mut self,
    mut pred: F,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
  {
    // if cache is empty, directly try expect on input
    if self.cache.is_empty() {
      return self.try_expect_on_input(pred);
    }

    // pop from cache if matching
    Ok(self.cache.pop_front_if(|t| pred(t.token)).map(|tok| {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;

      Spanned::new(span, tok)
    }))
  }

  // /// Advances to the next valid token and expects it to satisfy the predicate.
  // ///
  // /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  // /// If `pred` returns `Ok`, the token is consumed and returned.
  // /// Otherwise, the error is returned and the token remains in the cache.
  // pub fn try_expect_either<F>(
  //   &mut self,
  //   mut pred: F,
  // ) -> Result<Option<Either<Spanned<L::Token, L::Span>, Spanned<L::Token, L::Span>>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  // where
  //   F: FnMut(Spanned<&L::Token, &L::Span>) -> Option<Either<(), ()>>,
  // {
  //   // if cache is empty, directly try expect on input
  //   if self.cache.is_empty() {
  //     return self.try_expect_on_input(pred);
  //   }

  //   // pop from cache if matching
  //   let mut is_left = false;
  //   Ok(self.cache.pop_front_if(|t| {
  //     match pred(t.token) {
  //       Some(Either::Left(_)) => {
  //         is_left = true;
  //         true
  //       }
  //       Some(Either::Right(_)) => {
  //         true
  //       }
  //       None => false,
  //     }
  //   }).map(|tok| {
  //     let (lexed, state) = tok.into_components();
  //     let (span, tok) = lexed.into_components();
  //     self.set_span_after_consume((&span).into());
  //     *self.state = state;

  //     if is_left {
  //       Either::Left(Spanned::new(span, tok))
  //     } else {
  //       Either::Right(Spanned::new(span, tok))
  //     }
  //   }))
  // }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Some(output)`, the token is consumed and `(output, token)` is returned.
  /// If `pred` returns `None`, the token remains in the cache and `Ok(None)` is returned.
  pub fn try_expect_map<O, F>(
    &mut self,
    mut pred: F,
  ) -> Result<
    Option<(O, Spanned<L::Token, L::Span>)>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> Option<O>,
  {
    // if cache is empty, directly try expect on input
    if self.cache.is_empty() {
      return self.try_expect_map_on_input(pred);
    }

    let mut output = None;
    Ok(
      self
        .cache
        .pop_front_if(|t| match pred(t.token().copied()) {
          Some(out) => {
            output = Some(out);
            true
          }
          None => false,
        })
        .map(|tok| {
          let (lexed, state) = tok.into_components();
          let (span, tok) = lexed.into_components();
          self.set_span_after_consume((&span).into());
          *self.state = state;
          (output.unwrap(), Spanned::new(span, tok))
        }),
    )
  }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Some(Ok(output))`, the token is consumed and `(output, token)` is returned.
  /// If `pred` returns `Some(Err(error))`, the token is consumed and `Err(error)` is returned.
  /// If `pred` returns `None`, the token remains in the cache and `Ok(None)` is returned.
  pub fn try_expect_and_then<O, F>(
    &mut self,
    mut pred: F,
  ) -> Result<
    Option<(O, Spanned<L::Token, L::Span>)>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(
      Spanned<&L::Token, &L::Span>,
    ) -> Option<Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>>,
  {
    // if cache is empty, directly try expect on input
    if self.cache.is_empty() {
      return self.try_expect_and_then_on_input(pred);
    }

    let mut output = None;
    if let Some(tok) = self.cache.pop_front_if(|t| match pred(t.token().copied()) {
      Some(res) => {
        output = Some(res);
        true
      }
      None => false,
    }) {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;

      return match output {
        Some(res) => res.map(|o| Some((o, Spanned::new(span, tok)))),
        None => Ok(None),
      };
    }

    Ok(None)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_expect_and_then_on_input<O, F>(
    &mut self,
    mut pred: F,
  ) -> Result<
    Option<(O, Spanned<L::Token, L::Span>)>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(
      Spanned<&L::Token, &L::Span>,
    ) -> Option<Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>>,
  {
    let mut lexer = self.lexer();

    while let Some(Spanned { span, data: tok }) = Lexed::<L::Token>::lex_spanned(&mut lexer) {
      match tok {
        Lexed::Error(err) => match self.emitter().emit_lexer_error(Spanned::new(span, err)) {
          Ok(_) => {}
          Err(e) => {
            self.set_span_after_consume(lexer.span().into());
            *self.state = lexer.into_state();
            return Err(e);
          }
        },
        Lexed::Token(tok) => {
          let tok = Spanned::new(span, tok);
          // if the token matches, we return it
          match pred(tok.as_ref()) {
            Some(output) => {
              self.set_span_after_consume(tok.span_ref().into());
              *self.state = lexer.into_state();
              return output.map(|o| Some((o, tok)));
            }
            None => {
              let (span, tok) = tok.into_components();
              // put back the token into cache as it was peeked
              let ct = CachedToken::new(Spanned::new(span, tok), lexer.state().clone());
              let _ = self.cache_mut().push_back(ct);
              return Ok(None);
            }
          }
        }
      }
    }

    Ok(None)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_expect_on_input<F>(
    &mut self,
    mut pred: F,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
  {
    let mut lexer = self.lexer();

    while let Some(Spanned { span, data: tok }) = Lexed::<L::Token>::lex_spanned(&mut lexer) {
      match tok {
        Lexed::Error(err) => match self.emitter().emit_lexer_error(Spanned::new(span, err)) {
          Ok(_) => {}
          Err(e) => {
            self.set_span_after_consume(lexer.span().into());
            *self.state = lexer.into_state();
            return Err(e);
          }
        },
        Lexed::Token(tok) => {
          let tok = Spanned::new(span, tok);
          // if the token matches, we return it
          if pred(tok.as_ref()) {
            self.set_span_after_consume(tok.span_ref().into());
            *self.state = lexer.into_state();
            return Ok(Some(tok));
          } else {
            let (span, tok) = tok.into_components();
            // put back the token into cache as it was peeked
            let ct = CachedToken::new(Spanned::new(span, tok), lexer.state().clone());
            let _ = self.cache_mut().push_back(ct);
            return Ok(None);
          }
        }
      }
    }

    Ok(None)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_expect_map_on_input<O, F>(
    &mut self,
    mut pred: F,
  ) -> Result<
    Option<(O, Spanned<L::Token, L::Span>)>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> Option<O>,
  {
    let mut lexer = self.lexer();

    while let Some(Spanned { span, data: tok }) = Lexed::<L::Token>::lex_spanned(&mut lexer) {
      match tok {
        Lexed::Error(err) => match self.emitter().emit_lexer_error(Spanned::new(span, err)) {
          Ok(_) => {}
          Err(e) => {
            self.set_span_after_consume(lexer.span().into());
            *self.state = lexer.into_state();
            return Err(e);
          }
        },
        Lexed::Token(tok) => {
          let tok = Spanned::new(span, tok);
          // if the token matches, we return it
          if let Some(out) = pred(tok.as_ref()) {
            self.set_span_after_consume(tok.span_ref().into());
            *self.state = lexer.into_state();
            return Ok(Some((out, tok)));
          } else {
            let (span, tok) = tok.into_components();
            // put back the token into cache as it was peeked
            let ct = CachedToken::new(Spanned::new(span, tok), lexer.state().clone());
            let _ = self.cache_mut().push_back(ct);
            return Ok(None);
          }
        }
      }
    }

    Ok(None)
  }
}
