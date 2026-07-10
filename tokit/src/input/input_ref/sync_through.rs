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
  ///
  /// Diagnostics travel with progress: a match (or a resource-limit trip)
  /// commits the skipped prefix, so the diagnostics describing it persist. A
  /// no-match run to end of input commits nothing — the cursor stays at the
  /// pre-call position — and leaves no trace: the emissions made during the
  /// failed scan are unwound and the lexer-error deduplication watermark is
  /// restored, so a later genuine consume of the same region reports its
  /// errors exactly once.
  ///
  /// This holds even when the caller had prefilled the cache with peeked
  /// lookahead: a failed sync rewinds the drained cache prefix too, restoring
  /// the pre-call position, at the cost of re-lexing those formerly-cached
  /// tokens on the next read.
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
    // A no-match run to end of input must leave no trace — even across a prefilled cache.
    // `sync_matched_in_cache` below drains the non-matching cached prefix, advancing
    // span/state and emitting an unexpected-token diagnostic per drained token; the later
    // uncached scan may skip and diagnose more tokens and cross lexer errors (lifting the
    // dedup watermark). Snapshot the pre-call position (span + lexer state), the emitter's
    // emission mark, and the watermark HERE — BEFORE the drain — so the end-of-input exit
    // can restore the FULL pre-call state. A match or a limit trip commits the whole
    // diagnosed prefix (the drain was real progress en route to it), so this snapshot goes
    // unused on those paths; only the no-match end-of-input exit rewinds to it. This is an
    // internal positional rewind, not a `Checkpoint`: it threads no lineage entry.
    let entry_span = self.span.clone();
    let entry_state = self.state.clone();
    let entry_mark = self.emitter.checkpoint();
    let entry_error_end = self.emitted_error_end.clone();

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

    // A sticky limit trip latches a poison boundary: once the cursor reaches the
    // durable frontier no token remains to sync to, so return `Ok(None)` (the
    // end-of-input outcome) without rebuilding a lexer. Strictly before it, the
    // scan proceeds.
    if self.reached_boundary(self.offset()) {
      return Ok(None);
    }

    let mut lex_at = self.offset().clone();
    let mut lexer = self.lexer();
    // The frontier tracks the end of the last synced-over token; a trip latches
    // and commits there.
    let mut frontier = AtFrontier {
      span: self.span.clone(),
      state: self.state.clone(),
    };

    loop {
      match self.scan_with(&mut lexer, &mut lex_at, &mut frontier)? {
        Scan::Token(tok) => {
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
            frontier.advance(&lexer);
          }
        }
        Scan::Tripped => {
          // A trip commits the diagnosed prefix at the durable frontier — the end of
          // the last skipped token — so a later scan yields the poisoned outcome there
          // instead of stranding the diagnosed tokens at the cursor. That commit is
          // real progress, so its diagnostics persist.
          self.set_span_after_consume(frontier.span.into());
          *self.state = frontier.state;
          return Ok(None);
        }
        Scan::Eof => {
          // No match reached the end of input: this path commits no progress, so it
          // rewinds the FULL pre-call state — the drained cache prefix included. Restore
          // span and lexer state to their entry values, restore the dedup watermark, and
          // unwind every emission this call made (the drained AND scanned tokens'
          // unexpected-token diagnostics and any lexer errors crossed). The drained cache
          // entries were popped, not put back; by the `Lexer` determinism contract the
          // next read re-lexes them identically (the replay machinery's standing story),
          // so the caller sees the same tokens at the same spans — at the cost of
          // re-lexing them once. Restoring span/state BEFORE deriving the cursor lands it
          // exactly at the pre-call position: the cache is now empty, so the cursor
          // follows span.end, which equals the pre-call cursor. Restoring the watermark
          // keeps a rewound lexer error re-emittable, so the caller's genuine consume
          // reports it exactly once instead of deduplicating it silently away.
          self.set_span((&entry_span).into());
          *self.state = entry_state;
          *self.emitted_error_end = entry_error_end;
          let cursor = self.cursor().clone();
          self.emitter().rewind(&cursor, entry_mark);
          return Ok(None);
        }
      }
    }
  }

  /// Skip tokens until the predicate matches, emitting lexer errors along the way.
  ///
  /// If the predicate matches, the matching token is consumed and returned with the tokens
  /// peeked after it.
  ///
  /// Diagnostics travel with progress, exactly as in [`sync_through`](Self::sync_through): a
  /// match commits the skipped prefix, so the diagnostics describing it persist. A no-match
  /// run to end of input commits nothing — the cursor stays at the pre-call position — and
  /// leaves no trace: the failed scan's emissions are unwound and the lexer-error
  /// deduplication watermark is restored, so a later genuine consume of the same region
  /// reports its errors exactly once. The returned peek is then empty. As in
  /// [`sync_through`](Self::sync_through), the pre-call position is restored even when the
  /// caller had prefilled the cache with peeked lookahead — the drained cache prefix is
  /// rewound too, at the cost of re-lexing those tokens on the next read.
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
  ///
  /// Diagnostics travel with progress, exactly as in [`sync_through`](Self::sync_through): a
  /// match commits the skipped prefix, so its diagnostics persist. A no-match run to end of
  /// input commits nothing — the cursor stays at the pre-call position — and leaves no trace:
  /// the failed scan's emissions are unwound and the lexer-error deduplication watermark is
  /// restored, so a later genuine consume of the same region reports its errors exactly once.
  /// The returned peek is then empty. As in [`sync_through`](Self::sync_through), the pre-call
  /// position is restored even when the caller had prefilled the cache with peeked lookahead —
  /// the drained cache prefix is rewound too, at the cost of re-lexing those tokens on the
  /// next read.
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
    // A no-match run to end of input must leave no trace — even across a prefilled cache.
    // `sync_matched_in_cache` below drains the non-matching cached prefix, advancing
    // span/state and emitting an unexpected-token diagnostic per drained token; the later
    // uncached scan may skip and diagnose more tokens and cross lexer errors (lifting the
    // dedup watermark). Snapshot the pre-call position (span + lexer state), the emitter's
    // emission mark, and the watermark HERE — BEFORE the drain — so the end-of-input exit
    // can restore the FULL pre-call state. A match or a limit trip commits the whole
    // diagnosed prefix (the drain was real progress en route to it), so this snapshot goes
    // unused on those paths; only the no-match end-of-input exit rewinds to it. This is an
    // internal positional rewind, not a `Checkpoint`: it threads no lineage entry.
    let entry_span = self.span.clone();
    let entry_state = self.state.clone();
    let entry_mark = self.emitter.checkpoint();
    let entry_error_end = self.emitted_error_end.clone();

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
        // A sticky limit trip latches a poison boundary: once the cursor reaches
        // the durable frontier no token remains to sync to, so return the empty
        // result (the end-of-input outcome) without rebuilding a lexer. Strictly
        // before it, the scan proceeds.
        if self.reached_boundary(self.offset()) {
          return Ok((None, GenericArrayDeque::new(), self.emitter));
        }

        let mut lex_at = self.offset().clone();
        let mut lexer = self.lexer();
        // The frontier tracks the end of the last synced-over token; a trip latches
        // and commits there.
        let mut frontier = AtFrontier {
          span: self.span.clone(),
          state: self.state.clone(),
        };

        loop {
          match self.scan_with(&mut lexer, &mut lex_at, &mut frontier)? {
            Scan::Token(tok) => {
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
                frontier.advance(&lexer);
              }
            }
            Scan::Tripped => {
              // Commit the diagnosed prefix before the trip; the boundary latches at
              // the end of the last skipped token, so a later scan yields the poisoned
              // outcome there instead of stranding the diagnosed tokens at the cursor.
              self.set_span_after_consume(frontier.span.into());
              *self.state = frontier.state;
              return Ok((None, GenericArrayDeque::new(), self.emitter));
            }
            Scan::Eof => {
              // No match reached the end of input: this path commits no progress, so it
              // rewinds the FULL pre-call state — the drained cache prefix included — and
              // the returned peek is empty. Restore span and lexer state to their entry
              // values, restore the dedup watermark, and unwind every emission this call
              // made (the drained AND scanned tokens' unexpected-token diagnostics and any
              // lexer errors crossed). The drained cache entries were popped, not put back;
              // by the `Lexer` determinism contract the next read re-lexes them identically
              // (the replay machinery's standing story), so the caller sees the same tokens
              // at the same spans — at the cost of re-lexing them once. Restoring span/state
              // BEFORE deriving the cursor lands it exactly at the pre-call position: the
              // cache is now empty, so the cursor follows span.end, which equals the
              // pre-call cursor. Restoring the watermark keeps a rewound lexer error
              // re-emittable, so the caller's genuine consume reports it exactly once
              // instead of deduplicating it silently away.
              self.set_span((&entry_span).into());
              *self.state = entry_state;
              *self.emitted_error_end = entry_error_end;
              let cursor = self.cursor().clone();
              self.emitter().rewind(&cursor, entry_mark);
              return Ok((None, GenericArrayDeque::new(), self.emitter));
            }
          }
        }
      }
    }
  }
}
