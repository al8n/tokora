#![allow(clippy::type_complexity)]

use core::{
  marker::PhantomData,
  ops::{Range, RangeBounds},
};

use generic_arraydeque::{GenericArrayDeque, typenum::U1};
use mayber::{Maybe, MaybeRef};

use crate::{
  ParseContext, Token, Window,
  cache::{CachedToken, CachedTokenRefOf, MaybeRefCachedTokenOf, Peeked},
  emitter::Emitter,
  error::token::UnexpectedToken,
  span::Spanned,
  utils::Expected,
};

use super::{Cache, Checkpoint, Cursor, Lexed, Lexer, Source, Span};

mod consume_cached;
mod fold;
mod peek;
mod pratt;
mod skip_while;
mod sync_through;
mod sync_to;
mod try_expect;

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests;

/// A reference to an `Input` instance.
pub struct InputRef<'inp, 'closure, L, Ctx, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  pub(super) input: &'closure &'inp L::Source,
  pub(super) state: &'closure mut L::State,
  pub(super) span: &'closure mut L::Span,
  pub(super) cache: &'closure mut Ctx::Cache,
  pub(super) emitted_error_end: &'closure mut L::Offset,
  pub(super) poisoned: &'closure mut bool,
  pub(super) emitter: &'closure mut Ctx::Emitter,
  pub(super) _marker: PhantomData<Lang>,
}

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Returns a reference to the tokenizer's cache.
  ///
  /// The cache stores peeked tokens that have been lexed but not yet consumed.
  /// This can be useful for inspecting the cache state or implementing custom
  /// lookahead logic.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn cache(&self) -> &Ctx::Cache {
    self.cache
  }

  /// Returns a mutable reference to the tokenizer's cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn cache_mut(&mut self) -> &mut Ctx::Cache {
    self.cache
  }

  /// Returns a reference to the underlying input source.
  ///
  /// This allows access to the raw source being tokenized, which is typically
  /// a `&str` or `&[u8]` depending on your Logos token definition.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn source(&self) -> &'inp L::Source {
    self.input
  }

  /// Returns a reference to the current lexer state (extras).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn state(&self) -> &L::State {
    self.state
  }

  /// Returns a mutable reference to the current lexer state (extras).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn state_mut(&mut self) -> &mut L::State {
    self.state
  }

  /// Manually sets the lexer state (for context-sensitive lexing).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_state(&mut self, state: L::State) {
    *self.state = state;
  }

  /// Returns a mutable reference to the emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn emitter(&mut self) -> &mut Ctx::Emitter {
    self.emitter
  }

  /// Emits a lexer error unless the same region has already been reported.
  ///
  /// Peeking a window larger than the cache lexes past the cached region and emits
  /// any lexer errors it finds right away, so a peek-and-stop caller never loses
  /// them. Consuming that region later re-lexes it; this dedup — keyed on the error
  /// span's end against a high-water mark — guarantees every lexer error is reported
  /// exactly once, whether it is peeked, consumed, or both.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error_deduped(
    &mut self,
    err: Spanned<<L::Token as Token<'inp>>::Error, L::Span>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let end = err.span_ref().end_ref().clone();
    if end <= *self.emitted_error_end {
      return Ok(());
    }
    *self.emitted_error_end = end;
    self.emitter().emit_lexer_error(err)
  }

  /// Returns `true` if the input is poisoned by a sticky limit error.
  ///
  /// Once a lexer trips a state/limit error, this latch is set at the input level
  /// so subsequent scanning entry points can bail without rebuilding a lexer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) fn is_poisoned(&self) -> bool {
    *self.poisoned
  }

  /// Latches the input-level poison if `lexer`'s state has tripped a limit error.
  ///
  /// A limit-class error is sticky: it manifests as a failing
  /// [`check`](crate::Lexer::check) (the exact condition the lexer's own latch
  /// keys on). Because `InputRef` rebuilds a fresh lexer per operation, that
  /// per-lexer latch would be lost; recording it here bounds the work a recovering
  /// caller can trigger by re-entering `next()`/`peek()`. Returns whether it
  /// latched. A plain (non-limit) lexer error leaves `check()` `Ok` and does not
  /// latch, so the caller keeps scanning for the next valid token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn latch_if_limit_tripped(&mut self, lexer: &L) -> bool {
    if lexer.check().is_err() {
      *self.poisoned = true;
      true
    } else {
      false
    }
  }

  /// Returns `true` if reached the end of input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[doc(alias = "is_eof")]
  #[doc(alias = "end_of_input")]
  pub fn is_eoi(&self) -> bool {
    self.offset().ge(&self.input.len())
  }

  /// Creates a lexer positioned at the end of the cache or current cursor.
  ///
  /// This internal method constructs a fresh Logos lexer with the current state and
  /// positions it to continue lexing from where the cache ends (or from the cursor
  /// if the cache is empty).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn lexer(&self) -> L
  where
    L::State: Clone,
  {
    let mut lexer = L::with_state(self.input, self.state.clone());
    lexer.bump(self.offset());
    lexer
  }

  /// Sets the cursor to the specified position, clamped to the input length.
  ///
  /// This ensures the cursor never exceeds the bounds of the input source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn set_span(&mut self, new: MaybeRef<'_, L::Span>) {
    let end = self.input.len();
    *self.span = if new.end_ref().le(&end) {
      to_owned(new)
    } else {
      L::Span::new(new.start_ref().clone(), end)
    };
  }

  /// Records the span of the just-consumed token as the current input span.
  ///
  /// `span()`/`slice()` therefore report the most recently consumed token even
  /// when the cache still holds later peeked tokens. The span is clamped to the
  /// input length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn set_span_after_consume(&mut self, new: MaybeRef<'_, L::Span>) {
    self.set_span(new);
  }
}

