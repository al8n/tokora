//! `folding` against itself, and in the tiers.
//!
//! The loop that grows by segments is compiled where the crate cannot allocate, and in the crate's
//! own tests: here it runs in the `std` build too, at segments of one, two and four frames, over a
//! first storage of that many, and every result, fold, position and charge is the loop's over a
//! storage that never refuses. The cells that name `core` alone run in every tier, and those of
//! the tier with no allocator pin the bounds a growing storage meets there.

use hybrid_arraydeque::{
  ArrayDeque,
  typenum::{U4, U16},
};

use crate::{
  InputRef, Lexer, ParseAttempt, ParseInput, ReadFrontier, ScanLookahead, SimpleSpan, Token,
  cache::DefaultCache,
  container::Container,
  emitter::Fatal,
  error::{
    NonAssociativeChain, PrattStackFull, UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot,
    UnexpectedToken,
  },
  input::{Input, InputContext},
  pratt::{PrattFrame, PrattInfix, PrattLHS, PrattRHS, PrattTable, Precedenced, flat},
  span::Spanned,
  token::PrattToken,
};

// ── The token, the lexer and the table ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tok(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LexErr;

impl Token<'_> for Tok {
  type Kind = u8;
  type Error = LexErr;

  const SCAN_LOOKAHEAD: ScanLookahead = ScanLookahead::Unbounded;

  fn kind(&self) -> u8 {
    self.0
  }

  fn is_trivia(&self) -> bool {
    self.0 == b' '
  }
}

/// One token for each byte. With `SKIP` a space is skipped; without it, a space is a trivia token.
struct ByteLexer<'inp, const SKIP: bool> {
  src: &'inp str,
  tok_start: usize,
  pos: usize,
  state: (),
}

impl<'inp, const SKIP: bool> Lexer<'inp> for ByteLexer<'inp, SKIP> {
  type State = ();
  type Source = str;
  type Token = Tok;
  type Span = SimpleSpan;
  type Offset = usize;

  const SURFACES_TRIVIA: bool = !SKIP;

  fn new(src: &'inp str) -> Self {
    Self::with_state(src, ())
  }

  fn with_state(src: &'inp str, state: ()) -> Self {
    Self {
      src,
      tok_start: 0,
      pos: 0,
      state,
    }
  }

  fn check(&self) -> Result<(), LexErr> {
    Ok(())
  }

  fn state(&self) -> &Self::State {
    &self.state
  }

  fn state_mut(&mut self) -> &mut Self::State {
    &mut self.state
  }

  fn into_state(self) -> Self::State {
    self.state
  }

  fn source(&self) -> &'inp str {
    self.src
  }

  fn span(&self) -> SimpleSpan {
    SimpleSpan::new(self.tok_start, self.pos)
  }

  fn slice(&self) -> &'inp str {
    &self.src[self.tok_start..self.pos]
  }

  fn lex(&mut self) -> Option<Result<Tok, LexErr>> {
    let bytes = self.src.as_bytes();
    if SKIP {
      while bytes.get(self.pos) == Some(&b' ') {
        self.pos += 1;
      }
    }
    let byte = *bytes.get(self.pos)?;
    self.tok_start = self.pos;
    self.pos += 1;
    Some(Ok(Tok(byte)))
  }

  fn read_frontier(&self) -> ReadFrontier<usize> {
    ReadFrontier::SpanEnd
  }

  fn bump(&mut self, n: &usize) {
    self.pos += *n;
    self.tok_start = self.pos;
  }
}

/// The table's marker.
struct Expr;

const TABLE: PrattTable<Expr, u8> = PrattTable::new();

fn infix(associativity: PrattInfix<(), (), ()>, power: u8) -> Option<PrattRHS<(), (), (), (), u8>> {
  Some(PrattRHS::Infix(Precedenced::new(associativity, power)))
}

fn postfix(power: u8) -> Option<PrattRHS<(), (), (), (), u8>> {
  Some(PrattRHS::Postfix(Precedenced::new((), power)))
}

/// | byte | left-hand side | right-hand side |
/// |---|---|---|
/// | `0` to `9` | operand | |
/// | `-` | prefix, 40 | infix, left, 10 |
/// | `~` | prefix, 12 | |
/// | `p` | prefix, 9, whose fold takes a `+` after its operand | |
/// | `+` | | infix, left, 10 |
/// | `&` | | infix, right, 10 |
/// | `*` | | infix, left, 20 |
/// | `^` | | infix, right, 30 |
/// | `=` | | infix, right, 5 |
/// | `<` | | infix, non-associative, 15 |
/// | `>` | | infix, left, 15 |
/// | `i` | | infix, right, 8, whose fold reads `e` and an operand after its right one |
/// | `t` | | infix, left, 45, whose fold takes a `+` that follows |
/// | `!` | | postfix, 50 |
/// | `?` | | postfix, 12 |
/// | `%` | | postfix, 15 |
/// | `[` | | postfix, 50, whose fold reads a digit and a `]` |
impl PrattToken<'_, Expr, u8> for Tok {
  fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), u8>> {
    match self.0 {
      b'0'..=b'9' => Some(PrattLHS::Operand(())),
      b'-' => Some(PrattLHS::Prefix(Precedenced::new((), 40))),
      b'~' => Some(PrattLHS::Prefix(Precedenced::new((), 12))),
      b'p' => Some(PrattLHS::Prefix(Precedenced::new((), 9))),
      _ => None,
    }
  }

  fn try_pratt_rhs(&self) -> Option<PrattRHS<(), (), (), (), u8>> {
    match self.0 {
      b'+' | b'-' => infix(PrattInfix::Left(()), 10),
      b'&' => infix(PrattInfix::Right(()), 10),
      b'*' => infix(PrattInfix::Left(()), 20),
      b'^' => infix(PrattInfix::Right(()), 30),
      b'=' => infix(PrattInfix::Right(()), 5),
      b'<' => infix(PrattInfix::Neither(()), 15),
      b'>' => infix(PrattInfix::Left(()), 15),
      b'i' => infix(PrattInfix::Right(()), 8),
      b't' => infix(PrattInfix::Left(()), 45),
      b'!' | b'[' => postfix(50),
      b'?' => postfix(12),
      b'%' => postfix(15),
      _ => None,
    }
  }
}

