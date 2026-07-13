//! The shared scanner behind the `sync_to`/`sync_through`/`sync_balanced` family: **one** loop
//! over the token stream — cached tokens and freshly-lexed ones alike — parameterized by a
//! [`SyncMode`] policy that decides how a matched sync token settles and what end of input does.
//!
//! # One loop, because the cache is invisible
//!
//! Whether a token had already been peeked into the cache is an optimization the caller cannot
//! see: every observable of a sync call — its return, the committed position and lexer state, the
//! diagnostics it emits, the resume cursor, the poison boundary, the dedup watermark — is a
//! function of the token stream alone, never of how much of it had been prefetched. The family
//! used to keep that promise by *agreeing*: a cache-drain prologue and this scanner each
//! implemented skip-and-report, and each settled a match its own way. Nothing forced the two to
//! agree, and they repeatedly did not.
//!
//! So there is now exactly one implementation. The loop takes its next token from the cache while
//! the cache has one and from the lexer once it does not ([`Fetched`], carried as the crate's
//! [`CachedToken`] — a lexed token plus the state that produced it — whichever way it arrived).
//! That fetch is the *whole* of the difference: the predicate is evaluated at one site, the
//! skip-and-report is one method, and the match settles through one [`SyncMode`] hook that cannot
//! even tell where the token came from. A cached/uncached divergence is no longer a bug to be
//! caught by a test — it has nowhere to live.
//!
//! The public loops differ in only three decisions, and all three are the mode's:
//!
//! - **how a match settles** — a `to` scan stops *before* the token, leaving it unconsumed; a
//!   `through` scan consumes it;
//! - **what end of input does** — a `to` scan commits at the lexer's end; a `through` scan rewinds
//!   the full pre-call state, so a no-match run leaves no trace;
//! - **whether each skipped token is reported** — the to/through family diagnoses per skipped
//!   token; the balanced scan describes the whole hole with one diagnostic instead.
//!
//! Everything else — the poison-boundary short-circuit, the dedup watermark lifted through
//! [`scan_with`](InputRef::scan_with), and the trip-commit at the durable frontier — is identical,
//! so it lives here once and the contracts documented on the public methods are structural instead
//! of re-implemented per method.
//!
//! # An unconsumed token lives at the cache front
//!
//! That is the invariant a `to`-shaped match settles on, and it is not new: a token whose
//! predicate declined it is exactly what [`try_expect`](InputRef::try_expect) puts back into the
//! cache, and the cache front is the one place [`cursor`](InputRef::cursor) reads. The old scanner
//! broke it — it threw a lexed match away and let the caller re-lex it — which is why the same
//! call returned a different zero-skip [`Hole`](super::Hole) depending on how deep the caller had
//! peeked. Settling a match through [`InputRef::unconsume`] restores the invariant on *both*
//! origins, so the cursor after a match is the sync token's start no matter who lexed it.

use super::*;

/// The normalized outcome of [`sync_with`](InputRef::sync_with), across the to/through and
/// plain/peek variants. The caller maps it to its own return shape; the position is already
/// settled per policy by the time it is handed back.
pub(super) enum Synced<'inp, L>
where
  L: Lexer<'inp>,
{
  /// The sync predicate matched. A `through` policy consumes the token and carries it here; a
  /// `to` policy leaves it unconsumed at the cache front and carries `None` (its caller peeks the
  /// match straight back out).
  Found(Option<Spanned<L::Token, L::Span>>),
  /// End of input or a poison trip — no sync point. The position is already settled per policy
  /// (committed at the frontier/end, or rewound to the pre-call snapshot), so the caller only
  /// produces its exhausted return.
  Exhausted,
}

/// Where the loop got the token it is deciding on.
///
/// The origin may change **how** a token is obtained and **how** its consumption is committed —
/// never what a caller can observe afterwards. It survives the fetch for exactly one reason: the
/// cache's push history. Putting a *popped* token back is a no-op on that history, while a *lexed*
/// one becomes a new cache entry; see [`InputRef::unconsume`], the single place that knows.
#[derive(Clone, Copy)]
enum Origin {
  /// Popped off the cache front — it was lexed and counted by an earlier peek.
  Cache,
  /// Lexed by this loop, once the cache had run out.
  Lexer,
}

