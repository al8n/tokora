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
  error::{
    UnexpectedEot,
    syntax::{FullContainer, MissingSyntax, TooFew, TooMany},
    token::{MissingToken, SeparatedError, UnexpectedToken},
  },
  parser::{angles, braces, brackets, delimited, list_of, opt, parens, peek_kind, separated1},
  punct::{Brace, CloseBrace, Comma, Paren, Semicolon},
  span::Spanned,
  token::IdentifierToken,
  types::Ident,
  utils::Expected,
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

// ── Delimited shapes (smear: shape/tests.rs) ─────────────────────────────────
//
// `parens`/`braces`/`brackets`/`angles` commit the opener, run the inner sub-parser,
// commit the closer, and wrap the three in a `Delimited` spanning the whole construct;
// the generic `delimited::<D>` does the same through the `Delimiter` pair type. The
// accept paths assert the wrapped data (slice and span), each delimiter's span, and the
// construct span; the unterminated paths run under both emitter modes. Smear's `ident`
// inner becomes the fixture's `Ident::parse`.

/// An inner sub-parser that records one non-fatal diagnostic and then yields `()`. Under
/// a fail-fast emitter the emit is fatal and this errors; under a collecting emitter the
/// diagnostic is recorded, the emit recovers, and this returns `Ok` — so an enclosing
/// delimited atom threads on to commit its closer. It is a bare `fn` so the atom's
/// higher-order bound pins the lexer for the emitter call, the way the atom fns compose.
fn emit_then_recover<'inp, Em>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, ParserContext<'inp, TestLexer<'inp>, Em>>,
) -> Result<(), E>
where
  Em: Emitter<'inp, TestLexer<'inp>, Error = E>,
{
  inp
    .emitter()
    .emit_error(Spanned::new(SimpleSpan::new(0, 0), E))?;
  Ok(())
}

#[test]
fn braces_wrap_ident() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let delimited = braces(Ident::parse)(inp)?;
      assert_eq!(*delimited.data().source_ref(), "x");
      assert_eq!(delimited.data().span(), SimpleSpan::new(1, 2));
      assert_eq!(delimited.span(), SimpleSpan::new(0, 3));
      assert_eq!(delimited.open_ref().span(), &SimpleSpan::new(0, 1));
      assert_eq!(delimited.close_ref().span(), &SimpleSpan::new(2, 3));
      Ok::<_, E>(())
    },
    "{x}",
  );
  assert!(out.is_ok());
}

#[test]
fn parens_wrap_ident() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let delimited = parens(Ident::parse)(inp)?;
      assert_eq!(*delimited.data().source_ref(), "x");
      assert_eq!(delimited.data().span(), SimpleSpan::new(1, 2));
      assert_eq!(delimited.span(), SimpleSpan::new(0, 3));
      assert_eq!(delimited.open_ref().span(), &SimpleSpan::new(0, 1));
      assert_eq!(delimited.close_ref().span(), &SimpleSpan::new(2, 3));
      Ok::<_, E>(())
    },
    "(x)",
  );
  assert!(out.is_ok());
}

#[test]
fn brackets_wrap_ident() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let delimited = brackets(Ident::parse)(inp)?;
      assert_eq!(*delimited.data().source_ref(), "x");
      assert_eq!(delimited.data().span(), SimpleSpan::new(1, 2));
      assert_eq!(delimited.span(), SimpleSpan::new(0, 3));
      assert_eq!(delimited.open_ref().span(), &SimpleSpan::new(0, 1));
      assert_eq!(delimited.close_ref().span(), &SimpleSpan::new(2, 3));
      Ok::<_, E>(())
    },
    "[x]",
  );
  assert!(out.is_ok());
}

// An empty group with an `opt(Ident::try_parse)` inner yields `None` inside, proving the
// inner runs between the delimiters and that a declining inner leaves the closer for the
// atom to commit.
#[test]
fn braces_empty_with_opt_inner_yields_none() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let delimited = braces(opt(Ident::try_parse))(inp)?;
      assert!(delimited.data().is_none());
      assert_eq!(delimited.span(), SimpleSpan::new(0, 2));
      assert_eq!(delimited.open_ref().span(), &SimpleSpan::new(0, 1));
      assert_eq!(delimited.close_ref().span(), &SimpleSpan::new(1, 2));
      Ok::<_, E>(())
    },
    "{}",
  );
  assert!(out.is_ok());
}

// Unterminated groups: the inner consumes the `a` ident, then the missing closer makes
// the committed closer atom error on end of input. That error propagates identically
// under a fail-fast and a collecting emitter — a missing closer is not recovered into a
// fabricated delimiter — so both modes return `Err`.

