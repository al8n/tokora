//! The grammar every Pratt row is built on: integers, `+`, `^` and `;`, over a lexer whose state is
//! 64 bytes, and the one operator table the three drivers read.
//!
//! It is `tokora-benches/benches/pratt_typed.rs`'s grammar, with the same lexer state, the same
//! operand width and the same two operators, so a row here and a cell there read one shape. That
//! file says why the state is 64 bytes: a checkpoint, and a staged token, clone it, and a driver
//! priced against a `()` state would be priced against nothing.
//!
//! **One table.** `PTok`'s [`PrattToken`] implementation for [`PExpr`] is the table. The flat
//! driver and the token-level driver read it directly, and the typed driver reads it through
//! [`PrattTable::lhs`] and [`PrattTable::rhs`], so no row spells an operator of its own.
//!
//! **One source for each family of rows.** [`left_chain`], [`right_chain`] and [`shallow`] each
//! return the bytes and the two accumulators a driver must produce over them, computed from the
//! operands the generator wrote and never by running a parser.

use core::fmt::Write as _;
use std::hint::black_box;

use tokora::{
  Emitter, FatalContext, InputRef, ParseAttempt, ParseContext, Parser, ParserContext,
  ScanLookahead, SimpleSpan, Token,
  emitter::Fatal,
  error::{
    NonAssociativeChain, PrattStackFull, RecursionLimitReached, UnexpectedEnd, UnexpectedToken,
  },
  lexer::LogosLexer,
  logos::{self, Logos},
  pratt::{PrattHandler, PrattInfix, PrattLHS, PrattRHS, PrattTable, Precedenced},
  span::Spanned,
  state::{RecursionLimiter, State},
  token::PrattToken,
};

// ── The lexer state and the token enum ──────────────────────────────────────────────────────

/// What a mode-switching, layout-sensitive lexer holds between tokens: an indent stack, a mode
/// stack and three cursors. Exactly `ExprState` of `pratt_typed.rs`, field for field, so that the
/// two instruments clone the same 64 bytes.
#[derive(Debug, Clone, Default)]
pub struct PState {
  indents: [u16; 16],
  modes: [u8; 29],
  indent_top: u8,
  mode_top: u8,
  /// Written by `note_operand` and read by nothing: it is in the state so that a clone copies it,
  /// as `ExprState`'s is.
  #[allow(dead_code)]
  last_kind: u8,
}

const _: () = assert!(
  size_of::<PState>() == 64,
  "tokora-icount: the Pratt rows' lexer state is no longer one cache line. It is the size of \
   `ExprState` in `pratt_typed.rs`, which says why: change both together or the two instruments \
   price different states."
);

impl State for PState {
  type Error = ();

  /// The per-scan veto, and the read that keeps the state from being optimised down to the two
  /// bytes anything observes. See `ExprState::check` in `pratt_typed.rs` for the argument.
  fn check(&self) -> Result<(), ()> {
    let indent = black_box(self.indents[usize::from(self.indent_top) % self.indents.len()]);
    let mode = black_box(self.modes[usize::from(self.mode_top) % self.modes.len()]);
    if mode == MODE_POISON && indent == u16::MAX {
      Err(())
    } else {
      Ok(())
    }
  }
}

/// The mode an operand is read in: the only mode this grammar has.
const MODE_OPERAND: u8 = 1;

/// A mode that is never pushed, so the veto's failure arm never fires.
const MODE_POISON: u8 = 0xFF;

/// Records the operand in the lexer state the way a stateful lexer does, and returns its value.
fn note_operand(lex: &mut logos::Lexer<'_, PTok>) -> Option<i64> {
  let value = lex.slice().parse::<i64>().ok()?;
  let column = u16::try_from(lex.span().start % 4096).unwrap_or(0);
  let state = &mut lex.extras;

  let indent_slot = usize::from(state.indent_top) % state.indents.len();
  state.indents[indent_slot] = column;
  state.indent_top = state.indent_top.wrapping_add(1);

  let mode_slot = usize::from(state.mode_top) % state.modes.len();
  let previous_mode = state.modes[mode_slot];
  state.modes[mode_slot] = MODE_OPERAND;
  state.mode_top = state.mode_top.wrapping_add(1);
  state.last_kind = MODE_OPERAND.wrapping_add(previous_mode);

  Some(value)
}

