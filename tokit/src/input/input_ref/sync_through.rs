use super::*;

use super::sync::{SyncThrough, Synced, ThroughEntry};

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
    trace_event!(self, "sync_through");
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
    let snapshot = ThroughEntry::new(
      self.span.clone(),
      self.state.clone(),
      self.emitter.checkpoint(),
      self.emitted_error_end.clone(),
    );

    match self.sync_matched_in_cache(&mut pred, &mut exp)? {
      // The drain stopped at a cached token `pred` accepted, and left it at the front. Consume
      // THAT token: the decision is carried out of the drain, never re-derived. A second `pred`
      // call about it is observable to any stateful `FnMut` and free to answer differently, and
      // acting on that answer would drop us into the scanner below with a live cache.
      Drained::Matched => Ok(self.consume_cached_one()),
      // The cache is empty, so the scanner may lex: skip-and-diagnose to the first match.
      // `SyncThrough` consumes the match (`Synced::Found`); a poison trip commits the diagnosed
      // prefix at the durable frontier and a no-match run to end of input rewinds to `snapshot`,
      // both yielding the exhausted outcome (`Ok(None)`) with the position already settled.
      Drained::Empty => match self.sync_with::<SyncThrough, _, _>(&mut pred, &mut exp, snapshot)? {
        Synced::Found(tok) => Ok(tok),
        Synced::Exhausted => Ok(None),
      },
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
    trace_event!(self, "sync_through_then_peek");
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
    let snapshot = ThroughEntry::new(
      self.span.clone(),
      self.state.clone(),
      self.emitter.checkpoint(),
      self.emitted_error_end.clone(),
    );

    match self.sync_matched_in_cache(&mut pred, &mut exp)? {
      // The drain stopped at a cached token `pred` accepted, and left it at the front. Consume
      // THAT token — the decision leaves the drain with it, never re-derived (see
      // [`sync_through`](Self::sync_through)) — and peek what follows it.
      Drained::Matched => {
        let tok = self.consume_cached_one();
        let (peeked, emitter) = self.peek_with_emitter::<W>()?;
        Ok((tok, peeked, emitter))
      }
      // The cache is empty, so the scanner may lex: skip-and-diagnose to the first match.
      Drained::Empty => match self.sync_with::<SyncThrough, _, _>(&mut pred, &mut exp, snapshot)? {
        // The match is consumed; peek the tokens after it.
        Synced::Found(tok) => {
          let (peeked, emitter) = self.peek_with_emitter::<W>()?;
          Ok((tok, peeked, emitter))
        }
        // The exhausted outcomes — a poison trip committed at the durable frontier, or a
        // no-match run to end of input rewound to `snapshot` — yield no match and an empty peek.
        Synced::Exhausted => Ok((None, GenericArrayDeque::new(), self.emitter)),
      },
    }
  }
}
