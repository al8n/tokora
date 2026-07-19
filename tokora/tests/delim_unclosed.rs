#![cfg(all(feature = "std", feature = "logos"))]
#![allow(clippy::type_complexity)]
//! Regression suite for the unterminated-delimited-list fix.
//!
//! An unterminated delimited many-builder (`item…delimited::<D>().collect()`) used to accept
//! the input silently (returning `Ok` with the elements parsed so far). It now reports the
//! opener as [`Unclosed`] **through the emitter**:
//!
//! - a fail-fast [`Fatal`] emitter converts the emission to `Err` (carrying the opener's span
//!   and the delimiter pair's name);
//! - a recovering [`Verbose`] emitter records the diagnostic and the parse recovers, returning
//!   the elements collected so far.
//!
//! Both delimiter close-miss shapes are covered: (a) end-of-input with the opener still open
//! ⇒ `Unclosed`; (b) a wrong token where the closer belongs ⇒ the existing unexpected-token
//! (expected-close) vocabulary, **not** `Unclosed`.

mod common;

use common::{TestLexer, Token};
use generic_arraydeque::typenum::U1;
use tokora::{
  Accumulator, Emitter, InputRef, Parse, ParseContext, ParseInput, Parser, ParserContext,
  SimpleSpan, TryParseInput,
  cache::Peeked,
  emitter::{
    Fatal, FullContainerEmitter, SeparatedEmitter, TooManyEmitter, UnclosedEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter, Verbose,
  },
  error::{
    Unclosed, UnexpectedEot,
    syntax::{FullContainer, MissingSyntax, TooFew, TooMany},
    token::{MissingToken, SeparatedError, UnexpectedToken},
  },
  parser::Action,
  punct::{Brace, Bracket, Paren},
  try_parse_input::ParseAttempt,
};

// ── A rich error type that preserves the `Unclosed` payload ────────────────────
//
// Unlike the shared unit `E`, this captures whether a diagnostic came from `Unclosed`
// (and the opener name + start offset it carries) so the assertions can prove *what*
// was emitted, not merely *that* something was.

#[derive(Debug, Clone, PartialEq)]
enum RE {
  Unclosed { name: String, start: usize },
  Other,
}

// The migration arm: the delimited many-builders now require `From<Unclosed<…>>`. The tag is
// the erased `()`; the delimiter identity rides the carried name.
impl<D, Lang: ?Sized> From<Unclosed<D, SimpleSpan, Lang>> for RE {
  fn from(err: Unclosed<D, SimpleSpan, Lang>) -> Self {
    RE::Unclosed {
      name: err.name_ref().to_string(),
      start: err.span().start(),
    }
  }
}

impl From<()> for RE {
  fn from(_: ()) -> Self {
    RE::Other
  }
}
impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for RE {
  fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self {
    RE::Other
  }
}
impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for RE {
  fn from(_: FullContainer<S, Lang>) -> Self {
    RE::Other
  }
}
impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for RE {
  fn from(_: TooFew<S, Lang>) -> Self {
    RE::Other
  }
}
impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for RE {
  fn from(_: TooMany<S, Lang>) -> Self {
    RE::Other
  }
}
impl From<UnexpectedEot> for RE {
  fn from(_: UnexpectedEot) -> Self {
    RE::Other
  }
}
impl<'a, K: Clone, O, Lang: ?Sized> From<MissingToken<'a, K, O, Lang>> for RE {
  fn from(_: MissingToken<'a, K, O, Lang>) -> Self {
    RE::Other
  }
}
impl<'a, T, K: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, K, S, Lang>> for RE {
  fn from(_: SeparatedError<'a, T, K, S, Lang>) -> Self {
    RE::Other
  }
}
impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for RE {
  fn from(_: MissingSyntax<O, Lang>) -> Self {
    RE::Other
  }
}

type VerboseCtx<'inp> = ParserContext<'inp, TestLexer<'inp>, Verbose<RE>>;

fn fatal_ctx() -> ParserContext<'static, TestLexer<'static>, Fatal<RE>> {
  ParserContext::new(Fatal::new())
}
fn verbose_ctx() -> VerboseCtx<'static> {
  ParserContext::new(Verbose::new())
}

/// Pulls the recorded `Unclosed` diagnostics (name, start) out of a `Verbose<RE>` sink, in
/// span order.
fn recorded_unclosed(em: &Verbose<RE>) -> Vec<(String, usize)> {
  em.errors()
    .values()
    .flatten()
    .filter_map(|e| match e {
      RE::Unclosed { name, start } => Some((name.clone(), *start)),
      RE::Other => None,
    })
    .collect()
}

// ── Element parsers / stop condition ───────────────────────────────────────────

fn try_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, RE>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RE>,
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