#[derive(Debug, Clone, PartialEq, Logos)]
#[logos(crate = logos, extras = PState, skip r"[ \t\r\n]+")]
pub enum PTok {
  #[regex(r"[0-9]+", note_operand)]
  Int(i64),
  /// The right-associative operator.
  #[token("^")]
  Caret,
  /// The left-associative operator.
  #[token("+")]
  Plus,
  /// The expression's terminator.
  #[token(";")]
  Semi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PKind {
  Int,
  Caret,
  Plus,
  Semi,
}

impl core::fmt::Display for PKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(match self {
      PKind::Int => "integer",
      PKind::Caret => "'^'",
      PKind::Plus => "'+'",
      PKind::Semi => "';'",
    })
  }
}

impl core::fmt::Display for PTok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      PTok::Int(n) => write!(f, "{n}"),
      other => core::fmt::Display::fmt(&other.kind(), f),
    }
  }
}

impl Token<'_> for PTok {
  type Kind = PKind;
  type Error = ();

  const SCAN_LOOKAHEAD: ScanLookahead = ScanLookahead::Unbounded;

  fn kind(&self) -> PKind {
    match self {
      PTok::Int(_) => PKind::Int,
      PTok::Caret => PKind::Caret,
      PTok::Plus => PKind::Plus,
      PTok::Semi => PKind::Semi,
    }
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

pub type PLex<'a> = LogosLexer<'a, PTok>;

// ── The table ───────────────────────────────────────────────────────────────────────────────

/// The table's marker.
pub struct PExpr;

/// The table, named once. `+` is left-associative at [`PREC_SUM`] and `^` right-associative at
/// [`PREC_EXP`]: the two rows of `parse_rhs` in `pratt_typed.rs`, power for power.
pub const TABLE: PrattTable<PExpr, i64> = PrattTable::new();

/// The left-associative rung.
const PREC_SUM: i64 = 1;

/// The right-associative rung, above it.
const PREC_EXP: i64 = 2;

impl PrattToken<'_, PExpr, i64> for PTok {
  fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), i64>> {
    match self {
      PTok::Int(_) => Some(PrattLHS::Operand(())),
      PTok::Caret | PTok::Plus | PTok::Semi => None,
    }
  }

  fn try_pratt_rhs(&self) -> Option<PrattRHS<(), (), (), (), i64>> {
    match self {
      PTok::Plus => Some(PrattRHS::Infix(Precedenced::new(
        PrattInfix::Left(()),
        PREC_SUM,
      ))),
      PTok::Caret => Some(PrattRHS::Infix(Precedenced::new(
        PrattInfix::Right(()),
        PREC_EXP,
      ))),
      PTok::Int(_) | PTok::Semi => None,
    }
  }
}

// ── The error absorber ──────────────────────────────────────────────────────────────────────
//
// Every source below is well-formed, so none of these is constructed at runtime. They satisfy the
// bounds the three drivers put on the emitter's error.

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PErr;

impl From<()> for PErr {
  fn from(_: ()) -> Self {
    PErr
  }
}

impl<'a, T, K: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, K, S, Lang>> for PErr {
  fn from(_: UnexpectedToken<'a, T, K, S, Lang>) -> Self {
    PErr
  }
}

/// Covers `UnexpectedEot`, `UnexpectedEoLhs` and `UnexpectedEoRhs`: aliases of `UnexpectedEnd`
/// with different hints.
impl<H, O, Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEnd<H, O, Lang, Set>> for PErr {
  fn from(_: UnexpectedEnd<H, O, Lang, Set>) -> Self {
    PErr
  }
}

impl From<RecursionLimitReached> for PErr {
  fn from(_: RecursionLimitReached) -> Self {
    PErr
  }
}

impl From<NonAssociativeChain> for PErr {
  fn from(_: NonAssociativeChain) -> Self {
    PErr
  }
}

impl From<PrattStackFull> for PErr {
  fn from(_: PrattStackFull) -> Self {
    PErr
  }
}

impl<'inp, L, Lang: ?Sized> tokora::emitter::FromUnclosed<'inp, L, Lang> for PErr
where
  L: tokora::Lexer<'inp>,
{
  fn from_unclosed<D>(_: tokora::error::Unclosed<D, L::Span, Lang>) -> Self {
    PErr
  }
}

// ── The parser ──────────────────────────────────────────────────────────────────────────────

