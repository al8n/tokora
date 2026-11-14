#![cfg(all(feature = "std", feature = "logos_0_16"))]
#![allow(clippy::type_complexity)]

//! Token extents against the lookahead spans they sit beside.
//!
//! `common::TestLexer` skips whitespace, so a lookahead span and a token extent differ wherever a
//! production peeks past its last token or is entered with nothing cached. Each scenario runs both
//! measurements over the same production: the lookahead assertion is the witness that the
//! scenario is built as claimed, and the token-extent assertion is the one under test. Each test
//! names the wrong implementation it refuses.
//!
//! The productions are hand-written, so this suite needs no combinator family.

mod common;

use core::cell::RefCell;
use std::rc::Rc;

use common::{TestLexer, Token};
use tokora::{
  Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext, SimpleSpan,
  Token as TokenT,
  cache::{Cache, DefaultCache},
  error::{Incomplete, MaybeIncomplete, UnexpectedTokenOf},
  input::{Completeness, Cursor, Partial, SurfaceIncomplete},
  parse_partial,
  span::Spanned,
};

// ── errors and emitters ─────────────────────────────────────────────────────

/// What a parse in this suite fails with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fail {
  /// The lexer refused the bytes at this span.
  Lex(SimpleSpan),
  /// A production refused what it found, or found nothing.
  Grammar,
  /// A non-final partial input ran out at its frontier.
  Incomplete,
}

impl From<Incomplete<usize>> for Fail {
  fn from(_: Incomplete<usize>) -> Self {
    Fail::Incomplete
  }
}

impl MaybeIncomplete for Fail {
  fn is_incomplete(&self) -> bool {
    matches!(self, Fail::Incomplete)
  }
}

/// Rejects every diagnostic. A lexer error's rejection carries the lexeme's span, so two errors
/// compare equal only when they are about the same bytes.
struct Strict;

impl<'inp> Emitter<'inp, TestLexer<'inp>> for Strict {
  type Error = Fail;

  fn emit_lexer_error(
    &mut self,
    err: Spanned<
      <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenT<'inp>>::Error,
      <TestLexer<'inp> as Lexer<'inp>>::Span,
    >,
  ) -> Result<(), Fail> {
    Err(Fail::Lex(err.span))
  }

  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), Fail> {
    Err(Fail::Grammar)
  }

  fn emit_error(
    &mut self,
    err: Spanned<Fail, <TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), Fail> {
    Err(err.data)
  }

  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>, _: u64) {}
}

/// Records each lexer error's span and carries on; rejects everything else.
struct Lenient(Rc<RefCell<Vec<SimpleSpan>>>);

impl<'inp> Emitter<'inp, TestLexer<'inp>> for Lenient {
  type Error = Fail;

  fn emit_lexer_error(
    &mut self,
    err: Spanned<
      <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenT<'inp>>::Error,
      <TestLexer<'inp> as Lexer<'inp>>::Span,
    >,
  ) -> Result<(), Fail> {
    self.0.borrow_mut().push(err.span);
    Ok(())
  }

  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), Fail> {
    Err(Fail::Grammar)
  }

  fn emit_error(
    &mut self,
    err: Spanned<Fail, <TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), Fail> {
    Err(err.data)
  }

  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>, _: u64) {}
}

// ── contexts and drivers ────────────────────────────────────────────────────

/// The default cache, which retains up to three lookahead tokens. The other cache this suite
/// runs is `()`, which retains none.
type Retaining<'inp> = DefaultCache<'inp, TestLexer<'inp>>;

type StrictCtx<'inp, C> = ParserContext<'inp, TestLexer<'inp>, Strict, C>;
type LenientCtx<'inp, C> = ParserContext<'inp, TestLexer<'inp>, Lenient, C>;

/// Runs `f` over `src` under [`Strict`] and the cache `C`.
fn run<'inp, C, O>(
  src: &'inp str,
  f: impl FnMut(&mut InputRef<'inp, '_, TestLexer<'inp>, StrictCtx<'inp, C>>) -> Result<O, Fail>,
) -> Result<O, Fail>
where
  C: Cache<'inp, TestLexer<'inp>>,
{
  Parser::with_parser_and_context(f, ParserContext::new(Strict)).parse_str(src)
}