fn parse_num<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<i64, RE>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RE>,
{
  match inp.next()? {
    None => Err(RE::Other),
    Some(tok) => match tok.into_data() {
      Token::Num(n) => Ok(n),
      _ => Err(RE::Other),
    },
  }
}

fn decide_num<'inp, Ctx>(
  mut peeked: Peeked<'_, 'inp, TestLexer<'inp>, U1>,
  _: &mut Ctx::Emitter,
) -> Result<Action, <Ctx::Emitter as Emitter<'inp, TestLexer<'inp>>>::Error>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
{
  Ok(match peeked.pop_front() {
    None => Action::Stop,
    Some(tok) => {
      let tok = tok
        .as_maybe_ref()
        .map(|t| t.token().copied(), |t| t.token())
        .into_inner();
      if matches!(**tok.data(), Token::Num(_)) {
        Action::Continue
      } else {
        Action::Stop
      }
    }
  })
}

// ═══════════════════════════════════════════════════════════════════════════════
// repeated_while + delimited — THE confirmed probe, full pair × emitter matrix.
// The three inputs "(1 2" / "[1 2" / "{1 2" are the exact regression inputs.
// ═══════════════════════════════════════════════════════════════════════════════

macro_rules! rw_matrix {
  ($fatal:ident, $verbose:ident, $delim:ty, $src:literal, $name:literal) => {
    #[test]
    fn $fatal() {
      fn go<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, RE>
      where
        Ctx: ParseContext<'inp, TestLexer<'inp>>,
        Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RE>
          + FullContainerEmitter<'inp, TestLexer<'inp>>
          + SeparatedEmitter<'inp, TestLexer<'inp>>
          + UnclosedEmitter<'inp, TestLexer<'inp>>,
      {
        parse_num
          .repeated_while::<_, U1>(decide_num::<Ctx>)
          .delimited::<$delim>()
          .collect()
          .parse_input(inp)
      }
      let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx()).apply(go).parse_str($src);
      assert_eq!(
        r,
        Err(RE::Unclosed {
          name: $name.to_string(),
          start: 0
        }),
        "fatal: unterminated {} must Err with Unclosed at the opener",
        $src
      );
    }

    #[test]
    fn $verbose() {
      fn go<'inp>(
        inp: &mut InputRef<'inp, '_, TestLexer<'inp>, VerboseCtx<'inp>>,
      ) -> Result<(Vec<i64>, Vec<(String, usize)>), RE> {
        let items: Vec<i64> = parse_num
          .repeated_while::<_, U1>(decide_num::<VerboseCtx<'inp>>)
          .delimited::<$delim>()
          .collect()
          .parse_input(inp)?;
        let diags = recorded_unclosed(inp.emitter());
        Ok((items, diags))
      }
      let (items, diags) = Parser::with_context(verbose_ctx())
        .apply(go)
        .parse_str($src)
        .unwrap();
      assert_eq!(
        items,
        vec![1, 2],
        "verbose: recovery yields the collected elements"
      );
      assert_eq!(
        diags,
        vec![($name.to_string(), 0)],
        "verbose: records exactly one Unclosed at the opener"
      );
    }
  };
}

rw_matrix!(rw_paren_fatal, rw_paren_verbose, Paren<(), (), ()>, "(1 2", "()");
rw_matrix!(rw_bracket_fatal, rw_bracket_verbose, Bracket<(), (), ()>, "[1 2", "[]");
rw_matrix!(rw_brace_fatal, rw_brace_verbose, Brace<(), (), ()>, "{1 2", "{}");

// Miss shape (b): a wrong token where the closer belongs is unexpected-token
// (expected-close), NOT Unclosed. `[1 2 )` stops at `)`, which is not `]`.
#[test]
fn rw_wrong_close_is_unexpected_token_not_unclosed() {
  fn go<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, RE>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RE>
      + FullContainerEmitter<'inp, TestLexer<'inp>>
      + SeparatedEmitter<'inp, TestLexer<'inp>>
      + UnclosedEmitter<'inp, TestLexer<'inp>>,
  {
    parse_num
      .repeated_while::<_, U1>(decide_num::<Ctx>)
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)
  }
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx())
    .apply(go)
    .parse_str("[1 2 )");
  assert_eq!(
    r,
    Err(RE::Other),
    "wrong closer must be unexpected-token, not Unclosed"
  );
}

