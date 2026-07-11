#![cfg(all(feature = "std", feature = "logos"))]

//! Behavioral tests for `ParseChoice::dispatch_on_kind` — the kind-keyed dispatch
//! combinator whose committed failure carries the full static expected set.

mod common;

use common::{TestLexer, Token, TokenKind};
use tokit::{
  InputRef, Parse, ParseChoice, ParseContext, ParseInput, Parser, ParserContext,
  Token as TokenTrait,
  emitter::Verbose,
  error::{UnexpectedEot, token::UnexpectedToken},
  parser::Any,
  utils::Expected,
};

// ── Error type that *preserves* the expected set for value assertions ──────────

#[derive(Debug, Clone, PartialEq)]
enum DispatchError {
  Unexpected {
    expected: Vec<TokenKind>,
    found: Option<TokenKind>,
  },
  Eot,
  Lexer,
}

impl From<()> for DispatchError {
  fn from(_: ()) -> Self {
    DispatchError::Lexer
  }
}

impl<'a, S, Lang: ?Sized> From<UnexpectedToken<'a, Token, TokenKind, S, Lang>> for DispatchError {
  fn from(err: UnexpectedToken<'a, Token, TokenKind, S, Lang>) -> Self {
    let (_span, found, expected) = err.into_components();
    let expected = match expected {
      Some(Expected::One(kind)) => vec![kind],
      Some(Expected::OneOf(one_of)) => one_of.as_slice().to_vec(),
      _ => Vec::new(),
    };
    DispatchError::Unexpected {
      expected,
      found: found.as_ref().map(|token| token.kind()),
    }
  }
}

impl<O, Lang: ?Sized> From<UnexpectedEot<O, Lang>> for DispatchError {
  fn from(_: UnexpectedEot<O, Lang>) -> Self {
    DispatchError::Eot
  }
}

// ── The parsers under test ─────────────────────────────────────────────────────

/// A three-alternative dispatch: number → B0, `+` → B1, `(` → B2. Each branch
/// consumes its token and reports which branch ran (its kind).
fn dispatch3<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<TokenKind, DispatchError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: tokit::Emitter<'inp, TestLexer<'inp>, Error = DispatchError>,
{
  (
    Any::new().map(|_| TokenKind::Num),
    Any::new().map(|_| TokenKind::Plus),
    Any::new().map(|_| TokenKind::LParen),
  )
    .dispatch_on_kind(&[TokenKind::Num, TokenKind::Plus, TokenKind::LParen])
    .parse_input(inp)
}

/// A single-alternative dispatch: number → B0.
fn dispatch1<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<TokenKind, DispatchError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: tokit::Emitter<'inp, TestLexer<'inp>, Error = DispatchError>,
{
  (Any::new().map(|_| TokenKind::Num),)
    .dispatch_on_kind(&[TokenKind::Num])
    .parse_input(inp)
}

fn verbose_ctx() -> ParserContext<'static, TestLexer<'static>, Verbose<DispatchError>> {
  ParserContext::new(Verbose::new())
}

// ── Tests ───────────────────────────────────────────────────────────────────────

#[test]
fn dispatch_failure_reports_full_expected_set_in_table_order() {
  // `;` (Semi) is not a viable first token; the failure reports the whole table,
  // in table order, alongside the found token.
  let r = Parser::new().apply(dispatch3).parse_str(";");
  assert_eq!(
    r,
    Err(DispatchError::Unexpected {
      expected: vec![TokenKind::Num, TokenKind::Plus, TokenKind::LParen],
      found: Some(TokenKind::Semi),
    })
  );
}

#[test]
fn dispatch_eof_reports_eot() {
  // End-of-input at the dispatch point: `UnexpectedEot`, which carries no set.
  let r = Parser::new().apply(dispatch3).parse_str("");
  assert_eq!(r, Err(DispatchError::Eot));
}

#[test]
fn single_alternative_dispatch_reports_single_expectation() {
  // Miss over a one-entry table still reports that one expectation.
  let r = Parser::new().apply(dispatch1).parse_str(";");
  assert_eq!(
    r,
    Err(DispatchError::Unexpected {
      expected: vec![TokenKind::Num],
      found: Some(TokenKind::Semi),
    })
  );
  // ...and dispatches successfully on a hit.
  assert_eq!(
    Parser::new().apply(dispatch1).parse_str("7"),
    Ok(TokenKind::Num)
  );
}

#[test]
fn fatal_and_verbose_observe_the_same_payload() {
  let expected = Err(DispatchError::Unexpected {
    expected: vec![TokenKind::Num, TokenKind::Plus, TokenKind::LParen],
    found: Some(TokenKind::Semi),
  });

  // Fatal (fail-fast) and Verbose (error-collecting) surface the identical payload.
  let fatal = Parser::new().apply(dispatch3).parse_str(";");
  let verbose = Parser::with_context(verbose_ctx())
    .apply(dispatch3)
    .parse_str(";");

  assert_eq!(fatal, expected);
  assert_eq!(verbose, expected);
  assert_eq!(fatal, verbose);
}

#[test]
fn successful_dispatch_runs_the_selected_branch_unchanged() {
  // Scoping guard: on a hit there is no diagnostic and the correct branch runs,
  // identically under a fail-fast and an error-collecting emitter.
  for (src, want) in [
    ("42", TokenKind::Num),
    ("+", TokenKind::Plus),
    ("(", TokenKind::LParen),
  ] {
    assert_eq!(Parser::new().apply(dispatch3).parse_str(src), Ok(want));
    assert_eq!(
      Parser::with_context(verbose_ctx())
        .apply(dispatch3)
        .parse_str(src),
      Ok(want)
    );
  }
}