/// Runs `f` over `src` under [`Lenient`] and the cache `C`, and returns the spans of the lexer
/// errors it emitted beside `f`'s result.
fn run_lenient<'inp, C, O>(
  src: &'inp str,
  f: impl FnMut(&mut InputRef<'inp, '_, TestLexer<'inp>, LenientCtx<'inp, C>>) -> Result<O, Fail>,
) -> (Result<O, Fail>, Vec<SimpleSpan>)
where
  C: Cache<'inp, TestLexer<'inp>>,
{
  let log = Rc::new(RefCell::new(Vec::new()));
  let out =
    Parser::with_parser_and_context(f, ParserContext::new(Lenient(Rc::clone(&log)))).parse_str(src);
  let emitted = log.take();
  (out, emitted)
}

/// Runs `f` over `src` as a partial input under [`Strict`], sealed when `is_final`.
fn run_partial<'inp, O>(
  src: &'inp str,
  is_final: bool,
  f: impl FnMut(
    &mut InputRef<'inp, '_, TestLexer<'inp>, StrictCtx<'inp, Retaining<'inp>>, (), Partial>,
  ) -> Result<O, Fail>,
) -> Result<O, Fail> {
  parse_partial(ParserContext::new(Strict), src, (), is_final, f)
}

// ── productions ─────────────────────────────────────────────────────────────

/// One identifier.
fn word<'inp, Ctx, Cmpl>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx, (), Cmpl>,
) -> Result<(), Fail>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Fail>,
  Cmpl: SurfaceIncomplete<'inp, TestLexer<'inp>, Ctx, ()>,
{
  match inp.next()? {
    Some(tok) if tok.data == Token::Ident => Ok(()),
    _ => Err(Fail::Grammar),
  }
}

/// One identifier and an optional `;`. Deciding on the `;` means looking at the token after the
/// identifier, so when there is none this production has peeked past its own last token.
fn word_opt_semi<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<(), Fail>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Fail>,
{
  word(inp)?;
  inp.try_expect(|t| matches!(t.data, Token::Semi))?;
  Ok(())
}

/// Consumes nothing.
fn nothing<'inp, Ctx, Cmpl>(
  _inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx, (), Cmpl>,
) -> Result<(), Fail>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Fail>,
  Cmpl: Completeness,
{
  Ok(())
}

/// Speculates three identifiers ahead, rolls the transaction back, then settles for
/// [`word_opt_semi`]: the abandoned branch reaches further than the accepted one.
fn reach_then_settle<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<(), Fail>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Fail>,
{
  let mut txn = inp.begin();
  word(&mut txn)?;
  word(&mut txn)?;
  word(&mut txn)?;
  txn.rollback();
  word_opt_semi(inp)
}

/// Two identifiers, each measured as a token extent of its own.
fn pair<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<(SimpleSpan, SimpleSpan), Fail>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Fail>,
{
  let first = word.token_spanned().parse_input(inp)?.span;
  let second = word_opt_semi.token_spanned().parse_input(inp)?.span;
  Ok((first, second))
}

// ── a peek past the last token ──────────────────────────────────────────────
//
// `a   b`: `word_opt_semi` consumes `a` (0..1) and peeks `b` (4..5) for a `;` it does not find.

/// Refuses the lookahead rule at the end: a span closed at the lookahead position ends at `b`.
#[test]
fn a_peek_past_the_last_token_is_outside_the_token_span() {
  let lookahead = run::<Retaining<'_>, _>("a   b", |inp| {
    Ok(word_opt_semi.spanned().parse_input(inp)?.span)
  });
  assert_eq!(
    lookahead,
    Ok(SimpleSpan::new(0, 4)),
    "witness: the peek left the lookahead at `b`"
  );

  let extent = run::<Retaining<'_>, _>("a   b", |inp| {
    Ok(word_opt_semi.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(extent, Ok(SimpleSpan::new(0, 1)));
}

/// The same scenario through `token_sliced`: refuses a slice closed at the lookahead position.
#[test]
fn a_peek_past_the_last_token_is_outside_the_token_slice() {
  let lookahead = run::<Retaining<'_>, _>("a   b", |inp| {
    Ok(word_opt_semi.sliced().parse_input(inp)?.slice())
  });
  assert_eq!(
    lookahead,
    Ok("a   "),
    "witness: the lookahead slice runs on to `b`"
  );

  let extent = run::<Retaining<'_>, _>("a   b", |inp| {
    Ok(word_opt_semi.token_sliced().parse_input(inp)?.slice())
  });
  assert_eq!(extent, Ok("a"));
}

