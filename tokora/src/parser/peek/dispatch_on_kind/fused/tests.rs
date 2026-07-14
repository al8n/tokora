use crate::{
  SimpleSpan,
  lexer::{DummyLexer, DummyToken},
  parser::{Parse, Parser},
  span::Spanned,
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

#[test]
fn assert_parse_impl() {
  let _ = assert_fused_dispatch_on_kind_parse_impl();
}
