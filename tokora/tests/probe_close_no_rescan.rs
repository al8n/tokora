#![cfg(all(feature = "std", feature = "combinators", feature = "logos_0_16"))]
#![allow(clippy::type_complexity)]

//! Regression: the delimited many-builders commit the closing delimiter **without
//! re-lexing it**, so a scan-counting lexer observes the closer lexed the minimum
//! number of times — including under the blackhole `()` cache
//! (`ParserContext<_, _, ()>`).
//!
//! A probe that classifies the closer by scanning it and then pushes the scanned token back to
//! the cache for a follow-up `try_expect` to commit hands it, under `()`, to the parked slot (the
//! cache refuses it), and since 0.9.0 (#116) the follow-up read takes it from there without
//! lexing. Before that, a put-back under `()` was dropped and the follow-up `try_expect`
//! **re-scanned** it. `InputRef::probe_close` carries the classified closer out of the probe and
//! commits it by value (`commit_probed`), so the closer is lexed once by the probe and never
//! again.
//!
//! These tests pin the closer-lex count through the **public builder API**, so they compile against
//! either shape: the count is the minimum under both the put-back and the by-value commit. A
//! shared `Rc<Cell<usize>>` in the lexer state counts every time `)` is lexed (surviving the
//! input's lexer rebuilds).
//!
//! Coverage per delimited family:
//! - `repeated` / `repeated_while`: a simple valid list (`(a)` / `[a]`) reaches the
//!   closer through `probe_close`, so the no-cache count drops by one with the fix.
//! - `separated`: a simple `(a)` commits the closer via the in-loop `try_expect_map`,
//!   NOT the epilogue `probe_close`, so the closer must be reached through the epilogue
//!   with an element parser that CONSUMES-then-DECLINES, leaving the closer at the
//!   cursor when the element loop breaks.
//! - `separated_while`: its in-loop `try_expect` catches the closer whenever it is at
//!   the cursor, so the `probe_close` Close arm is unreachable via the driver; its
//!   `commit_probed` mechanism is covered at the primitive level in the crate's
//!   `input_ref` unit tests (`probe_close_carries_the_closer_out_and_commit_probed_advances`).
//!
//! The close MISS has two cells at the end. A probe that misses hands back no token, so a
//! delimited `repeated_while` list clones none for an element, and under `()` a reported miss is
//! read from the parked slot and spends no item of the token budget twice.

use core::cell::Cell;
use std::rc::Rc;
use std::vec::Vec;
use tokora::EmitterView;

use hybrid_arraydeque::ArrayDeque;
use hybrid_arraydeque::typenum::{U1, U8};
use tokora::{
  Accumulator, Emitter, InputRef, Parse, ParseAttempt, ParseContext, ParseInput, Parser,
  ParserContext, Token as TokenTrait, TryParseInput,
  cache::{CachedTokenOf, Peeked},
  emitter::{
    Fatal, FullContainerEmitter, MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter,
    SeparatedEmitter, TooFewEmitter, TooManyEmitter, UnclosedEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter, Verbose,
  },
  error::{
    FullContainer, MissingSyntax, MissingToken, SeparatedError, TooFew, TooMany, Unclosed,
    UnexpectedEot, UnexpectedToken,
  },
  input::TokenBudget,
  lexer::LogosLexer,
  logos::{self, Logos},
  parse::Action,
  punct::{CloseParen, Comma, OpenParen, Paren},
  state::State,
  token::PunctuatorToken,
};

// ── Scan-counting lexer state ──────────────────────────────────────────────────
//
// The tally is shared through an `Rc<Cell<_>>` so it survives every lexer the input
// rebuilds — exactly the `tests/partial.rs` `LimitTracker` pattern, but the bump is on
// the CLOSER `)`, not on words. `check()` never trips: the tally itself is the assertion.

#[derive(Debug, Clone)]
struct CloserScans {
  count: Rc<Cell<usize>>,
}

impl Default for CloserScans {
  fn default() -> Self {
    Self {
      count: Rc::new(Cell::new(0)),
    }
  }
}

