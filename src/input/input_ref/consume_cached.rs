use super::*;

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
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
}
