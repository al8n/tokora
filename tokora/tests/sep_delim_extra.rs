#![cfg(all(feature = "std", feature = "logos"))]

//! Tests exercising the **spanned** (`With<Collect<...>, PhantomSpan>`) and
//! **mut-ref** (`Collect<&mut ..., &mut Container>`) code paths for every
//! separator-policy x count-modifier combination in the `sep/delim` directory.
//!
//! 32 policy combos x 2 paths = 64 tests.

mod common;

use common::E;

use tokora::{
  Accumulator, Emitter, InputRef, Parse, ParseContext, ParseInput, Parser, ParserContext,
  SimpleSpan, TryParseInput,
  emitter::{
    Fatal, FullContainerEmitter, MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter,
    SeparatedEmitter, TooFewEmitter, TooManyEmitter, UnclosedEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter, Verbose,
  },
  parser::With,
  punct::Bracket,
  span::Spanned,
  try_parse_input::ParseAttempt,
  utils::marker::PhantomSpan,
};

use common::{TestLexer, Token};

fn full_ctx() -> ParserContext<'static, TestLexer<'static>, Fatal<E>> {
  ParserContext::new(Fatal::new())
}

fn verbose_ctx() -> ParserContext<'static, TestLexer<'static>, Verbose<E>> {
  ParserContext::new(Verbose::new())
}

// ── Element parser ────────────────────────────────────────────────────────────

fn try_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, E>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>,
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

// ── Test macro ────────────────────────────────────────────────────────────────

macro_rules! sep_delim_tests {
  ($name:ident, { $($policy:tt)* }, $input:expr) => {
    paste::paste! {
      fn [<$name _sp>]<'inp, Ctx>(
        inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
      ) -> Result<Spanned<Vec<i64>, SimpleSpan>, E>
      where
        Ctx: ParseContext<'inp, TestLexer<'inp>>,
        Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = E>
          + SeparatedEmitter<'inp, TestLexer<'inp>>
          + FullContainerEmitter<'inp, TestLexer<'inp>>
          + UnclosedEmitter<'inp, TestLexer<'inp>>
          + TooFewEmitter<'inp, TestLexer<'inp>>
          + TooManyEmitter<'inp, TestLexer<'inp>>
          + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
          + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
          + MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
          + MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>>,
      {
        With::new(
          try_num
            .separated_by_comma()
            $($policy)*
            .delimited::<Bracket<(), (), ()>>()
            .collect(),
          PhantomSpan::PHANTOM,
        )
        .parse_input(inp)
      }

      #[test]
      fn [<$name _spanned>]() {
        let r = Parser::with_context(full_ctx())
          .apply([<$name _sp>])
          .parse_str($input)
          .unwrap();
        assert!(!r.data().is_empty());
      }
    }
  };
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. allow_leading (4 count variants)
// ═══════════════════════════════════════════════════════════════════════════════

sep_delim_tests!(al_unb, { .allow_leading() }, "[,1,2,3]");
sep_delim_tests!(al_min, { .allow_leading().at_least(2) }, "[,1,2,3]");
sep_delim_tests!(al_max, { .allow_leading().at_most(3) }, "[,1,2,3]");
sep_delim_tests!(al_bnd, { .allow_leading().bounded(2, 4) }, "[,1,2,3]");

// ═══════════════════════════════════════════════════════════════════════════════
// 2. allow_trailing (4 count variants)
// ═══════════════════════════════════════════════════════════════════════════════

sep_delim_tests!(at_unb, { .allow_trailing() }, "[1,2,3,]");
sep_delim_tests!(at_min, { .allow_trailing().at_least(2) }, "[1,2,3,]");
sep_delim_tests!(at_max, { .allow_trailing().at_most(3) }, "[1,2,3,]");
sep_delim_tests!(at_bnd, { .allow_trailing().bounded(2, 4) }, "[1,2,3,]");

