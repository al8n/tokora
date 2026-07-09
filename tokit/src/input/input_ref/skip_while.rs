use super::*;

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Consumes consecutive tokens matching `pred` without reporting them.
  ///
  /// Advances the cursor past every leading token for which `pred` returns
  /// `true`, stopping before the first token for which it returns `false` (that
  /// token is left unconsumed) or at end of input.
  ///
  /// Unlike [`sync_to`](Self::sync_to), the skipped tokens are **not** reported
  /// through `emit_unexpected_token`: they are expected and simply dropped.
  /// Genuine lexer errors encountered while skipping are still emitted, so a
  /// fatal emitter can abort on a malformed token. Already-cached (peeked)
  /// tokens are drained identically to freshly-lexed ones.
  ///
  /// This is the primitive used to skip trivia (whitespace, comments) in the
  /// `padded`, `padded_left`, and `padded_right` combinators, where trivia must
  /// be consumed but must never surface as an error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn skip_while<F>(
    &mut self,
    mut pred: F,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
  {
    // Drain matching tokens already sitting in the cache. `pop_front_if` stops
    // (and leaves the token in place) at the first non-matching token, so a
    // cached stopper — and anything peeked after it — is preserved.
    while let Some(tok) = self
      .cache
      .pop_front_if(|t| pred(Spanned::new(t.token().span(), t.token().data())))
    {
      let (lexed, state) = tok.into_components();
      let (span, _) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;
    }

    // If a non-matching token remains cached, the cursor is already positioned
    // before it and there is nothing left to lex.
    if !self.cache().is_empty() {
      return Ok(());
    }

    // Otherwise keep skipping straight from the lexer.
    let mut lexer = self.lexer();
    let mut end = self.span.clone();
    let mut state = self.state.clone();

    while let Some(Spanned { span, data: tok }) = Lexed::<L::Token>::lex_spanned(&mut lexer) {
      match tok {
        Lexed::Error(err) => match self.emit_lexer_error_deduped(Spanned::new(span, err)) {
          Ok(_) => {
            end = lexer.span();
            state = lexer.state().clone();
          }
          Err(e) => {
            self.set_span_after_consume(lexer.span().into());
            *self.state = lexer.into_state();
            return Err(e);
          }
        },
        Lexed::Token(tok) => {
          let tok = Spanned::new(span, tok);
          if pred(tok.as_ref()) {
            // Matching (e.g. trivia): consume it and keep going.
            end = lexer.span();
            state = lexer.state().clone();
          } else {
            // Non-matching: stop before it, leaving it unconsumed.
            self.set_span_after_consume(end.into());
            *self.state = state;
            return Ok(());
          }
        }
      }
    }

    // Reached end of input: everything from the cursor matched and was consumed.
    self.set_span_after_consume(lexer.span().into());
    *self.state = lexer.into_state();
    Ok(())
  }
}
