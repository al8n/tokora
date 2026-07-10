use super::*;

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Skip tokens until the predicate matches, emitting lexer errors along the way.
  ///
  /// If the predicate matches, the matching token is consumed and returned.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn sync_through<F, Exp>(
    &mut self,
    mut pred: F,
    mut exp: Exp,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
  {
    if let Some(tok) = self.sync_matched_in_cache(&mut pred, &mut exp)? {
      return Ok(Some(tok));
    }

    // sync_matched_in_cache skips non-matching tokens but leaves a matching
    // token in the cache (since pop_front_if doesn't pop when pred matches).
    // Check if the front of cache now matches and consume it.
    if !self.cache.is_empty() {
      if let Some(front) = self.cache.front() {
        let span = front.token().span();
        if pred(Spanned::new(span, front.token().data())) {
          if let Some(tok) = self.cache.pop_front() {
            let (lexed, state) = tok.into_components();
            let (span, tok) = lexed.into_components();
            self.set_span_after_consume((&span).into());
            *self.state = state;
            return Ok(Some(Spanned::new(span, tok)));
          }
        }
      }
    }

    // A sticky limit trip latches the input: no token remains to sync to, so
    // return `Ok(None)` (the end-of-input outcome) without rebuilding a lexer.
    if self.is_poisoned() {
      return Ok(None);
    }

    let mut lexer = self.lexer();

    while let Some(Spanned { span, data: tok }) = Lexed::<L::Token>::lex_spanned(&mut lexer) {
      match tok {
        Lexed::Error(err) => {
          // A limit trip is sticky: latch the input so re-entry cannot rescan.
          let limit_hit = self.latch_if_limit_tripped(&lexer);
          match self.emit_lexer_error_deduped(Spanned::new(span, err)) {
            Ok(_) => {
              if limit_hit {
                return Ok(None);
              }
            }
            Err(e) => {
              self.set_span_after_consume(lexer.span().into());
              *self.state = lexer.into_state();
              return Err(e);
            }
          }
        }
        Lexed::Token(tok) => {
          let tok = Spanned::new(span, tok);
          // if the token matches, we return it
          if pred(tok.as_ref()) {
            self.set_span_after_consume(tok.span_ref().into());
            *self.state = lexer.into_state();
            return Ok(Some(tok));
          } else {
            let (span, tok) = tok.into_components();
            self.emitter().emit_unexpected_token(
              UnexpectedToken::maybe_expected_of(span, exp()).with_found(tok),
            )?;
          }
        }
      }
    }

    Ok(None)
  }

  /// Skip tokens until the predicate matches, emitting lexer errors along the way.
  ///
  /// If the predicate matches, the matching token is consumed.
  /// Returns the matched token and peeked tokens after it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn sync_through_then_peek<'p, F, Exp, W>(
    &'p mut self,
    pred: F,
    exp: Exp,
  ) -> Result<
    (Option<Spanned<L::Token, L::Span>>, Peeked<'p, 'inp, L, W>),
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
    W: Window,
  {
    let (tok, peeked, _) = self.sync_through_then_peek_with_emitter::<_, _, W>(pred, exp)?;
    Ok((tok, peeked))
  }

  /// Skip tokens until the predicate matches, emitting lexer errors along the way.
  ///
  /// Returns the matched token, peeked tokens, and a mutable reference to the emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn sync_through_then_peek_with_emitter<'p, F, Exp, W>(
    &'p mut self,
    mut pred: F,
    mut exp: Exp,
  ) -> Result<
    (
      Option<Spanned<L::Token, L::Span>>,
      Peeked<'p, 'inp, L, W>,
      &'p mut Ctx::Emitter,
    ),
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
    W: Window,
  {
    if let Some(tok) = self.sync_matched_in_cache(&mut pred, &mut exp)? {
      let (peeked, emitter) = self.peek_with_emitter::<W>()?;
      return Ok((Some(tok), peeked, emitter));
    }

    // sync_matched_in_cache skips non-matching tokens but leaves a matching
    // token in the cache. Check if the front of cache now matches and consume it.
    if !self.cache.is_empty() {
      if let Some(front) = self.cache.front() {
        let span = front.token().span();
        if pred(Spanned::new(span, front.token().data())) {
          if let Some(tok) = self.cache.pop_front() {
            let (lexed, state) = tok.into_components();
            let (span, tok) = lexed.into_components();
            self.set_span_after_consume((&span).into());
            *self.state = state;
            let (peeked, emitter) = self.peek_with_emitter::<W>()?;
            return Ok((Some(Spanned::new(span, tok)), peeked, emitter));
          }
        }
      }
    }

    match !self.cache().is_empty() {
      // If the cache is non-empty but no match, peek remaining
      true => {
        let (peeked, emitter) = self.peek_with_emitter::<W>()?;
        Ok((None, peeked, emitter))
      }
      // Otherwise, let's skip the input
      false => {
        // A sticky limit trip latches the input: no token remains to sync to, so
        // return the empty result (the end-of-input outcome) without rebuilding a
        // lexer.
        if self.is_poisoned() {
          return Ok((None, GenericArrayDeque::new(), self.emitter));
        }

        let mut lexer = self.lexer();

        while let Some(Spanned { span, data: tok }) = Lexed::<L::Token>::lex_spanned(&mut lexer) {
          match tok {
            Lexed::Error(err) => {
              // A limit trip is sticky: latch the input so re-entry cannot rescan.
              let limit_hit = self.latch_if_limit_tripped(&lexer);
              match self.emit_lexer_error_deduped(Spanned::new(span, err)) {
                Ok(_) => {
                  if limit_hit {
                    return Ok((None, GenericArrayDeque::new(), self.emitter));
                  }
                }
                Err(e) => {
                  self.set_span_after_consume(lexer.span().into());
                  *self.state = lexer.into_state();
                  return Err(e);
                }
              }
            }
            Lexed::Token(tok) => {
              let tok = Spanned::new(span, tok);
              // if the token matches, we cache it and return it
              if pred(tok.as_ref()) {
                self.set_span_after_consume(tok.span_ref().into());
                *self.state = lexer.into_state();
                let (peeked, emitter) = self.peek_with_emitter::<W>()?;
                return Ok((Some(tok), peeked, emitter));
              } else {
                let (span, tok) = tok.into_components();
                self.emitter().emit_unexpected_token(
                  UnexpectedToken::maybe_expected_of(span, exp()).with_found(tok),
                )?;
              }
            }
          }
        }

        // No matched token found, we just update the cursor and state
        self.set_span_after_consume(lexer.span().into());
        *self.state = lexer.into_state();

        Ok((None, GenericArrayDeque::new(), self.emitter))
      }
    }
  }
}