#[test]
fn braces_unterminated_errors() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| braces(Ident::parse)(inp).map(|_| ()),
    "{a",
  );
  assert!(out.is_err());
  let out = drive(
    Verbose::<E>::new(),
    |inp| braces(Ident::parse)(inp).map(|_| ()),
    "{a",
  );
  assert!(out.is_err());
}

#[test]
fn parens_unterminated_errors() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| parens(Ident::parse)(inp).map(|_| ()),
    "(a",
  );
  assert!(out.is_err());
  let out = drive(
    Verbose::<E>::new(),
    |inp| parens(Ident::parse)(inp).map(|_| ()),
    "(a",
  );
  assert!(out.is_err());
}

#[test]
fn brackets_unterminated_errors() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| brackets(Ident::parse)(inp).map(|_| ()),
    "[a",
  );
  assert!(out.is_err());
  let out = drive(
    Verbose::<E>::new(),
    |inp| brackets(Ident::parse)(inp).map(|_| ()),
    "[a",
  );
  assert!(out.is_err());
}

// The collecting-mode recovery continuation: an inner sub-parser that records a non-fatal
// diagnostic and then yields. Under a fail-fast emitter the emit is fatal, so `braces`
// aborts before the closer and returns `Err`. Under a collecting emitter the emit is
// recorded and recovers, so the inner returns `Ok`, `braces` threads on to commit the `}`
// closer, and it returns the delimited construct spanning `{}`.
#[test]
fn braces_thread_collecting_mode_inner_emit() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| braces(emit_then_recover)(inp).map(|_| ()),
    "{}",
  );
  assert!(out.is_err());
  let out = drive(
    Verbose::<E>::new(),
    |inp| {
      let delimited = braces(emit_then_recover)(inp)?;
      assert_eq!(delimited.span(), SimpleSpan::new(0, 2));
      assert_eq!(delimited.open_ref().span(), &SimpleSpan::new(0, 1));
      assert_eq!(delimited.close_ref().span(), &SimpleSpan::new(1, 2));
      Ok::<_, E>(())
    },
    "{}",
  );
  assert!(out.is_ok());
}

// New coverage: `angles` (the pair with no smear-side dialect vehicle) over the fixture,
// which wires both capability routes for `<`/`>`.
#[test]
fn angles_wrap_ident() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let delimited = angles(Ident::parse)(inp)?;
      assert_eq!(*delimited.data().source_ref(), "x");
      assert_eq!(delimited.data().span(), SimpleSpan::new(1, 2));
      assert_eq!(delimited.span(), SimpleSpan::new(0, 3));
      assert_eq!(delimited.open_ref().span(), &SimpleSpan::new(0, 1));
      assert_eq!(delimited.close_ref().span(), &SimpleSpan::new(2, 3));
      Ok::<_, E>(())
    },
    "<x>",
  );
  assert!(out.is_ok());
}

#[test]
fn angles_unterminated_errors() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| angles(Ident::parse)(inp).map(|_| ()),
    "<a",
  );
  assert!(out.is_err());
  let out = drive(
    Verbose::<E>::new(),
    |inp| angles(Ident::parse)(inp).map(|_| ()),
    "<a",
  );
  assert!(out.is_err());
}

// The `≡` proof: the generic `delimited::<Paren, …>` over `(x)` yields the same data and
// spans as `parens` (test `parens_wrap_ident`) — and, because the built-in pair's
// `OpenValue`/`CloseValue` normalize to the named alias inner types, the same result type.
#[test]
fn delimited_generic_paren_equals_parens() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let d = delimited::<Paren, _, _, _, _, _>(Ident::parse)(inp)?;
      assert_eq!(*d.data().source_ref(), "x");
      assert_eq!(d.data().span(), SimpleSpan::new(1, 2));
      assert_eq!(d.span(), SimpleSpan::new(0, 3));
      assert_eq!(d.open_ref().span(), &SimpleSpan::new(0, 1));
      assert_eq!(d.close_ref().span(), &SimpleSpan::new(2, 3));
      Ok::<_, E>(())
    },
    "(x)",
  );
  assert!(out.is_ok());
}

