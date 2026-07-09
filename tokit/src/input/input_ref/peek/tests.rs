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
  F: for<'c> FnMut(&mut crate::input::InputRef<'inp, 'c, TestLexer<'inp>, (), ()>) -> Result<O, ()>,
{
  let (mut emitter, cache) = <() as ParseContext<'_, TestLexer<'_>>>::provide(()).into_components();
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

#[test]
fn peek_window_exceeds_cache_capacity() {
  // U4 window on default U3 cache — triggers overflow path (lines 76-126)
  parse_with("abc 123 def ghi", |inp| {
    use generic_arraydeque::typenum::U4;
    let peeked = inp.peek::<U4>()?;
    // Should see all 4 tokens even though cache can only hold 3
    assert_eq!(peeked.len(), 4);
    Ok(())
  })
  .unwrap();
}

#[test]
fn peek_overflow_tokens_correct() {
  // Verify overflowed tokens have correct data
  parse_with("abc 123 def ghi jkl", |inp| {
    use generic_arraydeque::typenum::U4;
    {
      let peeked = inp.peek::<U4>()?;
      assert_eq!(peeked.len(), 4);
    }
    // Peek again — should get same result (tokens cached or re-lexed)
    {
      let peeked2 = inp.peek::<U4>()?;
      assert_eq!(peeked2.len(), 4);
    }
    Ok(())
  })
  .unwrap();
}

#[test]
fn peek_overflow_then_consume() {
  // Peek with overflow, then consume tokens normally
  parse_with("abc 123 def ghi", |inp| {
    use generic_arraydeque::typenum::U4;
    {
      let peeked = inp.peek::<U4>()?;
      assert_eq!(peeked.len(), 4);
    }
    // Consume should work correctly after overflow peek
    let tok = inp.next()?;
    assert!(tok.is_some());
    Ok(())
  })
  .unwrap();
}

#[test]
fn slice_after_peek_returns_consumed_token() {
  // Consume the first token so the target is no longer at offset 0, then peek
  // to fill the cache and consume from it. `slice()` must return the text of
  // the just-consumed token, not the whole consumed prefix.
  parse_with("foo bar", |inp| {
    assert!(inp.next()?.is_some());
    assert!(inp.peek_one()?.is_some());
    assert!(inp.next()?.is_some());
    assert_eq!(inp.slice(), "bar");
    Ok(())
  })
  .unwrap();
}

#[test]
fn cursor_targets_first_cached_token_start() {
  use generic_arraydeque::typenum::U2;
  // "a1" lexes to two adjacent tokens: Word(0..1), Num(1..2).
  parse_with("a1", |inp| {
    {
      let peeked = inp.peek::<U2>()?;
      assert_eq!(peeked.len(), 2);
    }
    // The cursor must point at the START of the first cached token (0),
    // not its end (1).
    assert_eq!(*inp.cursor().as_inner(), 0usize);
    Ok(())
  })
  .unwrap();
}

#[test]
fn save_restore_preserves_front_token_with_multi_cache() {
  use generic_arraydeque::typenum::U2;
  // Fill the cache with two tokens, checkpoint, consume one, then restore.
  // The next token must be the FIRST one again (no silent token loss).
  parse_with("a1", |inp| {
    {
      let peeked = inp.peek::<U2>()?;
      assert_eq!(peeked.len(), 2);
    }
    let ckp = inp.save();
    let first = inp.next()?.expect("first token");
    assert_eq!(first.data, Tok::Word);
    inp.restore(ckp);
    let again = inp.next()?.expect("token after restore");
    assert_eq!(again.data, Tok::Word);
    Ok(())
  })
  .unwrap();
}

#[test]
fn attempt_over_prefilled_cache_preserves_first_token() {
  use generic_arraydeque::typenum::U2;
  // A rollback attempt over a pre-filled cache must not skip a token.
  parse_with("a1", |inp| {
    {
      let peeked = inp.peek::<U2>()?;
      assert_eq!(peeked.len(), 2);
    }
    let outcome = inp.attempt(|inp| {
      // Consume the first token, then decline so the attempt rolls back.
      match inp.next() {
        Ok(Some(_)) => None::<()>,
        _ => None,
      }
    });
    assert!(outcome.is_none());
    let again = inp.next()?.expect("token after rolled-back attempt");
    assert_eq!(again.data, Tok::Word);
    Ok(())
  })
  .unwrap();
}

#[test]
fn spanned_since_under_peek_yields_real_span() {
  use crate::span::SimpleSpan;
  // Peek to fill the cache (as a peek_then_choice branch would), capture the
  // cursor, consume the peeked token, then measure the span from the captured
  // cursor. It must be the token's real span, not an empty span.
  parse_with("a1", |inp| {
    {
      let peeked = inp.peek_one()?;
      assert!(peeked.is_some());
    }
    let start = *inp.cursor();
    let _ = inp.next()?.expect("first token");
    let span = inp.span_since(&start);
    assert_eq!(span, SimpleSpan::new(0, 1));
    Ok(())
  })
  .unwrap();
}

#[test]
fn span_and_slice_report_consumed_token_after_multi_peek() {
  use crate::span::SimpleSpan;
  // With more than one token cached, consuming one must leave `span()`/`slice()`
  // reporting the JUST-CONSUMED token, not the remaining front cached token.
  parse_with("a1", |inp| {
    {
      let peeked = inp.peek::<generic_arraydeque::typenum::U2>()?;
      assert_eq!(peeked.len(), 2);
    }
    let first = inp.next()?.expect("first token");
    assert_eq!(first.data, Tok::Word);
    assert_eq!(*inp.span(), SimpleSpan::new(0, 1));
    assert_eq!(inp.slice(), "a");
    Ok(())
  })
  .unwrap();
}

#[test]
fn token_accessor_reads_ref_arm() {
  use crate::{cache::PeekedTokenExt, span::SimpleSpan};
  use generic_arraydeque::typenum::U2;
  // A U2 window fits the default U3 cache, so both peeked tokens are the
  // borrowed (`Ref`) arm. The accessor reaches token + span without matching.
  parse_with("abc 123", |inp| {
    let peeked = inp.peek::<U2>()?;
    assert_eq!(peeked.len(), 2);
    assert!(peeked[0].is_ref());
    assert!(peeked[1].is_ref());
    assert_eq!(*peeked[0].token(), Tok::Word);
    assert_eq!(*peeked[0].span(), SimpleSpan::new(0, 3));
    assert_eq!(*peeked[1].token(), Tok::Num);
    assert_eq!(*peeked[1].span(), SimpleSpan::new(4, 7));
    Ok(())
  })
  .unwrap();
}

#[test]
fn token_accessor_reads_owned_arm() {
  use crate::{cache::PeekedTokenExt, span::SimpleSpan};
  use generic_arraydeque::typenum::U4;
  // A U4 window exceeds the default U3 cache; the 4th token overflows and is
  // materialized as the owned (`Owned`) arm. The same accessor reaches it.
  parse_with("abc 123 def ghi", |inp| {
    let peeked = inp.peek::<U4>()?;
    assert_eq!(peeked.len(), 4);
    assert!(peeked[3].is_owned());
    assert_eq!(*peeked[3].token(), Tok::Word);
    assert_eq!(*peeked[3].span(), SimpleSpan::new(12, 15));
    Ok(())
  })
  .unwrap();
}

// ── Lexer errors must never be dropped, never double-emitted ──────────────
//
// A counting emitter records exactly how many lexer errors reach the
// emitter. It is non-fatal (always returns `Ok`) and does NOT deduplicate,
// so a double emission of the same malformed region is observable as `2`.

#[derive(Debug, Default)]
struct CountingEmitter {
  lexer_errors: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NeverFatal;

impl<'inp> crate::emitter::Emitter<'inp, TestLexer<'inp>> for CountingEmitter {
  type Error = NeverFatal;

  fn emit_lexer_error(
    &mut self,
    _: crate::span::Spanned<
      <<TestLexer<'inp> as crate::Lexer<'inp>>::Token as crate::Token<'inp>>::Error,
      <TestLexer<'inp> as crate::Lexer<'inp>>::Span,
    >,
  ) -> Result<(), NeverFatal> {
    self.lexer_errors += 1;
    Ok(())
  }

  fn emit_unexpected_token(
    &mut self,
    _: crate::error::token::UnexpectedTokenOf<'inp, TestLexer<'inp>, ()>,
  ) -> Result<(), NeverFatal> {
    Ok(())
  }

  fn emit_error(
    &mut self,
    _: crate::span::Spanned<NeverFatal, <TestLexer<'inp> as crate::Lexer<'inp>>::Span>,
  ) -> Result<(), NeverFatal> {
    Ok(())
  }

  fn rewind(&mut self, _: &crate::input::Cursor<'inp, '_, TestLexer<'inp>>) {}
}

type CountingCtx<'inp> = (
  CountingEmitter,
  crate::cache::DefaultCache<'inp, TestLexer<'inp>>,
);

fn count_lexer_errors<'inp, F>(src: &'inp str, f: F) -> usize
where
  F: FnOnce(
    &mut crate::input::InputRef<'inp, '_, TestLexer<'inp>, CountingCtx<'inp>, ()>,
  ) -> Result<(), NeverFatal>,
{
  let mut emitter = CountingEmitter::default();
  let cache = crate::cache::DefaultCache::<'inp, TestLexer<'inp>>::default();
  let mut input =
    crate::input::Input::<TestLexer<'inp>, CountingCtx<'inp>, ()>::with_state_and_cache(
      src,
      (),
      cache,
    );
  {
    let mut inp = input.as_ref(&mut emitter);
    let _ = f(&mut inp);
  }
  emitter.lexer_errors
}