impl CloserScans {
  /// A shared handle on the closer-scan counter, kept after the state moves into the input.
  fn handle(&self) -> Rc<Cell<usize>> {
    self.count.clone()
  }

  fn bump(&self) {
    self.count.set(self.count.get() + 1);
  }
}

impl State for CloserScans {
  type Error = ();

  fn check(&self) -> Result<(), Self::Error> {
    Ok(())
  }
}

// ── Token ──────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Logos)]
#[logos(crate = logos, extras = CloserScans, skip r"[ \t\r\n]+")]
enum PcTok {
  #[token("(")]
  Open,
  // Every lex of the closer bumps the shared tally — the second lex the fix removes is
  // exactly one extra bump here.
  #[token(")", |lex| lex.extras.bump())]
  Close,
  #[token(",")]
  Comma,
  #[regex(r"[a-z]+")]
  Word,
}

thread_local! {
  /// One increment per `PcTok` clone. Thread-local, so a cell on another test thread cannot move
  /// it.
  static TOKEN_CLONES: Cell<usize> = const { Cell::new(0) };
}

// `Clone` by hand, so that it counts: the close-miss cell at the end reads the tally.
impl Clone for PcTok {
  fn clone(&self) -> Self {
    TOKEN_CLONES.with(|c| c.set(c.get() + 1));
    match self {
      PcTok::Open => PcTok::Open,
      PcTok::Close => PcTok::Close,
      PcTok::Comma => PcTok::Comma,
      PcTok::Word => PcTok::Word,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PcKind {
  Open,
  Close,
  Comma,
  Word,
}

impl core::fmt::Display for PcKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      PcKind::Open => write!(f, "("),
      PcKind::Close => write!(f, ")"),
      PcKind::Comma => write!(f, ","),
      PcKind::Word => write!(f, "word"),
    }
  }
}

impl core::fmt::Display for PcTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.kind().fmt(f)
  }
}

impl TokenTrait<'_> for PcTok {
  type Kind = PcKind;
  type Error = ();

  const SCAN_LOOKAHEAD: tokora::ScanLookahead = tokora::ScanLookahead::Unbounded;

  fn kind(&self) -> PcKind {
    match self {
      PcTok::Open => PcKind::Open,
      PcTok::Close => PcKind::Close,
      PcTok::Comma => PcKind::Comma,
      PcTok::Word => PcKind::Word,
    }
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

impl PunctuatorToken<'_> for PcTok {
  fn open_paren() -> Option<Self::Kind> {
    Some(PcKind::Open)
  }
  fn close_paren() -> Option<Self::Kind> {
    Some(PcKind::Close)
  }
  fn comma() -> Option<Self::Kind> {
    Some(PcKind::Comma)
  }
}

impl From<OpenParen<(), (), ()>> for PcKind {
  fn from(_: OpenParen<(), (), ()>) -> Self {
    PcKind::Open
  }
}
impl From<CloseParen<(), (), ()>> for PcKind {
  fn from(_: CloseParen<(), (), ()>) -> Self {
    PcKind::Close
  }
}
impl From<Comma<(), (), ()>> for PcKind {
  fn from(_: Comma<(), (), ()>) -> Self {
    PcKind::Comma
  }
}

type PcLex<'a> = LogosLexer<'a, PcTok>;

// ── Error type (the union of From arms the delimited drivers require) ────────────

#[derive(Debug, Clone, PartialEq)]
struct PcErr;