// ── The error and the input ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fail {
  Lex,
  Unexpected,
  Eot { at: usize },
  EoLhs { at: usize },
  EoRhs { at: usize },
  Chain { at: usize },
  Full { at: usize, capacity: usize },
}

impl From<LexErr> for Fail {
  fn from(_: LexErr) -> Self {
    Fail::Lex
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for Fail {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    Fail::Unexpected
  }
}

impl<Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEot<usize, Lang, Set>> for Fail {
  fn from(e: UnexpectedEot<usize, Lang, Set>) -> Self {
    Fail::Eot { at: e.offset() }
  }
}

impl<Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEoLhs<usize, Lang, Set>> for Fail {
  fn from(e: UnexpectedEoLhs<usize, Lang, Set>) -> Self {
    Fail::EoLhs { at: e.offset() }
  }
}

impl<Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEoRhs<usize, Lang, Set>> for Fail {
  fn from(e: UnexpectedEoRhs<usize, Lang, Set>) -> Self {
    Fail::EoRhs { at: e.offset() }
  }
}

impl<Lang: ?Sized> From<NonAssociativeChain<usize, Lang>> for Fail {
  fn from(e: NonAssociativeChain<usize, Lang>) -> Self {
    Fail::Chain { at: e.offset() }
  }
}

impl<Lang: ?Sized> From<PrattStackFull<usize, Lang>> for Fail {
  fn from(e: PrattStackFull<usize, Lang>) -> Self {
    Fail::Full {
      at: e.offset(),
      capacity: e.capacity(),
    }
  }
}

type Ctx<'inp, const SKIP: bool> = (Fatal<Fail>, DefaultCache<'inp, ByteLexer<'inp, SKIP>>);
type Ir<'inp, 'c, const SKIP: bool> = InputRef<'inp, 'c, ByteLexer<'inp, SKIP>, Ctx<'inp, SKIP>>;

/// An operator token, as a fold is handed one.
type Op = Spanned<Tok, SimpleSpan>;

/// Runs `f` over an input of `src`.
fn over<'inp, const SKIP: bool, R>(
  src: &'inp str,
  f: impl FnOnce(&mut Ir<'inp, '_, SKIP>) -> R,
) -> R {
  let mut input = Input::<ByteLexer<'inp, SKIP>, Ctx<'inp, SKIP>, ()>::with_state_and_context(
    src,
    (),
    InputContext::new(
      Fatal::<Fail>::new(),
      DefaultCache::<'inp, ByteLexer<'inp, SKIP>>::default(),
    ),
  );
  let mut inp = input.as_ref();
  f(&mut inp)
}

// ── Values that need no allocator ─────────────────────────────────────────────────────────────

/// One pending operator of a driver whose value is a count.
type CountFrame = PrattFrame<u32, Op, Op, Op, Op, u8>;

/// A digit, as its value. It crosses no trivia, and declines anything else.
fn digit(input: &mut Ir<'static, '_, true>) -> Result<ParseAttempt<u32>, Fail> {
  Ok(
    match input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
      Some(tok) => ParseAttempt::Accept(u32::from(tok.into_data().0 - b'0')),
      None => ParseAttempt::Decline,
    },
  )
}

fn count_prefix(
  _: &mut Ir<'static, '_, true>,
  operand: u32,
  _: Precedenced<Op, u8>,
) -> Result<u32, Fail> {
  Ok(operand + 1)
}

fn sum_infix(
  _: &mut Ir<'static, '_, true>,
  left: u32,
  right: u32,
  _: Precedenced<PrattInfix<Op, Op, Op>, u8>,
) -> Result<u32, Fail> {
  Ok(left + right)
}

fn keep_postfix(
  _: &mut Ir<'static, '_, true>,
  operand: u32,
  _: Precedenced<Op, u8>,
) -> Result<u32, Fail> {
  Ok(operand)
}

/// `N - 2` prefix operators, then `1;`.
const fn prefixed<const N: usize>() -> [u8; N] {
  let mut bytes = [b'-'; N];
  bytes[N - 2] = b'1';
  bytes[N - 1] = b';';
  bytes
}

