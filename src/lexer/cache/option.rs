use generic_arraydeque::GenericArrayDeque;
use mayber::Maybe;

use crate::lexer::{CachedToken, CachedTokenRefOf, Lexed, MaybeRefCachedTokenOf};

use super::{Cache, CachedTokenOf, Checkpoint, Lexer, Span};

impl<'a, L> Cache<'a, L> for Option<CachedToken<Lexed<'a, L::Token>, L::State, L::Span>>
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
  fn first(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
    self.as_ref().map(|tok| tok.as_ref())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn last(&self) -> Option<CachedTokenRefOf<'_, 'a, L>> {
    self.as_ref().map(|tok| tok.as_ref())
  }
}
