#![allow(clippy::type_complexity)]

use core::{
  convert::identity,
  marker::PhantomData,
  mem::ManuallyDrop,
  ops::{Range, RangeBounds},
};

use generic_arraydeque::{GenericArrayDeque, typenum::U1};
use mayber::{Maybe, MaybeRef};

use crate::{
  ParseContext, Token, Window,
  cache::{CachedToken, CachedTokenRefOf, MaybeRefCachedTokenOf, Peeked},
  emitter::Emitter,
  error::{UnexpectedEot, token::UnexpectedToken},
  span::Spanned,
  utils::Expected,
};

use super::{Cache, Checkpoint, Cursor, Lexed, Lexer, Source, Span};

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

  /// Creates a lexer without state positioned at the current offset.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lexer_no_state(&self) -> L {
    let mut lexer = L::new(self.input);
    lexer.bump(self.offset());
    lexer
  }

  /// Sets the cursor to the specified position, clamped to the input length.
  ///
  /// This ensures the cursor never exceeds the bounds of the input source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn set_span(&mut self, new: MaybeRef<'_, L::Span>) {
    let end = self.input.len();
    *self.span = if new.end_ref().lt(&end) {
      to_owned(new)
    } else {
      L::Span::new(L::Offset::default(), end)
    };
  }

  /// Sets the cursor to the latest position between the new value and the cache start.
  ///
  /// This method ensures the cursor is positioned at or after the first cached token
  /// (if any), preventing the cursor from moving backwards past cached tokens.
  /// The cursor is also clamped to the input length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn set_span_after_consume(&mut self, new: MaybeRef<'_, L::Span>) {
    let end = self.input.len();
    let cache = self.cache().front_span();

    let off = match cache {
      Some(off) => {
        if new.end_ref().lt(off.end_ref()) {
          off.clone()
        } else {
          to_owned(new)
        }
      }
      None => {
        if new.end_ref().lt(&end) {
          to_owned(new)
        } else {
          L::Span::new(new.start_ref().clone(), end)
        }
      }
    };

    *self.span = off;
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
    let cur = self.cursor().span().clone();
    let state = self.state.clone();

    match f(self) {
      Some(result) => Some(result),
      None => {
        let ckp = Checkpoint::new(Cursor::new(cur), state);
        self.restore(ckp);
        None
      }
    }
  }

  /// Returns a slice of the current token from the input source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice(&self) -> <L::Source as Source<L::Offset>>::Slice<'inp> {
    self.lexer_no_state().slice()
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

  /// Consumes one token from the peeked tokens and returns the consumed token if any, the cursor is advanced.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn consume_cached_one(&mut self) -> Option<Spanned<L::Token, L::Span>> {
    let tok = self.cache_mut().pop_front()?;
    let (tok, extras): (Spanned<L::Token, L::Span>, _) = tok.into_components();
    self.set_span_after_consume(tok.span_ref().into());
    *self.state = extras;
    Some(tok)
  }

  /// Consumes tokens from cache until the predicate returns `true`.
  ///
  /// Advances the cursor to the end of the last consumed token.
  /// Returns the last consumed token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn consume_cached_to<F>(&mut self, mut f: F) -> Option<Spanned<L::Token, L::Span>>
  where
    F: FnMut(CachedTokenRefOf<'_, 'inp, L>) -> bool,
  {
    let mut last = None;
    // pop from cache if not matching
    while let Some(tok) = self.cache_mut().pop_front_if(|t| !f(t)) {
      self.set_span_after_consume(tok.token().span().into());
      let (tok, state) = tok.into_components();
      *self.state = state;
      last = Some(tok);
    }

    last
  }

  /// Consumes tokens from cache while the predicate returns `true`.
  ///
  /// Advances the cursor to the end of the last consumed token.
  /// Returns the last consumed token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn consume_cached_while<F>(&mut self, mut f: F) -> Option<Spanned<L::Token, L::Span>>
  where
    F: FnMut(CachedTokenRefOf<'_, 'inp, L>) -> bool,
  {
    self.consume_cached_to(|t| !f(t))
  }

  /// Consumes all cached tokens.
  ///
  /// Advances the cursor to the end of the last cached token.
  /// Returns the last consumed token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn consume_all_cached(&mut self) -> Option<Spanned<L::Token, L::Span>> {
    let last = self.cache_mut().pop_back()?;
    self.cache_mut().clear();
    let (tok, extras): (Spanned<L::Token, L::Span>, _) = last.into_components();
    self.set_span_after_consume(tok.span_ref().into());
    *self.state = extras;
    Some(tok)
  }

  /// Resynchronize by skipping and emitting lexer errors until a valid token or end of input.
  ///
  /// Emits every lexer error encountered while advancing. Stops before the first
  /// non-error token (if any) and returns it without consuming.
  /// Non-matching non-error tokens are skipped but also reported via `emit_unexpected_token`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn sync_errors(
    &mut self,
  ) -> Result<
    Option<MaybeRefCachedTokenOf<'_, 'inp, L, L::Token>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    self
      .sync_errors_then_peek::<U1>()
      .map(|mut val| val.pop_front())
  }

  /// Resynchronize by skipping and emitting lexer errors until a valid token or end of input.
  ///
  /// Returns peeked tokens (up to window size) after synchronization.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn sync_errors_then_peek<'p, W>(
    &'p mut self,
  ) -> Result<Peeked<'p, 'inp, L, W>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    W: Window,
  {
    self
      .sync_errors_then_peek_with_emitter::<W>()
      .map(|(out, _)| out)
  }

  /// Resynchronize by skipping and emitting lexer errors until a valid token or end of input.
  ///
  /// Returns peeked tokens and a mutable reference to the emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn sync_errors_then_peek_with_emitter<'p, W>(
    &'p mut self,
  ) -> Result<
    (Peeked<'p, 'inp, L, W>, &'p mut Ctx::Emitter),
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    W: Window,
  {
    self.sync_to_then_peek_with_emitter::<_, _, W>(|_| true, || None)
  }

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
    self.sync_matched_in_cache(&mut pred, &mut exp)?;

    // as the matched token will not be consumed, we just peek it
    match !self.cache().is_empty() {
      // If the matched token is in cache, return it
      true => self.peek_with_emitter::<W>(),
      // Otherwise, let's skip the input
      false => {
        let mut lexer = self.lexer();

        let mut end = self.span.clone();
        let mut state = self.state.clone();

        while let Some(Spanned { span, data: tok }) = Lexed::<L::Token>::lex_spanned(&mut lexer) {
          match tok {
            Lexed::Error(err) => match self.emitter().emit_lexer_error(Spanned::new(span, err)) {
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
              // if the token matches, we cache it and return it
              if pred(tok.as_ref()) {
                self.set_span_after_consume(end.into());
                *self.state = state;
                return self.peek_with_emitter::<W>();
              } else {
                let (span, tok) = tok.into_components();
                self.emitter().emit_unexpected_token(
                  UnexpectedToken::maybe_expected_of(span, exp()).with_found(tok),
                )?;
              }

              end = lexer.span();
              state = lexer.state().clone();
            }
          }
        }

        // No matched token found, we just update the cursor and state
        self.set_span_after_consume(lexer.span().into());
        *self.state = lexer.into_state();

        Ok((GenericArrayDeque::new(), self.emitter))
      }
    }
  }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Ok`, the token is consumed and returned.
  /// Otherwise, the error is returned and the token remains in the cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn expect<F>(
    &mut self,
    mut pred: F,
  ) -> Result<Spanned<L::Token, L::Span>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(
      Spanned<&L::Token, &L::Span>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    let tok = self.sync_to_next_valid_then_try_match_in_cache(&mut pred)?;

    match tok {
      // found the token in cache
      Some(tok) => Ok(tok),
      // need to lex from input
      None => {
        // find the next valid token
        match self.sync_through(|_| true, || None)? {
          // no more tokens, end of input
          None => Err(UnexpectedEot::eot_of(self.span().end()).into()),
          // got a valid token, try to match it
          Some(tok) => {
            match pred(tok.as_ref(), self.emitter) {
              // matched, consume it
              Ok(_) => Ok(tok),
              // not matched, put back into cache and return error
              Err(e) => {
                // put back the token into cache as it was peeked
                let ct = CachedToken::new(tok, self.state.clone());
                let _ = self.cache_mut().push_back(ct);
                Err(e)
              }
            }
          }
        }
      }
    }
  }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Ok`, the token is consumed and returned.
  /// Otherwise, the error is returned and the token remains in the cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn expect_then_map<O, F, M>(
    &mut self,
    mut pred: F,
    map: M,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(
      Spanned<&L::Token, &L::Span>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
    M: FnOnce(Spanned<L::Token, L::Span>) -> O,
  {
    let tok = self.sync_to_next_valid_then_try_match_in_cache(&mut pred)?;

    match tok {
      // found the token in cache
      Some(tok) => Ok(map(tok)),
      // need to lex from input
      None => self.lex_next_valid(pred).and_then(|tok| match tok {
        Some(tok) => Ok(map(tok)),
        None => Err(UnexpectedEot::eot_of(self.span().end()).into()),
      }),
    }
  }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Ok`, the token is consumed and returned.
  /// Otherwise, the error is returned and the token remains in the cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn try_expect<F>(
    &mut self,
    pred: F,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    self.try_expect_then_map(pred, identity)
  }

  /// Advances to the next valid token and expects it to satisfy the predicate.
  ///
  /// Emits any lexer errors encountered. If a valid token is found, calls `pred`.
  /// If `pred` returns `Ok`, the token is consumed and returned.
  /// Otherwise, the error is returned and the token remains in the cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn try_expect_then_map<O, F, M>(
    &mut self,
    mut pred: F,
    map: M,
  ) -> Result<Option<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>) -> bool,
    M: FnOnce(Spanned<L::Token, L::Span>) -> O,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  {
    let (exhausted, tok) = self.try_sync_to_next_valid_then_try_match_in_cache(&mut pred)?;

    if !exhausted {
      return Ok(tok.map(map));
    }

    match tok {
      // found the token in cache
      Some(tok) => Ok(Some(map(tok))),
      // need to lex from input
      None => self.lex_next_matches(pred).map(|tok| tok.map(map)),
    }
  }

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

    // as the matched token will not be consumed, we just peek it
    match !self.cache().is_empty() {
      // If the matched token is in cache, return it
      true => {
        let (peeked, emitter) = self.peek_with_emitter::<W>()?;
        Ok((None, peeked, emitter))
      }
      // Otherwise, let's skip the input
      false => {
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

  /// Peeks the next token without advancing the cursor.
  #[inline]
  pub fn peek_one(
    &mut self,
  ) -> Result<
    Option<MaybeRefCachedTokenOf<'_, 'inp, L>>,
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  > {
    let mut buf = GenericArrayDeque::<_, U1>::new();
    self
      .peek_with_emitter_inner::<U1>(&mut buf)
      .map(|_| buf.pop_front())
  }

  /// Peeks tokens to fill the provided buffer.
  ///
  /// If not enough tokens are cached, lexes more tokens to fill the buffer.
  /// The returned deque contains references to peeked tokens.
  #[inline]
  pub fn peek<'p, W>(
    &'p mut self,
  ) -> Result<Peeked<'p, 'inp, L, W>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    W: Window,
  {
    self.peek_with_emitter::<W>().map(|(peeked, _)| peeked)
  }

  /// Peeks tokens to fill the provided buffer and returns the emitter.
  #[inline]
  pub fn peek_with_emitter<'p, W>(
    &'p mut self,
  ) -> Result<
    (Peeked<'p, 'inp, L, W>, &'p mut Ctx::Emitter),
    <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  >
  where
    W: Window,
  {
    let mut peeked = GenericArrayDeque::new();
    self
      .peek_with_emitter_inner::<W>(&mut peeked)
      .map(|emitter| (peeked, emitter))
  }

  /// Internal implementation for peeking tokens.
  #[inline]
  #[allow(unused_assignments)]
  fn peek_with_emitter_inner<'p, W>(
    &'p mut self,
    buf: &mut Peeked<'p, 'inp, L, W>,
  ) -> Result<&'p mut Ctx::Emitter, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    W: Window,
  {
    let buf_len = buf.len();
    let remaining_cap = buf.capacity() - buf_len;
    let mut in_cache = self.cache().len();
    let mut want = remaining_cap.saturating_sub(in_cache);
    let exp = want;

    // If we already have enough tokens cached, just peek from cache
    if want == 0 {
      self.cache.peek::<W>(buf);
      return Ok(self.emitter);
    }

    let mut overflowed = ManuallyDrop::new(W::array());

    let mut yielded = 0;
    // Otherwise, lex additional tokens to fill the request
    let mut lexer = self.lexer();
    while want > 0 {
      if let Some(lexed) = Lexed::lex_spanned(&mut lexer) {
        let (span, lexed) = lexed.into_components();

        match lexed {
          Lexed::Error(e) => {
            if self.cache.remaining() > 0 {
              self.emitter().emit_lexer_error(Spanned::new(span, e))?;
            }
          }
          Lexed::Token(tok) => {
            let cached = CachedToken::new(Spanned::new(span, tok), lexer.state().clone());

            // Try to cache the token; if cache is full, write directly to output buffer
            match self.cache_mut().push_back(cached) {
              Ok(_) => {
                in_cache += 1;
              }
              Err(ct) => {
                // Cache full: write overflow tokens directly to overflow buffer
                overflowed[yielded].write(Maybe::Owned(ct));
                yielded += 1;
              }
            }
            want -= 1;
          }
        }
      } else {
        break;
      }
    }

    // Fill buffer from cache (this covers both cached tokens and any we just added)
    // SAFETY: Cache.peek() returns slice of initialized tokens, guaranteed by trait contract
    self.cache.peek::<W>(buf);
    debug_assert!(
      buf_len + in_cache == buf.len(),
      "Cache peek returned unexpected number of tokens"
    );

    for i in 0..yielded {
      // SAFETY: We just wrote `yielded` elements into `overflowed`, so the first `yielded` elements are initialized.
      unsafe {
        buf.push_back(overflowed[i].assume_init_read());
      }
    }
    debug_assert!(
      buf.len() == buf_len + in_cache + yielded,
      "buffer length mismatch after adding overflowed tokens"
    );
    debug_assert!(
      exp == in_cache + yielded,
      "expected peeked token count mismatch"
    );

    Ok(self.emitter)
  }

  /// Saves the current state of the tokenizer as a checkpoint.
  ///
  /// This creates a snapshot of the current position and lexer state, which can
  /// later be restored using [`restore`](Self::restore). Checkpoints are essential for
  /// implementing backtracking in parsers.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn save(&self) -> Checkpoint<'inp, 'closure, L> {
    Checkpoint::new(self.cursor().clone(), self.state.clone())
  }

  /// Returns the current cursor position.
  ///
  /// If there are cached tokens, the cursor points to the start
  /// of the first cached token; otherwise, it points to the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn cursor(&self) -> &Cursor<'inp, 'closure, L> {
    Cursor::from_ref(self.cache().front_span().unwrap_or(self.span))
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
  #[doc(alias = "rewinds")]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn restore(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    self.cache_mut().rewind(&checkpoint);
    let cur = checkpoint.cursor();
    self.emitter().rewind(cur);
    self.set_span(cur.span().into());
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

    self.next_inner()
  }

  /// Internal implementation for advancing to the next token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next_inner(
    &mut self,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  {
    let mut lexer = self.lexer();
    loop {
      match Lexed::lex_spanned(&mut lexer).inspect(|_| {
        self.set_span_after_consume(lexer.span().into());
        *self.state = lexer.state().clone();
      }) {
        Some(lexed) => {
          let (span, lexed) = lexed.into_components();
          match lexed {
            Lexed::Token(t) => return Ok(Some(Spanned::new(span, t))),
            Lexed::Error(e) => {
              self.emitter().emit_lexer_error(Spanned::new(span, e))?;
            }
          }
        }
        None => return Ok(None),
      }
    }
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

  /// Internal implementation for syncing tokens in the cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn sync_to_next_valid_then_try_match_in_cache<P>(
    &mut self,
    mut pred: P,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    P: FnMut(
      Spanned<&L::Token, &L::Span>,
      &mut Ctx::Emitter,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    // pop from cache if not matching
    if let Some(tok) = self.cache.pop_front() {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;

      // Note: cursor/state are updated before emission. If emission fails,
      // the error token has still been consumed (no backtracking here).

      return match pred(Spanned::new(&span, &tok), self.emitter) {
        Ok(_) => Ok(Some(Spanned::new(span, tok))),
        Err(e) => Err(e),
      };
    }
    Ok(None)
  }

  /// Internal implementation for syncing tokens in the cache.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn try_sync_to_next_valid_then_try_match_in_cache<P>(
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
    if let Some(tok) = self.cache.pop_front() {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;

      // Note: cursor/state are updated before emission. If emission fails,
      // the error token has still been consumed (no backtracking here).

      return match pred(Spanned::new(&span, &tok)) {
        true => Ok((false, Some(Spanned::new(span, tok)))),
        false => Ok((false, None)),
      };
    }
    Ok((true, None))
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

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn lex_next_matches<F>(
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
