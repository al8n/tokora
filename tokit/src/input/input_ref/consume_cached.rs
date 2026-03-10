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
  fn consume_cached_one_after_peek() {
    parse_with("abc 123", |inp| {
      use generic_arraydeque::typenum::U2;
      let peeked = inp.peek::<U2>()?;
      drop(peeked);
      let tok = inp.consume_cached_one();
      assert!(tok.is_some());
      let tok = tok.unwrap();
      assert_eq!(tok.data, Tok::Word);
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn consume_cached_one_empty_cache() {
    parse_with("abc", |inp| {
      let tok = inp.consume_cached_one();
      assert!(tok.is_none());
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn consume_cached_to_predicate() {
    parse_with("abc 123 def", |inp| {
      use generic_arraydeque::typenum::U3;
      let peeked = inp.peek::<U3>()?;
      drop(peeked);
      let last = inp.consume_cached_to(|t| matches!(t.token().data(), Tok::Num));
      assert!(last.is_some());
      let last = last.unwrap();
      assert_eq!(last.data, Tok::Word);
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn consume_cached_while_predicate() {
    parse_with("abc 123 def", |inp| {
      use generic_arraydeque::typenum::U3;
      let peeked = inp.peek::<U3>()?;
      drop(peeked);
      let last = inp.consume_cached_while(|t| matches!(t.token().data(), Tok::Word));
      assert!(last.is_some());
      let last = last.unwrap();
      assert_eq!(last.data, Tok::Word);
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn consume_all_cached() {
    parse_with("abc 123 def", |inp| {
      use generic_arraydeque::typenum::U3;
      let peeked = inp.peek::<U3>()?;
      drop(peeked);
      let last = inp.consume_all_cached();
      assert!(last.is_some());
      let last = last.unwrap();
      assert_eq!(last.data, Tok::Word);
      Ok(())
    })
    .unwrap();
  }

  #[test]
  fn consume_all_cached_empty() {
    parse_with("abc", |inp| {
      let last = inp.consume_all_cached();
      assert!(last.is_none());
      Ok(())
    })
    .unwrap();
  }
}
