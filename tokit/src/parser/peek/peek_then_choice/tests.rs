use generic_arraydeque::typenum::U2;

use crate::{
  Branch,
  lexer::{DummyLexer, DummyToken},
};

use super::*;

fn assert_peek_then_choice_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
  Parser::new().apply((Any::new(), Any::new()).peek_then_choice::<_, U2>(|_toks, _| Ok(Branch::B1)))
}

#[test]
fn assert_parse_impl() {
  let _ = assert_peek_then_choice_parse_impl();
}
