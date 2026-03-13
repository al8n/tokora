#![cfg(all(feature = "std", feature = "logos"))]
#![allow(warnings)]

//! Coverage tests for:
//! - `parser/many/handler/mod.rs` lines 47-49, 57-59, 75, 228-237, 245-254, 270, 277
//!   — SeparatorHandler and DelimiterHandler impls on `()`, `PhantomData<T>`,
//!     and `GenericArrayDeque` (exercised by running parsers that collect into those types)
//! - `emitter/mod.rs` lines 170, 177, 181, 188, 192, 196, 200, 204
//!   — `&mut U` delegation impl for `Emitter`
//! - `emitter/pratt.rs` lines 30, 37, 41, 48
//!   — `&mut U` delegation impl for `PrattEmitter`
//! - `emitter/separated/` files
//!   — `&mut U` delegation impls for separated emitter traits

mod common;

use core::marker::PhantomData;

use tokit::{
  Accumulator, Emitter, InputRef, Lexer, Parse, ParseContext, ParseInput, Parser, ParserContext,
  Token as TokenTrait, TryParseInput,
  emitter::{
    Fatal, FullContainerEmitter, MissingLeadingSeparatorEmitter, MissingTrailingSeparatorEmitter,
    PrattEmitter, SeparatedEmitter, Silent, TooFewEmitter, TooManyEmitter,
    UnexpectedLeadingSeparatorEmitter, UnexpectedTrailingSeparatorEmitter,
  },
  error::{
    UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot,
    syntax::{FullContainer, MissingSyntax, MissingSyntaxOf, TooFew, TooMany},
    token::{MissingToken, MissingTokenOf, UnexpectedToken, UnexpectedTokenOf},
  },
  input::Cursor,
  span::{SimpleSpan, Spanned},
  try_parse_input::ParseAttempt,
  utils::{CowStr, GenericArrayDeque, marker::Ignored, typenum::U2},
};

use common::{TestLexer, Token, TokenKind};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum Err {
  Any,
}

impl From<()> for Err {
  fn from(_: ()) -> Self {
    Err::Any
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for Err {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    Err::Any
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for Err {
  fn from(_: TooFew<S, Lang>) -> Self {
    Err::Any
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for Err {
  fn from(_: TooMany<S, Lang>) -> Self {
    Err::Any
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for Err {
  fn from(_: FullContainer<S, Lang>) -> Self {
    Err::Any
  }
}

impl From<UnexpectedEot> for Err {
  fn from(_: UnexpectedEot) -> Self {
    Err::Any
  }
}

impl<O, Lang: ?Sized> From<UnexpectedEoLhs<O, Lang>> for Err {
  fn from(_: UnexpectedEoLhs<O, Lang>) -> Self {
    Err::Any
  }
}

impl<O, Lang: ?Sized> From<UnexpectedEoRhs<O, Lang>> for Err {
  fn from(_: UnexpectedEoRhs<O, Lang>) -> Self {
    Err::Any
  }
}

impl<'inp> tokit::emitter::FromSeparatedError<'inp, TestLexer<'inp>> for Err {
  fn from_missing_separator(_: CowStr, _: MissingTokenOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err::Any
  }

  fn from_missing_element(_: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err::Any
  }
}

impl<'inp> tokit::emitter::FromUnexpectedLeadingSeparatorError<'inp, TestLexer<'inp>> for Err {
  fn from_unexpected_leading_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err::Any
  }
}

impl<'inp> tokit::emitter::FromUnexpectedTrailingSeparatorError<'inp, TestLexer<'inp>> for Err {
  fn from_unexpected_trailing_separator(
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    Err::Any
  }
}

// ── Minimal tracking emitter ─────────────────────────────────────────────────

/// An emitter that tracks calls made through it, used to verify `&mut U`
/// delegation actually reaches the inner emitter.
struct TrackingEmitter {
  calls: usize,
}

impl TrackingEmitter {
  fn new() -> Self {
    Self { calls: 0 }
  }
}

impl<'inp> Emitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  type Error = Err;

  fn emit_lexer_error(
    &mut self,
    _: Spanned<
      <<TestLexer<'inp> as Lexer<'inp>>::Token as TokenTrait<'inp>>::Error,
      <TestLexer<'inp> as Lexer<'inp>>::Span,
    >,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }

  fn emit_unexpected_token(
    &mut self,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }

  fn emit_error(
    &mut self,
    _: Spanned<Err, <TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }

  fn rewind(&mut self, _: &Cursor<'inp, '_, TestLexer<'inp>>)
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
  }
}

impl<'inp> PrattEmitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  fn emit_unexpected_end_of_lhs(
    &mut self,
    _: UnexpectedEoLhs<<TestLexer<'inp> as Lexer<'inp>>::Offset>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }

  fn emit_unexpected_end_of_rhs(
    &mut self,
    _: UnexpectedEoRhs<<TestLexer<'inp> as Lexer<'inp>>::Offset>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }
}

impl<'inp> SeparatedEmitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  fn emit_missing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }

  fn emit_missing_element(&mut self, _: MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }
}

impl<'inp> UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  fn emit_unexpected_leading_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }
}

impl<'inp> UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  fn emit_unexpected_trailing_separator(
    &mut self,
    _: CowStr,
    _: UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }
}

impl<'inp> MissingLeadingSeparatorEmitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  fn emit_missing_leading_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }
}

impl<'inp> MissingTrailingSeparatorEmitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  fn emit_missing_trailing_separator(
    &mut self,
    _: CowStr,
    _: MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }
}

impl<'inp> FullContainerEmitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  fn emit_full_container(
    &mut self,
    _: FullContainer<<TestLexer<'inp> as Lexer<'inp>>::Span>,
  ) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }
}

impl<'inp> TooFewEmitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  fn emit_too_few(&mut self, _: TooFew<<TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }
}

impl<'inp> TooManyEmitter<'inp, TestLexer<'inp>> for TrackingEmitter {
  fn emit_too_many(&mut self, _: TooMany<<TestLexer<'inp> as Lexer<'inp>>::Span>) -> Result<(), Err>
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    self.calls += 1;
    Ok(())
  }
}

fn tracking_ctx() -> ParserContext<'static, TestLexer<'static>, TrackingEmitter> {
  ParserContext::new(TrackingEmitter::new())
}

fn silent_ctx() -> ParserContext<'static, TestLexer<'static>, Silent<Err>> {
  ParserContext::new(Silent::new())
}

// ── Element parser helper ──────────────────────────────────────────────────────

fn try_num<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, Err>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Err>,
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

// ═══════════════════════════════════════════════════════════════════════════════
// emitter/mod.rs — &mut U delegation for Emitter (lines 170, 177, 181, 188,
//                   192, 196, 200, 204)
// Calling a method on `&mut emitter` explicitly invokes the &mut U forwarding
// impl rather than the concrete impl.
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn emitter_mut_ref_emit_lexer_error() {
  let mut emitter = TrackingEmitter::new();
  // r is &mut TrackingEmitter; calling through &mut r invokes the &mut U impl
  let mut r: &mut TrackingEmitter = &mut emitter;
  let spanned = Spanned::new(SimpleSpan::new(0usize, 1usize), ());
  <&mut TrackingEmitter as Emitter<'_, TestLexer<'_>>>::emit_lexer_error(&mut r, spanned).unwrap();
  assert_eq!(emitter.calls, 1);
}

#[test]
fn emitter_mut_ref_emit_unexpected_token() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let ut = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  <&mut TrackingEmitter as Emitter<'_, TestLexer<'_>>>::emit_unexpected_token(&mut r, ut).unwrap();
  assert_eq!(emitter.calls, 1);
}

#[test]
fn emitter_mut_ref_emit_error() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let spanned = Spanned::new(SimpleSpan::new(0usize, 1usize), Err::Any);
  <&mut TrackingEmitter as Emitter<'_, TestLexer<'_>>>::emit_error(&mut r, spanned).unwrap();
  assert_eq!(emitter.calls, 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// emitter/pratt.rs — &mut U delegation for PrattEmitter (lines 30, 37, 41, 48)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn pratt_emitter_mut_ref_emit_lhs() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let err = UnexpectedEoLhs::eolhs(0usize);
  <&mut TrackingEmitter as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_lhs(
    &mut r, err,
  )
  .unwrap();
  assert_eq!(emitter.calls, 1);
}

