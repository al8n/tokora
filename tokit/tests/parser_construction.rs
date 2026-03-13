#![cfg(all(feature = "std", feature = "logos"))]
mod common;

use tokit::{
  DefaultCache, Emitter, InputRef, Lexer, Parse, ParseContext, Parser, ParserContext,
  Token as TokenTrait,
  error::{UnexpectedEot, token::UnexpectedToken},
  input::Cursor,
  span::Spanned,
};

use common::{TestLexer, Token};

#[derive(Debug)]
struct E;

impl From<()> for E {
  fn from(_: ()) -> Self {
    E
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for E {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    E
  }
}

impl From<UnexpectedEot> for E {
  fn from(_: UnexpectedEot) -> Self {
    E
  }
}

struct TestEmitter;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for TestEmitter {
  type Error = E;

  fn emit_lexer_error(
    &mut self,
    _: Spanned<
      <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
      <TestLexer<'inp> as Lexer<'inp>>::Span,
    >,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }

  fn emit_unexpected_token(
    &mut self,
    _: tokit::error::token::UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(E)
  }

  fn emit_error(&mut self, err: Spanned<E, <TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), E>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err(err.into_data())
  }

  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>)
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
  }
}

fn parse_first_num<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
{
  match inp.next()? {
    Some(tok) => match tok.into_data() {
      Token::Num(n) => Ok(n),
      _ => Err(E),
    },
    None => Err(E),
  }
}

#[test]
fn parser_new_and_apply() {
  let r: Result<i64, _> = Parser::new().apply(parse_first_num).parse_str("42");
  assert_eq!(r.unwrap(), 42);
}

#[test]
fn parser_with_context() {
  let ctx: ParserContext<'_, TestLexer<'_>, TestEmitter, DefaultCache<'_, TestLexer<'_>>> =
    ParserContext::new(TestEmitter);
  let r: Result<i64, _> = Parser::with_context(ctx)
    .apply(parse_first_num)
    .parse_str("42");
  assert_eq!(r.unwrap(), 42);
}

#[test]
fn parser_with_parser() {
  let r: Result<i64, _> = Parser::with_parser(parse_first_num).parse_str("42");
  assert_eq!(r.unwrap(), 42);
}

#[test]
fn parser_with_parser_and_context() {
  let ctx: ParserContext<'_, TestLexer<'_>, TestEmitter, DefaultCache<'_, TestLexer<'_>>> =
    ParserContext::new(TestEmitter);
  let r: Result<i64, _> = Parser::with_parser_and_context(parse_first_num, ctx).parse_str("42");
  assert_eq!(r.unwrap(), 42);
}

#[test]
fn parser_deref() {
  let p = Parser::with_parser(parse_first_num);
  let _: &_ = &*p;
}

#[test]
fn parser_deref_mut() {
  let mut p = Parser::with_parser(parse_first_num);
  let _: &mut _ = &mut *p;
}