/// The same scenario through `token_located`: refuses either half closed at the lookahead
/// position.
#[test]
fn a_peek_past_the_last_token_is_outside_the_token_location() {
  let lookahead = run::<Retaining<'_>, _>("a   b", |inp| {
    let located = word_opt_semi.located().parse_input(inp)?;
    Ok((located.span(), located.slice()))
  });
  assert_eq!(lookahead, Ok((SimpleSpan::new(0, 4), "a   ")), "witness");

  let extent = run::<Retaining<'_>, _>("a   b", |inp| {
    let located = word_opt_semi.token_located().parse_input(inp)?;
    Ok((located.span(), located.slice()))
  });
  assert_eq!(extent, Ok((SimpleSpan::new(0, 1), "a")));
}

// ── a production entered with a cold cache ──────────────────────────────────
//
// `a   b`: `a` is consumed first and nothing is left cached, so the lookahead position is the end
// of `a` (1) when `word` starts on `b` (4..5).

/// Refuses the lookahead rule at the start: a span opened at the lookahead position opens at the
/// end of `a`.
#[test]
fn skipped_bytes_in_front_of_a_cold_entry_are_outside_the_token_span() {
  let lookahead = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    Ok(word.spanned().parse_input(inp)?.span)
  });
  assert_eq!(
    lookahead,
    Ok(SimpleSpan::new(1, 5)),
    "witness: the cache was cold, so the lookahead opened at the end of `a`"
  );

  let extent = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    Ok(word.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(extent, Ok(SimpleSpan::new(4, 5)));
}

/// The same scenario through `token_sliced`: refuses a slice opened at the lookahead position.
#[test]
fn skipped_bytes_in_front_of_a_cold_entry_are_outside_the_token_slice() {
  let lookahead = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    Ok(word.sliced().parse_input(inp)?.slice())
  });
  assert_eq!(lookahead, Ok("   b"), "witness");

  let extent = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    Ok(word.token_sliced().parse_input(inp)?.slice())
  });
  assert_eq!(extent, Ok("b"));
}

/// The same scenario through `token_located`: refuses either half opened at the lookahead
/// position.
#[test]
fn skipped_bytes_in_front_of_a_cold_entry_are_outside_the_token_location() {
  let lookahead = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    let located = word.located().parse_input(inp)?;
    Ok((located.span(), located.slice()))
  });
  assert_eq!(lookahead, Ok((SimpleSpan::new(1, 5), "   b")), "witness");

  let extent = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    let located = word.token_located().parse_input(inp)?;
    Ok((located.span(), located.slice()))
  });
  assert_eq!(extent, Ok((SimpleSpan::new(4, 5), "b")));
}

// ── compact input ───────────────────────────────────────────────────────────

/// `+a+`, with the first `+` consumed: a cold entry and a peek past the last token, as above, but
/// with no skipped byte for the two rules to disagree about. Refuses a token extent that differs
/// from the lookahead span where no byte is skipped.
#[test]
fn compact_input_measures_the_same_under_both_rules() {
  let spans = run::<Retaining<'_>, _>("+a+", |inp| {
    inp.next()?;
    Ok(word_opt_semi.spanned().parse_input(inp)?.span)
  });
  let token_spans = run::<Retaining<'_>, _>("+a+", |inp| {
    inp.next()?;
    Ok(word_opt_semi.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(spans, Ok(SimpleSpan::new(1, 2)));
  assert_eq!(token_spans, spans);

  let slices = run::<Retaining<'_>, _>("+a+", |inp| {
    inp.next()?;
    Ok(word_opt_semi.sliced().parse_input(inp)?.slice())
  });
  let token_slices = run::<Retaining<'_>, _>("+a+", |inp| {
    inp.next()?;
    Ok(word_opt_semi.token_sliced().parse_input(inp)?.slice())
  });
  assert_eq!(slices, Ok("a"));
  assert_eq!(token_slices, slices);

  let locations = run::<Retaining<'_>, _>("+a+", |inp| {
    inp.next()?;
    let located = word_opt_semi.located().parse_input(inp)?;
    Ok((located.span(), located.slice()))
  });
  let token_locations = run::<Retaining<'_>, _>("+a+", |inp| {
    inp.next()?;
    let located = word_opt_semi.token_located().parse_input(inp)?;
    Ok((located.span(), located.slice()))
  });
  assert_eq!(locations, Ok((SimpleSpan::new(1, 2), "a")));
  assert_eq!(token_locations, locations);
}