// ═══════════════════════════════════════════════════════════════════════════════
// 3. allow_surrounded (allow_trailing + allow_leading) (4 count variants)
// ═══════════════════════════════════════════════════════════════════════════════

sep_delim_tests!(as_unb, { .allow_trailing().allow_leading() }, "[,1,2,3,]");
sep_delim_tests!(as_min, { .allow_trailing().at_least(2).allow_leading() }, "[,1,2,3,]");
sep_delim_tests!(as_max, { .allow_trailing().at_most(3).allow_leading() }, "[,1,2,3,]");
sep_delim_tests!(as_bnd, { .allow_trailing().bounded(2, 4).allow_leading() }, "[,1,2,3,]");

// ═══════════════════════════════════════════════════════════════════════════════
// 4. require_leading (4 count variants)
// ═══════════════════════════════════════════════════════════════════════════════

sep_delim_tests!(rl_unb, { .require_leading() }, "[,1,2,3]");
sep_delim_tests!(rl_min, { .require_leading().at_least(2) }, "[,1,2,3]");
sep_delim_tests!(rl_max, { .require_leading().at_most(3) }, "[,1,2,3]");
sep_delim_tests!(rl_bnd, { .require_leading().bounded(2, 4) }, "[,1,2,3]");

// ═══════════════════════════════════════════════════════════════════════════════
// 5. require_trailing (4 count variants)
// ═══════════════════════════════════════════════════════════════════════════════

sep_delim_tests!(rt_unb, { .require_trailing() }, "[1,2,3,]");
sep_delim_tests!(rt_min, { .require_trailing().at_least(2) }, "[1,2,3,]");
sep_delim_tests!(rt_max, { .require_trailing().at_most(3) }, "[1,2,3,]");
sep_delim_tests!(rt_bnd, { .require_trailing().bounded(2, 4) }, "[1,2,3,]");

// ═══════════════════════════════════════════════════════════════════════════════
// 6. require_surrounded (require_trailing + require_leading) (4 count variants)
// ═══════════════════════════════════════════════════════════════════════════════

sep_delim_tests!(rs_unb, { .require_trailing().require_leading() }, "[,1,2,3,]");
sep_delim_tests!(rs_min, { .require_trailing().at_least(2).require_leading() }, "[,1,2,3,]");
sep_delim_tests!(rs_max, { .require_trailing().at_most(3).require_leading() }, "[,1,2,3,]");
sep_delim_tests!(rs_bnd, { .require_trailing().bounded(2, 4).require_leading() }, "[,1,2,3,]");

// ═══════════════════════════════════════════════════════════════════════════════
// 7. allow_leading_require_trailing (require_trailing + allow_leading)
//    (4 count variants)
// ═══════════════════════════════════════════════════════════════════════════════

sep_delim_tests!(alrt_unb, { .require_trailing().allow_leading() }, "[,1,2,3,]");
sep_delim_tests!(alrt_min, { .require_trailing().at_least(2).allow_leading() }, "[,1,2,3,]");
sep_delim_tests!(alrt_max, { .require_trailing().at_most(3).allow_leading() }, "[,1,2,3,]");
sep_delim_tests!(alrt_bnd, { .require_trailing().bounded(2, 4).allow_leading() }, "[,1,2,3,]");

// ═══════════════════════════════════════════════════════════════════════════════
// 8. require_leading_allow_trailing (allow_trailing + require_leading)
//    (4 count variants)
// ═══════════════════════════════════════════════════════════════════════════════

sep_delim_tests!(rlat_unb, { .allow_trailing().require_leading() }, "[,1,2,3,]");
sep_delim_tests!(rlat_min, { .allow_trailing().at_least(2).require_leading() }, "[,1,2,3,]");
sep_delim_tests!(rlat_max, { .allow_trailing().at_most(3).require_leading() }, "[,1,2,3,]");
sep_delim_tests!(rlat_bnd, { .allow_trailing().bounded(2, 4).require_leading() }, "[,1,2,3,]");

