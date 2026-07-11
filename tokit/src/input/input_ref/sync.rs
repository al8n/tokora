//! The shared scanner behind the `sync_to`/`sync_through` family: one loop over
//! [`scan_with`](InputRef::scan_with), parameterized by a [`SyncMode`] policy that decides
//! how a matched sync token settles and what end of input does.
//!
//! The public sync loops differ in only three decisions — how a match settles (a `to` scan
//! stops *before* the token and commits at the frontier; a `through` scan consumes it), what
//! end of input does (a `to` scan commits at the lexer's end; a `through` scan rewinds the
//! full pre-call state so a no-match run leaves no trace), and whether each skipped token is
//! reported as unexpected (the to/through family does; the balanced scan describes the whole
//! hole with one diagnostic instead). Everything else — the poison-boundary short-circuit,
//! the dedup watermark lifted through [`scan_with`](InputRef::scan_with), and the
//! trip-commit at the durable frontier — is identical, so it lives here once and the
//! contracts documented on the public methods become structural instead of re-implemented
//! per method.

use super::*;

/// The normalized outcome of [`sync_with`](InputRef::sync_with), across the to/through and
/// plain/peek variants. The caller maps it to its own return shape; the position is already
/// settled per policy by the time it is handed back.
pub(super) enum Synced<'inp, L>
where
  L: Lexer<'inp>,
{
  /// The sync predicate matched. A `through` policy consumes the token and carries it here; a
  /// `to` policy commits *before* it and carries `None` (its caller peeks the match).
  Found(Option<Spanned<L::Token, L::Span>>),
  /// End of input or a poison trip — no sync point. The position is already settled per policy
  /// (committed at the frontier/end, or rewound to the pre-call snapshot), so the caller only
  /// produces its exhausted return.
  Exhausted,
}

/// The pre-call snapshot a `through` end-of-input arm rewinds to. Captured *before* the cache
/// drain so the rewind restores the FULL pre-call state — span, lexer state, emission mark, and
/// dedup watermark — leaving a no-match run to end of input with no trace (see
/// [`sync_through`](InputRef::sync_through)).
pub(super) struct ThroughEntry<Span, State, Offset> {
  span: Span,
  state: State,
  mark: u64,
  error_end: Offset,
}

impl<Span, State, Offset> ThroughEntry<Span, State, Offset> {
  /// Bundles the four facts the end-of-input rewind restores.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(span: Span, state: State, mark: u64, error_end: Offset) -> Self {
    Self {
      span,
      state,
      mark,
      error_end,
    }
  }
}

/// How a sync scan settles the two decisions that separate `sync_to` from `sync_through`.
///
/// [`SyncTo`] stops before the matched token (commits at the frontier) and, at end of input,
/// commits at the lexer's end. [`SyncThrough`] consumes the matched token and, at end of input,
/// rewinds the full pre-call state. Both are zero-sized; the pred/exp closures and the pre-call
/// snapshot are threaded through [`sync_with`](InputRef::sync_with) rather than held here.
pub(super) trait SyncMode<'inp, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  /// The pre-call snapshot the end-of-input arm needs: `()` for `to` (it commits at the end),
  /// a [`ThroughEntry`] for `through` (it rewinds to it).
  type Snapshot;

  /// Whether the scan reports each skipped token through `emit_unexpected_token`. The
  /// to/through family diagnoses per skipped token; the balanced scan suppresses the
  /// per-token reports because one skipped-region diagnostic describes the whole hole (see
  /// [`sync_balanced`](InputRef::sync_balanced)).
  const REPORT_SKIPPED: bool;

  /// Settle the input on a matched sync token and produce the carried token. `to` commits at
  /// `frontier` (the end of the last skipped token, i.e. before the match) and returns `None`;
  /// `through` consumes the match (commits at its span, adopting the lexer state) and returns
  /// `Some(tok)`.
  fn on_match(
    ir: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    frontier: AtFrontier<L::Span, L::State>,
    lexer: L,
    tok: Spanned<L::Token, L::Span>,
  ) -> Option<Spanned<L::Token, L::Span>>;

  /// Settle the input at end of input (no sync point found). `to` commits at the lexer's end;
  /// `through` rewinds span, lexer state, dedup watermark, and emissions to `snapshot`.
  fn on_eof(ir: &mut InputRef<'inp, '_, L, Ctx, Lang>, lexer: L, snapshot: Self::Snapshot);
}