// ── a parser that consumes nothing ──────────────────────────────────────────

/// After `a` in `a   b`, a parser that consumes nothing is anchored at `b`'s start. Refuses a
/// missing clamp (the committed end, 1, falls before the start, 4, and `SimpleSpan::new` panics on
/// the inverted span) and a start taken from the committed position.
#[test]
fn consuming_nothing_is_the_empty_extent_at_the_next_token() {
  let lookahead = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    Ok(nothing.spanned().parse_input(inp)?.span)
  });
  assert_eq!(
    lookahead,
    Ok(SimpleSpan::new(1, 1)),
    "witness: the lookahead stays at the end of `a`"
  );

  let span = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    Ok(nothing.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(span, Ok(SimpleSpan::new(4, 4)));

  let slice = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    Ok(nothing.token_sliced().parse_input(inp)?.slice())
  });
  assert_eq!(slice, Ok(""));

  let location = run::<Retaining<'_>, _>("a   b", |inp| {
    inp.next()?;
    let located = nothing.token_located().parse_input(inp)?;
    Ok((located.span(), located.slice()))
  });
  assert_eq!(location, Ok((SimpleSpan::new(4, 4), "")));
}

/// At end of input there is no next token, and the anchor is the committed position: the end of
/// `a`. The input ends in whitespace, so this refuses an anchor at the input's length or at the
/// lexer's exhaustion position (both 4).
#[test]
fn consuming_nothing_at_end_of_input_is_empty_at_the_committed_position() {
  let span = run::<Retaining<'_>, _>("a   ", |inp| {
    inp.next()?;
    Ok(nothing.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(span, Ok(SimpleSpan::new(1, 1)));

  let slice = run::<Retaining<'_>, _>("a   ", |inp| {
    inp.next()?;
    Ok(nothing.token_sliced().parse_input(inp)?.slice())
  });
  assert_eq!(slice, Ok(""));
}

// ── nesting and backtracking ────────────────────────────────────────────────

/// ` a  b  c`: an outer token extent around two inner ones. The inner `word`'s start is a peek
/// that hits the token the outer start cached, and `word_opt_semi` peeks `c`. Refuses a start
/// kept on the input rather than in each wrapper: an inner start would overwrite the outer one.
#[test]
fn nested_token_extents_are_each_their_own() {
  let out = run::<Retaining<'_>, _>(" a  b  c", |inp| {
    let outer = pair.token_spanned().parse_input(inp)?;
    Ok((outer.span, outer.data))
  });
  assert_eq!(
    out,
    Ok((
      SimpleSpan::new(1, 5),
      (SimpleSpan::new(1, 2), SimpleSpan::new(4, 5)),
    ))
  );
}

/// `a b c d`: the rolled-back branch commits through `c` (end 5); the accepted one commits `a`
/// (end 1) and peeks `b` (2..3). Refuses an end read as a high-water mark — the furthest committed
/// position (5) or `offset()`, the end of the newest lexed token (3) — as well as the lookahead
/// position (2).
#[test]
fn a_rolled_back_reach_is_outside_the_token_span() {
  let lookahead = run::<Retaining<'_>, _>("a b c d", |inp| {
    Ok(reach_then_settle.spanned().parse_input(inp)?.span)
  });
  assert_eq!(
    lookahead,
    Ok(SimpleSpan::new(0, 2)),
    "witness: the accepted branch peeked `b`"
  );

  let extent = run::<Retaining<'_>, _>("a b c d", |inp| {
    Ok(reach_then_settle.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(extent, Ok(SimpleSpan::new(0, 1)));
}

// ── caches that retain nothing ──────────────────────────────────────────────