impl From<()> for PcErr {
  fn from(_: ()) -> Self {
    PcErr
  }
}
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for PcErr {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    PcErr
  }
}
impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for PcErr {
  fn from(_: FullContainer<S, Lang>) -> Self {
    PcErr
  }
}
impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for PcErr {
  fn from(_: TooFew<S, Lang>) -> Self {
    PcErr
  }
}
impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for PcErr {
  fn from(_: TooMany<S, Lang>) -> Self {
    PcErr
  }
}
impl From<UnexpectedEot> for PcErr {
  fn from(_: UnexpectedEot) -> Self {
    PcErr
  }
}
impl<'a, Kind: Clone, O, Lang: ?Sized> From<MissingToken<'a, Kind, O, Lang>> for PcErr {
  fn from(_: MissingToken<'a, Kind, O, Lang>) -> Self {
    PcErr
  }
}
impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<SeparatedError<'a, T, Kind, S, Lang>> for PcErr {
  fn from(_: SeparatedError<'a, T, Kind, S, Lang>) -> Self {
    PcErr
  }
}
impl<O, Lang: ?Sized> From<MissingSyntax<O, Lang>> for PcErr {
  fn from(_: MissingSyntax<O, Lang>) -> Self {
    PcErr
  }
}
impl<Delimiter, S, Lang: ?Sized> From<Unclosed<Delimiter, S, Lang>> for PcErr {
  fn from(_: Unclosed<Delimiter, S, Lang>) -> Self {
    PcErr
  }
}

impl<'inp, L, Lang: ?Sized> tokora::emitter::FromUnclosed<'inp, L, Lang> for PcErr
where
  L: tokora::Lexer<'inp>,
{
  fn from_unclosed<Delimiter>(_: Unclosed<Delimiter, L::Span, Lang>) -> Self {
    PcErr
  }
}

// ── Contexts: blackhole `()` cache vs `DefaultCache` ─────────────────────────────

fn no_cache_ctx() -> ParserContext<'static, PcLex<'static>, Fatal<PcErr>, ()> {
  ParserContext::new(Fatal::new())
}

fn default_cache_ctx() -> ParserContext<'static, PcLex<'static>, Fatal<PcErr>> {
  ParserContext::new(Fatal::new())
}

// The two ends of the capacity range the law claims and the suite did not cover. The
// #75 regression this file exists for was capacity-dependent, so `()` and the default (U3)
// are the wrong two points to stop at: capacity 1 is the smallest cache that retains
// anything, and U8 is above the default.

/// The capacity-1 `Option` cache.
fn one_slot_ctx() -> ParserContext<
  'static,
  PcLex<'static>,
  Fatal<PcErr>,
  Option<CachedTokenOf<'static, PcLex<'static>>>,
> {
  ParserContext::new(Fatal::new())
}

/// A ring above the default capacity.
fn wide_cache_ctx() -> ParserContext<
  'static,
  PcLex<'static>,
  Fatal<PcErr>,
  ArrayDeque<CachedTokenOf<'static, PcLex<'static>>, U8>,
> {
  ParserContext::new(Fatal::new())
}

// ── Element parsers ──────────────────────────────────────────────────────────────

/// `repeated`: accept a word, else decline (the decline scans — and, at the closer,
/// parks under `()` — the token where the probe then classifies the closer).
fn try_word<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, PcLex<'inp>, Ctx>,
) -> Result<ParseAttempt<()>, PcErr>
where
  Ctx: ParseContext<'inp, PcLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PcLex<'inp>, Error = PcErr>,
{
  inp
    .try_expect(|t| matches!(t.data(), PcTok::Word))
    .map(|opt| match opt {
      None => ParseAttempt::Decline,
      Some(_) => ParseAttempt::Accept(()),
    })
}

/// `repeated_while` element: consume a word or fail.
fn parse_word<'inp, Ctx>(inp: &mut InputRef<'inp, '_, PcLex<'inp>, Ctx>) -> Result<(), PcErr>
where
  Ctx: ParseContext<'inp, PcLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PcLex<'inp>, Error = PcErr>,
{
  match inp.next()? {
    Some(t) if matches!(t.data(), PcTok::Word) => Ok(()),
    _ => Err(PcErr),
  }
}

