//! The token-level driver against the recursive body it replaced, and alone.
//!
//! The oracle is that body, kept in `oracle.rs`. Where the crate has `std`, the two are run over
//! generated token lists and every outcome is compared: the value or the error, with its offset,
//! its depth and its limit; the folds, the emissions and the lexer errors the input reported, as
//! one ordered log, each with the spans or the offset it was handed; the position the input is
//! left at; the token after it; and the token budget's charge. The loop runs with segments of one,
//! two and four frames, the segment's size being a type parameter of the crate-private loop, and
//! every one of those is the oracle's. The cells that name `core` alone run in every tier.

use hybrid_arraydeque::{ArraySize, typenum::U4};

use crate::{
  Lexer, ReadFrontier, ScanLookahead, SimpleSpan, Token,
  cache::DefaultCache,
  emitter::{EmitterView, Fatal},
  error::{
    NonAssociativeChain, RecursionLimitReached, UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot,
    UnexpectedToken,
  },
  input::{Input, InputContext},
  pratt::{PrattInfix, PrattLHS, PrattRHS, Precedenced},
  span::Spanned,
  state::RecursionLimiter,
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
    false
  }
}

/// One token for each byte. A space is skipped, and `$` is a lexer error.
struct ByteLexer<'inp> {
  src: &'inp str,
  tok_start: usize,
  pos: usize,
  state: (),
}

impl<'inp> Lexer<'inp> for ByteLexer<'inp> {
  type State = ();
  type Source = str;
  type Token = Tok;
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

  fn lex(&mut self) -> Option<Result<Tok, LexErr>> {
    let bytes = self.src.as_bytes();
    while bytes.get(self.pos) == Some(&b' ') {
      self.pos += 1;
    }
    let byte = *bytes.get(self.pos)?;
    self.tok_start = self.pos;
    self.pos += 1;
    Some(if byte == b'$' {
      Err(LexErr)
    } else {
      Ok(Tok(byte))
    })
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

fn infix(associativity: PrattInfix<(), (), ()>, power: u8) -> Option<PrattRHS<(), (), (), (), u8>> {
  Some(PrattRHS::Infix(Precedenced::new(associativity, power)))
}

fn postfix(power: u8) -> Option<PrattRHS<(), (), (), (), u8>> {
  Some(PrattRHS::Postfix(Precedenced::new((), power)))
}

fn adjacent(power: u8) -> Option<PrattRHS<(), (), (), (), u8>> {
  Some(PrattRHS::Adjacent(Precedenced::new((), power)))
}

/// | byte | left-hand side | right-hand side |
/// |---|---|---|
/// | `0` to `9` | operand | |
/// | `-` | prefix, 40 | infix, left, 10 |
/// | `~` | prefix, 12 | |
/// | `_` | prefix, 3 | |
/// | `+` | | infix, left, 10 |
/// | `&` | | infix, right, 10 |
/// | `\|` | | infix, non-associative, 10 |
/// | `*` | | infix, left, 20 |
/// | `^` | | infix, right, 30 |
/// | `=` | | infix, right, 5 |
/// | `<` | | infix, non-associative, 15 |
/// | `>` | | infix, left, 15 |
/// | `!` | | postfix, 50 |
/// | `?` | | postfix, 12 |
/// | `%` | | postfix, 15 |
/// | `,` | | adjacent, 10 |
/// | `@` | | adjacent, 15 |
///
/// A prefix operator above most infix operators, one below some, and one below all. A postfix
/// operator above every infix operator, one below some, and one at a non-associative power, which
/// tells "a postfix operator ignores the latch" from its opposite. Two operators of different
/// associativity at each of two powers, and a zero-token operator at each.
impl PrattToken<'_, Expr, u8> for Tok {
  fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), u8>> {
    match self.0 {
      b'0'..=b'9' => Some(PrattLHS::Operand(())),
      b'-' => Some(PrattLHS::Prefix(Precedenced::new((), 40))),
      b'~' => Some(PrattLHS::Prefix(Precedenced::new((), 12))),
      b'_' => Some(PrattLHS::Prefix(Precedenced::new((), 3))),
      _ => None,
    }
  }

  fn try_pratt_rhs(&self) -> Option<PrattRHS<(), (), (), (), u8>> {
    match self.0 {
      b'+' | b'-' => infix(PrattInfix::Left(()), 10),
      b'&' => infix(PrattInfix::Right(()), 10),
      b'|' => infix(PrattInfix::Neither(()), 10),
      b'*' => infix(PrattInfix::Left(()), 20),
      b'^' => infix(PrattInfix::Right(()), 30),
      b'=' => infix(PrattInfix::Right(()), 5),
      b'<' => infix(PrattInfix::Neither(()), 15),
      b'>' => infix(PrattInfix::Left(()), 15),
      b'!' => postfix(50),
      b'?' => postfix(12),
      b'%' => postfix(15),
      b',' => adjacent(10),
      b'@' => adjacent(15),
      _ => None,
    }
  }
}