/// The recursion ceiling every Pratt row's parse runs against. The typed and the token-level
/// drivers take one level for each operator of a right-associative chain, and the longest is
/// [`CHAIN`] operators; the flat driver takes none. The default budget is below that, so the
/// ceiling is raised for every row alike and the three drivers meet one limit.
const RECURSION_LIMIT: usize = 256;

const _: () = assert!(
  RECURSION_LIMIT > CHAIN + 1,
  "tokora-icount: the Pratt rows' recursion ceiling is at or below the depth of their deepest \
   right chain, so the typed and the token-level rows would return `RecursionLimitReached`."
);

/// The parser every Pratt row runs under: the fail-fast context, with [`RECURSION_LIMIT`] in place
/// of the default budget.
pub fn pratt_parser<'inp>() -> Parser<(), PLex<'inp>, i64, FatalContext<'inp, PLex<'inp>, PErr>> {
  Parser::with_context(
    ParserContext::of(Fatal::of())
      .with_recursion_limiter(RecursionLimiter::with_limitation(RECURSION_LIMIT)),
  )
}

// ── What every driver is given: the operand parser, the handler and the folds ───────────────

/// A committed token with its span, which is what the flat and the typed drivers hand a fold as
/// an operator.
pub type PTokSpanned = Spanned<PTok, SimpleSpan>;

/// The error `Ctx`'s emitter returns.
type ErrorIn<'inp, Ctx> =
  <<Ctx as ParseContext<'inp, PLex<'inp>>>::Emitter as Emitter<'inp, PLex<'inp>>>::Error;

/// One operand: an integer. It declines anything else, and leaves it on the input.
pub fn try_operand<'inp, Ctx>(
  inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>,
) -> Result<ParseAttempt<i64>, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  Ok(
    match inp.try_expect(|t| matches!(t.data(), PTok::Int(_)))? {
      Some(tok) => match tok.into_data() {
        PTok::Int(n) => ParseAttempt::Accept(n),
        _ => unreachable!("the predicate admits only integers"),
      },
      None => ParseAttempt::Decline,
    },
  )
}

/// The handler every `parse_into` row and the control share: it sums the operands and counts the
/// operators, and keeps nothing else.
#[derive(Debug, Default)]
pub struct Sum(pub i64);

impl<'inp, Ctx> PrattHandler<'inp, i64, PLex<'inp>, i64, Ctx> for Sum
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
{
  #[inline(always)]
  fn on_prefix(
    &mut self,
    _input: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>,
    _operator: Precedenced<PTokSpanned, i64>,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    Ok(())
  }

  #[inline(always)]
  fn on_operand(
    &mut self,
    _input: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>,
    operand: i64,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 = self.0.wrapping_add(operand);
    Ok(())
  }

  #[inline(always)]
  fn on_infix(
    &mut self,
    _input: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>,
    _operator: Precedenced<PrattInfix<PTokSpanned, PTokSpanned, PTokSpanned>, i64>,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    self.0 = self.0.wrapping_add(1);
    Ok(())
  }

  #[inline(always)]
  fn on_postfix(
    &mut self,
    _input: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>,
    _operator: Precedenced<PTokSpanned, i64>,
  ) -> Result<(), ErrorIn<'inp, Ctx>> {
    Ok(())
  }
}

/// A prefix fold. Never reached: the table has no prefix operator.
pub fn fold_prefix<'inp, Ctx>(
  _inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>,
  operand: i64,
  _operator: Precedenced<PTokSpanned, i64>,
) -> Result<i64, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  Ok(operand.wrapping_neg())
}

/// The one fold the sources reach, and it is associativity-blind: `+` and `^` run the same
/// arithmetic, so a left chain and a right chain of one length run the same number of the same
/// folds.
pub fn fold_infix<'inp, Ctx>(
  _inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>,
  left: i64,
  right: i64,
  _operator: Precedenced<PrattInfix<PTokSpanned, PTokSpanned, PTokSpanned>, i64>,
) -> Result<i64, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  Ok(left.wrapping_mul(31).wrapping_add(right))
}

/// A postfix fold. Never reached: the table has no postfix operator.
pub fn fold_postfix<'inp, Ctx>(
  _inp: &mut InputRef<'inp, '_, PLex<'inp>, Ctx>,
  operand: i64,
  _operator: Precedenced<PTokSpanned, i64>,
) -> Result<i64, PErr>
where
  Ctx: ParseContext<'inp, PLex<'inp>>,
  Ctx::Emitter: Emitter<'inp, PLex<'inp>, Error = PErr>,
{
  Ok(operand)
}

