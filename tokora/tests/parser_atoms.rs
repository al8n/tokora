#![cfg(all(feature = "std", feature = "logos"))]

//! The dialect-free atom surface: the `ComposableEmitter`/`ParseCtx` bundles and the
//! policy atoms promoted from smear-parser-next (W-MOVE).
//!
//! The cases are ported one-for-one from the smear-side suites (`combinator/tests.rs`
//! for the bundle gate, `combinator/shape/tests.rs` for the atoms), with the fixtures
//! adapted from smear's GraphQL lexers to the shared `common` logos fixture: smear's
//! three-source matrix (`str`/`[u8]`/`Bytes`) collapses to the fixture's `str`, its two
//! dialects collapse to `TestLexer` driven under two context shapes, and its `@`/`|`
//! tokens map to the fixture's `,`/`;` punctuators. Every assertion is otherwise the
//! same as the smear original it pins.

mod common;

use common::{E, TestLexer};

use tokora::{
  FatalContext, InputRef, Lexer, Parse, ParseCtx, Parser, ParserContext,
  emitter::{FullContainerEmitter, SeparatedEmitter, TooFewEmitter, Verbose},
};

// ── The bundle gate (smear: combinator/tests.rs) ──────────────────────────────

fn assert_ctx<'inp, L, Ctx>()
where
  L: Lexer<'inp>,
  Ctx: ParseCtx<'inp, L>,
{
}

fn requires_separated<'inp, L, Em>()
where
  L: Lexer<'inp>,
  Em: SeparatedEmitter<'inp, L>,
{
}

fn requires_too_few<'inp, L, Em>()
where
  L: Lexer<'inp>,
  Em: TooFewEmitter<'inp, L>,
{
}

fn requires_full_container<'inp, L, Em>()
where
  L: Lexer<'inp>,
  Em: FullContainerEmitter<'inp, L>,
{
}

fn elaborates<'inp, L, Ctx>()
where
  L: Lexer<'inp>,
  Ctx: ParseCtx<'inp, L>,
{
  requires_separated::<L, Ctx::Emitter>();
  requires_too_few::<L, Ctx::Emitter>();
  requires_full_container::<L, Ctx::Emitter>();
}

// Smear's `ctx_bundle_holds_for_both_dialects_and_sources`: two dialects over two
// sources there; here the fixture lexer under both a fail-fast and a collecting
// context, pinning that the bundle holds and elaborates for each.
#[test]
fn ctx_bundle_holds_for_fatal_and_verbose_contexts() {
  assert_ctx::<TestLexer<'_>, FatalContext<'_, TestLexer<'_>, E>>();
  assert_ctx::<TestLexer<'_>, ParserContext<'_, TestLexer<'_>, Verbose<E>>>();
  elaborates::<TestLexer<'_>, FatalContext<'_, TestLexer<'_>, E>>();
  elaborates::<TestLexer<'_>, ParserContext<'_, TestLexer<'_>, Verbose<E>>>();
}

// Smear's `trivial_parse_drives_over_str_and_slice`: a bundle-shaped closure drives an
// actual parse end to end (the fixture is `str`-only, so the slice arm collapses).
#[test]
fn trivial_parse_drives_through_the_bundle() {
  fn drive<'inp, O>(
    f: impl for<'c> FnMut(
      &mut InputRef<'inp, 'c, TestLexer<'inp>, FatalContext<'inp, TestLexer<'inp>, E>>,
    ) -> Result<O, E>,
    input: &'inp str,
  ) -> Result<O, E> {
    Parser::with_parser(f).parse_str(input)
  }

  let out = drive(|inp| inp.next().map(|t| t.is_some()), "query");
  assert!(matches!(out, Ok(true)));
}
