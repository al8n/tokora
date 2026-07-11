use super::*;

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Skip tokens until the predicate matches, emitting lexer errors along the way.
  ///
  /// Advances through the stream, emitting each lexer error via the emitter. Stops
  /// before the first token for which `pred` returns `true` and returns it (without
  /// consuming). Non-matching non-error tokens are skipped but also reported via
  /// `emit_unexpected_token`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn sync_to<F, Exp>(
    &mut self,
    pred: F,
    exp: Exp,
  ) -> Result<
    Option<MaybeRefCachedTokenOf<'_, 'inp, L, L::Token>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
  {
    self
      .sync_to_then_peek_with_emitter::<_, _, U1>(pred, exp)
      .map(|(mut out, _)| out.pop_front())
  }

  /// Skip tokens until the predicate matches, emitting lexer errors along the way.
  ///
  /// Returns peeked tokens and a mutable reference to the emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn sync_to_then_peek_with_emitter<'p, F, Exp, W>(
    &'p mut self,
    mut pred: F,
    mut exp: Exp,
  ) -> Result<
    (Peeked<'p, 'inp, L, W>, &'p mut Ctx::Emitter),
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
    W: Window,
  {
    trace_event!(self, "sync_to");
    self.sync_matched_in_cache(&mut pred, &mut exp)?;

    // as the matched token will not be consumed, we just peek it
    match !self.cache().is_empty() {
      // If the matched token is in cache, return it
      true => self.peek_with_emitter::<W>(),
      // Otherwise, let's skip the input
      false => {
        // A sticky limit trip latches a poison boundary: once the cursor reaches
        // the durable frontier no token remains to sync to, so return the empty
        // peek (the end-of-input outcome) without rebuilding a lexer. Strictly
        // before it, the scan proceeds.
        if self.reached_boundary(self.offset()) {
          return Ok((GenericArrayDeque::new(), self.emitter));
        }

        let mut lex_at = self.offset().clone();
        let mut lexer = self.lexer();
        // The frontier tracks the end of the last synced-over token; a trip
        // latches and commits there.
        let mut frontier = AtFrontier {
          span: self.span.clone(),
          state: self.state.clone(),
        };

        loop {
          match self.scan_with(&mut lexer, &mut lex_at, &mut frontier)? {
            Scan::Token(tok) => {
              // if the token matches, we cache it and return it
              if pred(tok.as_ref()) {
                self.set_span_after_consume(frontier.span.into());
                *self.state = frontier.state;
                return self.peek_with_emitter::<W>();
              } else {
                let (span, tok) = tok.into_components();
                self.emitter().emit_unexpected_token(
                  UnexpectedToken::maybe_expected_of(span, exp()).with_found(tok),
                )?;
                frontier.advance(&lexer);
              }
            }
            Scan::Tripped => {
              // Commit progress before the trip; stop at the poison.
              self.set_span_after_consume(frontier.span.into());
              *self.state = frontier.state;
              return Ok((GenericArrayDeque::new(), self.emitter));
            }
            Scan::Eof => {
              // No matched token found, we just update the cursor and state
              self.set_span_after_consume(lexer.span().into());
              *self.state = lexer.into_state();
              return Ok((GenericArrayDeque::new(), self.emitter));
            }
          }
        }
      }
    }
  }
}