/// Stop *before* the sync token: commit at the frontier on a match, commit at the lexer's end at
/// end of input. Drives `sync_to`.
pub(super) struct SyncTo;

impl<'inp, L, Ctx, Lang> SyncMode<'inp, L, Ctx, Lang> for SyncTo
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  type Snapshot = ();

  const REPORT_SKIPPED: bool = true;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_match(
    ir: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    frontier: AtFrontier<L::Span, L::State>,
    _lexer: L,
    _tok: Spanned<L::Token, L::Span>,
  ) -> Option<Spanned<L::Token, L::Span>> {
    // Commit at the frontier (the start of the match) and leave the match itself unconsumed;
    // the caller peeks it back by re-lexing from here.
    ir.set_span_after_consume(frontier.span.into());
    *ir.state = frontier.state;
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_eof(ir: &mut InputRef<'inp, '_, L, Ctx, Lang>, lexer: L, _snapshot: ()) {
    // No match found: commit the whole skipped run at the lexer's end. `sync_to` reports as it
    // goes and keeps that progress, so end of input is not a rewinding failure here.
    ir.set_span_after_consume(lexer.span().into());
    *ir.state = lexer.into_state();
  }
}

/// Consume the sync token: commit at its span on a match, rewind the full pre-call state at end
/// of input. Drives `sync_through` and `sync_through_then_peek`.
pub(super) struct SyncThrough;

impl<'inp, L, Ctx, Lang> SyncMode<'inp, L, Ctx, Lang> for SyncThrough
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  type Snapshot = ThroughEntry<L::Span, L::State, L::Offset>;

  const REPORT_SKIPPED: bool = true;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_match(
    ir: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    _frontier: AtFrontier<L::Span, L::State>,
    lexer: L,
    tok: Spanned<L::Token, L::Span>,
  ) -> Option<Spanned<L::Token, L::Span>> {
    // Consume the match: commit at its span, adopting the lexer state that produced it.
    ir.set_span_after_consume(tok.span_ref().into());
    *ir.state = lexer.into_state();
    Some(tok)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_eof(
    ir: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    _lexer: L,
    snapshot: ThroughEntry<L::Span, L::State, L::Offset>,
  ) {
    // No match reached the end of input: this path commits no progress, so it rewinds the FULL
    // pre-call state — the drained cache prefix included. Restore span/state, restore the dedup
    // watermark, and unwind every emission this call made. Restoring span/state BEFORE deriving
    // the cursor lands it exactly at the pre-call position (the cache is now empty, so the cursor
    // follows span.end). Restoring the watermark keeps a rewound lexer error re-emittable, so a
    // later genuine consume reports it exactly once instead of deduplicating it silently away.
    ir.set_span((&snapshot.span).into());
    *ir.state = snapshot.state;
    *ir.emitted_error_end = snapshot.error_end;
    let cursor = ir.cursor().clone();
    ir.emitter().rewind(&cursor, snapshot.mark);
  }
}

/// Stop *before* the sync token like [`SyncTo`], but rewind the full pre-call state at end of
/// input like [`SyncThrough`], and report no per-token diagnostics — the hole diagnostic that
/// [`sync_balanced`](InputRef::sync_balanced) emits on success describes the whole skipped
/// region. Composed from the other two modes' settles.
pub(super) struct SyncBalanced;

