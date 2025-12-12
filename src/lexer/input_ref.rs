#![allow(clippy::type_complexity)]

use core::{
  marker::PhantomData,
  mem::ManuallyDrop,
  ops::{Range, RangeBounds},
};

use generic_arraydeque::{GenericArrayDeque, typenum::U1};
use mayber::{Maybe, MaybeRef};

use crate::{
  Token, Window,
  emitter::Emitter,
  error::token::UnexpectedToken,
  lexer::{CachedTokenRefOf, MaybeRefCachedTokenOf, peek::Peeked},
  utils::{Expected, Spanned},
};

use super::{Cache, CachedToken, Checkpoint, Cursor, Lexed, Lexer, Source, Span};

mod iter;

/// A reference to an [`Input`] instance.
pub struct InputRef<'inp, 'closure, L, E, C, Lang: ?Sized = ()>
where
  L: Lexer<'inp>,
{
  pub(super) input: &'closure &'inp L::Source,
  pub(super) state: &'closure mut L::State,
  pub(super) span: &'closure mut L::Span,
  pub(super) cache: &'closure mut C,
  pub(super) emitter: &'closure mut E,
  pub(super) _marker: PhantomData<Lang>,
}

impl<'inp, L, E, C, Lang: ?Sized> InputRef<'inp, '_, L, E, C, Lang>
where
  L: Lexer<'inp>,
{
  /// Returns a reference to the tokenizer's cache.
  ///
  /// The cache stores peeked tokens that have been lexed but not yet consumed.
  /// This can be useful for inspecting the cache state or implementing custom
  /// lookahead logic.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn cache(&self) -> &C {
    self.cache
  }

  const fn cache_mut(&mut self) -> &mut C {
    self.cache
  }

  /// Returns a reference to the underlying input source.
  ///
  /// This allows access to the raw source being tokenized, which is typically
  /// a `&str` or `&[u8]` depending on your Logos token definition.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn input(&self) -> &L::Source {
    self.input
  }

  /// Returns a reference to the current lexer state (extras)
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn state(&self) -> &L::State {
    self.state
  }

  /// Manually sets the lexer state (for context-sensitive lexing)
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_state(&mut self, state: L::State) {
    *self.state = state;
  }

  /// Returns a mutable reference to the emitter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn emitter(&mut self) -> &mut E {
    self.emitter
  }

  // /// Returns an iterator over the tokens of the lexer.
  // #[cfg_attr(not(tarpaulin), inline(always))]
  // pub const fn iter(&mut self) -> iter::Iter<'inp, '_, L, C> {
  //   iter::Iter::new(self)
  // }

  // /// Consumes the lexer and returns an iterator over the tokens of the lexer.
  // #[cfg_attr(not(tarpaulin), inline(always))]
  // pub const fn into_iter(self) -> iter::IntoIter<'inp, '_, L, C> {
  //   iter::IntoIter::new(self)
  // }

  /// Creates a lexer positioned at the end of the cache or current cursor.
  ///
  /// This internal method constructs a fresh Logos lexer with the current state and
  /// positions it to continue lexing from where the cache ends (or from the cursor
  /// if the cache is empty).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn lexer(&self) -> L
  where
    L::State: Clone,
    C: Cache<'inp, L>,
  {
    let mut lexer = L::with_state(self.input, self.state.clone());
    lexer.bump(
      self
        .cache()
        .last_span()
        .map(|s| s.end_ref())
        .unwrap_or_else(|| self.span.end_ref()),
    );
    lexer
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn lexer_at(&self, off: &L::Offset) -> L
  where
    L::State: Clone,
    C: Cache<'inp, L>,
  {
    let mut lexer = L::with_state(self.input, self.state.clone());
    lexer.bump(off);
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
  fn set_span_after_consume(&mut self, new: MaybeRef<'_, L::Span>)
  where
    C: Cache<'inp, L>,
  {
    let end = self.input.len();
    let cache = self.cache().first_span();

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

impl<'inp, 'closure, L, E, C, Lang: ?Sized> InputRef<'inp, 'closure, L, E, C, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  C: Cache<'inp, L>,
  E: Emitter<'inp, L, Lang>,
{
  /// Try parsing, returns `None` on failure (no error propagation)
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
        self.go(ckp);
        None
      }
    }
  }

  /// Consumes a token if it matches the predicate, returns `None` otherwise (no cursor advance on failure)
  pub fn accept<F>(&mut self, pred: F) -> Option<Spanned<L::Token, L::Span>>
  where
    F: FnOnce(&L::Token) -> bool,
  {
    if let Some(peeked) = self.cache().first() {
      match peeked.token().data() {
        Lexed::Token(tk) if pred(tk) => {
          let tok = self.cache_mut().pop_front().unwrap();
          let (spanned_lexed, extras) = tok.into_components();
          let (span, lexed) = spanned_lexed.into_components();
          self.set_span_after_consume((&span).into());
          *self.state = extras;
          return Some(Spanned::new(span, lexed.unwrap_token()));
        }
        _ => return None,
      }
    }

    let mut lexer = self.lexer();
    if let Some(lexed) = Lexed::<L::Token>::lex_spanned(&mut lexer) {
      let (span, lexed) = lexed.into_components();

      if let Lexed::Token(tk) = &lexed {
        if pred(tk) {
          self.set_span_after_consume(lexer.span().into());
          *self.state = lexer.into_state();
          return Some(Spanned::new(span, lexed.unwrap_token()));
        }
      }

      // cache the token as it was peeked
      let ct = CachedToken::new(Spanned::new(span, lexed), lexer.into_state());
      match self.cache_mut().push_back(ct) {
        Ok(_) => {}
        Err(_) => {
          // cache full, do nothing
        }
      }
    }

    None
  }

  /// Consumes the next token if it matches the predicate, otherwise returns an error.
  pub fn expect<F, Error>(
    &mut self,
    pred: F,
    error_fn: impl FnOnce(Lexed<'inp, L::Token>) -> Error,
  ) -> Result<Option<Spanned<L::Token, L::Span>>, Error>
  where
    F: FnOnce(&L::Token) -> bool,
  {
    if let Some(peeked) = self.cache().first() {
      match peeked.token().data() {
        Lexed::Token(tk) if pred(tk) => {
          let tok = self.cache_mut().pop_front().unwrap();
          let (spanned_lexed, extras) = tok.into_components();
          let (span, lexed) = spanned_lexed.into_components();
          self.set_span_after_consume((&span).into());
          *self.state = extras;
          return Ok(Some(Spanned::new(span, lexed.unwrap_token())));
        }
        _ => {
          let tok = self.cache_mut().pop_front().unwrap();
          let (spanned_lexed, extras) = tok.into_components();
          let (span, lexed) = spanned_lexed.into_components();
          self.set_span_after_consume(span.into());
          *self.state = extras;
          return Err(error_fn(lexed));
        }
      }
    }

    let mut lexer = self.lexer();

    if let Some(lexed) = Lexed::lex_spanned(&mut lexer) {
      let (span, lexed) = lexed.into_components();

      match &lexed {
        Lexed::Token(tk) if pred(tk) => {
          self.set_span_after_consume(lexer.span().into());
          *self.state = lexer.into_state();
          return Ok(Some(Spanned::new(span, lexed.unwrap_token())));
        }
        _ => {
          self.set_span_after_consume(lexer.span().into());
          *self.state = lexer.into_state();
          return Err(error_fn(lexed));
        }
      }
    }

    Ok(None)
  }

  /// Returns a slice of the current token from the input source.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice(&self) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    self.input.slice(self.span.start_ref()..self.span.end_ref())
  }

  /// Returns a slice of the input source from the given cursor to the current cursor of the tokenizer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn slice_since(
    &self,
    cursor: &Cursor<'inp, 'closure, L>,
  ) -> Option<<L::Source as Source<L::Offset>>::Slice<'inp>> {
    // let start = cursor.cursor;
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

  /// Returns a slice of the input source from the current cursor of the tokenizer to the end of the input.
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

  /// Returns the span of the current token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> &L::Span {
    self.span
  }

  /// Returns a span from the given cursor to the current cursor of the tokenizer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span_since(&self, cursor: &Cursor<'inp, 'closure, L>) -> L::Span {
    Span::new(cursor.as_inner().clone(), self.cursor().as_inner().clone())
  }

  /// Returns a span from the given cursor to the end of the input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span_from(&self, cursor: &Cursor<'inp, 'closure, L>) -> L::Span {
    Span::new(cursor.as_inner().clone(), self.input.len())
  }

  /// Returns a span from the given range of cursors.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn span_range(&self, range: Range<&Cursor<'inp, 'closure, L>>) -> L::Span {
    Span::new(range.start.as_inner().clone(), range.end.as_inner().clone())
  }

  /// Consumes one token from the peeked tokens and returns the consumed token if any, the cursor is advanced.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn consume_one(&mut self) -> Option<Spanned<Lexed<'inp, L::Token>, L::Span>> {
    let tok = self.cache_mut().pop_front()?;
    let (tok, extras): (Spanned<Lexed<'inp, L::Token>, L::Span>, _) = tok.into_components();
    self.set_span_after_consume(tok.span_ref().into());
    *self.state = extras;
    Some(tok)
  }

  /// Consumes tokens from cache until the predicate returns `true`, the cursor is advanced to the end of the last consumed token.
  ///
  /// Returns the last consumed token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn consume_until<F>(&mut self, mut f: F) -> Option<Spanned<Lexed<'inp, L::Token>, L::Span>>
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

  /// Consumes tokens from cache while the predicate returns `true`, the cursor is advanced to the end of the last consumed token.
  ///
  /// Returns the last consumed token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn consume_while<F>(&mut self, mut f: F) -> Option<Spanned<Lexed<'inp, L::Token>, L::Span>>
  where
    F: FnMut(CachedTokenRefOf<'_, 'inp, L>) -> bool,
  {
    self.consume_until(|t| !f(t))
  }

  /// Consumes all cached tokens, the cursor is advanced to the end of the last cached token.
  ///
  /// Returns the last consumed token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn consume_cached(&mut self) -> Option<Spanned<Lexed<'inp, L::Token>, L::Span>> {
    let last = self.cache_mut().pop_back()?;
    self.cache_mut().clear();
    let (tok, extras): (Spanned<Lexed<'inp, L::Token>, L::Span>, _) = last.into_components();
    self.set_span_after_consume(tok.span_ref().into());
    *self.state = extras;
    Some(tok)
  }

  /// Skips one token, advancing the cursor.
  ///
  /// If there's a token in the cache, it pops and discards it. Otherwise,
  /// it lexes the next token and discards it.
  ///
  /// Returns `true` if a token was skipped, `false` if the end of input was reached.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn skip_one(&mut self) -> bool {
    if let Some(cached_token) = self.cache_mut().pop_front() {
      let (spanned_lexed, extras) = cached_token.into_components();
      let (span, _lexed) = spanned_lexed.into_components();
      self.set_span_after_consume(span.into());
      *self.state = extras;
      true
    } else {
      self.next().is_some()
    }
  }

  /// Skips tokens until a valid token is found or the end of input is reached.
  ///
  /// Returns the first valid token found, but without consuming it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn skip_until<F>(&mut self, pred: F) -> Option<MaybeRefCachedTokenOf<'_, 'inp, L>>
  where
    F: FnMut(Spanned<&Lexed<'inp, L::Token>, &L::Span>) -> bool,
  {
    self.skip_until_then_peek::<_, U1>(pred).pop_front()
  }

  /// Skips tokens until a valid token is found or the end of input is reached.
  ///
  /// Returns the first valid token found, but without consuming it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn skip_until_then_peek<F, W>(&mut self, mut pred: F) -> Peeked<'_, 'inp, L, W>
  where
    F: FnMut(Spanned<&Lexed<'inp, L::Token>, &L::Span>) -> bool,
    W: Window,
  {
    // pop from cache if not matching
    while let Some(tok) = self.cache_mut().pop_front_if(|t| !pred(t.token().copied())) {
      self.set_span_after_consume(tok.token().span().into());
      *self.state = tok.state;
    }

    // as the matched token will not be consumed, we just peek it
    match !self.cache().is_empty() {
      // If the matched token is in cache, return it
      true => self.peek::<W>(),
      // Otherwise, let's skip the input
      false => {
        let mut lexer = self.lexer();
        let mut end = self.span.clone();
        let mut state = self.state.clone();

        while let Some(lexed) = Lexed::<L::Token>::lex_spanned(&mut lexer) {
          // if the token matches, we cache it and return it
          if pred(lexed.as_ref()) {
            self.set_span_after_consume(end.into());
            *self.state = state;

            return self.peek::<W>();
          }

          end = lexer.span();
          state = lexer.state().clone();
        }

        // No matched token found, we just update the cursor and state
        self.set_span_after_consume(lexer.span().into());
        *self.state = lexer.into_state();

        GenericArrayDeque::new()
      }
    }
  }

  /// Skips tokens while the predicate returns `true`.
  ///
  /// Returns the first token that does not match the predicate, but without consuming it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn skip_while<F>(&mut self, mut pred: F) -> Option<MaybeRefCachedTokenOf<'_, 'inp, L>>
  where
    F: FnMut(Spanned<&Lexed<'inp, L::Token>, &L::Span>) -> bool,
  {
    self.skip_until(|t| !pred(t))
  }

  /// Skip past lexer errors until the next valid token or end of input (no emission).
  ///
  /// Advances over error tokens without emitting them, stopping before the first
  /// non-error token (if any). Returns that token without consuming it so the
  /// caller can decide what to do next. Any cached errors are discarded; use
  /// `sync_until*` to emit them.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn skip_until_token(&mut self) -> Option<MaybeRefCachedTokenOf<'_, 'inp, L, L::Token>> {
    self
      .skip_until_then_peek::<_, U1>(|t| matches!(t.data, Lexed::Token(_)))
      .pop_front()
      .map(|t| {
        t.map(
          |t| t.map_token(|t| t.unwrap_token_ref()),
          |t| t.map_token(|t| t.unwrap_token()),
        )
      })
  }

  /// Resynchronize by skipping and emitting lexer errors until a valid token or end of input.
  ///
  /// Emits every lexer error encountered while advancing. Stops before the first
  /// non-error token (if any) and returns it without consuming, allowing the caller
  /// to resume parsing. If emission fails, returns that error immediately.
  /// Non-matching non-error tokens are skipped but also reported via `emit_unexpected_token`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn sync_until_token(
    &mut self,
  ) -> Result<Option<MaybeRefCachedTokenOf<'_, 'inp, L, L::Token>>, E::Error> {
    self.sync_until_token_then_peek::<U1>().map(|mut val| {
      val.pop_front().map(|t| {
        t.map(
          |t| t.map_token(|t| t.unwrap_token_ref()),
          |t| t.map_token(|t| t.unwrap_token()),
        )
      })
    })
  }

  /// Resynchronize by skipping and emitting lexer errors until a valid token or end of input.
  ///
  /// Emits every lexer error encountered while advancing. Stops before the first
  /// non-error token (if any) and returns it without consuming, allowing the caller
  /// to resume parsing. If emission fails, returns that error immediately.
  /// Non-matching non-error tokens are skipped but also reported via `emit_unexpected_token`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn sync_until_token_then_peek<'p, W>(&'p mut self) -> Result<Peeked<'p, 'inp, L, W>, E::Error>
  where
    W: Window,
  {
    self
      .sync_until_token_then_peek_with_emitter::<W>()
      .map(|(out, _)| out)
  }

  /// Resynchronize by skipping and emitting lexer errors until a valid token or end of input.
  ///
  /// Emits every lexer error encountered while advancing. Stops before the first
  /// non-error token (if any) and returns it without consuming, allowing the caller
  /// to resume parsing. If emission fails, returns that error immediately.
  /// Non-matching non-error tokens are skipped but also reported via `emit_unexpected_token`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn sync_until_token_then_peek_with_emitter<'p, W>(
    &'p mut self,
  ) -> Result<(Peeked<'p, 'inp, L, W>, &'p mut E), E::Error>
  where
    W: Window,
  {
    self.sync_until_then_peek_with_emitter::<_, _, W>(|_, _| true, || None)
  }

  /// Skip tokens until the predicate matches, emitting lexer errors along the way.
  ///
  /// Advances through the stream, emitting each lexer error via the emitter. Stops
  /// before the first token for which `pred` returns `true` and returns it (without
  /// consuming). Non-matching non-error tokens are skipped but also reported via
  /// `emit_unexpected_token`. If emission fails, returns that error. If no matching
  /// token is found, returns `None`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn sync_until<F, Exp>(
    &mut self,
    pred: F,
    exp: Exp,
  ) -> Result<Option<MaybeRefCachedTokenOf<'_, 'inp, L>>, E::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>, &mut E) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
  {
    self
      .sync_until_then_peek_with_emitter::<_, _, U1>(pred, exp)
      .map(|(mut out, _)| out.pop_front())
  }

  /// Skip tokens until the predicate matches, emitting lexer errors along the way.
  ///
  /// Advances through the stream, emitting each lexer error via the emitter. Stops
  /// before the first token for which `pred` returns `true` and returns it (without
  /// consuming). Non-matching non-error tokens are skipped but also reported via
  /// `emit_unexpected_token`. If emission fails, returns that error. If no matching
  /// token is found, returns `None`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn sync_until_then_peek_with_emitter<'p, F, Exp, W>(
    &'p mut self,
    mut pred: F,
    mut exp: Exp,
  ) -> Result<(Peeked<'p, 'inp, L, W>, &'p mut E), E::Error>
  where
    F: FnMut(Spanned<&L::Token, &L::Span>, &mut E) -> bool,
    Exp: FnMut() -> Option<Expected<'inp, <L::Token as Token<'inp>>::Kind>>,
    W: Window,
  {
    // pop from cache if not matching
    while let Some(tok) = self.cache.pop_front_if(|t| {
      let span = t.token().span();
      match t.token().data() {
        Lexed::Token(tok) => !pred(Spanned::new(span, tok), self.emitter),
        Lexed::Error(_) => true,
      }
    }) {
      let (lexed, state) = tok.into_components();
      let (span, tok) = lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = state;

      // Note: cursor/state are updated before emission. If emission fails,
      // the error token has still been consumed (no backtracking here).

      match tok {
        Lexed::Error(e) => {
          self
            .emitter()
            .emit_lexer_error(Spanned::new(span.clone(), e))?;
        }
        Lexed::Token(unexpected) => {
          self.emitter().emit_unexpected_token(
            UnexpectedToken::maybe_expected_of(span, exp()).with_found(unexpected),
          )?;
        }
      }
    }

    // as the matched token will not be consumed, we just peek it
    match !self.cache().is_empty() {
      // If the matched token is in cache, return it
      true => Ok(self.peek_with_emitter::<W>()),
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
              if pred(tok.as_ref(), self.emitter) {
                self.set_span_after_consume(end.into());
                *self.state = state;
                return Ok(self.peek_with_emitter::<W>());
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

  /// Peeks the next token without advancing the cursor.
  #[inline]
  pub fn peek_one(&mut self) -> Option<MaybeRefCachedTokenOf<'_, 'inp, L>> {
    let mut buf = GenericArrayDeque::<_, U1>::new();
    self.peek_with_emitter_inner::<U1>(&mut buf);
    buf.pop_front()
  }

  /// Try to peeks tokens to fill the provided buffer, if not enough tokens are cached, lex more tokens to fill the buffer.
  ///
  /// The returned slice will contain only the initialized tokens.
  #[inline]
  pub fn peek<'p, W>(&'p mut self) -> Peeked<'p, 'inp, L, W>
  where
    W: Window,
  {
    self.peek_with_emitter::<W>().0
  }

  /// Try to peeks tokens to fill the provided buffer, if not enough tokens are cached, lex more tokens to fill the buffer.
  ///
  /// The returned slice will contain only the initialized tokens.
  #[inline]
  pub fn peek_with_emitter<'p, W>(&'p mut self) -> (Peeked<'p, 'inp, L, W>, &'p mut E)
  where
    W: Window,
  {
    let mut peeked = GenericArrayDeque::new();
    let emitter = self.peek_with_emitter_inner::<W>(&mut peeked);

    (peeked, emitter)
  }

  /// Try to peeks tokens to fill the provided buffer, if not enough tokens are cached, lex more tokens to fill the buffer.
  ///
  /// The returned slice will contain only the initialized tokens.
  #[inline]
  fn peek_with_emitter_inner<'p, W>(&'p mut self, buf: &mut Peeked<'p, 'inp, L, W>) -> &'p mut E
  where
    W: Window,
  {
    let remaining_cap = buf.remaining_capacity();
    let mut in_cache = self.cache().len();
    let mut want = remaining_cap.saturating_sub(in_cache);

    // If we already have enough tokens cached, just peek from cache
    if want == 0 {
      self.cache.peek::<W>(buf);
      return self.emitter;
    }

    let mut overflowed = ManuallyDrop::new(W::array());

    let mut yielded = 0;
    // Otherwise, lex additional tokens to fill the request
    let mut lexer = self.lexer();
    while want > 0 {
      if let Some(lexed) = Lexed::lex_spanned(&mut lexer) {
        let (span, lexed) = lexed.into_components();
        let cached = CachedToken::new(Spanned::new(span, lexed), lexer.state().clone());

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
      } else {
        break;
      }
    }

    // Fill buffer from cache (this covers both cached tokens and any we just added)
    // SAFETY: Cache.peek() returns slice of initialized tokens, guaranteed by trait contract
    self.cache.peek::<W>(buf);
    debug_assert!(
      buf.len() - remaining_cap == in_cache,
      "Cache peek returned unexpected number of tokens"
    );

    for i in 0..yielded {
      // SAFETY: We just wrote `yielded` elements into `overflowed`, so the first `yielded` elements are initialized.
      unsafe {
        buf.push_back(overflowed[i].assume_init_read());
      }
    }

    self.emitter
  }

  /// Saves the current state of the tokenizer as a checkpoint.
  ///
  /// This creates a snapshot of the current position and lexer state, which can
  /// later be restored using [`go`](Self::go). Checkpoints are essential for
  /// implementing backtracking in parsers.
  ///
  /// # Example
  ///
  /// ```ignore
  /// let checkpoint = tokenizer.save();
  /// // Try parsing something...
  /// if parsing_failed {
  ///     tokenizer.go(checkpoint); // Restore state
  /// }
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn save(&self) -> Checkpoint<'inp, 'closure, L> {
    Checkpoint::new(self.cursor().clone(), self.state.clone())
  }

  /// Returns the current cursor position of the tokenizer.
  ///
  /// The cursor represents the byte offset in the input where the tokenizer will
  /// continue lexing. If there are cached tokens, the cursor points to the start
  /// of the first cached token; otherwise, it points to the current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn cursor(&self) -> &Cursor<'inp, 'closure, L> {
    Cursor::from_ref(self.cache().first_span().unwrap_or(self.span))
  }

  /// Restores the tokenizer state to a previously saved checkpoint.
  ///
  /// This rewinds the cache, resets the cursor position, and restores the lexer
  /// state, effectively undoing all operations since the checkpoint was created.
  /// This is commonly used for parser backtracking.
  #[doc(alias = "rewinds")]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn go(&mut self, checkpoint: Checkpoint<'inp, '_, L>) {
    self.cache_mut().rewind(&checkpoint);
    let cur = checkpoint.cursor();
    self.emitter().rewind(cur);
    self.set_span(cur.span().into());
    *self.state = checkpoint.state;
  }

  /// Advances the cursor and returns the next valid token, emitting errors via the provided emitter.
  ///
  /// This method skips over lexer errors, emitting them through the provided emitter.
  /// Non-fatal errors are emitted and the method continues to the next token. If a
  /// fatal error occurs during emission, it's returned and processing stops.
  ///
  /// Returns `Ok(Some(token))` for valid tokens, `Ok(None)` at end of input, or
  /// `Err(error)` if a fatal error occurred.
  pub fn next_token(&mut self) -> Result<Option<Spanned<L::Token, L::Span>>, E::Error> {
    // First, consume from cache if available
    while let Some(cached_token) = self.cache_mut().pop_front() {
      let (spanned_lexed, extras) = cached_token.into_components();
      let (span, lexed) = spanned_lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = extras;
      match lexed {
        Lexed::Token(t) => return Ok(Some(Spanned::new(span, t))),
        Lexed::Error(e) => {
          self.emitter().emit_lexer_error(Spanned::new(span, e))?;
          continue;
        }
      }
    }

    // then, construct a lexer and lex until a valid token is found
    let mut lexer = self.lexer();

    while let Some(lexed) = Lexed::lex_spanned(&mut lexer) {
      let (span, lexed) = lexed.into_components();
      self.set_span_after_consume(lexer.span().into());
      *self.state = lexer.state().clone();

      match lexed {
        Lexed::Token(t) => return Ok(Some(Spanned::new(span, t))),
        Lexed::Error(e) => {
          self.emitter().emit_lexer_error(Spanned::new(span, e))?;
          continue;
        }
      }
    }

    Ok(None)
  }

  /// Advances the cursor and returns the next token (valid or error).
  ///
  /// Unlike [`next_valid_with`](Self::next_valid_with), this method returns both
  /// valid tokens and lexer errors wrapped in [`Lexed`]. The cursor advances
  /// regardless of whether a valid token or error is returned.
  ///
  /// Returns `Some(Spanned<Lexed>)` with either a token or error, or `None` at
  /// end of input.
  #[allow(clippy::should_implement_trait)]
  pub fn next(&mut self) -> Option<Spanned<Lexed<'inp, L::Token>, L::Span>> {
    if let Some(cached_token) = self.cache_mut().pop_front() {
      let (spanned_lexed, extras) = cached_token.into_components();
      let (span, lexed) = spanned_lexed.into_components();
      self.set_span_after_consume((&span).into());
      *self.state = extras;
      return Some(Spanned::new(span, lexed));
    }

    let mut lexer = self.lexer();
    Lexed::lex_spanned(&mut lexer).inspect(|_| {
      self.set_span_after_consume(lexer.span().into());
      *self.state = lexer.state().clone();
    })
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