// A wrong closer is a hard error carrying the expected close kind. `ShapeError` (local,
// discriminating — the shared `E` is unit) pins that the error IS an `UnexpectedToken`
// naming the close-brace kind, not a fabricated delimiter.
#[test]
fn delimited_wrong_close_reports_unexpected_token() {
  fn wrong_close<'a>(
    inp: &mut InputRef<'a, '_, TestLexer<'a>, FatalContext<'a, TestLexer<'a>, ShapeError>>,
  ) -> Result<(), ShapeError> {
    delimited::<Brace, _, _, _, _, _>(Ident::parse)(inp).map(|_| ())
  }
  let err = Parser::with_parser(wrong_close)
    .parse_str("{x)")
    .unwrap_err();
  assert!(matches!(
    err,
    ShapeError::Unexpected(Some(TokenKind::RBrace))
  ));
}

// A missing closer at end of input is an `UnexpectedEot`, pinned distinct from the
// unexpected-token path by the same local discriminating error.
#[test]
fn delimited_eof_reports_unexpected_eot() {
  fn unterminated<'a>(
    inp: &mut InputRef<'a, '_, TestLexer<'a>, FatalContext<'a, TestLexer<'a>, ShapeError>>,
  ) -> Result<(), ShapeError> {
    delimited::<Paren, _, _, _, _, _>(Ident::parse)(inp).map(|_| ())
  }
  let err = Parser::with_parser(unterminated)
    .parse_str("(a")
    .unwrap_err();
  assert!(matches!(err, ShapeError::Eot));
}

// Nesting composes: `braces(brackets(ident))` over `{[x]}` wraps the bracket construct as
// the brace construct's data, each carrying its own delimiter and construct spans.
#[test]
fn nested_delimiters_compose() {
  let out = drive(
    Fatal::<E>::new(),
    |inp| {
      let delimited = braces(brackets(Ident::parse))(inp)?;
      assert_eq!(delimited.span(), SimpleSpan::new(0, 5));
      assert_eq!(delimited.open_ref().span(), &SimpleSpan::new(0, 1));
      assert_eq!(delimited.close_ref().span(), &SimpleSpan::new(4, 5));
      let inner = delimited.data();
      assert_eq!(inner.span(), SimpleSpan::new(1, 4));
      assert_eq!(inner.open_ref().span(), &SimpleSpan::new(1, 2));
      assert_eq!(inner.close_ref().span(), &SimpleSpan::new(3, 4));
      assert_eq!(*inner.data().source_ref(), "x");
      Ok::<_, E>(())
    },
    "{[x]}",
  );
  assert!(out.is_ok());
}

// ── Local discriminating error for the generic error-path tests ──────────────
//
// `ShapeError` mirrors the shared `E`'s absorb-everything `From` family (so it is a
// `ComposableEmitter` error and backs a `FatalContext`), but keeps the two token-level
// families the shapes commit through as distinct variants, so a test can pin WHICH one a
// missing/wrong closer produced. The shared `E` is unit and cannot.
#[derive(Debug)]
enum ShapeError {
  /// An unexpected token, carrying the single expected kind (the delimiter).
  Unexpected(Option<TokenKind>),
  /// End of input where a delimiter was required.
  Eot,
  /// Any other diagnostic family (unexercised by these tests).
  Other,
}

impl From<()> for ShapeError {
  fn from(_: ()) -> Self {
    ShapeError::Other
  }
}

impl<'a, S, Lang: ?Sized> From<UnexpectedToken<'a, Token, TokenKind, S, Lang>> for ShapeError {
  fn from(e: UnexpectedToken<'a, Token, TokenKind, S, Lang>) -> Self {
    let kind = match e.expected() {
      Some(Expected::One(k)) => Some(*k),
      _ => None,
    };
    ShapeError::Unexpected(kind)
  }
}

impl From<UnexpectedEot> for ShapeError {
  fn from(_: UnexpectedEot) -> Self {
    ShapeError::Eot
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for ShapeError {
  fn from(_: FullContainer<S, Lang>) -> Self {
    ShapeError::Other
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for ShapeError {
  fn from(_: TooFew<S, Lang>) -> Self {
    ShapeError::Other
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for ShapeError {
  fn from(_: TooMany<S, Lang>) -> Self {
    ShapeError::Other
  }
}

impl<'a, Kind: Clone, O, Lang: ?Sized> From<MissingToken<'a, Kind, O, Lang>> for ShapeError {
  fn from(_: MissingToken<'a, Kind, O, Lang>) -> Self {
    ShapeError::Other
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, Kind, S, Lang>>
  for ShapeError
{
  fn from(_: SeparatedError<'a, T, Kind, S, Lang>) -> Self {
    ShapeError::Other
  }
}

impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for ShapeError {
  fn from(_: MissingSyntax<O, Lang>) -> Self {
    ShapeError::Other
  }
}
