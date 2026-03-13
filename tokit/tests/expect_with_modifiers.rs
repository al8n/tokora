#![cfg(all(feature = "std", feature = "logos"))]

//! Tests for `ParseInput` impls on `With<Expect, PhantomSpan/PhantomSliced/PhantomLocated>`.
//! These cover lines 142-216 in parser/expect.rs and parse_state.rs accessors.

mod common;

use common::{TestLexer, Token, TokenKind};
use tokit::{
  Emitter, InputRef, Located, Parse, ParseContext, ParseInput, SimpleSpan,
  parser::{Parser, With, expect},
  slice::Sliced,
  span::Spanned,
  utils::{
    Expected,
    marker::{PhantomLocated, PhantomSliced, PhantomSpan},
  },
};

type E = ();

// ── With<Expect, PhantomSpan> — ParseInput ──────────────────────────────────

#[test]
fn expect_parse_input_with_spanned_ok() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Spanned<Token, SimpleSpan>, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    let expect_parser = expect::<'inp, _, TestLexer<'inp>, Ctx>(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    });
    let mut with_spanned: With<_, PhantomSpan> = With::new(expect_parser, PhantomSpan::phantom());
    with_spanned.parse_input(inp)
  }
  let r = Parser::new().apply(parse).parse_str("42");
  let spanned = r.unwrap();
  assert!(matches!(spanned.data(), Token::Num(42)));
  assert_eq!(spanned.span(), SimpleSpan::new(0, 2));
}

#[test]
fn expect_parse_input_with_spanned_err() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Spanned<Token, SimpleSpan>, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    let expect_parser = expect::<'inp, _, TestLexer<'inp>, Ctx>(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    });
    let mut with_spanned: With<_, PhantomSpan> = With::new(expect_parser, PhantomSpan::phantom());
    with_spanned.parse_input(inp)
  }
  let r = Parser::new().apply(parse).parse_str("+");
  assert!(r.is_err());
}

// ── With<Expect, PhantomSliced> — ParseInput ────────────────────────────────

#[test]
fn expect_parse_input_with_sliced_ok() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Sliced<Token, &'inp str>, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    let expect_parser = expect::<'inp, _, TestLexer<'inp>, Ctx>(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    });
    let mut with_sliced: With<_, PhantomSliced> =
      With::new(expect_parser, PhantomSliced::phantom());
    with_sliced.parse_input(inp)
  }
  let r = Parser::new().apply(parse).parse_str("42");
  let sliced = r.unwrap();
  assert!(matches!(sliced.data(), Token::Num(42)));
}

#[test]
fn expect_parse_input_with_sliced_err() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Sliced<Token, &'inp str>, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    let expect_parser = expect::<'inp, _, TestLexer<'inp>, Ctx>(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    });
    let mut with_sliced: With<_, PhantomSliced> =
      With::new(expect_parser, PhantomSliced::phantom());
    with_sliced.parse_input(inp)
  }
  let r = Parser::new().apply(parse).parse_str("+");
  assert!(r.is_err());
}

// ── With<Expect, PhantomLocated> — ParseInput ───────────────────────────────

#[test]
fn expect_parse_input_with_located_ok() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Located<Token, SimpleSpan, &'inp str>, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    let expect_parser = expect::<'inp, _, TestLexer<'inp>, Ctx>(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    });
    let mut with_located: With<_, PhantomLocated> =
      With::new(expect_parser, PhantomLocated::phantom());
    with_located.parse_input(inp)
  }
  let r = Parser::new().apply(parse).parse_str("42");
  let located = r.unwrap();
  assert!(matches!(located.data(), Token::Num(42)));
  assert_eq!(located.span(), SimpleSpan::new(0, 2));
}

#[test]
fn expect_parse_input_with_located_err() {
  fn parse<'inp, Ctx>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
  ) -> Result<Located<Token, SimpleSpan, &'inp str>, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    let expect_parser = expect::<'inp, _, TestLexer<'inp>, Ctx>(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    });
    let mut with_located: With<_, PhantomLocated> =
      With::new(expect_parser, PhantomLocated::phantom());
    with_located.parse_input(inp)
  }
  let r = Parser::new().apply(parse).parse_str("+");
  assert!(r.is_err());
}

// ── ParseState accessor coverage ────────────────────────────────────────────
// parse_state.rs: span(), emitter(), state(), state_mut(), slice()

#[test]
fn parse_state_accessors_via_map() {
  use tokit::ParseState;
  fn parse<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Token, E>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
  {
    expect::<'inp, _, TestLexer<'inp>, Ctx>(|t: &Token| {
      if matches!(t, Token::Num(_)) {
        Ok(())
      } else {
        Err(Expected::one(TokenKind::Num))
      }
    })
    .map_with(
      |tok: Token, mut ps: ParseState<'_, 'inp, '_, TestLexer<'inp>, Ctx>| {
        // Exercise all parse_state accessors
        let _span = ps.span();
        let _emitter = ps.emitter();
        let _state = ps.state();
        let _state_mut = ps.state_mut();
        let _slice = ps.slice();
        tok
      },
    )
    .parse_input(inp)
  }
  let r = Parser::new().apply(parse).parse_str("42");
  assert!(matches!(r.unwrap(), Token::Num(42)));
}