// ═══════════════════════════════════════════════════════════════════════════════
// Bounds-VIOLATING inputs inside the delim family (issue #90)
// ═══════════════════════════════════════════════════════════════════════════════
//
// Every case above feeds an input that already SATISFIES its configured bounds/policy,
// so the 64-test matrix asserts only non-emptiness and never exercises the try-driven
// `sep/delim` closer on a violation. Issue #90 is exactly that: the mid-scan-closer
// arm (`sep/delim/mod.rs`) returns as soon as the closer is found, without ever reaching
// the post-loop `handle_end` pass that enforces count bounds and separator policy — so on
// a well-formed, properly-closed list the bound/policy check never runs. These two pin
// what the driver ACTUALLY does today on inputs that violate their own configured bounds:
// a clean `Ok`, with zero diagnostics recorded. Fixing this should flip both assertions to
// the diagnostic the sibling drivers already emit for the same shape (non-delim and
// `sep_while`-delim both report it).

/// Pins CURRENT WRONG behavior (issue #90): once fixed, this should record a
/// `TooFew(1, 2)` on the `[1]` parse.
///
/// `[1]` under `.at_least(2)`: the closer is found mid-scan on this already-well-formed,
/// properly-closed list, so the driver returns through the `is_closed` arm without ever
/// reaching `handle_end`, and the `at_least(2)` bound is never checked.
#[test]
fn at_least_violation_inside_delim_returns_clean_ok() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, Verbose<E>>>,
  ) -> Result<Vec<i64>, E> {
    let out = try_num
      .separated_by_comma()
      .at_least(2)
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)?;
    let errs = inp.emitter().errors();
    assert!(
      errs.is_empty(),
      "pinned bug (issue #90): today `handle_end` never runs on this path, so no \
       `TooFew` is recorded (found {errs:?}) — fixing it must record exactly one \
       TooFew(1, 2)"
    );
    Ok(out)
  }

  let r: Vec<i64> = Parser::with_context(verbose_ctx())
    .apply(parse)
    .parse_str("[1]")
    .unwrap();
  assert_eq!(
    r,
    vec![1],
    "pinned bug (issue #90): a bounds-violating list (1 element under at_least(2)) \
     still parses clean — the count bound never fires on the try-driven delim path"
  );
}

/// Pins CURRENT WRONG behavior (issue #90): once fixed, this should record an
/// unexpected-trailing-separator diagnostic on the `[1,]` parse.
///
/// `[1,]` under the **default** policy (no `.allow_trailing()`/`.require_trailing()` at
/// all — trailing separators are unexpected unless explicitly allowed): the mid-scan
/// closer arm accepts the list before the end-state pass that would reject the trailing
/// comma ever runs, regardless of which policy is configured — the bug is dead code on
/// this path, not a specific policy's gap.
#[test]
fn default_policy_trailing_separator_inside_delim_returns_clean_ok() {
  fn parse<'inp>(
    inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, Verbose<E>>>,
  ) -> Result<Vec<i64>, E> {
    let out = try_num
      .separated_by_comma()
      .delimited::<Bracket<(), (), ()>>()
      .collect()
      .parse_input(inp)?;
    let errs = inp.emitter().errors();
    assert!(
      errs.is_empty(),
      "pinned bug (issue #90): today `handle_end` never runs on this path, so no \
       trailing-separator diagnostic is recorded (found {errs:?}) — fixing it must \
       record an error"
    );
    Ok(out)
  }

  let r: Vec<i64> = Parser::with_context(verbose_ctx())
    .apply(parse)
    .parse_str("[1,]")
    .unwrap();
  assert_eq!(
    r,
    vec![1],
    "pinned bug (issue #90): a trailing separator the default policy should reject \
     still parses clean — no leading/trailing allowance was ever configured"
  );
}
