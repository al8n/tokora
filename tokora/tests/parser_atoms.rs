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

use common::{E, TestLexer, Token, TokenKind};

use tokora::{
  Emitter, FatalContext, InputRef, Lexer, Parse, ParseCtx, Parser, ParserContext, SimpleSpan,
  emitter::{Fatal, FullContainerEmitter, SeparatedEmitter, TooFewEmitter, Verbose},
  parser::{list_of, opt, peek_kind, separated1},
  punct::{CloseBrace, Comma, Semicolon},
  token::IdentifierToken,
  types::Ident,
};

// The atoms' item parser is `Ident::parse`, which classifies through
// `IdentifierToken`; the shared fixture token only wires `PunctuatorToken`, so the
// identifier facet lives here with the one suite that needs it.
impl IdentifierToken<'_> for Token {
  fn is_identifier(&self) -> bool {
    matches!(self, Token::Ident)
  }
}

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
  fn drive_fatal<'inp, O>(
    f: impl for<'c> FnMut(
      &mut InputRef<'inp, 'c, TestLexer<'inp>, FatalContext<'inp, TestLexer<'inp>, E>>,
    ) -> Result<O, E>,
    input: &'inp str,
  ) -> Result<O, E> {
    Parser::with_parser(f).parse_str(input)
  }

  let out = drive_fatal(|inp| inp.next().map(|t| t.is_some()), "query");
  assert!(matches!(out, Ok(true)));
}

// ── The atom harness (smear: shape/tests.rs) ─────────────────────────────────
//
// The port of smear's `drive_str`, the surviving arm of its `drive_all!` source
// matrix: one concrete lexer, still generic over the emitter, so the same closure
// runs under a fail-fast `Fatal` and a collecting `Verbose` context.

fn drive<'inp, O, Em>(
  emitter: Em,
  f: impl for<'c> FnMut(
    &mut InputRef<'inp, 'c, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, Em>>,
  ) -> Result<O, E>,
  input: &'inp str,
) -> Result<O, E>
where
  Em: Emitter<'inp, TestLexer<'inp>, Error = E>,
{
  let ctx: ParserContext<'inp, TestLexer<'inp>, Em> = ParserContext::new(emitter);
  Parser::with_parser_and_context(f, ctx).parse_str(input)
}

/// Continue-predicate for `separated1`: the next token is an identifier, so it starts
/// another item in a `,`-separated list of names.
fn starts_ident(tok: &Token) -> bool {
  matches!(tok, Token::Ident)
}

/// Stop-predicate for `list_of`: the next token is the closing brace that ends the
/// list, so the loop stops and leaves the `}` in place.
fn is_close_brace(tok: &Token) -> bool {
  matches!(tok, Token::RBrace)
}

// ── `separated1` (smear: shape/tests.rs) ─────────────────────────────────────
//
// One-or-more `,`-separated idents (smear drove `|`; the fixture's separator is the
// comma), leading separator allowed, trailing rejected, at least one required. The
// sources are spaced exactly like smear's, so every span assertion carries over
// unchanged.

#[test]
fn separated1_three_idents() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let items = separated1::<Comma, _, _, _, _, _, _>(Ident::parse, starts_ident)(inp)?;
      assert_eq!(items.len(), 3);
      assert_eq!(*items[0].source_ref(), "A");
      assert_eq!(items[0].span(), SimpleSpan::new(0, 1));
      assert_eq!(*items[1].source_ref(), "B");
      assert_eq!(items[1].span(), SimpleSpan::new(4, 5));
      assert_eq!(*items[2].source_ref(), "C");
      assert_eq!(items[2].span(), SimpleSpan::new(8, 9));
      Ok::<_, E>(())
    },
    "A , B , C",
  );
  assert!(out.is_ok());
}

#[test]
fn separated1_allows_leading_separator() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      // The leading `,` is consumed and does not count as an item, so the two
      // names remain and their spans skip the leading separator.
      let items = separated1::<Comma, _, _, _, _, _, _>(Ident::parse, starts_ident)(inp)?;
      assert_eq!(items.len(), 2);
      assert_eq!(*items[0].source_ref(), "A");
      assert_eq!(items[0].span(), SimpleSpan::new(2, 3));
      assert_eq!(*items[1].source_ref(), "B");
      assert_eq!(items[1].span(), SimpleSpan::new(6, 7));
      Ok::<_, E>(())
    },
    ", A , B",
  );
  assert!(out.is_ok());
}

// A trailing separator is unexpected (no `allow_trailing`): the closing `,` with no
// item after it is an unexpected-trailing-separator emit. Under a fail-fast emitter it
// aborts to `Err`; under a collecting emitter it is recorded and the parse threads on
// to return the one item gathered before the trailing separator.
#[test]
fn separated1_trailing_separator_errors() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| separated1::<Comma, _, _, _, _, _, _>(Ident::parse, starts_ident)(inp).map(|_| ()),
    "A ,",
  );
  assert!(out.is_err());

  let out = drive(
    Verbose::<E>::new(),
    |inp| {
      let items = separated1::<Comma, _, _, _, _, _, _>(Ident::parse, starts_ident)(inp)?;
      assert_eq!(items.len(), 1);
      assert_eq!(*items[0].source_ref(), "A");
      assert_eq!(items[0].span(), SimpleSpan::new(0, 1));
      Ok::<_, E>(())
    },
    "A ,",
  );
  assert!(out.is_ok());
}

