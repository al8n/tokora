use crate::{
  SimpleSpan, TryParseInput,
  lexer::{DummyLexer, DummyToken},
  parser::{Parse, Parser},
  span::Spanned,
  try_parse_input::ParseAttempt,
};

use super::*;

// A plain `fn` arm: proves fn items (not just closures) satisfy the fused arm bound.
fn head_arm<'inp, Ctx>(
  head: Spanned<DummyToken, SimpleSpan>,
  _inp: &mut InputRef<'inp, '_, DummyLexer, Ctx>,
) -> Result<DummyToken, ()>
where
  Ctx: ParseContext<'inp, DummyLexer, Emitter: Emitter<'inp, DummyLexer, Error = ()>>,
{
  Ok(head.data)
}

fn assert_fused_dispatch_on_kind_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()>
{
  Parser::new().apply((head_arm,).fused_dispatch_on_kind(&[DummyToken]))
}

fn try_fused_dispatch_on_kind<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, DummyLexer, Ctx>,
) -> Result<ParseAttempt<DummyToken>, ()>
where
  Ctx: ParseContext<'inp, DummyLexer, Emitter: Emitter<'inp, DummyLexer, Error = ()>>,
{
  (head_arm,)
    .fused_dispatch_on_kind(&[DummyToken])
    .try_parse_input(inp)
}

#[test]
fn assert_parse_impl() {
  let _ = assert_fused_dispatch_on_kind_parse_impl();
}

#[test]
fn assert_try_parse_input_impl() {
  // This helper calls the trait method with a generic context, so it proves the fused
  // combinator is available wherever a normal parser input is available.
  let _ = Parser::new().apply(try_fused_dispatch_on_kind);
}
