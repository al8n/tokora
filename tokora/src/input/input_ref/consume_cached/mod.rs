use super::*;

impl<'inp, L, Ctx, Lang: ?Sized, Cmpl> InputRef<'inp, '_, L, Ctx, Lang, Cmpl>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  /// Consumes one token from the peeked tokens and returns the consumed token if any, the cursor is advanced.
  #[inline(always)]
  pub fn consume_cached_one(&mut self) -> Option<Spanned<L::Token, L::Span>> {
    let tok = self.cache_mut().pop_front()?;
    let (tok, extras): (Spanned<L::Token, L::Span>, _) = tok.into_components();
    self.commit_token(tok.data(), tok.span_ref());
    *self.state = extras;
    Some(tok)
  }

  /// Consumes tokens from cache until the predicate returns `true`.
  ///
  /// Advances the cursor to the end of the last consumed token.
  /// Returns the last consumed token.
  #[inline(always)]
  pub fn consume_cached_to<F>(&mut self, mut f: F) -> Option<Spanned<L::Token, L::Span>>
  where
    F: FnMut(CachedTokenRefOf<'_, 'inp, L>) -> bool,
  {
    let mut last = None;
    // pop from cache if not matching
    while let Some(tok) = self.cache_mut().pop_front_if(|t| !f(t)) {
      let (tok, state) = tok.into_components();
      self.commit_token(tok.data(), tok.span_ref());
      *self.state = state;
      last = Some(tok);
    }

    last
  }

  /// Consumes tokens from cache while the predicate returns `true`.
  ///
  /// Advances the cursor to the end of the last consumed token.
  /// Returns the last consumed token.
  #[inline(always)]
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
  ///
  /// Drains **per token** through [`consume_cached_to`](Self::consume_cached_to) (with a
  /// never-matching predicate), so every cached token — not only the last — settles through
  /// the one commit primitive. The observable result is unchanged: the cache empties, the
  /// cursor lands at the end of the last cached token with its state, and the last token is
  /// returned; but each token in the run commits individually, exactly as it would have had
  /// the caller consumed them one by one.
  #[inline(always)]
  pub fn consume_all_cached(&mut self) -> Option<Spanned<L::Token, L::Span>> {
    self.consume_cached_to(|_| false)
  }
}

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests;