fn text(bytes: &'static [u8]) -> &'static str {
  core::str::from_utf8(bytes).expect("the sources are ASCII")
}

/// Folds `src` with `driver`.
fn fold<D>(driver: &mut D, src: &'static str) -> Result<u32, Fail>
where
  D: ParseInput<'static, ByteLexer<'static, true>, u32, Ctx<'static, true>>,
{
  over::<true, _>(src, |inp| driver.parse_input(inp))
}

// ── Every tier ────────────────────────────────────────────────────────────────────────────────

/// Under `max_depth(100)`, `folding` holds a hundred pending operators and refuses the 101st,
/// reporting the cap. Where the crate cannot allocate, it grows past its own four on the native
/// stack: 24 segments of four above them.
#[test]
fn folding_under_max_depth_100_holds_a_hundred_and_refuses_the_next() {
  static HUNDRED: [u8; 102] = prefixed();
  static PAST: [u8; 103] = prefixed();
  let mut driver = flat(TABLE, digit)
    .max_depth(100)
    .folding(count_prefix, sum_infix, keep_postfix);
  assert_eq!(fold(&mut driver, text(&HUNDRED)), Ok(101));
  assert_eq!(
    fold(&mut driver, text(&PAST)),
    Err(Fail::Full {
      at: 101,
      capacity: 100
    })
  );
  let deepest = if cfg!(any(feature = "std", feature = "alloc")) {
    0
  } else {
    24
  };
  assert_eq!(driver.growth.deepest, deepest);
}

/// A storage given by `with_stack` is not grown, in any tier: under `max_depth(100)`, one of 16
/// frames refuses the 17th pending operator, and the error reports 16.
#[test]
fn a_storage_given_by_with_stack_refuses_when_it_is_full() {
  static SIXTEEN: [u8; 18] = prefixed();
  static PAST: [u8; 19] = prefixed();
  let mut driver = flat(TABLE, digit)
    .max_depth(100)
    .folding(count_prefix, sum_infix, keep_postfix)
    .with_stack(ArrayDeque::<CountFrame, U16>::new());
  assert_eq!(fold(&mut driver, text(&SIXTEEN)), Ok(17));
  assert_eq!(
    fold(&mut driver, text(&PAST)),
    Err(Fail::Full {
      at: 17,
      capacity: 16
    })
  );
  assert_eq!(
    driver.growth.grown, 0,
    "a storage the caller gave is not grown"
  );
}

/// A long expression that is not deep does not grow the native stack: a chain that crosses a
/// segment's boundary again and again holds `ceil(depth / 4)` segments at its deepest, and grows
/// and returns once for each crossing. Driven through the growth path in every tier, over a first
/// storage of four frames, as the default is where the crate cannot allocate.
#[test]
fn a_chain_that_crosses_a_boundary_again_and_again_holds_ceil_depth_over_four_segments() {
  // After the first, each `1^1^1^1^1^1` keeps five `^` pending over a `+`: six, in two segments.
  const CHAIN: &str = "1^1^1^1^1^1+1^1^1^1^1^1+1^1^1^1^1^1+1^1^1^1^1^1+1^1^1^1^1^1;";
  let mut driver = flat(TABLE, digit)
    .folding(count_prefix, sum_infix, keep_postfix)
    .with_stack(ArrayDeque::<CountFrame, U4>::new());
  driver.grows = true;
  assert_eq!(
    over::<true, _>(CHAIN, |inp| driver.parse_at::<U4>(inp)),
    Ok(30)
  );
  assert_eq!(driver.growth.deepest, 1, "one segment above the first four");
  assert_eq!((driver.growth.grown, driver.growth.returned), (5, 5));
}

/// A parse that grows and then fails leaves the kept driver's storage empty, and the next parse
/// has its four frames again: it grows only past them.
#[test]
fn a_parse_that_grows_and_fails_leaves_the_driver_s_storage_empty() {
  let mut driver = flat(TABLE, digit)
    .folding(count_prefix, sum_infix, keep_postfix)
    .with_stack(ArrayDeque::<CountFrame, U4>::new());
  driver.grows = true;
  // Six prefix operators and no operand: the fifth grows a segment.
  assert_eq!(
    over::<true, _>("------;", |inp| driver.parse_at::<U4>(inp)),
    Err(Fail::EoLhs { at: 6 })
  );
  assert_eq!(driver.growth.grown, 1);
  assert_eq!(Container::len(&driver.stack), 0);
  assert_eq!(
    over::<true, _>("----1;", |inp| driver.parse_at::<U4>(inp)),
    Ok(5)
  );
  assert_eq!(
    driver.growth.grown, 1,
    "four pending operators fit the driver's own"
  );
  assert_eq!(
    over::<true, _>("-----1;", |inp| driver.parse_at::<U4>(inp)),
    Ok(6)
  );
  assert_eq!(driver.growth.grown, 2);
}

/// In `Checked` mode under `max_depth(8)`, `parse_into` refuses the ninth pending operator, and the
/// error reports 8.
#[test]
fn checked_under_max_depth_8_refuses_the_ninth_and_reports_8() {
  static EIGHT: [u8; 10] = prefixed();
  static PAST: [u8; 11] = prefixed();
  let mut driver = flat(TABLE, digit).max_depth(8);
  assert_eq!(
    over::<true, _>(text(&EIGHT), |inp| driver.parse_into(inp, &mut ())),
    Ok(())
  );
  assert_eq!(
    over::<true, _>(text(&PAST), |inp| driver.parse_into(inp, &mut ())),
    Err(Fail::Full { at: 9, capacity: 8 })
  );
}

/// In `Bare` mode `parse_into` holds no stack, and reads a run of 10,000 prefix operators. Miri reads
/// the run's last hundred.
#[test]
fn bare_reads_a_run_of_ten_thousand_prefix_operators() {
  static RUN: [u8; 10_002] = prefixed();
  let run = if cfg!(miri) {
    &RUN[RUN.len() - 102..]
  } else {
    &RUN[..]
  };
  let mut driver = flat(TABLE, digit).bare();
  assert_eq!(
    over::<true, _>(text(run), |inp| driver.parse_into(inp, &mut ())),
    Ok(())
  );
}

// ── The tier with no allocator ────────────────────────────────────────────────────────────────

/// Where the crate cannot allocate, `folding` holds the fifth to the 32nd pending operator in
/// segments of four on the native stack, seven above the driver's own, and refuses the 33rd,
/// reporting 32.
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[test]
fn without_an_allocator_folding_holds_32_and_refuses_the_33rd() {
  static THIRTY_TWO: [u8; 34] = prefixed();
  static PAST: [u8; 35] = prefixed();
  let mut driver = flat(TABLE, digit).folding(count_prefix, sum_infix, keep_postfix);
  assert_eq!(fold(&mut driver, text(&THIRTY_TWO)), Ok(33));
  assert_eq!(driver.growth.deepest, 7);
  assert_eq!(
    fold(&mut driver, text(&PAST)),
    Err(Fail::Full {
      at: 33,
      capacity: 32
    })
  );
}

/// Where the crate cannot allocate, `parse_into`'s own 32 unit frames refuse the 33rd pending
/// operator in `Checked` mode, and the error reports 32 with no cap set and under `max_depth(100)`.
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[test]
fn without_an_allocator_checked_refuses_the_33rd_and_reports_32() {
  static THIRTY_TWO: [u8; 34] = prefixed();
  static PAST: [u8; 35] = prefixed();
  let full = Err(Fail::Full {
    at: 33,
    capacity: 32,
  });

  let mut driver = flat(TABLE, digit);
  assert_eq!(
    over::<true, _>(text(&THIRTY_TWO), |inp| driver.parse_into(inp, &mut ())),
    Ok(())
  );
  assert_eq!(
    over::<true, _>(text(&PAST), |inp| driver.parse_into(inp, &mut ())),
    full
  );

  let mut driver = flat(TABLE, digit).max_depth(100);
  assert_eq!(
    over::<true, _>(text(&PAST), |inp| driver.parse_into(inp, &mut ())),
    full
  );
}

// ── The segmented loop against the heap loop ──────────────────────────────────────────────────

#[cfg(feature = "std")]
mod against_the_heap_loop {
  use core::cell::RefCell;
  use std::{format, string::String, thread_local, vec::Vec};

  use hybrid_arraydeque::{
    ArraySize,
    typenum::{U1, U2},
  };

  use super::{super::Growth, *};
  use crate::{Emitter, ParseContext, TryParseInput};

  /// How many sources each leg generates. Miri interprets each parse, so it reads a few.
  const CASES: usize = if cfg!(miri) { 3 } else { 400 };

  /// One below every power, one between the infix powers, one at the non-associative power, and
  /// one above every infix power but `^` and `t`.
  const FLOORS: [u8; 4] = [0, 11, 15, 25];

  /// No cap, one that a short chain meets, and the tier's default with no allocator.
  const CAPS: [Option<usize>; 3] = [None, Some(3), Some(32)];

  /// One pending operator of a driver whose value is a tree.
  type TreeFrame = PrattFrame<String, Op, Op, Op, Op, u8>;

  thread_local! {
    /// Each fold on this thread: its operator's byte and offset, and `span().end()` as it
    /// returns.
    static TRACE: RefCell<Vec<(u8, usize, usize)>> = const { RefCell::new(Vec::new()) };
  }

  fn record(operator: &Op, end: usize) {
    TRACE.with(|trace| {
      trace
        .borrow_mut()
        .push((operator.data().0, operator.span().start(), end));
    });
  }

  fn take_trace() -> Vec<(u8, usize, usize)> {
    TRACE.with(|trace| core::mem::take(&mut *trace.borrow_mut()))
  }

  /// Commits the next token when `wanted` takes its byte.
  fn take<const SKIP: bool>(
    input: &mut Ir<'_, '_, SKIP>,
    wanted: impl Fn(u8) -> bool,
  ) -> Result<Option<char>, Fail> {
    Ok(
      input
        .try_expect(|tok| wanted(tok.data().0))?
        .map(|tok| char::from(tok.into_data().0)),
    )
  }

  /// A digit, as its own text. With `crossing` it crosses the trivia before the digit itself.
  #[derive(Debug, Clone, Copy)]
  struct Text {
    crossing: bool,
  }

  impl<'inp, const SKIP: bool, C> TryParseInput<'inp, ByteLexer<'inp, SKIP>, String, C> for Text
  where
    C: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  {
    fn try_parse_input(
      &mut self,
      input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, C>,
    ) -> Result<ParseAttempt<String>, <C::Emitter as Emitter<'inp, ByteLexer<'inp, SKIP>>>::Error>
    {
      if self.crossing {
        input.skip_while(|tok| tok.is_trivia())?;
      }
      Ok(
        match input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
          Some(tok) => ParseAttempt::Accept(String::from(char::from(tok.into_data().0))),
          None => ParseAttempt::Decline,
        },
      )
    }
  }

  /// `-1` folds to `(-1)`. `p` takes a `+` after its operand.
  fn fold_prefix<const SKIP: bool>(
    input: &mut Ir<'_, '_, SKIP>,
    operand: String,
    operator: Precedenced<Op, u8>,
  ) -> Result<String, Fail> {
    let operator = operator.into_data();
    let byte = operator.data().0;
    let tail = match byte {
      b'p' => take(input, |next| next == b'+')?,
      _ => None,
    };
    record(&operator, input.span().end());
    Ok(format!(
      "({}{operand}{})",
      char::from(byte),
      tail.map(String::from).unwrap_or_default()
    ))
  }

  /// `1+2` folds to `(1+2)`. `i` reads `e` and a digit after its right operand, and `t` takes a
  /// `+` that follows.
  fn fold_infix<const SKIP: bool>(
    input: &mut Ir<'_, '_, SKIP>,
    left: String,
    right: String,
    operator: Precedenced<PrattInfix<Op, Op, Op>, u8>,
  ) -> Result<String, Fail> {
    let (PrattInfix::Left(operator) | PrattInfix::Right(operator) | PrattInfix::Neither(operator)) =
      operator.into_data();
    let byte = operator.data().0;
    let mut tail = String::new();
    match byte {
      b'i' => {
        if let Some(e) = take(input, |next| next == b'e')? {
          tail.push(e);
          tail.extend(take(input, |next| next.is_ascii_digit())?);
        }
      }
      b't' => tail.extend(take(input, |next| next == b'+')?),
      _ => {}
    }
    record(&operator, input.span().end());
    Ok(format!("({left}{}{right}{tail})", char::from(byte)))
  }

  /// `1!` folds to `(1!)`. `[` reads a digit and a `]`.
  fn fold_postfix<const SKIP: bool>(
    input: &mut Ir<'_, '_, SKIP>,
    operand: String,
    operator: Precedenced<Op, u8>,
  ) -> Result<String, Fail> {
    let operator = operator.into_data();
    let byte = operator.data().0;
    let mut tail = String::new();
    if byte == b'[' {
      tail.extend(take(input, |next| next.is_ascii_digit())?);
      tail.extend(take(input, |next| next == b']')?);
    }
    record(&operator, input.span().end());
    Ok(format!("({operand}{}{tail})", char::from(byte)))
  }

  /// A small deterministic generator: xorshift64.
  struct Rng(u64);

  impl Rng {
    fn below(&mut self, n: usize) -> usize {
      self.0 ^= self.0 << 13;
      self.0 ^= self.0 >> 7;
      self.0 ^= self.0 << 17;
      (self.0 % n as u64) as usize
    }

    fn pick(&mut self, bytes: &[u8]) -> u8 {
      bytes[self.below(bytes.len())]
    }
  }

  const OPERANDS: &[u8] = b"0123456789";
  const PREFIXES: &[u8] = b"-~p";
  const INFIXES: &[u8] = b"+-&*^=<>it";
  const POSTFIXES: &[u8] = b"!?%[";
  /// What a mutation draws from: every row of the table, the read-on folds' tails, the
  /// terminator, and a byte no row names.
  const ANY: &[u8] = b"0123456789-~p+&*^=<>it!?%[]e;#";

  /// A well-formed expression: terms joined by infix operators, each term an operand with prefix
  /// operators before it and postfix operators after it, and some of them followed by the tail an
  /// `i` reads.
  fn well_formed(rng: &mut Rng) -> Vec<u8> {
    let mut tokens = Vec::new();
    for term in 0..1 + rng.below(6) {
      if term > 0 {
        tokens.push(rng.pick(INFIXES));
      }
      for _ in 0..rng.below(3) {
        tokens.push(rng.pick(PREFIXES));
      }
      tokens.push(rng.pick(OPERANDS));
      for _ in 0..rng.below(3) {
        let postfix = rng.pick(POSTFIXES);
        tokens.push(postfix);
        if postfix == b'[' {
          tokens.extend([rng.pick(OPERANDS), b']']);
        }
      }
      if rng.below(4) == 0 {
        tokens.extend([b'e', rng.pick(OPERANDS)]);
      }
    }
    tokens
  }

  /// A well-formed expression with one to three tokens deleted, inserted or replaced.
  fn mutated(rng: &mut Rng) -> Vec<u8> {
    let mut tokens = well_formed(rng);
    for _ in 0..1 + rng.below(3) {
      let at = rng.below(tokens.len() + 1);
      match (rng.below(3), at < tokens.len()) {
        (0, true) => {
          tokens.remove(at);
        }
        (1, _) | (_, false) => tokens.insert(at, rng.pick(ANY)),
        (_, true) => tokens[at] = rng.pick(ANY),
      }
    }
    tokens
  }

  /// The sources of one leg, half well-formed and half mutated, each with a terminator and up to
  /// `spaces` spaces before each token.
  fn sources(seed: u64, spaces: usize) -> Vec<String> {
    let mut rng = Rng(seed);
    (0..CASES)
      .map(|case| {
        let tokens = if case % 2 == 0 {
          well_formed(&mut rng)
        } else {
          mutated(&mut rng)
        };
        let mut src = String::new();
        for &byte in tokens.iter().chain(b";") {
          for _ in 0..rng.below(spaces + 1) {
            src.push(' ');
          }
          src.push(char::from(byte));
        }
        src
      })
      .collect()
  }

  /// What one parse did: its result, its folds, where it left the input, and what it charged.
  #[derive(Debug, PartialEq, Eq)]
  struct Parsed {
    result: Result<String, Fail>,
    trace: Vec<(u8, usize, usize)>,
    handback: usize,
    next: Option<(u8, usize)>,
    spent: usize,
  }

  /// One parse of `src` by `folding` at `floor` and `cap`. With `grows`, through the loop that
  /// grows by segments of `K`, over a first storage of `K` frames; without it, as it ships with an
  /// allocator, over a storage that never refuses.
  fn parse<K: ArraySize, const SKIP: bool>(
    src: &str,
    crossing: bool,
    floor: u8,
    cap: Option<usize>,
    grows: bool,
  ) -> (Parsed, Growth) {
    over::<SKIP, _>(src, |input| {
      let driver = flat(TABLE, Text { crossing }).min_precedence(floor);
      let driver = match cap {
        Some(cap) => driver.max_depth(cap),
        None => driver,
      };
      let mut driver = driver.folding(
        fold_prefix::<SKIP>,
        fold_infix::<SKIP>,
        fold_postfix::<SKIP>,
      );
      take_trace();
      let (result, growth) = if grows {
        let mut driver = driver.with_stack(ArrayDeque::<TreeFrame, K>::new());
        driver.grows = true;
        let result = driver.parse_at::<K>(input);
        (result, driver.growth)
      } else {
        (driver.parse_input(input), driver.growth)
      };
      let trace = take_trace();
      let handback = input.span().end();
      let spent = input.token_budget().spent();
      let next = input
        .next()
        .expect("the next token")
        .map(|tok| (tok.data().0, tok.span().start()));
      let parsed = Parsed {
        result,
        trace,
        handback,
        next,
        spent,
      };
      (parsed, growth)
    })
  }

  /// Every source of one leg, under both operand parsers, at each floor and cap, through the loop
  /// that grows at segments of one, two and four frames, against the heap loop. Returns the
  /// growth at each size.
  fn leg<const SKIP: bool>(seed: u64, spaces: usize) -> [Growth; 3] {
    let mut sizes = [Growth::default(); 3];
    for (index, src) in sources(seed, spaces).iter().enumerate() {
      let crossing = index % 2 == 1;
      for floor in FLOORS {
        for cap in CAPS {
          let heap = parse::<U4, SKIP>(src, crossing, floor, cap, false).0;
          let segmented = [
            parse::<U1, SKIP>(src, crossing, floor, cap, true),
            parse::<U2, SKIP>(src, crossing, floor, cap, true),
            parse::<U4, SKIP>(src, crossing, floor, cap, true),
          ];
          for (size, (parsed, growth)) in sizes.iter_mut().zip(segmented) {
            assert_eq!(
              parsed, heap,
              "{src:?} at {floor}, cap {cap:?}, crossing {crossing}: {growth:?}"
            );
            size.grown += growth.grown;
            size.returned += growth.returned;
            size.deepest = size.deepest.max(growth.deepest);
          }
        }
      }
    }
    sizes
  }

  /// It cannot pass without growing: at each size the generated set grew and returned, and
  /// smaller segments grew more often and deeper.
  fn assert_grew(sizes: &[Growth; 3]) {
    if cfg!(miri) {
      return;
    }
    for size in sizes {
      assert!(size.grown > 0 && size.returned > 0, "{sizes:?}");
    }
    assert!(
      sizes[0].grown > sizes[1].grown && sizes[1].grown > sizes[2].grown,
      "{sizes:?}"
    );
    assert!(
      sizes[0].deepest > sizes[1].deepest && sizes[1].deepest > sizes[2].deepest,
      "{sizes:?}"
    );
  }

  /// Over a lexer that skips trivia: the generated set, its folds that read on included.
  #[test]
  fn the_segmented_loop_folds_as_the_heap_loop_over_a_skipping_lexer() {
    assert_grew(&leg::<true>(0x9e37_79b9_7f4a_7c15, 0));
  }

  /// Over a lexer that surfaces trivia, under an operand parser that crosses it and one that does
  /// not.
  #[test]
  fn the_segmented_loop_folds_as_the_heap_loop_over_a_surfacing_lexer() {
    assert_grew(&leg::<false>(0xd1b5_4a32_d192_ed03, 2));
  }

  /// One source at segments of one frame against the heap loop, and the result both give, with
  /// what the segmented parse grew and returned.
  fn crossing(src: &str) -> (Result<String, Fail>, Growth) {
    let heap = parse::<U4, true>(src, false, 0, None, false).0;
    let (parsed, growth) = parse::<U1, true>(src, false, 0, None, true);
    assert_eq!(parsed, heap, "{src:?}");
    (parsed.result, growth)
  }

  /// The `i`'s frame is below two segments of `^`. Both return, and its fold, in the frame below,
  /// reads `e5` on and moves the input.
  #[test]
  fn a_fold_that_reads_on_runs_in_the_frame_below_after_a_return() {
    let (result, growth) = crossing("1i2^3^4e5;");
    assert_eq!(result, Ok(String::from("(1i(2^(3^4))e5)")));
    assert_eq!((growth.grown, growth.returned), (2, 2));
  }

  /// The `<` is the upper segment's only frame. Its reduction latches the `+` below through the
  /// link, and the loop below refuses the second `<` as a chain.
  #[test]
  fn a_chain_is_refused_through_a_latch_written_through_the_link() {
    let (result, growth) = crossing("1+2<3<4;");
    assert_eq!(result, Err(Fail::Chain { at: 5 }));
    assert_eq!((growth.grown, growth.returned), (1, 1));
  }

  /// The `^` returns, the `*` grows a segment again, and its operand is missing.
  #[test]
  fn a_missing_operand_is_met_right_after_a_return() {
    let (result, growth) = crossing("1+2^3*;");
    assert_eq!(result, Err(Fail::EoRhs { at: 6 }));
    assert_eq!((growth.grown, growth.returned), (2, 1));
  }
}

/// A cache a context hands over holding entries: they count as pushed, so the head the first read
/// stages has its place in the push order like any other.
#[cfg(feature = "std")]
mod over_a_prefilled_cache {
  use core::cell::Cell;
  use std::{format, thread_local};

  use hybrid_arraydeque::typenum::U128;

  use super::*;
  use crate::cache::CachedToken;

  thread_local! {
    /// The right-hand sides the counting table has classified on this thread.
    static CLASSIFIED: Cell<usize> = const { Cell::new(0) };
  }

  /// The suites' table, under a marker whose right-hand classifier counts its calls.
  struct Counted;

  const COUNTED: PrattTable<Counted, u8> = PrattTable::new();

  impl PrattToken<'_, Counted, u8> for Tok {
    fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), u8>> {
      <Self as PrattToken<'_, Expr, u8>>::try_pratt_lhs(self)
    }

    fn try_pratt_rhs(&self) -> Option<PrattRHS<(), (), (), (), u8>> {
      CLASSIFIED.with(|classified| classified.set(classified.get() + 1));
      <Self as PrattToken<'_, Expr, u8>>::try_pratt_rhs(self)
    }
  }

  /// A cache with room for every token of the sources below.
  type Prefilled = ArrayDeque<CachedToken<Tok, (), SimpleSpan>, U128>;
  type PrefilledIr<'inp, 'c> = InputRef<'inp, 'c, ByteLexer<'inp, true>, (Fatal<Fail>, Prefilled)>;

  /// A digit, as its value.
  fn prefilled_digit(input: &mut PrefilledIr<'_, '_>) -> Result<ParseAttempt<u32>, Fail> {
    Ok(
      match input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
        Some(tok) => ParseAttempt::Accept(u32::from(tok.into_data().0 - b'0')),
        None => ParseAttempt::Decline,
      },
    )
  }

  fn keep(_: &mut PrefilledIr<'_, '_>, operand: u32, _: Precedenced<Op, u8>) -> Result<u32, Fail> {
    Ok(operand)
  }

  fn keep_left(
    _: &mut PrefilledIr<'_, '_>,
    left: u32,
    _: u32,
    _: Precedenced<PrattInfix<Op, Op, Op>, u8>,
  ) -> Result<u32, Fail> {
    Ok(left)
  }

  /// `folding` over N prefix operators, `1` and `x`, every token of which is in the cache the
  /// input is handed, with folds that read nothing: its value, and the right-hand sides it
  /// classified.
  fn over_prefilled(n: usize) -> (Result<u32, Fail>, usize) {
    let src = format!("{}1x", "-".repeat(n));
    let mut cache = Prefilled::new();
    for (at, byte) in src.bytes().enumerate() {
      let entry = CachedToken::new(Spanned::new(SimpleSpan::new(at, at + 1), Tok(byte)), ());
      assert!(
        cache.push_back_mut(entry).is_ok(),
        "the cache has room for the source"
      );
    }
    let mut input =
      Input::<ByteLexer<'_, true>, (Fatal<Fail>, Prefilled), ()>::with_state_and_context(
        src.as_str(),
        (),
        InputContext::new(Fatal::<Fail>::new(), cache),
      );
    CLASSIFIED.with(|classified| classified.set(0));
    let result = flat(COUNTED, prefilled_digit)
      .folding(keep, keep_left, keep)
      .parse_input(&mut input.as_ref());
    (result, CLASSIFIED.with(Cell::get))
  }

  /// The head `x` is an entry the cache arrived holding, and it is classified once, however many
  /// folds it waits behind. Without a place in the push order it would be classified after every
  /// fold, N + 1 times.
  #[test]
  fn a_head_the_cache_arrived_holding_is_classified_once_across_its_folds() {
    for n in [16, 64] {
      let (result, calls) = over_prefilled(n);
      assert_eq!(result, Ok(1), "{n}");
      assert!(
        calls <= 2,
        "{n} prefix operators over a prefilled cache: {calls} classifications"
      );
    }
  }
}