/// One token under decision: the token, its span, and the lexer state that produced it — the
/// crate's [`CachedToken`], which is precisely that triple, whichever origin it arrived from.
///
/// Normalizing both origins into this one carrier is what makes the rest of the loop origin-blind:
/// the predicate, the skip-and-report, and the match settle all take a `Fetched` and cannot tell
/// a drained cache from a fresh lex.
///
/// Visible to the enclosing module only because [`SyncMode`] is (its hooks take one); both fields
/// stay private to this one, so nothing outside the scanner can build a `Fetched` and mislabel its
/// origin.
pub(super) struct Fetched<'inp, L>
where
  L: Lexer<'inp>,
{
  tok: CachedTokenOf<'inp, L>,
  origin: Origin,
}

/// The pre-call snapshot a `through` end-of-input arm rewinds to. Captured *before* the sync loop
/// runs so the rewind restores the FULL pre-call state — span, lexer state, emission mark, and
/// dedup watermark — leaving a no-match run to end of input with no trace, including across a
/// prefilled cache the loop drained (see [`sync_through`](InputRef::sync_through)).
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
/// [`SyncTo`] stops before the matched token (commits at the frontier, leaving the match
/// unconsumed at the cache front) and, at end of input, commits at the lexer's end. [`SyncThrough`]
/// consumes the matched token and, at end of input, rewinds the full pre-call state. Both are
/// zero-sized; the pred/exp closures and the pre-call snapshot are threaded through
/// [`sync_with`](InputRef::sync_with) rather than held here.
///
/// Neither hook is told where the matched token came from — that is the point. Both are handed the
/// same [`Fetched`] carrier whether the loop popped it off the cache or lexed it, so a settle
/// cannot be written to depend on the cache even by accident.
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
  /// `frontier` (the end of the last skipped token, i.e. before the match), leaves the match
  /// unconsumed at the cache front, and returns `None`; `through` consumes the match (commits at
  /// its span, adopting the state that produced it) and returns `Some(tok)`.
  fn on_match(
    ir: &mut InputRef<'inp, '_, L, Ctx, Lang>,
    frontier: AtFrontier<L::Span, L::State>,
    matched: Fetched<'inp, L>,
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
    matched: Fetched<'inp, L>,
  ) -> Option<Spanned<L::Token, L::Span>> {
    // Leave the match unconsumed — which in this crate means AT THE CACHE FRONT, the home of every
    // lexed-but-not-consumed token (`try_expect`'s decline puts one back there too) and the one
    // place `cursor()` reads. The caller peeks it straight back out. Doing this for a token the
    // loop LEXED, and not only for one it popped, is what makes the resume cursor after a match a
    // fact about the stream instead of about the caller's lookahead depth.
    ir.unconsume(matched);
    // Commit before the match: the end of the last skipped token, with the state that produced it.
    ir.commit_at(frontier);
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
    matched: Fetched<'inp, L>,
  ) -> Option<Spanned<L::Token, L::Span>> {
    // Consume the match: commit at its span, adopting the state that produced it. This is
    // `consume_cached_one`'s body over the same carrier — and it is the same two lines whether the
    // token was popped off the cache or lexed a moment ago, because a `CachedToken` carries the
    // post-token state either way.
    let (tok, state) = matched.tok.into_components();
    ir.set_span_after_consume(tok.span_ref().into());
    *ir.state = state;
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
    // watermark, and unwind every emission this call made. The loop drained the cache, so
    // restoring span/state lands the cursor exactly at the pre-call position (with nothing cached,
    // the cursor follows span.end). Restoring the watermark keeps a rewound lexer error
    // re-emittable, so a later genuine consume reports it exactly once instead of deduplicating it
    // silently away.
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
    matched: Fetched<'inp, L>,
  ) -> Option<Spanned<L::Token, L::Span>> {
    // Stop before the sync point, exactly as `sync_to` does — which is also what places the
    // zero-skip hole: `sync_balanced` anchors it at `cursor()`, and the cursor is the match's start
    // because the match is left at the cache front here, cached or lexed.
    <SyncTo as SyncMode<'inp, L, Ctx, Lang>>::on_match(ir, frontier, matched)
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
  /// The shared sync scanner: skip tokens, diagnosing each as unexpected, until `pred` matches or
  /// the input is exhausted, then settle per the [`SyncMode`] `M`.
  ///
  /// The loop takes each token from the cache while the cache has one and from the lexer once it
  /// does not — and that is the only thing the two origins change. `pred` is evaluated at a single
  /// site, exactly once per token, so a stateful `FnMut` cannot tell a drained cache from a fresh
  /// lex; the skip-and-report is [`skip_and_report`](Self::skip_and_report), one method for both;
  /// and the match settles through [`SyncMode::on_match`], which is handed the same carrier either
  /// way ([`Synced::Found`]). A limit trip commits the diagnosed prefix at the durable frontier and
  /// end of input settles via [`SyncMode::on_eof`] (both [`Synced::Exhausted`]).
  ///
  /// # The frontier is the scan's uncommitted position
  ///
  /// Nothing is written back to the input while the loop runs: each skipped token settles behind
  /// the [`AtFrontier`] frontier — its span and the state that produced it, arriving with the token
  /// from the cache or read off the lexer — and every stop writes the input's position *from
  /// there* ([`commit_at`](Self::commit_at)). So the committed position after a sync is a function
  /// of the tokens the loop skipped, never of where they came from, and the lexer that takes over
  /// when the cache runs dry is built from that same frontier (its state, at its end — precisely
  /// where the drained cache left the lex position).
  ///
  /// # The fatal exit commits, so the cache stays invisible
  ///
  /// A fatal rejection of a skipped token's diagnostic commits that token before propagating — the
  /// family's fatal-exit discipline. It holds identically on both origins because there is only one
  /// path: [`skip_and_report`](Self::skip_and_report) settles the token behind the frontier
  /// *before* the report's verdict is honoured, and this loop commits at that frontier on the way
  /// out. Returning without the commit would leave the reported token unconsumed here and consumed
  /// there, so a recovery that retries would duplicate diagnostics — or spin — on exactly the runs
  /// where the token had not been prefetched.
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
    // The scan's uncommitted position: the pre-call span/state, then the end of each token the loop
    // settles behind it. A trip latches here, and every stop that keeps the loop's progress commits
    // here.
    let mut frontier = AtFrontier {
      span: self.span.clone(),
      state: self.state.clone(),
    };
    // The lexer, built the moment the cache runs out — under the frontier's state and at its end,
    // which is exactly where the drained cache left the lex position. A call answered entirely out
    // of the cache never builds one.
    let mut lexing: Option<(L, L::Offset)> = None;

    loop {
      // ── The one place the two origins differ: where the next token comes from ──
      let fetched = match self.cache.pop_front() {
        // A cached token arrives with the state that lexed it, already counted by the peek that
        // cached it.
        Some(tok) => Fetched {
          tok,
          origin: Origin::Cache,
        },
        None => {
          if lexing.is_none() {
            let at = frontier.span.end_ref().clone();
            // A sticky limit trip latches a poison boundary: once the lex position has reached the
            // durable frontier there is no token left to sync to. Commit what the loop already
            // skipped — real, diagnosed progress — and yield the exhausted outcome without
            // rebuilding a lexer.
            if self.reached_boundary(&at) {
              self.commit_at(frontier);
              return Ok(Synced::Exhausted);
            }
            lexing = Some((self.lexer_from(frontier.state.clone(), &at), at));
          }
          let (lexer, lex_at) = lexing.as_mut().expect("the lexer is built just above");
          // `scan_with` centralizes the poison latch, the dedup watermark, the partial-input
          // frontier rules, and the fatal-emit discipline, handing back only the events this loop
          // must decide.
          match self.scan_with(lexer, lex_at, &mut frontier)? {
            Scan::Token(tok) => Fetched {
              tok: CachedToken::new(tok, lexer.state().clone()),
              origin: Origin::Lexer,
            },
            Scan::Tripped => {
              // Commit the diagnosed prefix at the durable frontier — the end of the last skipped
              // token — so a later scan yields the poisoned outcome there instead of stranding the
              // diagnosed tokens at the cursor. That commit is real progress, so its diagnostics
              // persist.
              self.commit_at(frontier);
              return Ok(Synced::Exhausted);
            }
            Scan::Eof => {
              let (lexer, _) = lexing.take().expect("the lexer is built just above");
              M::on_eof(self, lexer, snapshot);
              return Ok(Synced::Exhausted);
            }
          }
        }
      };

      // ── One decision, one report, one settle — all of it blind to the origin ──
      // `pred` sees each token EXACTLY once, at this single site.
      if pred(fetched.tok.token()) {
        return Ok(Synced::Found(M::on_match(self, frontier, fetched)));
      }

      if let Err(e) = self.skip_and_report::<M, _>(fetched, &mut frontier, &mut exp) {
        // The family's fatal-exit discipline: the token that trips a fatal emitter is committed,
        // and the error propagates. The commit lands at the frontier — the skipped token's end,
        // with the state that produced it — because `skip_and_report` settled it there before
        // honouring the verdict. It also carries the prefix this loop already diagnosed, so nothing
        // already reported is left to be reported again.
        self.commit_at(frontier);
        return Err(e);
      }
    }
  }

  /// **The** skip-and-report path: settle a token the predicate rejected behind the frontier and —
  /// for the modes that diagnose each skipped token — report it as unexpected.
  ///
  /// Cached tokens and freshly-lexed ones reach this by the same call, carrying the same
  /// [`CachedToken`], so the settle and the report cannot drift apart: the crate has one answer to
  /// "skip a token and report it", not one per origin.
  ///
  /// The token settles behind the frontier **before** the report's verdict is honoured, so both
  /// outcomes leave it behind the frontier and the caller's fatal exit commits it — the family's
  /// trip-commit, on either origin. The balanced mode reports nothing (one hole diagnostic
  /// describes the whole region), so under it the diagnostic is never even built and `exp` is never
  /// called.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn skip_and_report<M, Exp>(
    &mut self,
    skipped: Fetched<'inp, L>,
    frontier: &mut AtFrontier<L::Span, L::State>,
    exp: &mut Exp,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    M: SyncMode<'inp, L, Ctx, Lang>,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
  {
    let (spanned, state) = skipped.tok.into_components();
    let (span, tok) = spanned.into_components();

    let report = M::REPORT_SKIPPED
      .then(|| UnexpectedToken::maybe_expected_of(span.clone(), exp()).with_found(tok));
    frontier.adopt(span, state);

    match report {
      Some(report) => self.emitter().emit_unexpected_token(report),
      None => Ok(()),
    }
  }

  /// Puts a token the scan decided **not** to consume back where an unconsumed token lives: the
  /// front of the cache. This is how a `to`-shaped match settles, and it is the same call whichever
  /// origin the token came from — so the cache after a match holds the sync token at its front
  /// either way, and [`cursor`](Self::cursor) reads the same resume position either way.
  ///
  /// # Only the push history knows the difference
  ///
  /// A token the loop **popped** off the cache goes straight back into the slot it left: the cache
  /// is then exactly what it was, so its push count must not move. A token the loop **lexed** is a
  /// NEW cache entry — precisely the one a peek would have made — so its push is recorded, and a
  /// checkpoint saved before this call drops it on restore, exactly as it drops a peek's.
  ///
  /// Getting that backwards is not cosmetic: [`restore_unchecked`](Self::restore_unchecked) drops
  /// the last `cache_pushes - saved` entries from the **back**, so counting a round-trip as a push
  /// would over-drop a genuinely pre-save entry — evicting lookahead the caller had already paid to
  /// lex, on a restore that should have kept it.
  ///
  /// A cache that accepts no push (a zero-capacity `BlackHole`) simply drops the token, which
  /// re-lexes on demand: the only behaviour such a cache can have — and, holding no tokens, the
  /// only origin it can ever produce is the lexer, so it has nothing to diverge from.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn unconsume(&mut self, fetched: Fetched<'inp, L>) {
    let Fetched { tok, origin } = fetched;
    if self.cache.push_front(tok).is_ok() && matches!(origin, Origin::Lexer) {
      self.session.lineage.record_cache_push();
    }
  }
}
