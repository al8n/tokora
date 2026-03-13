#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use tokit::{
  Emitter, InputRef, Lexer, Parse, ParseContext, Parser, ParserContext,
  Token as TokenTrait,
  error::{UnexpectedEot, token::{UnexpectedToken, UnexpectedTokenOf}},
  input::Cursor,
  span::Spanned,
};

use common::{TestLexer, Token};

#[derive(Debug)]
struct E;
impl From<()> for E { fn from(_: ()) -> Self { E } }
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self { E }
}
impl From<UnexpectedEot> for E { fn from(_: UnexpectedEot) -> Self { E } }

struct TestEm;
impl<'inp> Emitter<'inp, TestLexer<'inp>> for TestEm {
  type Error = E;
  fn emit_lexer_error(&mut self, _: Spanned<<<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(E) }
  fn emit_unexpected_token(&mut self, _: UnexpectedTokenOf<'inp, TestLexer<'inp>>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(E) }
  fn emit_error(&mut self, err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E> where TestLexer<'inp>: Lexer<'inp> { Err(err.into_data()) }
  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>) where TestLexer<'inp>: Lexer<'inp> {}
}

fn ctx() -> ParserContext<'static, TestLexer<'static>, TestEm> {
  ParserContext::new(TestEm)
}

#[test]
fn rewind_to_start_after_consuming() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<(i64, i64), E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    let ckp = inp.save();
    let first = match inp.next()? {
      Some(tok) => match tok.into_data() { Token::Num(n) => n, _ => return Err(E) },
      None => return Err(E),
    };
    inp.restore(ckp);
    let again = match inp.next()? {
      Some(tok) => match tok.into_data() { Token::Num(n) => n, _ => return Err(E) },
      None => return Err(E),
    };
    Ok((first, again))
  }
  let r: Result<(i64, i64), _> = Parser::with_context(ctx())
    .apply(parse)
    .parse_str("42");
  let (a, b) = r.unwrap();
  assert_eq!(a, b);
  assert_eq!(a, 42);
}

#[test]
fn rewind_after_peek_populates_cache() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<i64, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    // Save checkpoint before any peeking, then peek to populate cache,
    // then restore to verify the cache rewind works correctly.
    let ckp = inp.save();
    let _ = inp.peek_one()?;
    inp.restore(ckp);
    match inp.next()? {
      Some(tok) => match tok.into_data() { Token::Num(n) => Ok(n), _ => Err(E) },
      None => Err(E),
    }
  }
  let r: Result<i64, _> = Parser::with_context(ctx())
    .apply(parse)
    .parse_str("42 99");
  assert_eq!(r.unwrap(), 42);
}

#[test]
fn rewind_mid_stream() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Vec<i64>, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    let _ = inp.next()?;
    let ckp = inp.save();
    let _ = inp.next()?;
    let _ = inp.next()?;
    inp.restore(ckp);
    let mut results = Vec::new();
    while let Some(tok) = inp.next()? {
      if let Token::Num(n) = tok.into_data() {
        results.push(n);
      }
    }
    Ok(results)
  }
  let r: Result<Vec<i64>, _> = Parser::with_context(ctx())
    .apply(parse)
    .parse_str("1 2 3");
  let nums = r.unwrap();
  assert_eq!(nums, vec![2, 3]);
}

#[test]
fn rewind_with_empty_remaining_input() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<i64, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    let ckp = inp.save();
    while inp.next()?.is_some() {}
    inp.restore(ckp);
    match inp.next()? {
      Some(tok) => match tok.into_data() { Token::Num(n) => Ok(n), _ => Err(E) },
      None => Err(E),
    }
  }
  let r: Result<i64, _> = Parser::with_context(ctx())
    .apply(parse)
    .parse_str("42");
  assert_eq!(r.unwrap(), 42);
}