// ── The error, the folds and the input ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fail {
  Lex,
  Unexpected,
  Eot {
    at: usize,
  },
  EoLhs {
    at: usize,
  },
  EoRhs {
    at: usize,
  },
  Chain {
    at: usize,
  },
  Limit {
    at: usize,
    depth: usize,
    limitation: usize,
  },
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

impl<Lang: ?Sized> From<RecursionLimitReached<usize, Lang>> for Fail {
  fn from(e: RecursionLimitReached<usize, Lang>) -> Self {
    Fail::Limit {
      at: e.offset(),
      depth: e.depth(),
      limitation: e.limitation(),
    }
  }
}

/// A token, as a fold is handed one and returns one.
type Op = Spanned<Tok, SimpleSpan>;

/// The value of a sub-expression: a token that stands for it, over its operator and operands.
fn over(spans: &[SimpleSpan]) -> Op {
  let start = spans.iter().map(SimpleSpan::start).min().unwrap_or(0);
  let end = spans.iter().map(SimpleSpan::end).max().unwrap_or(0);
  Spanned::new(SimpleSpan::new(start, end), Tok(b'#'))
}

fn fold_prefix<'inp, E>(
  operator: Op,
  operand: Op,
  _: EmitterView<'_, 'inp, ByteLexer<'inp>, E>,
) -> Result<Op, Fail> {
  #[cfg(feature = "std")]
  log::push(log::Event::Prefix(operator.span(), operand.span()));
  Ok(over(&[operator.span(), operand.span()]))
}

fn fold_infix<'inp, E>(
  left: Op,
  right: Op,
  operator: Spanned<PrattInfix<Tok, Tok, Tok>, SimpleSpan>,
  _: EmitterView<'_, 'inp, ByteLexer<'inp>, E>,
) -> Result<Op, Fail> {
  #[cfg(feature = "std")]
  log::push(log::Event::Infix(
    match operator.data() {
      PrattInfix::Left(_) => b'L',
      PrattInfix::Right(_) => b'R',
      PrattInfix::Neither(_) => b'N',
    },
    left.span(),
    right.span(),
    operator.span(),
  ));
  Ok(over(&[left.span(), right.span(), operator.span()]))
}

fn fold_postfix<'inp, E>(
  operand: Op,
  operator: Op,
  _: EmitterView<'_, 'inp, ByteLexer<'inp>, E>,
) -> Result<Op, Fail> {
  #[cfg(feature = "std")]
  log::push(log::Event::Postfix(operand.span(), operator.span()));
  Ok(over(&[operand.span(), operator.span()]))
}

type FatalCtx<'inp> = (Fatal<Fail>, DefaultCache<'inp, ByteLexer<'inp>>);

/// What a parse under a fatal emitter returned, and where it left the input.
type Fatally = (Result<Option<Op>, Fail>, usize);