impl<'inp, 'closure, L, Ctx, Lang: ?Sized> InputRef<'inp, 'closure, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
  /// Attempts to parse with the given function, rolling back on failure.
  ///
  /// If the closure returns `None`, the input position and lexer state are
  /// restored to their original values. If it returns `Some`, the parser
  /// state is preserved.
  pub fn attempt<F, R>(&mut self, f: F) -> Option<R>
  where
    F: FnOnce(&mut Self) -> Option<R>,
  {
    let ckp = self.save();

    match f(self) {
      Some(result) => Some(result),
      None => {
        self.restore(ckp);
        None
      }
    }
  }

  /// Returns a slice of the current token from the input source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice(&self) -> <L::Source as Source<L::Offset>>::Slice<'inp> {
    self
      .input
      .slice(self.span.start_ref()..self.span.end_ref())
      .expect("lexer should guarantee slice")
  }

  /// Returns a slice of the input source from the given cursor to the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice_since(
    &self,
    cursor: &Cursor<'inp, 'closure, L>,
  ) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    let end = self.cursor();
    self.input.slice(cursor.as_inner()..end.as_inner())
  }

  /// Returns a slice of the input source from the given cursor to the end of the input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice_from(
    &self,
    cursor: &Cursor<'inp, 'closure, L>,
  ) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    let start = cursor.as_inner();
    self.input.slice(start..)
  }

  /// Returns a slice of the input source for the given cursor range.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice_range<'r, R>(
    &self,
    range: R,
  ) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>>
  where
    R: RangeBounds<&'r Cursor<'inp, 'closure, L>>,
    'closure: 'r,
  {
    let start = range.start_bound().map(|c| c.as_inner());
    let end = range.end_bound().map(|c| c.as_inner());
    // SAFETY: The range is guaranteed to be within bounds as both cursors are within input length and comes from the same input.
    self.input.slice((start, end))
  }

  /// Returns the span of the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> &L::Span {
    self.span
  }

  /// Returns a span from the given cursor to the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span_since(&self, cursor: &Cursor<'inp, 'closure, L>) -> L::Span {
    Span::new(cursor.as_inner().clone(), self.cursor().as_inner().clone())
  }

  /// Returns a span from the given cursor to the end of the input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span_from(&self, cursor: &Cursor<'inp, 'closure, L>) -> L::Span {
    Span::new(cursor.as_inner().clone(), self.input.len())
  }

  /// Returns a span for the given cursor range.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span_range(&self, range: Range<&Cursor<'inp, 'closure, L>>) -> L::Span {
    Span::new(range.start.as_inner().clone(), range.end.as_inner().clone())
  }

  /// Saves the current state of the tokenizer as a checkpoint.
  ///
  /// This creates a snapshot of the current position and lexer state, which can
  /// later be restored using [`restore`](Self::restore). Checkpoints are essential for
  /// implementing backtracking in parsers.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn save(&self) -> Checkpoint<'inp, 'closure, L> {
    Checkpoint::new(
      self.cursor().clone(),
      self.span.clone(),
      self.state.clone(),
      self.emitter.checkpoint(),
      self.emitted_error_end.clone(),
    )
  }

  /// Returns the current cursor position.
  ///
  /// If there are cached tokens, the cursor points to the start
  /// of the first cached token; otherwise, it points to the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn cursor(&self) -> &Cursor<'inp, 'closure, L> {
    Cursor::from_ref(
      self
        .cache()
        .front_span()
        .map(|span| span.start_ref())
        .unwrap_or_else(|| self.span.end_ref()),
    )
  }

  /// Returns the current offset of the tokenizer.
  ///
  /// This is the end of the last lexed token (cached or otherwise).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn offset(&self) -> &L::Offset {
    self
      .cache()
      .back_span()
      .map(|s| s.end_ref())
      .unwrap_or_else(|| self.span.end_ref())
  }

  /// Restores the tokenizer state to a previously saved checkpoint.
  ///
  /// This rewinds the cache, resets the cursor position, and restores the lexer
  /// state.
  ///
  /// A sticky limit-error latch is **not** cleared: a tripped input stays tripped
  /// across restores, mirroring the lexer-level latch, so backtracking cannot
  /// re-arm unbounded rescans.
  #[doc(alias = "rewinds")]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn restore(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    self.cache_mut().rewind(&checkpoint);
    let cur = checkpoint.cursor();
    self.emitter().rewind(cur, checkpoint.emitter_checkpoint);
    // Restore the dedup mark toward its value at save time, not to the cursor.
    // The emitter's emission-log rewind retains every error sealed *before* the
    // checkpoint — including one whose span sits above the cursor (a peek that
    // scanned ahead) — so dropping the mark to the cursor would let a re-lex
    // re-emit that retained error. Errors sealed *after* the checkpoint were
    // unwound, and the saved mark predates them, so a re-lexing commit path can
    // report them again.
    //
    // But restore must never *raise* the mark: clamp it to the min of the saved
    // and current values (`L::Offset: Ord`). Case walk:
    //   - LIFO restore (the common path): speculative work after `save` only ever
    //     raised the mark, so `current >= saved` → `min == saved` → the mark
    //     returns to its saved value, exactly as before (retained peek-ahead
    //     errors above the cursor stay deduped: exactly-once).
    //   - Stale younger restore after an older restore: the older restore already
    //     dropped the mark below this (younger, stale) checkpoint's saved mark and
    //     unwound the error from the emission log, so `current < saved` → `min ==
    //     current` keeps the mark low. The re-lex re-emits the error the log can
    //     no longer resurrect — no diagnostic lost.
    // Lowering the mark never double-emits: it only ever sinks into a range the
    // paired emitter rewind already cleared, so no still-retained error sits above
    // the clamped mark.
    if checkpoint.emitted_error_end < *self.emitted_error_end {
      *self.emitted_error_end = checkpoint.emitted_error_end;
    }
    self.set_span((&checkpoint.span).into());
    *self.state = checkpoint.state;
  }

  /// Advances the cursor and returns the next valid token, emitting errors encountered on the way.
  ///
  /// Skips over lexer errors, emitting them through the provided emitter.
  /// Non-fatal errors are emitted and the method continues to the next token.
  #[allow(clippy::should_implement_trait)]
  pub fn next(
    &mut self,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  {
    if let Some(cached_token) = self.cache_mut().pop_front() {
      let (spanned_lexed, extras) = cached_token.into_components();
      let (span, lexed) = spanned_lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = extras;
      return Ok(Some(Spanned::new(span, lexed)));
    }

    // A sticky limit trip latches the input: once the cache is drained, stop
    // without rebuilding a lexer or rescanning the tripping token.
    if self.is_poisoned() {
      return Ok(None);
    }

    self.lex_next_valid(|_, _| Ok(()))
  }

  /// Internal implementation for syncing tokens in the cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn sync_matched_in_cache<P, Exp>(
    &mut self,
    mut pred: P,
    mut exp: Exp,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    P: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
  {
    let matched = core::cell::RefCell::new(false);
    // pop from cache if not matching
    while let Some(tok) = self.cache.pop_front_if(|t| {
      let span = t.token().span();
      *matched.borrow_mut() = pred(Spanned::new(span, t.token().data()));
      !*matched.borrow()
    }) {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;

      // if matched, we stop here
      if *matched.borrow() {
        return Ok(Some(Spanned::new(span, tok)));
      }

      // Note: cursor/state are updated before emission. If emission fails,
      // the error token has still been consumed (no backtracking here).

      self
        .emitter()
        .emit_unexpected_token(UnexpectedToken::maybe_expected_of(span, exp()).with_found(tok))?;
    }
    Ok(None)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lex_next_valid<F>(
    &mut self,
    mut pred: F,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(
      Spanned<&L::Token, &L::Span>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
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

          let s = tok.span_ref().clone();

          // if the token matches, we return it
          let res = match pred(tok.as_ref(), self.emitter) {
            Ok(_) => Ok(Some(tok)),
            Err(e) => Err(e),
          };

          self.set_span_after_consume(s.into());
          *self.state = lexer.into_state();
          return res;
        }
      }
    }

    Ok(None)
  }
}

#[cfg_attr(not(tarpaulin), inline(always))]
fn to_owned<T>(maybe: MaybeRef<'_, T>) -> T
where
  T: Clone,
{
  match maybe {
    MaybeRef::Ref(r) => r.clone(),
    MaybeRef::Owned(o) => o,
  }
}
