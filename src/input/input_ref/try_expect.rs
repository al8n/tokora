use super::*;

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Ok`, the token is consumed and returned.
  /// Otherwise, the error is returned and the token remains in the cache.
  pub fn try_expect<F>(
    &mut self,
    mut pred: F,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
  {
    let (exhausted, tok) = self.try_expect_in_cache(&mut pred)?;

    if !exhausted {
      return Ok(tok);
    }

    match tok {
      // found the token in cache
      Some(tok) => Ok(Some(tok)),
      // need to lex from input
      None => self.try_expect_on_input(pred),
    }
  }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Ok`, the token is consumed and returned.
  /// Otherwise, the error is returned and the token remains in the cache.
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
    let (exhausted, tok) = self.try_expect_map_in_cache(&mut pred)?;

    if !exhausted {
      return Ok(tok);
    }

    match tok {
      // found the token in cache
      Some(tok) => Ok(Some(tok)),
      // need to lex from input
      None => self.try_expect_map_on_input(pred),
    }
  }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Ok`, the token is consumed and returned.
  /// Otherwise, the error is returned and the token remains in the cache.
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
    let (exhausted, tok) = self.try_expect_and_then_in_cache(&mut pred)?;

    if !exhausted {
      return Ok(tok);
    }

    match tok {
      // found the token in cache
      Some(tok) => Ok(Some(tok)),
      // need to lex from input
      None => self.try_expect_and_then_on_input(pred),
    }
  }

  /// Internal implementation for syncing tokens in the cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_expect_and_then_in_cache<O, P>(
    &mut self,
    mut pred: P,
  ) -> Result<
    (bool, Option<(O, Spanned<L::Token, L::Span>)>),
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    P: FnMut(
      Spanned<&L::Token, &L::Span>,
    ) -> Option<Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>>,
  {
    // pop from cache if not matching
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

      // Note: cursor/state are updated before emission. If emission fails,
      // the error token has still been consumed (no backtracking here).

      return match output {
        Some(res) => Ok((false, Some((res?, Spanned::new(span, tok))))),
        None => Ok((false, None)),
      };
    }
    Ok((true, None))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_expect_in_cache<P>(
    &mut self,
    mut pred: P,
  ) -> Result<
    (bool, Option<Spanned<L::Token, L::Span>>),
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    P: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
  {
    // pop from cache if not matching
    if let Some(tok) = self.cache.pop_front_if(|t| pred(t.token)) {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;

      // Note: cursor/state are updated before emission. If emission fails,
      // the error token has still been consumed (no backtracking here).

      return Ok((false, Some(Spanned::new(span, tok))));
    }
    Ok((true, None))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_expect_map_in_cache<O, P>(
    &mut self,
    mut pred: P,
  ) -> Result<
    (bool, Option<(O, Spanned<L::Token, L::Span>)>),
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    P: FnMut(Spanned<&L::Token, &L::Span>) -> Option<O>,
  {
    // pop from cache if not matching
    let mut output = None;
    if let Some(tok) = self.cache.pop_front_if(|t| match pred(t.token().copied()) {
      Some(out) => {
        output = Some(out);
        true
      }
      None => false,
    }) {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;
      return Ok((false, output.map(|out| (out, Spanned::new(span, tok)))));
    }
    Ok((true, None))
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