#[cfg(feature = "std")]
mod a_refused_operator_that_unwinds {
  use core::cell::Cell;
  use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    thread_local,
  };

  use hybrid_arraydeque::typenum::U1;

  use super::*;

  thread_local! {
    /// Whether the next `~` or `#` dropped on this thread unwinds. The drop that unwinds clears it.
    static ARMED: Cell<bool> = const { Cell::new(false) };
  }

  /// A token of one byte whose destructor unwinds, once, for a `~` or a `#` while `ARMED` is set.
  #[derive(Debug, Clone, PartialEq, Eq)]
  struct Live(u8);

  impl Drop for Live {
    fn drop(&mut self) {
      if matches!(self.0, b'~' | b'#') && ARMED.with(|armed| armed.replace(false)) {
        panic!("a token's destructor unwinds, as asked");
      }
    }
  }

  impl Token<'_> for Live {
    type Kind = u8;
    type Error = LexErr;

    const SCAN_LOOKAHEAD: ScanLookahead = ScanLookahead::Unbounded;

    fn kind(&self) -> u8 {
      self.0
    }

    fn is_trivia(&self) -> bool {
      false
    }
  }

  /// One `Live` token for each byte.
  struct LiveLexer<'inp> {
    src: &'inp str,
    tok_start: usize,
    pos: usize,
    state: (),
  }

  impl<'inp> Lexer<'inp> for LiveLexer<'inp> {
    type State = ();
    type Source = str;
    type Token = Live;
    type Span = SimpleSpan;
    type Offset = usize;

    fn new(src: &'inp str) -> Self {
      Self::with_state(src, ())
    }

    fn with_state(src: &'inp str, state: ()) -> Self {
      Self {
        src,
        tok_start: 0,
        pos: 0,
        state,
      }
    }

    fn check(&self) -> Result<(), LexErr> {
      Ok(())
    }

    fn state(&self) -> &Self::State {
      &self.state
    }

    fn state_mut(&mut self) -> &mut Self::State {
      &mut self.state
    }

    fn into_state(self) -> Self::State {
      self.state
    }

    fn source(&self) -> &'inp str {
      self.src
    }

    fn span(&self) -> SimpleSpan {
      SimpleSpan::new(self.tok_start, self.pos)
    }

    fn slice(&self) -> &'inp str {
      &self.src[self.tok_start..self.pos]
    }

    fn lex(&mut self) -> Option<Result<Live, LexErr>> {
      let byte = *self.src.as_bytes().get(self.pos)?;
      self.tok_start = self.pos;
      self.pos += 1;
      Some(Ok(Live(byte)))
    }

    fn read_frontier(&self) -> ReadFrontier<usize> {
      ReadFrontier::SpanEnd
    }

    fn bump(&mut self, n: &usize) {
      self.pos += *n;
      self.tok_start = self.pos;
    }
  }

  /// The table's marker.
  struct LiveExpr;

  const LIVE: PrattTable<LiveExpr, u8> = PrattTable::new();

  /// | byte | left-hand side | right-hand side |
  /// |---|---|---|
  /// | `0` to `9` | operand | |
  /// | `-`, `~` | prefix, 40 | |
  /// | `+` | | infix, left, 10 |
  /// | `#` | | infix, left, 20 |
  impl PrattToken<'_, LiveExpr, u8> for Live {
    fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), u8>> {
      match self.0 {
        b'0'..=b'9' => Some(PrattLHS::Operand(())),
        b'-' | b'~' => Some(PrattLHS::Prefix(Precedenced::new((), 40))),
        _ => None,
      }
    }

    fn try_pratt_rhs(&self) -> Option<PrattRHS<(), (), (), (), u8>> {
      match self.0 {
        b'+' => infix(PrattInfix::Left(()), 10),
        b'#' => infix(PrattInfix::Left(()), 20),
        _ => None,
      }
    }
  }

  type LiveCtx<'inp> = (Fatal<Fail>, DefaultCache<'inp, LiveLexer<'inp>>);
  type LiveIr<'inp, 'c> = InputRef<'inp, 'c, LiveLexer<'inp>, LiveCtx<'inp>>;
  type LiveOp = Spanned<Live, SimpleSpan>;
  type LiveFrame = PrattFrame<u32, LiveOp, LiveOp, LiveOp, LiveOp, u8>;

  /// A digit, as its value.
  fn live_digit(input: &mut LiveIr<'_, '_>) -> Result<ParseAttempt<u32>, Fail> {
    Ok(
      match input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
        Some(tok) => ParseAttempt::Accept(u32::from(tok.data().0 - b'0')),
        None => ParseAttempt::Decline,
      },
    )
  }

  fn live_keep(
    _: &mut LiveIr<'_, '_>,
    operand: u32,
    _: Precedenced<LiveOp, u8>,
  ) -> Result<u32, Fail> {
    Ok(operand)
  }

  fn live_keep_left(
    _: &mut LiveIr<'_, '_>,
    left: u32,
    _: u32,
    _: Precedenced<PrattInfix<LiveOp, LiveOp, LiveOp>, u8>,
  ) -> Result<u32, Fail> {
    Ok(left)
  }

  /// Folds `src` over a storage of one frame, which refuses the second pending operator, with the
  /// destructor of that operator's token armed to unwind: whether the parse counted the trip of a
  /// full storage before it unwound.
  fn counts_the_trip_through_the_unwind(src: &str) -> bool {
    let mut input = Input::<LiveLexer<'_>, LiveCtx<'_>, ()>::with_state_and_context(
      src,
      (),
      InputContext::new(
        Fatal::<Fail>::new(),
        DefaultCache::<'_, LiveLexer<'_>>::default(),
      ),
    );
    let mut inp = input.as_ref();
    let mut driver = flat(LIVE, live_digit)
      .folding(live_keep, live_keep_left, live_keep)
      .with_stack(ArrayDeque::<LiveFrame, U1>::new());
    let baseline = inp.trip_snapshot();
    ARMED.with(|armed| armed.set(true));
    let unwound = catch_unwind(AssertUnwindSafe(|| driver.parse_input(&mut inp))).is_err();
    let still_armed = ARMED.with(|armed| armed.replace(false));
    assert!(
      unwound && !still_armed,
      "{src}: the refused operator's destructor unwinds"
    );
    inp.tripped_during_attempt(baseline)
  }

  /// A full storage that refuses a pending operator counts its trip before it drops the operator,
  /// so a token's destructor that unwinds cannot leave the refusal uncounted: for a prefix
  /// operator, the `~` after a `-`, and for an infix one, the `#` after `1+`.
  #[test]
  fn a_refused_operator_whose_destructor_unwinds_leaves_its_trip_counted() {
    for src in ["-~1;", "1+2#3;"] {
      assert!(
        counts_the_trip_through_the_unwind(src),
        "{src}: the trip of the full storage is counted"
      );
    }
  }
}