/// `repeated_while` condition: continue on a word, stop otherwise (the `Close` short-
/// circuits before the condition is consulted).
fn decide_word<'inp, Ctx>(
  mut peeked: Peeked<'_, 'inp, PcLex<'inp>, U1>,
  _: EmitterView<'_, 'inp, PcLex<'inp>, Ctx::Emitter>,
) -> Result<Action, <Ctx::Emitter as Emitter<'inp, PcLex<'inp>>>::Error>
where
  Ctx: ParseContext<'inp, PcLex<'inp>>,
{
  Ok(match peeked.pop_front() {
    None => Action::Stop,
    Some(tok) => {
      let tok = tok
        .as_maybe_ref()
        .map(|t| t.token().copied(), |t| t.token())
        .into_inner();
      if matches!(**tok.data(), PcTok::Word) {
        Action::Continue
      } else {
        Action::Stop
      }
    }
  })
}

/// `separated`: CONSUME one word then DECLINE, so the element loop breaks with the
/// closer at the cursor — reaching the epilogue `probe_close` Close arm that a plain
/// `(a)` never hits (there the in-loop `try_expect_map` commits the closer).
fn consume_then_decline<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, PcLex<'inp>, Ctx>,
) -> Result<ParseAttempt<()>, PcErr>
where
  Ctx: ParseContext<'inp, PcLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PcLex<'inp>, Error = PcErr>,
{
  let _ = inp.try_expect(|t| matches!(t.data(), PcTok::Word))?;
  Ok(ParseAttempt::Decline)
}

// ── Parsers (generic over the context, so one fn runs under both caches) ─────────

fn parse_repeated<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, PcLex<'inp>, Ctx>,
) -> Result<Vec<()>, PcErr>
where
  Ctx: ParseContext<'inp, PcLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PcLex<'inp>, Error = PcErr>
    + FullContainerEmitter<'inp, PcLex<'inp>>
    + UnclosedEmitter<'inp, PcLex<'inp>>,
{
  try_word
    .repeated()
    .delimited::<Paren<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

fn parse_repeated_while<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, PcLex<'inp>, Ctx>,
) -> Result<Vec<()>, PcErr>
where
  Ctx: ParseContext<'inp, PcLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PcLex<'inp>, Error = PcErr>
    + FullContainerEmitter<'inp, PcLex<'inp>>
    + UnclosedEmitter<'inp, PcLex<'inp>>,
{
  parse_word
    .repeated_while::<_, U1>(decide_word::<Ctx>)
    .delimited::<Paren<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

fn parse_separated<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, PcLex<'inp>, Ctx>,
) -> Result<Vec<()>, PcErr>
where
  Ctx: ParseContext<'inp, PcLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PcLex<'inp>, Error = PcErr>
    + FullContainerEmitter<'inp, PcLex<'inp>>
    + UnclosedEmitter<'inp, PcLex<'inp>>
    + SeparatedEmitter<'inp, PcLex<'inp>>
    + TooFewEmitter<'inp, PcLex<'inp>>
    + TooManyEmitter<'inp, PcLex<'inp>>
    + MissingLeadingSeparatorEmitter<'inp, PcLex<'inp>>
    + MissingTrailingSeparatorEmitter<'inp, PcLex<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, PcLex<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, PcLex<'inp>>,
{
  consume_then_decline
    .separated_by_comma()
    .delimited::<Paren<(), (), ()>>()
    .collect()
    .parse_input(inp)
}

// ── repeated: no-cache + DefaultCache twin ───────────────────────────────────────

#[test]
fn repeated_no_cache_lexes_the_closer_once() {
  // `(a)`: the element parser declines on `)`. That decline leaves the closer at the front
  // of the stream — the parked slot, since `()` retains nothing — so `probe_close` classifies
  // the token that is already there and `commit_probed` commits it with no re-lex. One lex,
  // matching the DefaultCache twin below exactly.
  let state = CloserScans::default();
  let counter = state.handle();
  let r: Result<Vec<()>, PcErr> = Parser::with_context(no_cache_ctx())
    .apply(parse_repeated)
    .parse_str_with_state("(a)", state);
  assert!(
    r.is_ok(),
    "a valid `(a)` parses under the blackhole `()` cache"
  );
  assert_eq!(
    counter.get(),
    1,
    "no-cache: the closer is lexed exactly once — the decline retains it at the front \
     instead of dropping it for the probe to re-lex"
  );
}

