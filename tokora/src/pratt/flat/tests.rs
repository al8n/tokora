//! A kept driver in `Checked` mode reuses its stack, and holds nothing after a parse returns,
//! whichever way it returns. A parse that unwinds leaves its frames, and the next one's start
//! removes them.

use core::{cell::RefCell, panic::AssertUnwindSafe};
use std::{format, panic, string::String, thread_local, vec, vec::Vec};

use crate::{
  InputRef, Lexer, ParseAttempt, ReadFrontier, ScanLookahead, SimpleSpan, Token,
  cache::DefaultCache,
  emitter::Fatal,
  error::{
    NonAssociativeChain, PrattStackFull, UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot,
    UnexpectedToken,
  },
  input::{Input, InputContext},
  pratt::{PrattInfix, PrattLHS, PrattPower, PrattRHS, PrattTable, Precedenced, flat},
  span::Spanned,
  token::PrattToken,
};

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

/// One token for each byte.
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
    let byte = *self.src.as_bytes().get(self.pos)?;
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

/// Digits are operands, `-` is a prefix operator, `+` is left- and `^` right-associative.
impl PrattToken<'_, Expr, u8> for Tok {
  fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), u8>> {
    match self.0 {
      b'0'..=b'9' => Some(PrattLHS::Operand(())),
      b'-' => Some(PrattLHS::Prefix(Precedenced::new((), 9))),
      _ => None,
    }
  }

  fn try_pratt_rhs(&self) -> Option<PrattRHS<(), (), (), (), u8>> {
    match self.0 {
      b'+' => Some(PrattRHS::Infix(Precedenced::new(PrattInfix::Left(()), 1))),
      b'^' => Some(PrattRHS::Infix(Precedenced::new(PrattInfix::Right(()), 2))),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Fail {
  Lex,
  Unexpected,
  Driver,
  Handler,
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

impl<O, Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEot<O, Lang, Set>> for Fail {
  fn from(_: UnexpectedEot<O, Lang, Set>) -> Self {
    Fail::Driver
  }
}

impl<O, Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEoLhs<O, Lang, Set>> for Fail {
  fn from(_: UnexpectedEoLhs<O, Lang, Set>) -> Self {
    Fail::Driver
  }
}

impl<O, Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEoRhs<O, Lang, Set>> for Fail {
  fn from(_: UnexpectedEoRhs<O, Lang, Set>) -> Self {
    Fail::Driver
  }
}

impl<O, Lang: ?Sized> From<NonAssociativeChain<O, Lang>> for Fail {
  fn from(_: NonAssociativeChain<O, Lang>) -> Self {
    Fail::Driver
  }
}

impl<O, Lang: ?Sized> From<PrattStackFull<O, Lang>> for Fail {
  fn from(_: PrattStackFull<O, Lang>) -> Self {
    Fail::Driver
  }
}

type Lex = ByteLexer<'static>;
type Ctx = (Fatal<Fail>, DefaultCache<'static, Lex>);
type Ir<'c> = InputRef<'static, 'c, Lex, Ctx>;

/// Runs `f` over an input of `src`.
fn over<O>(src: &'static str, f: impl FnOnce(&mut Ir<'_>) -> O) -> O {
  let mut input = Input::<Lex, Ctx, ()>::with_state_and_context(
    src,
    (),
    InputContext::new(
      Fatal::<Fail>::new(),
      DefaultCache::<'static, Lex>::default(),
    ),
  );
  let mut inp = input.as_ref();
  f(&mut inp)
}

/// Accepts a digit, and declines anything else.
fn digit(inp: &mut Ir<'_>) -> Result<ParseAttempt<u8>, Fail> {
  Ok(match inp.try_expect(|tok| tok.data().0.is_ascii_digit())? {
    Some(tok) => ParseAttempt::Accept(tok.into_data().0),
    None => ParseAttempt::Decline,
  })
}

/// Writes each item down, and fails or panics at the item numbered `fail_at` or `panic_at`.
struct Probe {
  items: Vec<String>,
  fail_at: usize,
  panic_at: usize,
}

impl Probe {
  fn new(fail_at: usize, panic_at: usize) -> Self {
    Self {
      items: Vec::new(),
      fail_at,
      panic_at,
    }
  }

  fn tell(&mut self, item: String) -> Result<(), Fail> {
    self.items.push(item);
    let told = self.items.len();
    if told == self.panic_at {
      panic!("the probe panics at item {told}, as asked");
    }
    if told == self.fail_at {
      return Err(Fail::Handler);
    }
    Ok(())
  }
}

impl crate::pratt::PrattHandler<'static, u8, Lex, u8, Ctx> for Probe {
  fn on_prefix(
    &mut self,
    _input: &mut InputRef<'static, '_, Lex, Ctx>,
    operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), Fail> {
    self.tell(format!(
      "pre {}",
      char::from(operator.into_data().into_data().0)
    ))
  }

  fn on_operand(
    &mut self,
    _input: &mut InputRef<'static, '_, Lex, Ctx>,
    operand: u8,
  ) -> Result<(), Fail> {
    self.tell(format!("operand {}", char::from(operand)))
  }

  fn on_infix(
    &mut self,
    _input: &mut InputRef<'static, '_, Lex, Ctx>,
    operator: Precedenced<
      PrattInfix<Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>>,
      u8,
    >,
  ) -> Result<(), Fail> {
    let (PrattInfix::Left(tok) | PrattInfix::Right(tok) | PrattInfix::Neither(tok)) =
      operator.into_data();
    self.tell(format!("infix {}", char::from(tok.into_data().0)))
  }

  fn on_postfix(
    &mut self,
    _input: &mut InputRef<'static, '_, Lex, Ctx>,
    operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
  ) -> Result<(), Fail> {
    self.tell(format!(
      "post {}",
      char::from(operator.into_data().into_data().0)
    ))
  }
}

/// One driver, three parses: a handler that fails in the middle of a right-associative chain, one
/// that panics there, and a valid parse. The first leaves nothing on the stack; the unwind leaves
/// its frames, which the third parse's start removes; and the third tells every item, in order,
/// and leaves nothing.
#[test]
fn a_kept_checked_driver_holds_nothing_once_a_parse_returns() {
  const CHAIN: &str = "1^2^3^4^5^6^7^8;";
  let mut driver = flat(TABLE, digit);

  // The ninth item is the operand `5`, with four `^` pending.
  let mut failing = Probe::new(9, usize::MAX);
  let failed = over(CHAIN, |inp| driver.parse_into(inp, &mut failing));
  assert_eq!(failed, Err(Fail::Handler));
  assert_eq!(failing.items.len(), 9);
  assert_eq!(driver.mode.held(), 0, "an `Err` empties the stack");

  // The eleventh item is the operand `6`, with five `^` pending: past the four inline frames.
  let mut panicking = Probe::new(usize::MAX, 11);
  let unwound = panic::catch_unwind(AssertUnwindSafe(|| {
    over(CHAIN, |inp| driver.parse_into(inp, &mut panicking))
  }));
  assert!(unwound.is_err(), "the probe panicked");
  assert_eq!(
    driver.mode.held(),
    5,
    "an unwind runs no exit, so the frames pending at it stay"
  );

  let mut valid = Probe::new(usize::MAX, usize::MAX);
  let parsed = over("-1+2^3;", |inp| driver.parse_into(inp, &mut valid));
  assert_eq!(parsed, Ok(()));
  assert_eq!(
    valid.items,
    vec![
      "pre -",
      "operand 1",
      "infix +",
      "operand 2",
      "infix ^",
      "operand 3"
    ]
  );
  assert_eq!(driver.mode.held(), 0, "an `Ok` empties the stack too");
}

thread_local! {
  /// Each clone and each drop of a `Logged` power on this thread, in order, with its rank.
  static LOG: RefCell<Vec<(char, u8)>> = const { RefCell::new(Vec::new()) };
}

/// A power that writes down each clone of itself, as `'c'`, and each drop, as `'d'`.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Logged(u8);

impl Clone for Logged {
  fn clone(&self) -> Self {
    LOG.with_borrow_mut(|log| log.push(('c', self.0)));
    Logged(self.0)
  }
}

impl Drop for Logged {
  fn drop(&mut self) {
    LOG.with_borrow_mut(|log| log.push(('d', self.0)));
  }
}

impl PrattPower for Logged {}

/// The marker of a table whose powers are `Logged`.
struct Logs;

const LOGGED: PrattTable<Logs, Logged> = PrattTable::new();

/// Digits are operands, and `-` is a prefix operator at 9.
impl PrattToken<'_, Logs, Logged> for Tok {
  fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), Logged>> {
    match self.0 {
      b'0'..=b'9' => Some(PrattLHS::Operand(())),
      b'-' => Some(PrattLHS::Prefix(Precedenced::new((), Logged(9)))),
      _ => None,
    }
  }

  fn try_pratt_rhs(&self) -> Option<PrattRHS<(), (), (), (), Logged>> {
    None
  }
}