// No opener at all: nothing is unclosed. `1 2` (bracket expected) reports the wrong opener
// (unexpected-token), never Unclosed.
#[test]
fn rw_no_opener_is_not_unclosed() {
  fn go<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, RE>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RE>
      + FullContainerEmitter<'inp, TestLexer<'inp>>
      + SeparatedEmitter<'inp, TestLexer<'inp>>
      + UnclosedEmitter<'inp, TestLexer<'inp>>,
  {
    parse_num
      .repeated_while::<_, U1>(decide_num::<Ctx>)
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)
  }
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx()).apply(go).parse_str("1 2");
  assert_eq!(
    r,
    Err(RE::Other),
    "no opener ⇒ unexpected-token, not Unclosed"
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// The other three delim drivers — one EOI case each, both emitters, bracket pair.
// ═══════════════════════════════════════════════════════════════════════════════

// repeated + delimited
#[test]
fn rd_bracket_fatal_errors_unclosed() {
  fn go<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, RE>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RE>
      + FullContainerEmitter<'inp, TestLexer<'inp>>
      + UnclosedEmitter<'inp, TestLexer<'inp>>,
  {
    try_num
      .repeated()
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)
  }
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx())
    .apply(go)
    .parse_str("[1 2");
  assert_eq!(
    r,
    Err(RE::Unclosed {
      name: "[]".to_string(),
      start: 0
    })
  );
}

#[test]
fn rd_bracket_verbose_records_and_recovers() {
  fn go<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, VerboseCtx<'inp>>,
  ) -> Result<(Vec<i64>, Vec<(String, usize)>), RE> {
    let items: Vec<i64> = try_num
      .repeated()
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)?;
    Ok((items, recorded_unclosed(inp.emitter())))
  }
  let (items, diags) = Parser::with_context(verbose_ctx())
    .apply(go)
    .parse_str("[1 2")
    .unwrap();
  assert_eq!(items, vec![1, 2]);
  assert_eq!(diags, vec![("[]".to_string(), 0)]);
}

// separated + delimited (this driver used to error with the WRONG vocabulary — a stale
// unexpected-token on the last element — rather than Unclosed; now it is Unclosed).
#[test]
fn sep_bracket_fatal_errors_unclosed() {
  fn go<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, RE>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RE>
      + SeparatedEmitter<'inp, TestLexer<'inp>>
      + FullContainerEmitter<'inp, TestLexer<'inp>>
      + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
      + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
      + TooManyEmitter<'inp, TestLexer<'inp>>
      + UnclosedEmitter<'inp, TestLexer<'inp>>,
  {
    try_num
      .separated_by_comma()
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)
  }
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx())
    .apply(go)
    .parse_str("[1,2");
  assert_eq!(
    r,
    Err(RE::Unclosed {
      name: "[]".to_string(),
      start: 0
    })
  );
}

#[test]
fn sep_bracket_verbose_records_and_recovers() {
  fn go<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, VerboseCtx<'inp>>,
  ) -> Result<(Vec<i64>, Vec<(String, usize)>), RE> {
    let items: Vec<i64> = try_num
      .separated_by_comma()
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)?;
    Ok((items, recorded_unclosed(inp.emitter())))
  }
  let (items, diags) = Parser::with_context(verbose_ctx())
    .apply(go)
    .parse_str("[1,2")
    .unwrap();
  assert_eq!(items, vec![1, 2]);
  assert_eq!(diags, vec![("[]".to_string(), 0)]);
}

// separated_while + delimited
#[test]
fn sw_bracket_fatal_errors_unclosed() {
  fn go<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<Vec<i64>, RE>
  where
    Ctx: ParseContext<'inp, TestLexer<'inp>>,
    Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = RE>
      + SeparatedEmitter<'inp, TestLexer<'inp>>
      + FullContainerEmitter<'inp, TestLexer<'inp>>
      + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
      + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
      + TooManyEmitter<'inp, TestLexer<'inp>>
      + UnclosedEmitter<'inp, TestLexer<'inp>>,
  {
    parse_num
      .separated_by_comma_while::<_, U1>(decide_num::<Ctx>)
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)
  }
  let r: Result<Vec<i64>, _> = Parser::with_context(fatal_ctx())
    .apply(go)
    .parse_str("[1,2");
  assert_eq!(
    r,
    Err(RE::Unclosed {
      name: "[]".to_string(),
      start: 0
    })
  );
}

#[test]
fn sw_bracket_verbose_records_and_recovers() {
  fn go<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, VerboseCtx<'inp>>,
  ) -> Result<(Vec<i64>, Vec<(String, usize)>), RE> {
    let items: Vec<i64> = parse_num
      .separated_by_comma_while::<_, U1>(decide_num::<VerboseCtx<'inp>>)
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)?;
    Ok((items, recorded_unclosed(inp.emitter())))
  }
  let (items, diags) = Parser::with_context(verbose_ctx())
    .apply(go)
    .parse_str("[1,2")
    .unwrap();
  assert_eq!(items, vec![1, 2]);
  assert_eq!(diags, vec![("[]".to_string(), 0)]);
}
