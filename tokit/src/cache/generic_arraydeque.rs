use mayber::Maybe;

use crate::lexer::Lexer;

use super::{
  Cache, CachedToken, CachedTokenOf, CachedTokenRefOf, Checkpoint, MaybeRefCachedTokenOf, Span,
};

use generic_arraydeque::{ArrayLength, GenericArrayDeque};

impl<'a, L, Lang: ?Sized, N> Cache<'a, L, Lang>
  for GenericArrayDeque<CachedToken<L::Token, L::State, L::Span>, N>
where
  L: Lexer<'a>,
  N: ArrayLength,
{
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn new() -> Self {
    Self::new()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn with_options(_options: ()) -> Self {
    Self::new()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    self.len()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn remaining(&self) -> usize {
    self.remaining_capacity()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, ckp: &Checkpoint<'a, '_, L>)
  where
    Self: Sized,
  {
    if self.is_empty() {
      return;
    }

    let cursor = ckp.cursor();
    // if the rewind position is before the start of the cache, clear the cache
    if let Some(span) = self.front().map(|tok| tok.token().span()) {
      if cursor.as_inner() < span.start_ref() {
        self.clear();
        return;
      }

      // If the rewind position is exactly at the start of the cache, do nothing
      if cursor.as_inner() == span.start_ref() {
        return;
      }
    }

    // if the rewind position is after the end of the cache, clear the cache
    if let Some(span) = self.back().map(|tok| tok.token().span()) {
      if cursor.as_inner() >= span.end_ref() {
        self.clear();
        return;
      }
    }

    let off = cursor.as_inner();
    match self.binary_search_by_key(off, |tok| tok.token().span_ref().start()) {
      Ok(_) => {
        self.retain(|tok| tok.token().span_ref().start().ge(off));
      }
      Err(_) => {
        self.clear();
      }
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_front(
    &mut self,
    tok: CachedTokenOf<'a, L>,
  ) -> Result<CachedTokenRefOf<'_, 'a, L>, CachedTokenOf<'a, L>> {
    match self.push_front_mut(tok) {
      Ok(tok) => Ok(tok.as_ref()),
      Err(tok) => Err(tok),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_back(
    &mut self,
    tok: CachedTokenOf<'a, L>,
  ) -> Result<CachedTokenRefOf<'_, 'a, L>, CachedTokenOf<'a, L>> {
    match self.push_back_mut(tok) {
      Ok(tok) => Ok(tok.as_ref()),
      Err(tok) => Err(tok),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn pop_front(&mut self) -> Option<CachedTokenOf<'a, L>> {
    self.pop_front()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn pop_back(&mut self) -> Option<CachedTokenOf<'a, L>> {
    self.pop_back()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn pop_front_if<F>(&mut self, predicate: F) -> Option<CachedTokenOf<'a, L>>
  where
    F: FnOnce(CachedTokenRefOf<'_, 'a, L>) -> bool,
    L: Lexer<'a>,
  {
    self.pop_front_if(|tok| predicate(tok.as_ref()))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clear(&mut self) {
    self.clear();
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn peek<'p, W>(
    &'p self,
    buf: &mut GenericArrayDeque<MaybeRefCachedTokenOf<'p, 'a, L>, W::CAPACITY>,
  ) where
    W: crate::Window,
  {
    let fill = buf.remaining_capacity().min(self.len());
    for tok in self.iter().take(fill) {
      buf.push_back(Maybe::Ref(tok.as_ref()));
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn front(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
    self.front().map(|tok| tok.as_ref())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn back(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
    self.back().map(|tok| tok.as_ref())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{DummyLexer, DummyToken};
  use crate::span::{SimpleSpan, Spanned};
  use generic_arraydeque::typenum::U3;

  type DequeCache = GenericArrayDeque<CachedToken<DummyToken, (), SimpleSpan>, U3>;

  fn make_token(start: usize, end: usize) -> CachedToken<DummyToken, (), SimpleSpan> {
    CachedToken::new(Spanned::new(SimpleSpan::new(start, end), DummyToken), ())
  }

  #[test]
  fn deque_cache_new_is_empty() {
    let cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 0);
    assert!(<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
  }

  #[test]
  fn deque_cache_with_options_is_empty() {
    let cache = <DequeCache as Cache<'_, DummyLexer>>::with_options(());
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 0);
  }

  #[test]
  fn deque_cache_push_back_and_len() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let tok = make_token(0, 5);
    let result = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, tok);
    assert!(result.is_ok());
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
    assert!(!<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
  }

  #[test]
  fn deque_cache_push_back_multiple() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 3);
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 0);
  }

  #[test]
  fn deque_cache_push_back_when_full_returns_err() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
    let result = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(15, 20));
    assert!(result.is_err());
  }

  #[test]
  fn deque_cache_push_front_and_len() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let tok = make_token(0, 5);
    let result = <DequeCache as Cache<'_, DummyLexer>>::push_front(&mut cache, tok);
    assert!(result.is_ok());
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
  }

  #[test]
  fn deque_cache_pop_front() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    assert!(<DequeCache as Cache<'_, DummyLexer>>::pop_front(&mut cache).is_none());
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
    let popped = <DequeCache as Cache<'_, DummyLexer>>::pop_front(&mut cache);
    assert!(popped.is_some());
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
  }

  #[test]
  fn deque_cache_pop_back() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    assert!(<DequeCache as Cache<'_, DummyLexer>>::pop_back(&mut cache).is_none());
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let popped = <DequeCache as Cache<'_, DummyLexer>>::pop_back(&mut cache);
    assert!(popped.is_some());
    assert!(<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
  }

  #[test]
  fn deque_cache_pop_front_if_match() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let popped = <DequeCache as Cache<'_, DummyLexer>>::pop_front_if(&mut cache, |_| true);
    assert!(popped.is_some());
    assert!(<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
  }

  #[test]
  fn deque_cache_pop_front_if_no_match() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let popped = <DequeCache as Cache<'_, DummyLexer>>::pop_front_if(&mut cache, |_| false);
    assert!(popped.is_none());
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::len(&cache), 1);
  }

  #[test]
  fn deque_cache_clear() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
    <DequeCache as Cache<'_, DummyLexer>>::clear(&mut cache);
    assert!(<DequeCache as Cache<'_, DummyLexer>>::is_empty(&cache));
  }

  #[test]
  fn deque_cache_front_and_back() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    assert!(<DequeCache as Cache<'_, DummyLexer>>::front(&cache).is_none());
    assert!(<DequeCache as Cache<'_, DummyLexer>>::back(&cache).is_none());
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
    let front = <DequeCache as Cache<'_, DummyLexer>>::front(&cache);
    let back = <DequeCache as Cache<'_, DummyLexer>>::back(&cache);
    assert!(front.is_some());
    assert!(back.is_some());
  }

  #[test]
  fn deque_cache_remaining() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 3);
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 2);
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 1);
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
    assert_eq!(<DequeCache as Cache<'_, DummyLexer>>::remaining(&cache), 0);
  }

  #[test]
  fn deque_cache_peek_empty() {
    use generic_arraydeque::typenum::U2;
    let cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let mut buf = GenericArrayDeque::new();
    <DequeCache as Cache<'_, DummyLexer>>::peek::<U2>(&cache, &mut buf);
    assert!(buf.is_empty());
  }

  #[test]
  fn deque_cache_peek_with_tokens() {
    use generic_arraydeque::typenum::U2;
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
    let mut buf = GenericArrayDeque::new();
    <DequeCache as Cache<'_, DummyLexer>>::peek::<U2>(&cache, &mut buf);
    assert_eq!(buf.len(), 2);
  }

  #[test]
  fn deque_cache_peek_capped_by_buffer() {
    use generic_arraydeque::typenum::U1;
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
    let mut buf = GenericArrayDeque::new();
    // Buffer capacity is 1, should only get 1 token even though cache has 3
    <DequeCache as Cache<'_, DummyLexer>>::peek::<U1>(&cache, &mut buf);
    assert_eq!(buf.len(), 1);
  }

  #[test]
  fn deque_cache_push_front_when_full_returns_err() {
    let mut cache = <DequeCache as Cache<'_, DummyLexer>>::new();
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(5, 10));
    let _ = <DequeCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(10, 15));
    let result = <DequeCache as Cache<'_, DummyLexer>>::push_front(&mut cache, make_token(15, 20));
    assert!(result.is_err());
  }
}