// At least one item is required: empty input is a too-few emit. Under a fail-fast
// emitter it aborts to `Err`; under a collecting emitter it is recorded and the parse
// returns the empty collection.
#[test]
fn separated1_empty_errors() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| separated1::<Comma, _, _, _, _, _, _>(Ident::parse, starts_ident)(inp).map(|_| ()),
    "",
  );
  assert!(out.is_err());

  let out = drive(
    Verbose::<E>::new(),
    |inp| {
      let items = separated1::<Comma, _, _, _, _, _, _>(Ident::parse, starts_ident)(inp)?;
      assert!(items.is_empty());
      Ok::<_, E>(())
    },
    "",
  );
  assert!(out.is_ok());
}

// ── `list_of` (smear: shape/tests.rs) ────────────────────────────────────────
//
// Zero-or-more idents, no separator, stopping at the `}` the stop predicate accepts.
// The accept path asserts each item's slice and span, then commits `CloseBrace` to
// prove `list_of` stopped before the `}` and left it in place.

#[test]
fn list_of_three_idents_until_brace() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let items = list_of(Ident::parse, is_close_brace)(inp)?;
      assert_eq!(items.len(), 3);
      assert_eq!(*items[0].source_ref(), "a");
      assert_eq!(items[0].span(), SimpleSpan::new(0, 1));
      assert_eq!(*items[1].source_ref(), "b");
      assert_eq!(items[1].span(), SimpleSpan::new(2, 3));
      assert_eq!(*items[2].source_ref(), "c");
      assert_eq!(items[2].span(), SimpleSpan::new(4, 5));
      // The `}` was the stop token, left in place, so the committed closer parses it.
      let close = CloseBrace::parse(inp)?;
      assert_eq!(close.span(), &SimpleSpan::new(6, 7));
      Ok::<_, E>(())
    },
    "a b c }",
  );
  assert!(out.is_ok());
}

// An immediate stop yields an empty list with the stop token left in place — the
// zero-or-more lower bound, with no diagnostic.
#[test]
fn list_of_empty_leaves_stop_token() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let items = list_of(Ident::parse, is_close_brace)(inp)?;
      assert!(items.is_empty());
      let close = CloseBrace::parse(inp)?;
      assert_eq!(close.span(), &SimpleSpan::new(0, 1));
      Ok::<_, E>(())
    },
    "}",
  );
  assert!(out.is_ok());
}

// ── `peek_kind` (smear: shape/tests.rs) ──────────────────────────────────────
//
// Reports the next kind without consuming, so the same peek repeats and a committed
// atom still parses the leftover; end of input peeks as `None`. Smear peeked an `@`;
// the fixture's stand-in punctuator is the comma.

#[test]
fn peek_kind_reports_comma_twice_then_comma_parses() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let first = peek_kind(inp)?;
      let second = peek_kind(inp)?;
      assert_eq!(first, Some(TokenKind::Comma));
      assert_eq!(first, second);
      // Peeking consumed nothing, so the committed `Comma` atom pulls the `,` straight
      // off the input; its span proves the leftover and that peeking left it in place.
      let comma = Comma::parse(inp)?;
      assert_eq!(comma.span(), &SimpleSpan::new(0, 1));
      Ok::<_, E>(())
    },
    ",x",
  );
  assert!(out.is_ok());
}

#[test]
fn peek_kind_none_on_empty_input() {
  let out = drive(Fatal::<E>::new(), peek_kind, "");
  assert!(matches!(out, Ok(None)));
}

// ── `opt` (smear: shape/tests.rs) ────────────────────────────────────────────
//
// An accepted `try_`-attempt becomes `Some`, a decline becomes `None` with the
// leftover left in place for the next atom. Smear lifted `try_at` over `@`/`:`; the
// fixture's stand-ins are the comma and the semicolon.

#[test]
fn opt_try_comma_accepts_comma() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let opt_comma = opt(Comma::try_parse)(inp)?;
      assert!(opt_comma.is_some());
      let comma = opt_comma.unwrap();
      assert_eq!(comma.span(), &SimpleSpan::new(0, 1));
      Ok::<_, E>(())
    },
    ",",
  );
  assert!(out.is_ok());
}

#[test]
fn opt_try_comma_declines_on_semicolon_and_leaves_it() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let declined = opt(Comma::try_parse)(inp)?.is_none();
      // The `;` is untouched, so the committed `Semicolon` atom parses it; its span
      // proves the leftover's type and that the decline consumed nothing.
      let semi = Semicolon::parse(inp)?;
      assert_eq!(semi.span(), &SimpleSpan::new(0, 1));
      Ok::<_, E>(declined)
    },
    ";",
  );
  assert!(matches!(out, Ok(true)));
}