#[test]
fn pratt_emitter_mut_ref_emit_rhs() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let err = UnexpectedEoRhs::eorhs(0usize);
  <&mut TrackingEmitter as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_rhs(
    &mut r, err,
  )
  .unwrap();
  assert_eq!(emitter.calls, 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// emitter/separated/mod.rs — &mut U delegation for SeparatedEmitter
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn separated_emitter_mut_ref_missing_separator() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let name = CowStr::from_static("comma");
  let err: MissingToken<'_, TokenKind, usize> = MissingToken::new(0usize);
  <&mut TrackingEmitter as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_separator(
    &mut r, name, err,
  )
  .unwrap();
  assert_eq!(emitter.calls, 1);
}

#[test]
fn separated_emitter_mut_ref_missing_element() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let err = MissingSyntax::new(0usize);
  <&mut TrackingEmitter as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_element(&mut r, err)
    .unwrap();
  assert_eq!(emitter.calls, 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// emitter/separated/unexpected_leading.rs — &mut U delegation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn unexpected_leading_separator_emitter_mut_ref() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  <&mut TrackingEmitter as UnexpectedLeadingSeparatorEmitter<
    '_,
    TestLexer<'_>,
  >>::emit_unexpected_leading_separator(&mut r, name, err)
  .unwrap();
  assert_eq!(emitter.calls, 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// emitter/separated/unexpected_trailing.rs — &mut U delegation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn unexpected_trailing_separator_emitter_mut_ref() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  <&mut TrackingEmitter as UnexpectedTrailingSeparatorEmitter<
    '_,
    TestLexer<'_>,
  >>::emit_unexpected_trailing_separator(&mut r, name, err)
  .unwrap();
  assert_eq!(emitter.calls, 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// emitter/separated/missing_leading.rs — &mut U delegation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn missing_leading_separator_emitter_mut_ref() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let name = CowStr::from_static("comma");
  let err: MissingToken<'_, TokenKind, usize> = MissingToken::new(0usize);
  <&mut TrackingEmitter as MissingLeadingSeparatorEmitter<
    '_,
    TestLexer<'_>,
  >>::emit_missing_leading_separator(&mut r, name, err)
  .unwrap();
  assert_eq!(emitter.calls, 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// emitter/separated/missing_trailing.rs — &mut U delegation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn missing_trailing_separator_emitter_mut_ref() {
  let mut emitter = TrackingEmitter::new();
  let mut r: &mut TrackingEmitter = &mut emitter;
  let name = CowStr::from_static("comma");
  let err: MissingToken<'_, TokenKind, usize> = MissingToken::new(0usize);
  <&mut TrackingEmitter as MissingTrailingSeparatorEmitter<
    '_,
    TestLexer<'_>,
  >>::emit_missing_trailing_separator(&mut r, name, err)
  .unwrap();
  assert_eq!(emitter.calls, 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// parser/many/handler/mod.rs — SeparatorHandler impl for ()
// (lines 47-49)
//
// To exercise on_separator for (), we run a separated parser that collects
// into () — the library calls container.on_separator() during parsing.
// ═══════════════════════════════════════════════════════════════════════════════

fn sep_into_unit<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<(), Err>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Err>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num.separated_by_comma().collect().parse_input(inp)
}

#[test]
fn separator_handler_unit_via_parser() {
  // Parsing "1,2,3" collecting into () triggers on_separator on ()
  let r: Result<(), _> = Parser::with_context(tracking_ctx())
    .apply(sep_into_unit)
    .parse_str("1,2,3");
  assert!(r.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// parser/many/handler/mod.rs — SeparatorHandler impl for PhantomData<T>
// (lines 57-59)
//
// Collect into PhantomData<i64> — triggers on_separator for PhantomData.
// ═══════════════════════════════════════════════════════════════════════════════

fn sep_into_phantom<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<PhantomData<i64>, Err>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Err>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num.separated_by_comma().collect().parse_input(inp)
}

#[test]
fn separator_handler_phantom_data_via_parser() {
  let r: Result<PhantomData<i64>, _> = Parser::with_context(tracking_ctx())
    .apply(sep_into_phantom)
    .parse_str("1,2,3");
  assert!(r.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// parser/many/handler/mod.rs — SeparatorHandler impl for GenericArrayDeque
// (line 75)
//
// Collect into GenericArrayDeque<i64, U2>.
// ═══════════════════════════════════════════════════════════════════════════════

fn sep_into_gad<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<GenericArrayDeque<i64, U2>, Err>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Err>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num.separated_by_comma().collect().parse_input(inp)
}

#[test]
fn separator_handler_gad_via_parser() {
  // Parse 2 elements (capacity of U2 GAD)
  let r: Result<GenericArrayDeque<i64, U2>, _> = Parser::with_context(tracking_ctx())
    .apply(sep_into_gad)
    .parse_str("1,2");
  assert!(r.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// parser/many/handler/mod.rs — SeparatorHandler impl for Ignored<T>
// (line 68 via @generic macro)
//
// Collect into Ignored<i64> — triggers on_separator for Ignored.
// ═══════════════════════════════════════════════════════════════════════════════

fn sep_into_ignored<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Ignored<i64>, Err>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Err>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  try_num.separated_by_comma().collect().parse_input(inp)
}

#[test]
fn separator_handler_ignored_via_parser() {
  let r: Result<Ignored<i64>, _> = Parser::with_context(tracking_ctx())
    .apply(sep_into_ignored)
    .parse_str("1,2,3");
  assert!(r.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// parser/many/handler/mod.rs — DelimiterHandler impls on (), PhantomData, GAD
// (lines 228-237, 245-254, 270, 277)
//
// Delimited parsers call on_open_delimiter and on_close_delimiter on the
// container type. Using bracket-delimited parsers with those container types
// exercises those impls.
// ═══════════════════════════════════════════════════════════════════════════════

fn delim_into_unit<'inp, Ctx>(inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>) -> Result<(), Err>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Err>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  use tokit::{parser::With, punct::Bracket};
  try_num
    .separated_by_comma()
    .delimited::<Bracket>()
    .collect()
    .parse_input(inp)
}

#[test]
fn delimiter_handler_unit_via_parser() {
  // "[1,2,3]" exercises on_open_delimiter and on_close_delimiter for ()
  let r: Result<(), _> = Parser::with_context(tracking_ctx())
    .apply(delim_into_unit)
    .parse_str("[1,2,3]");
  assert!(r.is_ok());
}

fn delim_into_phantom<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<PhantomData<i64>, Err>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Err>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  use tokit::{parser::With, punct::Bracket};
  try_num
    .separated_by_comma()
    .delimited::<Bracket>()
    .collect()
    .parse_input(inp)
}

#[test]
fn delimiter_handler_phantom_data_via_parser() {
  let r: Result<PhantomData<i64>, _> = Parser::with_context(tracking_ctx())
    .apply(delim_into_phantom)
    .parse_str("[1,2]");
  assert!(r.is_ok());
}

fn delim_into_gad<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<GenericArrayDeque<i64, U2>, Err>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Err>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  use tokit::{parser::With, punct::Bracket};
  try_num
    .separated_by_comma()
    .delimited::<Bracket>()
    .collect()
    .parse_input(inp)
}

#[test]
fn delimiter_handler_gad_via_parser() {
  let r: Result<GenericArrayDeque<i64, U2>, _> = Parser::with_context(tracking_ctx())
    .apply(delim_into_gad)
    .parse_str("[1,2]");
  assert!(r.is_ok());
}

fn delim_into_ignored<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, TestLexer<'inp>, Ctx>,
) -> Result<Ignored<i64>, Err>
where
  Ctx: ParseContext<'inp, TestLexer<'inp>>,
  Ctx::Emitter: Emitter<'inp, TestLexer<'inp>, Error = Err>
    + SeparatedEmitter<'inp, TestLexer<'inp>>
    + FullContainerEmitter<'inp, TestLexer<'inp>>
    + UnexpectedLeadingSeparatorEmitter<'inp, TestLexer<'inp>>
    + UnexpectedTrailingSeparatorEmitter<'inp, TestLexer<'inp>>
    + TooFewEmitter<'inp, TestLexer<'inp>>
    + TooManyEmitter<'inp, TestLexer<'inp>>,
{
  use tokit::{parser::With, punct::Bracket};
  try_num
    .separated_by_comma()
    .delimited::<Bracket>()
    .collect()
    .parse_input(inp)
}

#[test]
fn delimiter_handler_ignored_via_parser() {
  let r: Result<Ignored<i64>, _> = Parser::with_context(tracking_ctx())
    .apply(delim_into_ignored)
    .parse_str("[1,2,3]");
  assert!(r.is_ok());
}
