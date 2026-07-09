#![cfg(all(feature = "std", feature = "logos"))]

//! Tests that the shipped emitters implement `Missing{Leading,Trailing}SeparatorEmitter`,
//! making the `require_*` separator families usable without a hand-rolled emitter.

mod common;

use tokit::{
  Accumulator, Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
  TryParseInput,
  emitter::{
    Fatal, FromSeparatedError, FromUnexpectedLeadingSeparatorError, FullContainerEmitter,
    MissingTrailingSeparatorEmitter, SeparatedEmitter, UnexpectedLeadingSeparatorEmitter, Verbose,
  },
  error::{
    UnexpectedEot,
    syntax::{FullContainer, MissingSyntaxOf},
    token::{MissingToken, MissingTokenOf, UnexpectedToken, UnexpectedTokenOf},
  },
  span::SimpleSpan,
  try_parse_input::ParseAttempt,
  utils::CowStr,
};

use common::{TestLexer, Token};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct ReqError;

impl From<()> for ReqError {
  fn from(_: ()) -> Self {
    ReqError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for ReqError {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    ReqError
  }
}

impl<'a, Kind: Clone, O, Lang: ?Sized> From<MissingToken<'a, Kind, O, Lang>> for ReqError {
  fn from(_: MissingToken<'a, Kind, O, Lang>) -> Self {
    ReqError
  }
}

impl From<UnexpectedEot> for ReqError {
  fn from(_: UnexpectedEot) -> Self {
    ReqError
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for ReqError {
  fn from(_: FullContainer<S, Lang>) -> Self {
    ReqError
  }
}

impl<'inp> FromSeparatedError<'inp, TestLexer<'inp>> for ReqError {
  fn from_missing_separator(_: CowStr, _: MissingTokenOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    ReqError
  }

  fn from_missing_element(_: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    ReqError
  }
}

impl<'inp> FromUnexpectedLeadingSeparatorError<'inp, TestLexer<'inp>> for ReqError {
  fn from_unexpected_leading_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    ReqError
  }
}

// ── Context constructors ──────────────────────────────────────────────────────

fn fatal_ctx() -> ParserContext<'static, TestLexer<'static>, Fatal<ReqError>> {
  ParserContext::new(Fatal::new())
}

fn verbose_ctx() -> ParserContext<'static, TestLexer<'static>, Verbose<ReqError>> {
  ParserContext::new(Verbose::new())
}

// ── Element parser ────────────────────────────────────────────────────────────

fn try_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, ReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ReqError>,
{
  inp
    .try_expect(|t| matches!(t.data(), Token::Num(_)))
    .map(|opt| match opt {
      None => ParseAttempt::Decline,
      Some(tok) => ParseAttempt::Accept(match tok.into_data() {
        Token::Num(n) => n,
        _ => unreachable!(),
      }),
    })
}

fn parse_require_trailing<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Vec<i64>, ReqError>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = ReqError>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>,
{
  try_num
    .separated_by_comma()
    .require_trailing()
    .collect()
    .parse_input(inp)
}

// ── Fatal: a missing required trailing separator is fatal ─────────────────────

#[test]
fn require_trailing_fatal_missing_is_err() {
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx())
    .apply(parse_require_trailing)
    .parse_str("1,2,3");
  assert!(r.is_err());
}

#[test]
fn require_trailing_fatal_present_is_ok() {
  let r: Vec<i64> = Parser::with_context(fatal_ctx())
    .apply(parse_require_trailing)
    .parse_str("1,2,3,")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── Verbose: a missing required trailing separator is recorded, not fatal ─────

#[test]
fn require_trailing_verbose_records_zero_width_span() {
  fn parse<'inp>(
    inp: &mut InputRef<
      'inp,
      '_,
      TestLexer<'inp>,
      ParserContext<'inp, TestLexer<'inp>, Verbose<ReqError>>,
    >,
  ) -> Result<Vec<i64>, ReqError> {
    let out = try_num
      .separated_by_comma()
      .require_trailing()
      .collect()
      .parse_input(inp)?;
    let errs = inp.emitter().errors();
    assert_eq!(errs.len(), 1, "one missing-trailing-separator error");
    // "1,2,3" ends at offset 5; the error is recorded at the zero-width span there.
    assert!(
      errs.contains_key(&SimpleSpan::new(5usize, 5usize)),
      "missing trailing separator recorded at the zero-width span at end of input"
    );
    Ok(out)
  }

  let r: Vec<i64> = Parser::with_context(verbose_ctx())
    .apply(parse)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}

// ── Verbose: require_surrounded records both leading and trailing at zero-width ─

#[test]
fn require_surrounded_verbose_records_both_zero_width_spans() {
  fn parse<'inp>(
    inp: &mut InputRef<
      'inp,
      '_,
      TestLexer<'inp>,
      ParserContext<'inp, TestLexer<'inp>, Verbose<ReqError>>,
    >,
  ) -> Result<Vec<i64>, ReqError> {
    let out = try_num
      .separated_by_comma()
      .require_trailing()
      .require_leading()
      .collect()
      .parse_input(inp)?;
    let errs = inp.emitter().errors();
    assert_eq!(errs.len(), 2, "missing leading and trailing separators");
    // Leading missing at the start (offset 0), trailing missing at the end (offset 5).
    assert!(errs.contains_key(&SimpleSpan::new(0usize, 0usize)));
    assert!(errs.contains_key(&SimpleSpan::new(5usize, 5usize)));
    Ok(out)
  }

  let r: Vec<i64> = Parser::with_context(verbose_ctx())
    .apply(parse)
    .parse_str("1,2,3")
    .unwrap();
  assert_eq!(r, vec![1, 2, 3]);
}