/// Parses `src` under a budget of `limit` and a fatal emitter, with the loop over segments of `K`
/// frames, or with the oracle.
fn fatally<K: ArraySize>(src: &str, limit: usize, oracle: bool) -> Fatally {
  let mut input = Input::<ByteLexer<'_>, FatalCtx<'_>, ()>::with_state_and_context(
    src,
    (),
    InputContext::new(Fatal::<Fail>::new(), DefaultCache::default())
      .with_recursion_limiter(RecursionLimiter::with_limitation(limit)),
  );
  let mut inp = input.as_ref();
  let result = if oracle {
    inp.pratt_oracle::<_, _, _, Expr, u8>(
      fold_prefix::<Fatal<Fail>>,
      fold_infix::<Fatal<Fail>>,
      fold_postfix::<Fatal<Fail>>,
      0,
    )
  } else {
    inp.pratt_in::<_, _, _, Expr, u8, K>(
      0,
      &mut fold_prefix::<Fatal<Fail>>,
      &mut fold_infix::<Fatal<Fail>>,
      &mut fold_postfix::<Fatal<Fail>>,
    )
  };
  (result, inp.span().end())
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

// ── Every tier ────────────────────────────────────────────────────────────────────────────────

/// The budget is the one bound, in every tier: raised to 100, it holds 99 pending operators, the
/// expression's own level being the hundredth, and refuses the next as the recursive driver
/// refuses it, at the same offset, with the same depth and limit. With no allocator the loop holds
/// them in segments of four on the native stack, as it does with one.
#[test]
fn a_budget_of_a_hundred_holds_ninety_nine_pending_operators_and_refuses_the_next() {
  static FITS: [u8; 101] = prefixed();
  static PAST: [u8; 102] = prefixed();
  let fits = fatally::<U4>(text(&FITS), 100, false);
  assert_eq!(
    fits,
    (
      Ok(Some(Spanned::new(SimpleSpan::new(0, 100), Tok(b'#')))),
      100
    )
  );
  assert_eq!(fits, fatally::<U4>(text(&FITS), 100, true));
  let past = fatally::<U4>(text(&PAST), 100, false);
  assert_eq!(
    past,
    (
      Err(Fail::Limit {
        at: 100,
        depth: 101,
        limitation: 100
      }),
      100
    )
  );
  assert_eq!(past, fatally::<U4>(text(&PAST), 100, true));
}

/// Counts occurrences of `needle` on the lines of `hay` that are not comments.
fn count(hay: &str, needle: &str) -> usize {
  hay
    .lines()
    .filter(|line| !line.trim_start().starts_with("//"))
    .map(|line| line.matches(needle).count())
    .sum()
}

/// The production file takes the budget's level once, for the expression's own, and asks the
/// native stack for room once, where it makes a segment. The oracle is a file of its own, so the
/// count is of the loop alone; and the oracle takes a level for each frame, with no `stacker`
/// segment, as the recursive driver took one.
#[test]
#[cfg_attr(
  miri,
  ignore = "reads crate source and string-matches: no UB surface, and miri interprets every byte"
)]
fn the_driver_takes_one_level_and_asks_for_the_native_stack_once() {
  const DRIVER: &str = include_str!("../pratt.rs");
  const ORACLE: &str = include_str!("oracle.rs");
  assert_eq!(count(DRIVER, ".descend("), 1, "the expression's own level");
  assert_eq!(
    count(DRIVER, "maybe_grow("),
    1,
    "in `pratt_grow`, around each new segment"
  );
  assert_eq!(count(ORACLE, ".descend("), 1, "one for each frame");
  assert_eq!(count(ORACLE, "maybe_grow("), 0);
}

// ── With `std`: the differential, and the native stack ────────────────────────────────────────

/// The ordered log: what the folds were handed, what was emitted, and what the input reported.
#[cfg(feature = "std")]
mod log {
  use core::cell::RefCell;

  use crate::SimpleSpan;

  /// One entry.
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub(super) enum Event {
    /// A prefix fold: the operator's span, and the operand's.
    Prefix(SimpleSpan, SimpleSpan),
    /// An infix fold: its associativity, and the spans of its left operand, right operand and
    /// operator.
    Infix(u8, SimpleSpan, SimpleSpan, SimpleSpan),
    /// A postfix fold: the operand's span, and the operator's.
    Postfix(SimpleSpan, SimpleSpan),
    /// A missing left-hand side, emitted at its offset.
    EoLhs(usize),
    /// A missing right-hand side, emitted at its offset.
    EoRhs(usize),
    /// A lexer error the input reported, over its span.
    Lex(SimpleSpan),
  }

  std::thread_local! {
    static LOG: RefCell<Vec<Event>> = const { RefCell::new(Vec::new()) };
  }

  pub(super) fn push(event: Event) {
    LOG.with(|log| log.borrow_mut().push(event));
  }

  /// The log so far, which it empties.
  pub(super) fn take() -> Vec<Event> {
    LOG.with(|log| core::mem::take(&mut *log.borrow_mut()))
  }
}

#[cfg(feature = "std")]
mod with_std {
  use hybrid_arraydeque::{
    ArraySize,
    typenum::{U1, U2},
  };