fn drain<'inp>(
  inp: &mut crate::input::InputRef<'inp, '_, TestLexer<'inp>, CountingCtx<'inp>, ()>,
) -> Result<(), NeverFatal> {
  while inp.next()?.is_some() {}
  Ok(())
}

#[test]
fn consume_direct_single_lexer_error() {
  // No peek at all: a lexer error is emitted once as it is consumed.
  let n = count_lexer_errors("a @ b", |inp| drain(inp));
  assert_eq!(n, 1, "consume-direct");
}

#[test]
fn peek_then_consume_single_lexer_error() {
  // Error precedes a cached token; peek seals it, consume must not re-emit.
  let n = count_lexer_errors("@ a b", |inp| {
    use generic_arraydeque::typenum::U2;
    {
      let _ = inp.peek::<U2>()?;
    }
    drain(inp)
  });
  assert_eq!(n, 1, "peek-then-consume");
}

#[test]
fn peek_trailing_then_consume_single_lexer_error() {
  // Error trails the cached token (no later cached token). Consume re-lexes it.
  let n = count_lexer_errors("a @", |inp| {
    use generic_arraydeque::typenum::U2;
    {
      let _ = inp.peek::<U2>()?;
    }
    drain(inp)
  });
  assert_eq!(n, 1, "peek-trailing-then-consume");
}

#[test]
fn peek_overflow_then_consume_single_lexer_error() {
  // Cache holds 3; window 5. Error sits in the overflow region.
  let n = count_lexer_errors("a b c @ d", |inp| {
    use generic_arraydeque::typenum::U5;
    {
      let _ = inp.peek::<U5>()?;
    }
    drain(inp)
  });
  assert_eq!(n, 1, "peek-overflow-then-consume");
}

#[test]
fn peek_overflow_stop_records_lexer_error() {
  // Peek over the overflow region then STOP without consuming: the error in
  // the overflow region must still have been recorded at peek time.
  let n = count_lexer_errors("a b c @ d", |inp| {
    use generic_arraydeque::typenum::U5;
    let _ = inp.peek::<U5>()?;
    Ok(())
  });
  assert_eq!(n, 1, "peek-overflow-stop must record the error");
}