#[test]
fn repeated_default_cache_lexes_the_closer_once() {
  // The capacity-independence twin: with a real cache the element's declining scan of
  // `)` is cached, so the probe and commit reuse it — one lex, with or without the rescan.
  let state = CloserScans::default();
  let counter = state.handle();
  let r: Result<Vec<()>, PcErr> = Parser::with_context(default_cache_ctx())
    .apply(parse_repeated)
    .parse_str_with_state("(a)", state);
  assert!(r.is_ok(), "a valid `(a)` parses under DefaultCache");
  assert_eq!(
    counter.get(),
    1,
    "DefaultCache: the closer is lexed exactly once"
  );
}

// ── repeated_while: no-cache RED→GREEN + DefaultCache twin ───────────────────────

#[test]
fn repeated_while_no_cache_lexes_the_closer_once() {
  // `repeated_while` probes the close position FIRST each iteration, so the element
  // parser never scans `)`. Under `()` the probe lexes `)` once and `commit_probed`
  // commits it with no re-lex: the closer is lexed exactly once. The assertion's "pre-fix this
  // reads 2" records the count before 0.9.0 (#116), when a put-back under `()` was dropped.
  let state = CloserScans::default();
  let counter = state.handle();
  let r: Result<Vec<()>, PcErr> = Parser::with_context(no_cache_ctx())
    .apply(parse_repeated_while)
    .parse_str_with_state("(a)", state);
  assert!(
    r.is_ok(),
    "a valid `(a)` parses under the blackhole `()` cache"
  );
  assert_eq!(
    counter.get(),
    1,
    "no-cache: the closer is lexed exactly once — pre-fix this reads 2"
  );
}

#[test]
fn repeated_while_default_cache_lexes_the_closer_once() {
  let state = CloserScans::default();
  let counter = state.handle();
  let r: Result<Vec<()>, PcErr> = Parser::with_context(default_cache_ctx())
    .apply(parse_repeated_while)
    .parse_str_with_state("(a)", state);
  assert!(r.is_ok(), "a valid `(a)` parses under DefaultCache");
  assert_eq!(
    counter.get(),
    1,
    "DefaultCache: the closer is lexed exactly once"
  );
}

// ── separated: no-cache RED→GREEN + DefaultCache twin (epilogue Close arm) ────────

#[test]
fn separated_no_cache_lexes_the_closer_once_via_the_epilogue() {
  // The `consume_then_decline` element eats `a` and declines, so the element loop breaks
  // with `)` at the cursor — reaching the epilogue `probe_close` Close arm (Shape B).
  // Under `()` the probe lexes `)` once and `commit_probed` commits it after the
  // end-state pass with no re-lex. The assertion's "pre-fix reads 2" records the count when a
  // put-back under `()` was dropped, before 0.9.0 (#116).
  let state = CloserScans::default();
  let counter = state.handle();
  let r: Result<Vec<()>, PcErr> = Parser::with_context(no_cache_ctx())
    .apply(parse_separated)
    .parse_str_with_state("(a)", state);
  assert!(
    r.is_ok(),
    "a valid `(a)` parses under the blackhole `()` cache"
  );
  assert_eq!(
    counter.get(),
    1,
    "no-cache: the epilogue commits the probed closer by value — lexed once, pre-fix reads 2"
  );
}

#[test]
fn separated_one_slot_cache_lexes_the_closer_once_via_the_epilogue() {
  // The capacity-1 cell at the driver level: the same Shape-B epilogue site, reached with
  // the smallest cache that retains anything.
  let state = CloserScans::default();
  let counter = state.handle();
  let r: Result<Vec<()>, PcErr> = Parser::with_context(one_slot_ctx())
    .apply(parse_separated)
    .parse_str_with_state("(a)", state);
  assert!(r.is_ok(), "a valid `(a)` parses under the capacity-1 cache");
  assert_eq!(
    counter.get(),
    1,
    "capacity-1: the epilogue commits the probed closer by value — lexed once"
  );
}