  use super::{ByteLexer, Expr, Fail, Op, Tok, fold_infix, fold_postfix, fold_prefix, log};
  use crate::{
    InputRef, SimpleSpan,
    cache::{Cache, DefaultCache},
    emitter::{Emitter, PrattEmitter},
    error::{UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedTokenOf},
    input::{Cursor, Input, InputContext},
    span::Spanned,
    state::RecursionLimiter,
  };

  /// An emitter that writes what it is handed into the ordered log and goes on.
  struct Logged;

  impl<'inp> Emitter<'inp, ByteLexer<'inp>> for Logged {
    type Error = Fail;

    fn emit_lexer_error(&mut self, err: Spanned<super::LexErr, SimpleSpan>) -> Result<(), Fail> {
      log::push(log::Event::Lex(err.span()));
      Ok(())
    }

    fn emit_unexpected_token(
      &mut self,
      _: UnexpectedTokenOf<'inp, ByteLexer<'inp>>,
    ) -> Result<(), Fail> {
      Err(Fail::Unexpected)
    }

    fn emit_error(&mut self, err: Spanned<Fail, SimpleSpan>) -> Result<(), Fail> {
      Err(err.into_data())
    }

    fn rewind(&mut self, _: &Cursor<'inp, '_, ByteLexer<'inp>>, _: u64) {}
  }

  impl<'inp> PrattEmitter<'inp, ByteLexer<'inp>> for Logged {
    fn emit_unexpected_end_of_lhs(&mut self, err: UnexpectedEoLhs<usize>) -> Result<(), Fail> {
      log::push(log::Event::EoLhs(err.offset()));
      Ok(())
    }