/// Under the `()` cache a peeked token is retained nowhere, so `cursor()` read after the start's
/// peek is still the end of `a`. Refuses a start read from `cursor()` after the peek instead of
/// from the peeked token: on a cold entry it would open at 1.
#[test]
fn a_zero_capacity_cache_takes_the_start_from_the_peeked_token() {
  let lookahead = run::<(), _>("a   b", |inp| {
    inp.next()?;
    Ok(word.spanned().parse_input(inp)?.span)
  });
  assert_eq!(
    lookahead,
    Ok(SimpleSpan::new(1, 5)),
    "witness: a cold entry"
  );

  let extent = run::<(), _>("a   b", |inp| {
    inp.next()?;
    Ok(word.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(extent, Ok(SimpleSpan::new(4, 5)));

  // A peek past the last token parks `b` here rather than caching it, and the extent still ends
  // at the committed position.
  let lookahead = run::<(), _>("a   b", |inp| {
    Ok(word_opt_semi.spanned().parse_input(inp)?.span)
  });
  assert_eq!(
    lookahead,
    Ok(SimpleSpan::new(0, 4)),
    "witness: `b` is parked"
  );

  let extent = run::<(), _>("a   b", |inp| {
    Ok(word_opt_semi.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(extent, Ok(SimpleSpan::new(0, 1)));
}

/// Under the `()` cache a declined `try_expect` parks the token it lexed. The start must be the
/// parked token's: refuses a start that reads the cache and misses the parked slot.
#[test]
fn a_parked_head_is_the_start() {
  let extent = run::<(), _>("a   b", |inp| {
    inp.next()?;
    let declined = inp.try_expect(|t| matches!(t.data, Token::Semi))?;
    assert!(declined.is_none(), "`b` is not a `;`, so it is parked");
    Ok(word.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(extent, Ok(SimpleSpan::new(4, 5)));
}

// ── a lexer error on the first token ────────────────────────────────────────
//
// `@ a`: `@` is a lexer error at 0..1, and `a` is at 2..3.

/// Under a fatal emitter the start's peek rejects the lexer error before the wrapped parser reads
/// it, and the error value is the one the wrapped parser raises without the wrapper. Only the
/// value is compared: the peek leaves the committed position before the lexeme, where the
/// consume would have committed past it. Refuses a start that swallows or rewrites the error.
#[test]
fn a_fatal_lexer_error_at_the_start_is_the_wrapped_parsers_error() {
  let bare = run::<Retaining<'_>, _>("@ a", word);
  let wrapped =
    run::<Retaining<'_>, _>("@ a", |inp| Ok(word.token_spanned().parse_input(inp)?.span));
  assert!(matches!(bare, Err(Fail::Lex(_))), "a lexer error: {bare:?}");
  assert_eq!(wrapped.err(), bare.err());
}

/// The outcome change the start's peek is documented to make: a parser that reads nothing never
/// meets the lexer error on its own, and fails on it once wrapped, because the start lexes the
/// next token anyway. Refuses a start that reads nothing until the wrapped parser has run.
#[test]
fn a_fatal_lexer_error_at_the_start_fails_a_parser_that_reads_nothing() {
  let bare = run::<Retaining<'_>, _>("@ a", nothing);
  assert_eq!(
    bare,
    Ok(()),
    "witness: on its own the parser never lexes `@`"
  );

  let wrapped = run::<Retaining<'_>, _>("@ a", |inp| {
    Ok(nothing.token_spanned().parse_input(inp)?.span)
  });
  let reading = run::<Retaining<'_>, _>("@ a", word);
  assert!(
    matches!(wrapped, Err(Fail::Lex(_))),
    "a lexer error: {wrapped:?}"
  );
  assert_eq!(
    wrapped.err(),
    reading.err(),
    "the error a parser that reads `@` meets"
  );
}

/// The non-fatal twin: the diagnostic is emitted once, and the extent starts at the first good
/// token after the error lexeme. Under the `()` cache the consume lexes `@` a second time, so this
/// also refuses a second diagnostic for the same lexeme.
#[test]
fn a_lenient_lexer_error_is_emitted_once_and_the_extent_starts_after_it() {
  let (extent, emitted) =
    run_lenient::<Retaining<'_>, _>("@ a", |inp| Ok(word.token_spanned().parse_input(inp)?.span));
  assert_eq!(extent, Ok(SimpleSpan::new(2, 3)));
  assert_eq!(emitted.len(), 1, "one diagnostic: {emitted:?}");

  let (extent, emitted) =
    run_lenient::<(), _>("@ a", |inp| Ok(word.token_spanned().parse_input(inp)?.span));
  assert_eq!(extent, Ok(SimpleSpan::new(2, 3)));
  assert_eq!(
    emitted.len(),
    1,
    "the re-lex under `()` emits nothing new: {emitted:?}"
  );
}

/// The non-fatal twin of the outcome change: a parser that reads nothing emits no diagnostic on
/// its own, and one once wrapped, anchored at the first good token.
#[test]
fn a_lenient_lexer_error_is_emitted_for_a_parser_that_reads_nothing() {
  let (bare, emitted) = run_lenient::<Retaining<'_>, _>("@ a", nothing);
  assert_eq!(bare, Ok(()));
  assert!(
    emitted.is_empty(),
    "witness: on its own nothing is lexed: {emitted:?}"
  );

  let (extent, emitted) = run_lenient::<Retaining<'_>, _>("@ a", |inp| {
    Ok(nothing.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(extent, Ok(SimpleSpan::new(2, 2)));
  assert_eq!(emitted.len(), 1, "{emitted:?}");
}

// ── partial input ───────────────────────────────────────────────────────────

/// `  a` as a non-final partial input. `TestLexer` claims an unbounded read frontier, so the
/// non-final input withholds `a` and the start's peek yields no token: the start is the committed
/// position, 0, and it raises nothing. Sealed, the same input yields `a`, and the start is 2.
/// Refuses a start that raises `Incomplete` or anchors past the committed position.
#[test]
fn a_withheld_head_starts_at_the_committed_position() {
  let open = run_partial("  a", false, |inp| {
    Ok(nothing.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(open, Ok(SimpleSpan::new(0, 0)));

  let sealed = run_partial("  a", true, |inp| {
    Ok(nothing.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(sealed, Ok(SimpleSpan::new(2, 2)));

  // A parser that consumes meets the frontier exactly as it does unwrapped.
  let bare = run_partial("  a", false, word);
  assert_eq!(bare, Err(Fail::Incomplete), "witness: `a` is withheld");
  let wrapped = run_partial("  a", false, |inp| {
    Ok(word.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(wrapped.err(), bare.err());
}

// ── where the end is not a token's end ──────────────────────────────────────

/// `a  `: `padded` skips at both ends, and a skip that reaches end of input commits the lexer's
/// end, so the extent runs to the end of the input. Pins the documented exception; the control
/// without `padded` ends at `a`.
#[test]
fn a_skip_to_end_of_input_ends_the_extent_at_the_input_end() {
  let control =
    run::<Retaining<'_>, _>("a  ", |inp| Ok(word.token_spanned().parse_input(inp)?.span));
  assert_eq!(control, Ok(SimpleSpan::new(0, 1)));

  let padded = run::<Retaining<'_>, _>("a  ", |inp| {
    Ok(word.padded().token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(padded, Ok(SimpleSpan::new(0, 3)));
}

// ── the imperative pieces ───────────────────────────────────────────────────

/// `a   b   c`, by hand: the start taken before anything is consumed gives the clamped empty
/// extent, then `b` is consumed and `c` peeked. Refuses a missing clamp in `token_span_since` and
/// `token_slice_since`, and an end at the lookahead position, which `span_since` shows.
#[test]
fn the_imperative_pieces_measure_what_the_combinator_measures() {
  let out = run::<Retaining<'_>, _>("a   b   c", |inp| {
    inp.next()?;
    let start = inp.token_cursor()?;
    let empty = (inp.token_span_since(&start), inp.token_slice_since(&start));
    word_opt_semi(inp)?;
    let extent = (inp.token_span_since(&start), inp.token_slice_since(&start));
    let lookahead = inp.span_since(&start);
    Ok((start.into_inner(), empty, extent, lookahead))
  });
  assert_eq!(
    out,
    Ok((
      4,
      (SimpleSpan::new(4, 4), Some("")),
      (SimpleSpan::new(4, 5), Some("b")),
      SimpleSpan::new(4, 8),
    ))
  );

  let spanning = run::<Retaining<'_>, _>("a   b   c", |inp| {
    inp.next()?;
    let (span, ()) = inp.token_spanning(word_opt_semi)?;
    Ok(span)
  });
  let combinator = run::<Retaining<'_>, _>("a   b   c", |inp| {
    inp.next()?;
    Ok(word_opt_semi.token_spanned().parse_input(inp)?.span)
  });
  assert_eq!(spanning, Ok(SimpleSpan::new(4, 5)));
  assert_eq!(combinator, spanning);
}