#[test]
fn separated_wide_cache_lexes_the_closer_once_via_the_epilogue() {
  // The above-the-default cell: capacity independence in the other direction.
  let state = CloserScans::default();
  let counter = state.handle();
  let r: Result<Vec<()>, PcErr> = Parser::with_context(wide_cache_ctx())
    .apply(parse_separated)
    .parse_str_with_state("(a)", state);
  assert!(r.is_ok(), "a valid `(a)` parses under the capacity-8 ring");
  assert_eq!(
    counter.get(),
    1,
    "capacity-8: the epilogue commits the probed closer by value — lexed once"
  );
}

#[test]
fn separated_default_cache_lexes_the_closer_once_via_the_epilogue() {
  let state = CloserScans::default();
  let counter = state.handle();
  let r: Result<Vec<()>, PcErr> = Parser::with_context(default_cache_ctx())
    .apply(parse_separated)
    .parse_str_with_state("(a)", state);
  assert!(r.is_ok(), "a valid `(a)` parses under DefaultCache");
  assert_eq!(
    counter.get(),
    1,
    "DefaultCache: the closer is lexed exactly once"
  );
}

// ── A close miss copies nothing out of the stream (#373) ─────────────────────────

type NoCacheVerboseCtx<'inp> = ParserContext<'inp, PcLex<'inp>, Verbose<PcErr>, ()>;

/// The blackhole `()` cache under a recording emitter, with a budget of `items` lexed items.
fn no_cache_verbose_ctx(items: usize) -> NoCacheVerboseCtx<'static> {
  ParserContext::new(Verbose::new()).with_token_budget(TokenBudget::with_limitation(items))
}

#[test]
fn a_delimited_while_list_clones_no_token_for_an_element() {
  // `repeated_while` probes the close position at the top of every cycle, and before each element
  // the probe misses: a word stands where the closer would. A miss hands back no token, and a list
  // that goes on never asks for one, so an element clones none and eight elements clone exactly
  // what four do. A probe that copied the token it missed cloned one more for each.
  fn clones_over(src: &'static str, elements: usize) -> usize {
    TOKEN_CLONES.with(|c| c.set(0));
    let r: Result<Vec<()>, PcErr> = Parser::with_context(default_cache_ctx())
      .apply(parse_repeated_while)
      .parse_str_with_state(src, CloserScans::default());
    assert_eq!(r.map(|items| items.len()), Ok(elements), "{src:?} parses");
    TOKEN_CLONES.with(Cell::get)
  }
  assert_eq!(
    clones_over("(a a a a a a a a)", 8),
    clones_over("(a a a a)", 4),
    "four more elements cloned more tokens: the close probe copies the token it misses on"
  );
}

#[test]
fn a_close_miss_under_no_cache_spends_no_item_twice() {
  // `(a a ,` is four items, and the budget is four. Under `()` the miss on `,` is read from the
  // parked slot, where the probe left it, and reported once. A fifth lex would be refused, and the
  // tally would say so.
  fn probe<'inp>(
    inp: &mut InputRef<'inp, '_, PcLex<'inp>, NoCacheVerboseCtx<'inp>>,
  ) -> Result<(usize, usize, bool, usize), PcErr> {
    let items = parse_repeated_while(inp)?.len();
    let tally = inp.token_budget();
    let (spent, refused) = (tally.spent(), tally.refused_an_item());
    let reports = inp.emitter_ref().errors().values().flatten().count();
    Ok((items, spent, refused, reports))
  }
  assert_eq!(
    Parser::with_context(no_cache_verbose_ctx(4))
      .apply(probe)
      .parse_str_with_state("(a a ,", CloserScans::default()),
    Ok((2, 4, false, 1)),
    "the two words collect, the `,` miss is reported once, and four items are spent with none \
     refused"
  );
}
