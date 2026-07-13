use super::*;

use super::sync::{SyncTo, Synced};

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
  ///
  /// # The fatal exit commits
  ///
  /// A fatal emitter rejection mid-skip follows the sync family's fatal-exit discipline: the
  /// token that trips the emitter is committed and the error propagates, so a caller that
  /// catches it resumes *after* the reported token. This does not depend on whether the token
  /// was already in the peek cache — the cache is an invisible optimization (see
  /// [`sync_through`](Self::sync_through)).
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
  /// Returns peeked tokens and a mutable reference to the emitter. A fatal emitter rejection
  /// mid-skip commits the token that tripped it, exactly as in [`sync_to`](Self::sync_to).
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
    // The matched token is left unconsumed, so peek it: either the drain stopped at it and it is
    // the cache front (tested exactly once — the decision comes back in `Drained`, and re-testing
    // it here would ask a stateful `pred` about the same token twice), or the cache drained empty,
    // the shared scanner committed at the frontier before the match (`SyncTo::on_match`), and the
    // peek re-lexes it from there. The exhausted outcomes — a poison trip mid-scan or a no-match
    // run to end of input, both of which `sync_with` has already committed — return the empty peek.
    match self.sync_matched_in_cache(&mut pred, &mut exp)? {
      Drained::Matched => self.peek_with_emitter::<W>(),
      Drained::Empty => match self.sync_with::<SyncTo, _, _>(&mut pred, &mut exp, ())? {
        Synced::Found(_) => self.peek_with_emitter::<W>(),
        Synced::Exhausted => Ok((GenericArrayDeque::new(), self.emitter)),
      },
    }
  }
}
