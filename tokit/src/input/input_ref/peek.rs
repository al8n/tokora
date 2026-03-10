use super::*;

impl<'inp, L, Ctx, Lang: ?Sized> InputRef<'inp, '_, L, Ctx, Lang>
where
  L: Lexer<'inp>,
  L::State: Clone,
  Ctx: ParseContext<'inp, L, Lang>,
{
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

    #[cfg(debug_assertions)]
    if want == 0 {
      debug_assert!(
        exp == in_cache + yielded,
        "expected peeked token count mismatch"
      );
    }

    Ok(self.emitter)
  }
}

#[cfg(all(test, feature = "logos", feature = "std"))]
mod tests {
  use crate::{ParseContext, input::Input, lexer::LogosLexer};

  #[derive(Debug, Clone, PartialEq, crate::logos::Logos)]
  #[logos(crate = crate::logos, skip r"[ \t\r\n]+")]
  enum Tok {
    #[regex(r"[a-z]+")]
    Word,
    #[regex(r"[0-9]+")]
    Num,
  }

  impl core::fmt::Display for Tok {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        Tok::Word => write!(f, "word"),
        Tok::Num => write!(f, "num"),
      }
    }
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum TokKind {
    Word,
    Num,
  }

  impl core::fmt::Display for TokKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        TokKind::Word => write!(f, "word"),
        TokKind::Num => write!(f, "num"),
      }
    }
  }

  impl crate::Token<'_> for Tok {
    type Kind = TokKind;
    type Error = ();
    fn kind(&self) -> TokKind {
      match self {
        Tok::Word => TokKind::Word,
        Tok::Num => TokKind::Num,
      }
    }
    fn is_trivia(&self) -> bool {
      false
    }
  }

  type TestLexer<'a> = LogosLexer<'a, Tok>;

  fn parse_with<'inp, F, O>(src: &'inp str, mut f: F) -> Result<O, ()>
  where
    F: for<'c> FnMut(
      &mut crate::input::InputRef<'inp, 'c, TestLexer<'inp>, (), ()>,
    ) -> Result<O, ()>,
  {
    let (mut emitter, cache) =
      <() as ParseContext<'_, TestLexer<'_>>>::provide(()).into_components();
    let mut input = Input::<TestLexer<'inp>, (), ()>::with_state_and_cache(src, (), cache);
    let mut inp_ref = input.as_ref(&mut emitter);
    f(&mut inp_ref)
  }

  #[test]
  fn peek_one_returns_token() {
    parse_with("abc 123", |inp| {
      let peeked = inp.peek_one()?;
      assert!(peeked.is_some());
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn peek_one_empty_input() {
    parse_with("", |inp| {
      let peeked = inp.peek_one()?;
      assert!(peeked.is_none());
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn peek_window() {
    parse_with("abc 123 def", |inp| {
      use generic_arraydeque::typenum::U2;
      let peeked = inp.peek::<U2>()?;
      assert_eq!(peeked.len(), 2);
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn peek_with_emitter_test() {
    parse_with("abc 123", |inp| {
      use generic_arraydeque::typenum::U2;
      let (peeked, _emitter) = inp.peek_with_emitter::<U2>()?;
      assert_eq!(peeked.len(), 2);
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn peek_window_larger_than_input() {
    parse_with("abc", |inp| {
      use generic_arraydeque::typenum::U3;
      let peeked = inp.peek::<U3>()?;
      assert_eq!(peeked.len(), 1);
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn peek_does_not_consume() {
    parse_with("abc 123", |inp| {
      use generic_arraydeque::typenum::U1;
      {
        let peeked = inp.peek::<U1>()?;
        assert_eq!(peeked.len(), 1);
      }
      {
        let peeked = inp.peek::<U1>()?;
        assert_eq!(peeked.len(), 1);
      }
      Ok(())
    })
    .unwrap();
  }
}