/// Panics when it is told of a prefix operator.
struct PanicsAtPrefix;

impl crate::pratt::PrattHandler<'static, Logged, Lex, u8, Ctx> for PanicsAtPrefix {
  fn on_prefix(
    &mut self,
    _input: &mut InputRef<'static, '_, Lex, Ctx>,
    _operator: Precedenced<Spanned<Tok, SimpleSpan>, Logged>,
  ) -> Result<(), Fail> {
    panic!("the handler panics at a prefix operator, as asked");
  }

  fn on_operand(
    &mut self,
    _input: &mut InputRef<'static, '_, Lex, Ctx>,
    _operand: u8,
  ) -> Result<(), Fail> {
    Ok(())
  }

  fn on_infix(
    &mut self,
    _input: &mut InputRef<'static, '_, Lex, Ctx>,
    _operator: Precedenced<
      PrattInfix<Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>, Spanned<Tok, SimpleSpan>>,
      Logged,
    >,
  ) -> Result<(), Fail> {
    Ok(())
  }

  fn on_postfix(
    &mut self,
    _input: &mut InputRef<'static, '_, Lex, Ctx>,
    _operator: Precedenced<Spanned<Tok, SimpleSpan>, Logged>,
  ) -> Result<(), Fail> {
    Ok(())
  }
}

/// A parse drops the frames an unwound one left before it clones the driver's floor for its
/// machine: no code of the new floor's runs while a power of the stale frames is still held.
#[test]
fn a_parse_drops_the_frames_an_unwound_one_left_before_it_clones_the_floor() {
  let mut driver = flat(LOGGED, digit);
  let unwound = panic::catch_unwind(AssertUnwindSafe(|| {
    over("-1;", |inp| driver.parse_into(inp, &mut PanicsAtPrefix))
  }));
  assert!(unwound.is_err(), "the handler panicked");
  assert_eq!(driver.mode.held(), 1, "the frame of the `-` stays");

  LOG.with_borrow_mut(Vec::clear);
  let parsed = over("1;", |inp| driver.parse_into(inp, &mut ()));
  assert_eq!(parsed, Ok(()));
  let log = LOG.take();
  let stale_dropped = log.iter().rposition(|&event| event == ('d', 9));
  let floor_cloned = log.iter().position(|&event| event == ('c', 0));
  assert!(
    matches!(
      (stale_dropped, floor_cloned),
      (Some(dropped), Some(cloned)) if dropped < cloned
    ),
    "the stale `-`'s power is dropped before the floor is cloned: {log:?}"
  );
}
