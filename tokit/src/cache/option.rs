use generic_arraydeque::GenericArrayDeque;
use mayber::Maybe;

use crate::lexer::Lexer;

use super::{
  Cache, CachedToken, CachedTokenOf, CachedTokenRefOf, Checkpoint, MaybeRefCachedTokenOf, Span,
};

impl<'a, L, Lang: ?Sized> Cache<'a, L, Lang> for Option<CachedToken<L::Token, L::State, L::Span>>
where
  L: Lexer<'a>,
{
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn new() -> Self {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn with_options(_options: ()) -> Self {
    None
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn len(&self) -> usize {
    self.as_ref().map(|_| 1).unwrap_or(0)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn remaining(&self) -> usize {
    if self.is_none() { 1 } else { 0 }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, ckp: &Checkpoint<'a, '_, L>)
  where
    Self: Sized,
  {
    if self.is_none() {
      return;
    }

    let cursor = ckp.cursor();
    // if the rewind position is before the start of the cache, clear the cache
    if let Some(span) = self.as_ref().map(|tok| tok.token().span()) {
      let off = cursor.as_inner();
      if off < span.start_ref() {
        *self = None;
        return;
      }

      // If the rewind position is exactly at the start of the cache, do nothing
      if off == span.start_ref() {
        return;
      }

      if off == span.end_ref() {
        return;
      }
    }

    *self = None;
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_front(
    &mut self,
    tok: CachedTokenOf<'a, L>,
  ) -> Result<CachedTokenRefOf<'_, 'a, L>, CachedTokenOf<'a, L>> {
    match self {
      Some(_) => Err(tok),
      None => {
        *self = Some(tok);
        Ok(self.as_ref().expect("there must be a token").as_ref())
      }
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn push_back(
    &mut self,
    tok: CachedTokenOf<'a, L>,
  ) -> Result<CachedTokenRefOf<'_, 'a, L>, CachedTokenOf<'a, L>> {
    match self {
      Some(_) => Err(tok),
      None => {
        *self = Some(tok);
        Ok(self.as_ref().expect("there must be a token").as_ref())
      }
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn pop_front(&mut self) -> Option<CachedTokenOf<'a, L>> {
    self.take()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn pop_back(&mut self) -> Option<CachedTokenOf<'a, L>> {
    self.take()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clear(&mut self) {
    *self = None;
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn peek<'p, W>(
    &'p self,
    buf: &mut GenericArrayDeque<MaybeRefCachedTokenOf<'p, 'a, L>, W::CAPACITY>,
  ) where
    W: crate::Window,
  {
    if let Some(tok) = self.as_ref() {
      buf.push_back(Maybe::Ref(tok.as_ref()));
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn front(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
    self.as_ref().map(|tok| tok.as_ref())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn back(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
    self.as_ref().map(|tok| tok.as_ref())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{DummyLexer, DummyToken};
  use crate::span::{SimpleSpan, Spanned};

  type OptionCache = Option<CachedToken<DummyToken, (), SimpleSpan>>;

  fn make_token(start: usize, end: usize) -> CachedToken<DummyToken, (), SimpleSpan> {
    CachedToken::new(Spanned::new(SimpleSpan::new(start, end), DummyToken), ())
  }

  #[test]
  fn option_cache_new_is_none() {
    let cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    assert!(cache.is_none());
    assert_eq!(<OptionCache as Cache<'_, DummyLexer>>::len(&cache), 0);
    assert!(<OptionCache as Cache<'_, DummyLexer>>::is_empty(&cache));
    assert_eq!(<OptionCache as Cache<'_, DummyLexer>>::remaining(&cache), 1);
  }

  #[test]
  fn option_cache_with_options_is_none() {
    let cache = <OptionCache as Cache<'_, DummyLexer>>::with_options(());
    assert!(cache.is_none());
  }

  #[test]
  fn option_cache_push_back_and_len() {
    let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    let tok = make_token(0, 5);
    let result = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, tok);
    assert!(result.is_ok());
    assert_eq!(<OptionCache as Cache<'_, DummyLexer>>::len(&cache), 1);
    assert!(!<OptionCache as Cache<'_, DummyLexer>>::is_empty(&cache));
    assert_eq!(<OptionCache as Cache<'_, DummyLexer>>::remaining(&cache), 0);
  }

  #[test]
  fn option_cache_push_back_when_full_returns_err() {
    let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    let tok1 = make_token(0, 5);
    let tok2 = make_token(5, 10);
    let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, tok1);
    let result = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, tok2);
    assert!(result.is_err());
  }

  #[test]
  fn option_cache_push_front_and_len() {
    let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    let tok = make_token(0, 5);
    let result = <OptionCache as Cache<'_, DummyLexer>>::push_front(&mut cache, tok);
    assert!(result.is_ok());
    assert_eq!(<OptionCache as Cache<'_, DummyLexer>>::len(&cache), 1);
  }

  #[test]
  fn option_cache_push_front_when_full_returns_err() {
    let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    let tok1 = make_token(0, 5);
    let tok2 = make_token(5, 10);
    let _ = <OptionCache as Cache<'_, DummyLexer>>::push_front(&mut cache, tok1);
    let result = <OptionCache as Cache<'_, DummyLexer>>::push_front(&mut cache, tok2);
    assert!(result.is_err());
  }

  #[test]
  fn option_cache_pop_front() {
    let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    assert!(<OptionCache as Cache<'_, DummyLexer>>::pop_front(&mut cache).is_none());
    let tok = make_token(0, 5);
    let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, tok);
    let popped = <OptionCache as Cache<'_, DummyLexer>>::pop_front(&mut cache);
    assert!(popped.is_some());
    assert!(<OptionCache as Cache<'_, DummyLexer>>::is_empty(&cache));
  }

  #[test]
  fn option_cache_pop_back() {
    let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    assert!(<OptionCache as Cache<'_, DummyLexer>>::pop_back(&mut cache).is_none());
    let tok = make_token(0, 5);
    let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, tok);
    let popped = <OptionCache as Cache<'_, DummyLexer>>::pop_back(&mut cache);
    assert!(popped.is_some());
    assert!(<OptionCache as Cache<'_, DummyLexer>>::is_empty(&cache));
  }

  #[test]
  fn option_cache_clear() {
    let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    let tok = make_token(0, 5);
    let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, tok);
    <OptionCache as Cache<'_, DummyLexer>>::clear(&mut cache);
    assert!(<OptionCache as Cache<'_, DummyLexer>>::is_empty(&cache));
  }

  #[test]
  fn option_cache_front_and_back() {
    let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    assert!(<OptionCache as Cache<'_, DummyLexer>>::front(&cache).is_none());
    assert!(<OptionCache as Cache<'_, DummyLexer>>::back(&cache).is_none());
    let tok = make_token(0, 5);
    let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, tok);
    assert!(<OptionCache as Cache<'_, DummyLexer>>::front(&cache).is_some());
    assert!(<OptionCache as Cache<'_, DummyLexer>>::back(&cache).is_some());
  }

  #[test]
  fn option_cache_peek_empty() {
    use generic_arraydeque::typenum::U1;
    let cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    let mut buf = GenericArrayDeque::new();
    <OptionCache as Cache<'_, DummyLexer>>::peek::<U1>(&cache, &mut buf);
    assert!(buf.is_empty());
  }

  #[test]
  fn option_cache_peek_with_token() {
    use generic_arraydeque::typenum::U1;
    let mut cache = <OptionCache as Cache<'_, DummyLexer>>::new();
    let _ = <OptionCache as Cache<'_, DummyLexer>>::push_back(&mut cache, make_token(0, 5));
    let mut buf = GenericArrayDeque::new();
    <OptionCache as Cache<'_, DummyLexer>>::peek::<U1>(&cache, &mut buf);
    assert_eq!(buf.len(), 1);
  }
}