/// A head parked under the capacity-0 cache, read around folds that peek it, or consume it and
/// park the token behind it, through a look-ahead. The driver keeps the head's classification by
/// its park id, so the look-ahead must give the copy it re-parks the id the head wore.
#[cfg(feature = "std")]
mod a_parked_head_across_look_aheads {
  use core::cell::Cell;
  use std::{format, thread_local};

  use super::*;

  thread_local! {
    /// The right-hand sides the counting table has classified on this thread.
    static CLASSIFIED: Cell<usize> = const { Cell::new(0) };
  }

  /// The suites' table, under a marker whose right-hand classifier counts its calls.
  struct Counted;

  const COUNTED: PrattTable<Counted, u8> = PrattTable::new();

  impl PrattToken<'_, Counted, u8> for Tok {
    fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), u8>> {
      <Self as PrattToken<'_, Expr, u8>>::try_pratt_lhs(self)
    }

    fn try_pratt_rhs(&self) -> Option<PrattRHS<(), (), (), (), u8>> {
      CLASSIFIED.with(|classified| classified.set(classified.get() + 1));
      <Self as PrattToken<'_, Expr, u8>>::try_pratt_rhs(self)
    }
  }

  type ParkedIr<'inp, 'c> = InputRef<'inp, 'c, ByteLexer<'inp, true>, (Fatal<Fail>, ())>;

  /// A digit, as its value.
  fn digit(input: &mut ParkedIr<'_, '_>) -> Result<ParseAttempt<u32>, Fail> {
    Ok(
      match input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
        Some(tok) => ParseAttempt::Accept(u32::from(tok.into_data().0 - b'0')),
        None => ParseAttempt::Decline,
      },
    )
  }

  /// A prefix fold that peeks the head through a look-ahead before it keeps its operand.
  fn peek_then_keep(
    input: &mut ParkedIr<'_, '_>,
    operand: u32,
    _: Precedenced<Op, u8>,
  ) -> Result<u32, Fail> {
    let _ = input.look_ahead(|input| input.peek_one().map(|head| head.is_some()));
    Ok(operand)
  }

  /// A prefix fold whose look-ahead consumes the head and declines the token behind it, which
  /// parks that token in the slot the head left, under the next park id.
  fn consume_then_decline(
    input: &mut ParkedIr<'_, '_>,
    operand: u32,
    _: Precedenced<Op, u8>,
  ) -> Result<u32, Fail> {
    let _ = input.look_ahead(|input| -> Result<bool, Fail> {
      let _ = input.next()?;
      Ok(input.try_expect(|_| false)?.is_none())
    });
    Ok(operand)
  }

  fn keep(_: &mut ParkedIr<'_, '_>, operand: u32, _: Precedenced<Op, u8>) -> Result<u32, Fail> {
    Ok(operand)
  }

  fn keep_left(
    _: &mut ParkedIr<'_, '_>,
    left: u32,
    _: u32,
    _: Precedenced<PrattInfix<Op, Op, Op>, u8>,
  ) -> Result<u32, Fail> {
    Ok(left)
  }

  /// `folding` over N prefix operators, `1` and `x` under the capacity-0 cache, with prefix folds
  /// that each peek the head through a look-ahead: its value, and the right-hand sides it
  /// classified.
  fn over_parked(n: usize) -> (Result<u32, Fail>, usize) {
    let src = format!("{}1x", "-".repeat(n));
    let mut input = Input::<ByteLexer<'_, true>, (Fatal<Fail>, ()), ()>::with_state_and_context(
      src.as_str(),
      (),
      InputContext::new(Fatal::<Fail>::new(), ()),
    );
    CLASSIFIED.with(|classified| classified.set(0));
    let result = flat(COUNTED, digit)
      .folding(peek_then_keep, keep_left, keep)
      .parse_input(&mut input.as_ref());
    (result, CLASSIFIED.with(Cell::get))
  }

  /// `folding` over N prefix operators, `1`, `x` and `y` under the capacity-0 cache, with prefix
  /// folds whose look-aheads each consume the parked `x` and park `y`: its value, and the
  /// right-hand sides it classified.
  fn over_parked_behind(n: usize) -> (Result<u32, Fail>, usize) {
    let src = format!("{}1xy", "-".repeat(n));
    let mut input = Input::<ByteLexer<'_, true>, (Fatal<Fail>, ()), ()>::with_state_and_context(
      src.as_str(),
      (),
      InputContext::new(Fatal::<Fail>::new(), ()),
    );
    CLASSIFIED.with(|classified| classified.set(0));
    let result = flat(COUNTED, digit)
      .folding(consume_then_decline, keep_left, keep)
      .parse_input(&mut input.as_ref());
    (result, CLASSIFIED.with(Cell::get))
  }

  /// The head `x` is parked, and it is classified once, however many folds peek it. A look-ahead
  /// that gave its copy a new park id on every return had the driver classify the unchanged head
  /// after every fold, N + 1 times.
  #[test]
  fn a_parked_head_is_classified_once_across_folds_that_peek_it_through_a_look_ahead() {
    for n in [16, 64] {
      let (result, calls) = over_parked(n);
      assert_eq!(result, Ok(1), "{n}");
      assert!(
        calls <= 2,
        "{n} prefix operators, each fold peeking the parked head: {calls} classifications"
      );
    }
  }

  /// The head `x` is parked, and it is classified once, however many folds consume it and park
  /// `y` in its place inside a look-ahead. `y` takes a new park id in every fold. A settle that
  /// read that as the head's identity moving gave the copy of `x` one more, and the driver
  /// classified the unchanged head after every fold, N + 1 times.
  #[test]
  fn a_parked_head_is_classified_once_across_folds_that_park_the_token_behind_it() {
    for n in [16, 64] {
      let (result, calls) = over_parked_behind(n);
      assert_eq!(result, Ok(1), "{n}");
      assert!(
        calls <= 2,
        "{n} prefix operators, each fold parking the token behind the head: {calls} \
         classifications"
      );
    }
  }
}