impl<'inp, L, Ctx, Lang> SyncMode<'inp, L, Ctx, Lang> for SyncBalanced
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  type Snapshot = ThroughEntry<L::Span, L::State, L::Offset>;

  const REPORT_SKIPPED: bool = false;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_match(
    ir: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    frontier: AtFrontier<L::Span, L::State>,
    lexer: L,
    tok: Spanned<L::Token, L::Span>,
  ) -> Option<Spanned<L::Token, L::Span>> {
    // Stop before the sync point, exactly as `sync_to` does.
    <SyncTo as SyncMode<'inp, L, Ctx, Lang>>::on_match(ir, frontier, lexer, tok)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn on_eof(ir: &mut InputRef<'inp, '_, L, Ctx, Lang>, lexer: L, snapshot: Self::Snapshot) {
    // A failed balanced sync leaves no trace, exactly as `sync_through`'s no-match exit.
    <SyncThrough as SyncMode<'inp, L, Ctx, Lang>>::on_eof(ir, lexer, snapshot)
  }
}

impl<'inp, L, Ctx, Lang> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  /// The shared sync scanner: skip tokens, diagnosing each as unexpected, until `pred` matches
  /// or the input is exhausted, then settle per the [`SyncMode`] `M`.
  ///
  /// Entered only with an empty cache (the callers drain the cache first). Once the lex position
  /// reaches the poison boundary there is no token to sync to, so it returns
  /// [`Synced::Exhausted`] without settling — the caller's exhausted return with the position
  /// left as the drain committed it. Otherwise it loops over
  /// [`scan_with`](Self::scan_with), which centralizes the poison-latch, dedup, and fatal-emit
  /// discipline: a matched token settles via [`SyncMode::on_match`] ([`Synced::Found`]); a
  /// non-matching token is skipped — and reported once via `emit_unexpected_token` when
  /// [`SyncMode::REPORT_SKIPPED`] holds; a limit trip commits the diagnosed prefix at the
  /// durable frontier; end of input settles via [`SyncMode::on_eof`] (both
  /// [`Synced::Exhausted`]).
  #[cfg_attr(not(tarpaulin), inline)]
  pub(super) fn sync_with<M, F, Exp>(
    &mut self,
    mut pred: F,
    mut exp: Exp,
    snapshot: M::Snapshot,
  ) -> Result<Synced<'inp, L>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    M: SyncMode<'inp, L, Ctx, Lang>,
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
  {
    // A sticky limit trip latches a poison boundary: once the cursor reaches the durable
    // frontier no token remains to sync to, so yield the exhausted outcome without rebuilding a
    // lexer. Strictly before it, the scan proceeds.
    if self.reached_boundary(self.offset()) {
      return Ok(Synced::Exhausted);
    }

    let mut lex_at = self.offset().clone();
    let mut lexer = self.lexer();
    // The frontier tracks the end of the last synced-over token; a trip latches and commits there.
    let mut frontier = AtFrontier {
      span: self.span.clone(),
      state: self.state.clone(),
    };

    loop {
      match self.scan_with(&mut lexer, &mut lex_at, &mut frontier)? {
        Scan::Token(tok) => {
          if pred(tok.as_ref()) {
            return Ok(Synced::Found(M::on_match(self, frontier, lexer, tok)));
          }
          if M::REPORT_SKIPPED {
            let (span, tok) = tok.into_components();
            self.emitter().emit_unexpected_token(
              UnexpectedToken::maybe_expected_of(span, exp()).with_found(tok),
            )?;
          }
          frontier.advance(&lexer);
        }
        Scan::Tripped => {
          // Commit the diagnosed prefix at the durable frontier — the end of the last skipped
          // token — so a later scan yields the poisoned outcome there instead of stranding the
          // diagnosed tokens at the cursor. That commit is real progress, so its diagnostics
          // persist.
          self.set_span_after_consume(frontier.span.into());
          *self.state = frontier.state;
          return Ok(Synced::Exhausted);
        }
        Scan::Eof => {
          M::on_eof(self, lexer, snapshot);
          return Ok(Synced::Exhausted);
        }
      }
    }
  }
}