// ── The sources ─────────────────────────────────────────────────────────────────────────────

/// Operands in every source: the repetition rows' `ELEMENTS` over two.
const ELEMENTS: usize = 256;

/// Operators in each expression of a chain source. The expression holds `CHAIN + 1` operands, a
/// power of two, so a source holds `ELEMENTS / (CHAIN + 1)` of them and no operand is left over.
pub const CHAIN: usize = 63;

/// What a source is, and what a driver must make of it.
pub struct Source {
  /// The bytes.
  pub text: String,
  /// What the three AST drivers must return: each expression folded with `fold_infix`, in the
  /// order its associativity gives, and the folds summed.
  pub fold: i64,
  /// What `parse_into` must have told a [`Sum`] by the end of the source: its operands' sum and
  /// its operators' count. It is the same for a left chain and a right chain.
  pub collect: i64,
}

#[derive(Debug, Clone, Copy)]
enum Assoc {
  Left,
  Right,
}

/// A four-digit operand, from a counter.
fn operand(i: usize) -> i64 {
  1000 + i64::from((i as u32).wrapping_mul(2654435761) % 9000)
}

/// The generator's own copy of [`fold_infix`]'s arithmetic. It is a second spelling on purpose:
/// the expectation is computed from the operands the generator wrote, with no lexer and no
/// parser, so an edit to the fold alone changes the parse and not the prediction.
fn expected_fold(left: i64, right: i64) -> i64 {
  left.wrapping_mul(31).wrapping_add(right)
}

/// One expression's operands folded the way its associativity says.
fn expected_expression(operands: &[i64], assoc: Assoc) -> i64 {
  const EMPTY: &str = "tokora-icount: an expression was generated with no operands";
  match assoc {
    Assoc::Left => {
      let (first, rest) = operands.split_first().expect(EMPTY);
      rest
        .iter()
        .fold(*first, |left, &right| expected_fold(left, right))
    }
    Assoc::Right => {
      let (last, rest) = operands.split_last().expect(EMPTY);
      rest
        .iter()
        .rfold(*last, |right, &left| expected_fold(left, right))
    }
  }
}

/// `1 OP 2 OP … OP 64 ;` over and over, with exactly [`CHAIN`] operators in each expression.
fn chain(assoc: Assoc) -> Source {
  let op = match assoc {
    Assoc::Left => '+',
    Assoc::Right => '^',
  };
  let mut text = String::with_capacity(ELEMENTS * 8);
  let mut operands: Vec<i64> = Vec::with_capacity(CHAIN + 1);
  let mut fold = 0i64;
  let mut collect = 0i64;
  let mut i = 0usize;
  for _ in 0..ELEMENTS / (CHAIN + 1) {
    operands.clear();
    for k in 0..=CHAIN {
      let value = operand(i);
      i += 1;
      operands.push(value);
      if k == 0 {
        let _ = write!(text, "{value}");
      } else {
        let _ = write!(text, " {op} {value}");
      }
    }
    text.push_str(" ;\n");
    fold = fold.wrapping_add(expected_expression(&operands, assoc));
    collect = collect.wrapping_add(operands.iter().sum::<i64>() + CHAIN as i64);
  }
  Source {
    text,
    fold,
    collect,
  }
}

/// A left-associative chain in each expression.
pub fn left_chain() -> Source {
  chain(Assoc::Left)
}

/// A right-associative chain in each expression.
pub fn right_chain() -> Source {
  chain(Assoc::Right)
}

/// `1 + 2 ;` over and over: many expressions of one operator each, so that a cost paid once for
/// each expression is read where a chain would divide it by its length. One operator makes the two
/// associativities the same expression.
pub fn shallow() -> Source {
  let mut text = String::with_capacity(ELEMENTS * 8);
  let mut fold = 0i64;
  let mut collect = 0i64;
  for e in 0..ELEMENTS / 2 {
    let (a, b) = (operand(2 * e), operand(2 * e + 1));
    let _ = writeln!(text, "{a} + {b} ;");
    fold = fold.wrapping_add(expected_fold(a, b));
    collect = collect.wrapping_add(a + b + 1);
  }
  Source {
    text,
    fold,
    collect,
  }
}
