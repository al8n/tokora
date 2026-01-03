use mayber::Maybe;

use crate::lexer::{Lexed, Lexer};

use super::{
  Cache, CachedToken, CachedTokenOf, CachedTokenRefOf, Checkpoint, MaybeRefCachedTokenOf, Span,
};

use generic_arraydeque::{ArrayLength, GenericArrayDeque};

impl<'a, L, N> Cache<'a, L>
  for GenericArrayDeque<CachedToken<Lexed<'a, L::Token>, L::State, L::Span>, N>
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
  fn first(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
    self.front().map(|tok| tok.as_ref())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn last(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
    self.back().map(|tok| tok.as_ref())
  }
}