    fn emit_unexpected_end_of_rhs(&mut self, err: UnexpectedEoRhs<usize>) -> Result<(), Fail> {
      log::push(log::Event::EoRhs(err.offset()));
      Ok(())
    }
  }

  /// Everything one parse did that the other driver must have done alike.
  #[derive(Debug, Clone, PartialEq, Eq)]
  struct Outcome {
    /// The value, or the error.
    result: Result<Option<Op>, Fail>,
    /// The folds, the emissions and the lexer errors, in order.
    log: Vec<log::Event>,
    /// `span().end()` once the parse has returned.
    at: usize,
    /// The token budget's charge.
    spent: usize,
    /// The token after the parse, read once the rest is recorded.
    next: Result<Option<Op>, Fail>,
  }

  type Ir<'inp, 'c, C> = InputRef<'inp, 'c, ByteLexer<'inp>, (Logged, C)>;

  /// Which driver a run is of.
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  enum Driver {
    /// The recursive body.
    Oracle,
    /// The loop, over segments of one, two or four frames; four is the public method.
    Loop(usize),
  }

  /// Parses `src` with `driver` under a budget of `limit`, none when `None`, at the floor `floor`,
  /// over the cache `C`.
  fn run<'inp, C>(src: &'inp str, driver: Driver, limit: Option<usize>, floor: u8) -> Outcome
  where
    C: Cache<'inp, ByteLexer<'inp>>,
  {
    let limiter = limit.map_or(
      RecursionLimiter::unlimited(),
      RecursionLimiter::with_limitation,
    );
    let mut input = Input::<ByteLexer<'_>, (Logged, C), ()>::with_state_and_context(
      src,
      (),
      InputContext::new(Logged, C::new()).with_recursion_limiter(limiter),
    );
    let mut inp = input.as_ref();
    log::take();
    let result = match driver {
      Driver::Oracle => inp.pratt_oracle::<_, _, _, Expr, u8>(
        fold_prefix::<Logged>,
        fold_infix::<Logged>,
        fold_postfix::<Logged>,
        floor,
      ),
      Driver::Loop(1) => looped::<C, U1>(&mut inp, floor),
      Driver::Loop(2) => looped::<C, U2>(&mut inp, floor),
      Driver::Loop(_) => inp.pratt_with_min_precedence::<_, _, _, Expr, u8>(
        fold_prefix::<Logged>,
        fold_infix::<Logged>,
        fold_postfix::<Logged>,
        floor,
      ),
    };
    let log = log::take();
    let at = inp.span().end();
    let spent = inp.token_budget().spent();
    let next = inp.next();
    Outcome {
      result,
      log,
      at,
      spent,
      next,
    }
  }

  fn looped<'inp, C, K>(inp: &mut Ir<'inp, '_, C>, floor: u8) -> Result<Option<Op>, Fail>
  where
    C: Cache<'inp, ByteLexer<'inp>>,
    K: ArraySize,
  {
    inp.pratt_in::<_, _, _, Expr, u8, K>(
      floor,
      &mut fold_prefix::<Logged>,
      &mut fold_infix::<Logged>,
      &mut fold_postfix::<Logged>,
    )
  }

  // ── The generator ───────────────────────────────────────────────────────────────────────────

  /// splitmix64: small, deterministic, and no dependency.
  struct Rng(u64);

  impl Rng {
    fn draw(&mut self) -> u64 {
      self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
      let mut z = self.0;
      z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
      z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
      z ^ (z >> 31)
    }

    /// A number below `n`.
    fn below(&mut self, n: usize) -> usize {
      (self.draw() % n as u64) as usize
    }

    fn pick(&mut self, from: &[u8]) -> u8 {
      from[self.below(from.len())]
    }
  }

  const OPERANDS: &[u8] = b"0123456789";
  const PREFIXES: &[u8] = b"--~_";
  const INFIXES: &[u8] = b"+-&|*^=<>";
  const POSTFIXES: &[u8] = b"!?%";
  const ADJACENT: &[u8] = b",@";
  /// Everything a mutation may write: the table's rows, the terminator, a lexer error, a space.
  const ANY: &[u8] = b"0123456789-~_+&|*^=<>!?%,@;$ ";

  /// A well-formed expression, then up to three tokens deleted, inserted or replaced; spaces;
  /// and an ending: none, a terminator, or lexer errors before or after one.
  fn generate(rng: &mut Rng) -> String {
    let mut out = Vec::new();
    // A run of one infix operator now and then, so that chains keep many operators pending.
    let run = (rng.below(4) == 0).then(|| rng.pick(INFIXES));
    let terms = 1 + rng.below(if run.is_some() { 14 } else { 7 });
    for term in 0..terms {
      if term > 0 {
        out.push(match run {
          Some(operator) if rng.below(5) > 0 => operator,
          _ if rng.below(12) == 0 => rng.pick(ADJACENT),
          _ => rng.pick(INFIXES),
        });
      }
      let widest = if rng.below(6) == 0 { 7 } else { 3 };
      for _ in 0..rng.below(widest) {
        out.push(rng.pick(PREFIXES));
      }
      out.push(rng.pick(OPERANDS));
      for _ in 0..rng.below(2) {
        out.push(rng.pick(POSTFIXES));
      }
    }
    if rng.below(3) == 0 {
      for _ in 0..1 + rng.below(3) {
        let at = rng.below(out.len() + 1);
        match rng.below(3) {
          0 if at < out.len() => {
            out.remove(at);
          }
          1 => out.insert(at, rng.pick(ANY)),
          _ if at < out.len() => out[at] = rng.pick(ANY),
          _ => {}
        }
      }
    }
    for _ in 0..rng.below(4) {
      let at = rng.below(out.len() + 1);
      out.insert(at, b' ');
    }
    match rng.below(5) {
      0 => {}
      1 => out.extend_from_slice(b";"),
      2 => out.extend_from_slice(b" ;$"),
      3 => out.extend_from_slice(b"$ $"),
      _ => out.extend_from_slice(b"$;"),
    }
    String::from_utf8(out).expect("the alphabet is ASCII")
  }

  /// The recursion budgets: none, and four limits that the generated chains reach.
  const LIMITS: [Option<usize>; 5] = [None, Some(1), Some(2), Some(3), Some(5)];
  /// The floors: the default, one at the power of `+`, and one above every infix operator but `^`.
  const FLOORS: [u8; 3] = [0, 10, 16];

  /// Compares every driver with the oracle on one source, under every budget and floor, over the
  /// cache `C`, and counts the segments each size grew.
  fn compare<'inp, C>(src: &'inp str, grown: &mut [usize; 3])
  where
    C: Cache<'inp, ByteLexer<'inp>>,
  {
    for limit in LIMITS {
      for floor in FLOORS {
        let want = run::<C>(src, Driver::Oracle, limit, floor);
        super::super::probe::take();
        for (size, grew) in [1, 2, 4].into_iter().zip(grown.iter_mut()) {
          let got = run::<C>(src, Driver::Loop(size), limit, floor);
          *grew += super::super::probe::take().0;
          assert_eq!(
            got, want,
            "`{src}` under a budget of {limit:?} at the floor {floor}, in segments of {size}: the \
             loop's outcome, then the recursive driver's"
          );
        }
      }
    }
  }

  /// Every outcome the loop gives is the recursive driver's, over generated token lists, in
  /// segments of one, two and four frames, under the default cache and under the one that holds
  /// nothing, where a declined token waits in the parked slot. And the comparison cannot pass
  /// without growing: each size grows a segment somewhere in the set.
  #[test]
  fn every_outcome_of_the_loop_is_the_recursive_driver_s() {
    // Miri interprets each step, and `trace` writes a line for every read of the input: both
    // read a short set, which need not grow at every size.
    let whole = !cfg!(miri) && !cfg!(feature = "trace");
    let cases = if whole {
      1000
    } else if cfg!(miri) {
      2
    } else {
      40
    };
    let mut rng = Rng(0x0198_1701_7377_0001);
    let mut grown = [0usize; 3];
    for _ in 0..cases {
      let src = generate(&mut rng);
      compare::<DefaultCache<'_, ByteLexer<'_>>>(&src, &mut grown);
      compare::<()>(&src, &mut grown);
    }
    if whole {
      assert!(
        grown.iter().all(|&grew| grew > 0),
        "a segment size that never grew was compared on nothing but its first segment: {grown:?}"
      );
    }
  }

  /// The crossings a segment's boundary could break, each pinned by name in segments of one
  /// frame, where every push past the first makes a segment and every reduction that empties one
  /// returns: a non-associative chain refused through the latch a reduction wrote through the link;
  /// a missing operand met right after a return; and a staged operator offered again after a
  /// return, below the operator it waited for.
  #[test]
  fn the_crossings_of_a_segment_s_boundary_are_the_recursive_driver_s() {
    for src in [
      // `2 < 3` is reduced in the segment above, and the latch it writes through the link is the
      // `+` frame's, below, where the second `<` is then refused.
      "1 + 2 < 3 < 4",
      // `2 ^ 3` is reduced in the segment above and clears the `<` frame's latch through the link;
      // `1 < (2 ^ 3)` then latches the root, and the second `<` is refused there.
      "1 < 2 ^ 3 < 4",
      // The `^` frames empty their segments one at a time, and `+` then misses its operand.
      "1 ^ 2 ^ 3 + ;",
      // `*` waits for `^ ^` to reduce across two returns, and is then taken.
      "1 + 2 ^ 3 ^ 4 * 5",
      // A prefix run that crosses boundaries, then an operator below it.
      "- - - - - 1 = 2",
    ] {
      for limit in [None, Some(4)] {
        let want = run::<DefaultCache<'_, ByteLexer<'_>>>(src, Driver::Oracle, limit, 0);
        let got = run::<DefaultCache<'_, ByteLexer<'_>>>(src, Driver::Loop(1), limit, 0);
        assert_eq!(got, want, "`{src}` under a budget of {limit:?}");
      }
    }
  }

  /// `1 ^ 1 ^ … ^ 1`, right-associative, with `n` operators: each keeps the one before it pending.
  fn right_chain(n: usize) -> String {
    let mut src = String::from("1");
    for _ in 0..n {
      src.push_str("^1");
    }
    src
  }

  /// Under the default budget a deep right-associative chain is refused at the operator the
  /// recursive driver refuses, with the same depth and limit in the error.
  #[test]
  fn the_default_budget_refuses_a_deep_chain_where_the_recursive_driver_does() {
    let src = right_chain(RecursionLimiter::PARSE_DEFAULT_DEPTH + 8);
    let want = run_defaulted(&src, Driver::Oracle);
    let got = run_defaulted(&src, Driver::Loop(4));
    assert_eq!(got, want);
    assert!(
      matches!(
        got,
        Err(Fail::Limit { depth, limitation, .. })
          if limitation == RecursionLimiter::PARSE_DEFAULT_DEPTH && depth == limitation + 1
      ),
      "{got:?}"
    );
  }

  /// A parse under the input's default budget, which is the shipped one.
  fn run_defaulted(src: &str, driver: Driver) -> Result<Option<Op>, Fail> {
    let mut input =
      Input::<ByteLexer<'_>, (Logged, DefaultCache<'_, ByteLexer<'_>>), ()>::with_state_and_context(
        src,
        (),
        InputContext::new(Logged, DefaultCache::default()),
      );
    let mut inp = input.as_ref();
    match driver {
      Driver::Oracle => inp.pratt_oracle::<_, _, _, Expr, u8>(
        fold_prefix::<Logged>,
        fold_infix::<Logged>,
        fold_postfix::<Logged>,
        0,
      ),
      Driver::Loop(_) => inp.pratt::<_, _, _, Expr, u8>(
        fold_prefix::<Logged>,
        fold_infix::<Logged>,
        fold_postfix::<Logged>,
      ),
    }
  }

  /// The thread the crate's native-stack figures are stated on.
  const STACK: usize = 2 * 1024 * 1024;

  /// The chain of 100 under a budget of 128, on a thread of [`STACK`]: what it returns, the native
  /// frames it entered, and where each sat.
  fn on_a_2mib_thread(driver: Driver) -> (Result<Option<Op>, Fail>, usize, Vec<usize>) {
    std::thread::Builder::new()
      .stack_size(STACK)
      .spawn(move || {
        let src = right_chain(100);
        let outcome = run::<DefaultCache<'_, ByteLexer<'_>>>(&src, driver, Some(128), 0);
        let (frames, marks) = super::super::probe::take();
        (outcome.result, frames, marks)
      })
      .expect("a thread of 2 MiB is spawned")
      .join()
      .expect("the parse finished on the thread")
  }

  /// Under a budget of 128, a chain of 100 right-associative operators parses, and the loop enters
  /// a native frame 24 times: four pending operators in the method's own frame, and 96 more in
  /// segments of four. The recursive oracle enters 101, one for each operator and one for the
  /// root.
  #[test]
  fn a_chain_of_a_hundred_enters_twenty_four_native_frames_where_the_oracle_enters_101() {
    let (looped, segments, _) = on_a_2mib_thread(Driver::Loop(4));
    let (oracle, frames, _) = on_a_2mib_thread(Driver::Oracle);
    assert_eq!(looped, oracle);
    assert_eq!(
      looped,
      Ok(Some(Spanned::new(SimpleSpan::new(0, 201), Tok(b'#'))))
    );
    assert_eq!((segments, frames), (24, 101));
  }

  /// The native stack the loop takes for that chain is below the recursive oracle's: one
  /// segment's native frames against four of the oracle's, each the distance between the marks
  /// its frames left. Compared in whatever profile the test is built in, debug and release alike.
  ///
  /// The distance between two addresses is a stack measurement only while they are offsets into
  /// one native stack. Under Miri they are not, and under a sanitizer, or with flags the build
  /// script could not read, they need not be; there the reading is not taken, and the line on the
  /// process's real stderr says so.
  #[test]
  fn the_loop_takes_less_native_stack_than_the_oracle_for_each_four_levels() {
    let refusal = if cfg!(miri) {
      Some("Miri interprets the frames")
    } else if option_env!("TOKORA_BUILD_SANITIZERS").is_some() {
      Some("the build is instrumented by a sanitizer")
    } else if option_env!("TOKORA_BUILD_FLAGS_OPAQUE").is_some() {
      Some("the build's flags could not be read")
    } else {
      None
    };
    if let Some(why) = refusal {
      use std::io::Write as _;
      let _ = writeln!(
        std::io::stderr().lock(),
        "the_loop_takes_less_native_stack_than_the_oracle_for_each_four_levels: no reading taken: {why}"
      );
      return;
    }
    let (_, _, segments) = on_a_2mib_thread(Driver::Loop(4));
    let (_, _, frames) = on_a_2mib_thread(Driver::Oracle);
    let per_segment: Vec<usize> = segments
      .windows(2)
      .map(|pair| pair[0].abs_diff(pair[1]))
      .collect();
    // From the first frame below the root on: the root is entered from another function.
    let per_four: Vec<usize> = frames[1..]
      .windows(5)
      .map(|five| five[0].abs_diff(five[4]))
      .collect();
    let widest = per_segment
      .iter()
      .copied()
      .max()
      .expect("the chain grew segments");
    let narrowest = per_four
      .iter()
      .copied()
      .min()
      .expect("the oracle entered its frames");
    assert!(
      widest < narrowest,
      "one segment's native frames take up to {widest} bytes, and four of the oracle's as few as \
       {narrowest}"
    );
  }
}
