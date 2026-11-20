//! The fixture the flat driver's suites and the table channels' suite share: a lexer of one byte
//! for each token, the operator table over it, an error type that keeps what the drivers report,
//! an operand parser, and the typed driver's folds. It names nothing of the flat driver's.

// Each suite uses a part of it.
#![allow(dead_code)]

use core::cell::RefCell;

use tokora::{
  Emitter, InputRef, Lexer, ParseAttempt, ParseContext, ParserContext, ReadFrontier, ScanLookahead,
  SimpleSpan, Token, TryParseInput,
  cache::PeekedTokenExt as _,
  emitter::{Fatal, Verbose},
  error::{
    MaybeIncomplete, MaybeTerminal, NonAssociativeChain, PrattStackFull, RecursionLimitReached,
    UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot, UnexpectedToken,
  },
  pratt::{PrattInfix, PrattLHS, PrattRHS, PrattTable, Precedenced},
  span::Spanned,
  state::{State, TokenLimiter},
  token::PrattToken,
  utils::typenum::U2,
};

// ── The token and the lexer ───────────────────────────────────────────────────────────────────

/// One byte of the source. A space is trivia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tok(pub u8);

/// The lexer's one error: a `$`, which no token stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexErr;

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

/// The lexer's state: a mode, which each lex records beside the offset of the token it lexed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mode(pub u8);

impl State for Mode {
  type Error = LexErr;

  fn check(&self) -> Result<(), LexErr> {
    Ok(())
  }
}

thread_local! {
  /// Every lex on this thread: the token's offset, and the mode it was lexed under.
  static LEXED: RefCell<Vec<(usize, u8)>> = const { RefCell::new(Vec::new()) };
}

/// Forgets the lexes recorded on this thread.
pub fn forget_lexes() {
  LEXED.with(|lexed| lexed.borrow_mut().clear());
}

/// The lexes recorded on this thread since they were last forgotten.
pub fn lexes() -> Vec<(usize, u8)> {
  LEXED.with(|lexed| lexed.borrow().clone())
}

/// Forgets the lexes recorded on this thread, and makes room for `n` more, so that recording them
/// allocates nothing.
pub fn reserve_lexes(n: usize) {
  LEXED.with(|lexed| {
    let mut lexed = lexed.borrow_mut();
    lexed.clear();
    lexed.reserve(n);
  });
}

/// One token for each byte, but `$`, which is a lexer error. With `SKIP`, spaces are skipped and no
/// token stands for them; without it, each space is a trivia token.
pub struct ByteLexer<'inp, const SKIP: bool> {
  src: &'inp str,
  tok_start: usize,
  pos: usize,
  state: Mode,
}

/// The lexer that skips trivia.
pub type Skip<'inp> = ByteLexer<'inp, true>;
/// The lexer that surfaces trivia as tokens.
pub type Surface<'inp> = ByteLexer<'inp, false>;

impl<'inp, const SKIP: bool> Lexer<'inp> for ByteLexer<'inp, SKIP> {
  type State = Mode;
  type Source = str;
  type Token = Tok;
  type Span = SimpleSpan;
  type Offset = usize;

  const SURFACES_TRIVIA: bool = !SKIP;

  fn new(src: &'inp str) -> Self {
    Self::with_state(src, Mode::default())
  }

  fn with_state(src: &'inp str, state: Mode) -> Self {
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
    let lexed = (self.tok_start, self.state.0);
    LEXED.with(|log| log.borrow_mut().push(lexed));
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

/// One token for each byte, trivia included, under a token limit the lexer's own state keeps: it
/// counts the tokens lexed, and reports the item past the limit as an error, which `check` then
/// confirms, and the input takes for a scanner trip. A rollback restores the count, and the latch
/// the trip set with it.
pub struct Limited<'inp> {
  src: &'inp str,
  tok_start: usize,
  pos: usize,
  state: TokenLimiter,
}

impl<'inp> Lexer<'inp> for Limited<'inp> {
  type State = TokenLimiter;
  type Source = str;
  type Token = Tok;
  type Span = SimpleSpan;
  type Offset = usize;

  const SURFACES_TRIVIA: bool = true;

  fn new(src: &'inp str) -> Self {
    Self::with_state(src, TokenLimiter::new())
  }

  fn with_state(src: &'inp str, state: TokenLimiter) -> Self {
    Self {
      src,
      tok_start: 0,
      pos: 0,
      state,
    }
  }

  fn check(&self) -> Result<(), LexErr> {
    self.state.check().map_err(|_| LexErr)
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
    self.state.increase();
    // A lexer reports its own limit: the item past it is an error, which the input confirms through
    // `check`, and takes for a trip. A token is never one.
    if self.state.check().is_err() {
      return Some(Err(LexErr));
    }
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

// ── The table ─────────────────────────────────────────────────────────────────────────────────

/// The marker of the suites' table.
pub struct Expr;

/// The suites' table.
pub const TABLE: PrattTable<Expr, u8> = PrattTable::new();

/// `-` as a prefix operator, above most infix operators.
pub const PREFIX_HIGH: u8 = 40;
/// `~` as a prefix operator, below some infix operators.
pub const PREFIX_LOW: u8 = 12;

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
/// | `+` | | infix, left, 10 |
/// | `&` | | infix, right, 10 |
/// | `*` | | infix, left, 20 |
/// | `^` | | infix, right, 30 |
/// | `=` | | infix, right, 5 |
/// | `<` | | infix, non-associative, 15 |
/// | `>` | | infix, left, 15 |
/// | `!` | | postfix, 50 |
/// | `?` | | postfix, 12 |
/// | `%` | | postfix, 15 |
/// | `@` | | adjacent, 15 |
impl PrattToken<'_, Expr, u8> for Tok {
  fn try_pratt_lhs(&self) -> Option<PrattLHS<(), (), u8>> {
    match self.0 {
      b'0'..=b'9' => Some(PrattLHS::Operand(())),
      b'-' => Some(PrattLHS::Prefix(Precedenced::new((), PREFIX_HIGH))),
      b'~' => Some(PrattLHS::Prefix(Precedenced::new((), PREFIX_LOW))),
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
      b'!' => postfix(50),
      b'?' => postfix(12),
      b'%' => postfix(15),
      b'@' => Some(PrattRHS::Adjacent(Precedenced::new((), 15))),
      _ => None,
    }
  }
}

// ── The error ─────────────────────────────────────────────────────────────────────────────────

/// Keeps what the drivers report, so a cell can assert the offset and the terminal mark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fail {
  Lex,
  Unexpected,
  Eot {
    at: usize,
    terminal: bool,
  },
  EoLhs {
    at: usize,
    terminal: bool,
  },
  EoRhs {
    at: usize,
    terminal: bool,
  },
  Chain {
    at: usize,
  },
  Full {
    at: usize,
    capacity: usize,
  },
  Limit {
    at: usize,
  },
  /// An error a cell's own parser or handler raised.
  Own(&'static str),
}

impl Fail {
  /// Whether this is a missing operand, of either side, at `at`.
  pub fn is_missing_at(&self, at: usize) -> bool {
    matches!(self, Fail::EoLhs { at: a, .. } | Fail::EoRhs { at: a, .. } if *a == at)
  }
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
    Fail::Eot {
      at: e.offset(),
      terminal: e.is_terminal(),
    }
  }
}

impl<Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEoLhs<usize, Lang, Set>> for Fail {
  fn from(e: UnexpectedEoLhs<usize, Lang, Set>) -> Self {
    Fail::EoLhs {
      at: e.offset(),
      terminal: e.is_terminal(),
    }
  }
}

impl<Lang: ?Sized, Set: Clone + 'static> From<UnexpectedEoRhs<usize, Lang, Set>> for Fail {
  fn from(e: UnexpectedEoRhs<usize, Lang, Set>) -> Self {
    Fail::EoRhs {
      at: e.offset(),
      terminal: e.is_terminal(),
    }
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

impl<Lang: ?Sized> From<RecursionLimitReached<usize, Lang>> for Fail {
  fn from(e: RecursionLimitReached<usize, Lang>) -> Self {
    Fail::Limit { at: e.offset() }
  }
}

/// Delegates: what the drivers marked terminal stays terminal.
impl MaybeTerminal for Fail {
  fn is_terminal(&self) -> bool {
    match self {
      Fail::Eot { terminal, .. } | Fail::EoLhs { terminal, .. } | Fail::EoRhs { terminal, .. } => {
        *terminal
      }
      Fail::Full { .. } | Fail::Limit { .. } => true,
      Fail::Lex | Fail::Unexpected | Fail::Chain { .. } | Fail::Own(_) => false,
    }
  }
}

impl MaybeIncomplete for Fail {}

// ── Contexts ──────────────────────────────────────────────────────────────────────────────────

/// A rejecting emitter over the default cache.
pub type FatalCtx<'inp, const SKIP: bool> = ParserContext<'inp, ByteLexer<'inp, SKIP>, Fatal<Fail>>;
/// A rejecting emitter over a cache of no capacity.
pub type BareCacheCtx<'inp, const SKIP: bool> =
  ParserContext<'inp, ByteLexer<'inp, SKIP>, Fatal<Fail>, ()>;
/// An accepting emitter over the default cache.
pub type VerboseCtx<'inp, const SKIP: bool> =
  ParserContext<'inp, ByteLexer<'inp, SKIP>, Verbose<Fail>>;

/// The input a closure over [`FatalCtx`] is handed.
pub type FatalIr<'inp, 'c, const SKIP: bool> =
  InputRef<'inp, 'c, ByteLexer<'inp, SKIP>, FatalCtx<'inp, SKIP>>;
/// The input a closure over [`BareCacheCtx`] is handed.
pub type BareCacheIr<'inp, 'c, const SKIP: bool> =
  InputRef<'inp, 'c, ByteLexer<'inp, SKIP>, BareCacheCtx<'inp, SKIP>>;
/// The input a closure over [`VerboseCtx`] is handed.
pub type VerboseIr<'inp, 'c, const SKIP: bool> =
  InputRef<'inp, 'c, ByteLexer<'inp, SKIP>, VerboseCtx<'inp, SKIP>>;
/// The input a closure over the limited lexer, with a rejecting emitter, is handed.
pub type LimitedIr<'inp, 'c> =
  InputRef<'inp, 'c, Limited<'inp>, ParserContext<'inp, Limited<'inp>, Fatal<Fail>>>;

// ── The operand parser ────────────────────────────────────────────────────────────────────────

/// A digit, as its own text. With `crossing` it crosses the trivia before the digit itself, and
/// `extra` is one more byte it takes as an operand. It declines, consuming nothing more, wherever
/// no operand begins.
#[derive(Debug, Clone, Copy)]
pub struct Operand {
  pub crossing: bool,
  pub extra: Option<u8>,
}

impl Operand {
  /// Digits alone, and no trivia crossed.
  pub const STRICT: Self = Self {
    crossing: false,
    extra: None,
  };
  /// Digits, with the trivia before them crossed.
  pub const CROSSING: Self = Self {
    crossing: true,
    extra: None,
  };
}

impl<'inp, const SKIP: bool, Ctx> TryParseInput<'inp, ByteLexer<'inp, SKIP>, String, Ctx>
  for Operand
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
{
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  ) -> Result<ParseAttempt<String>, <Ctx::Emitter as Emitter<'inp, ByteLexer<'inp, SKIP>>>::Error>
  {
    if self.crossing {
      input.skip_while(|tok| tok.is_trivia())?;
    }
    let extra = self.extra;
    let taken = input.try_expect(|tok| {
      let byte = tok.data().0;
      byte.is_ascii_digit() || Some(byte) == extra
    })?;
    Ok(match taken {
      Some(tok) => ParseAttempt::Accept(char::from(tok.into_data().0).to_string()),
      None => ParseAttempt::Decline,
    })
  }
}

/// Two digits, decided by a peek at both before either is read: an operand parser that declines
/// a window the input cut short.
#[derive(Debug, Clone, Copy)]
pub struct TwoDigits;

impl<'inp, const SKIP: bool, Ctx> TryParseInput<'inp, ByteLexer<'inp, SKIP>, String, Ctx>
  for TwoDigits
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
{
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  ) -> Result<ParseAttempt<String>, <Ctx::Emitter as Emitter<'inp, ByteLexer<'inp, SKIP>>>::Error>
  {
    let both = {
      let window = input.peek::<U2>()?;
      window.len() == 2 && window.iter().all(|tok| tok.token().0.is_ascii_digit())
    };
    if !both {
      return Ok(ParseAttempt::Decline);
    }
    let mut text = String::new();
    while let Some(tok) = input.try_expect(|tok| tok.data().0.is_ascii_digit())? {
      text.push(char::from(tok.into_data().0));
      if text.len() == 2 {
        break;
      }
    }
    Ok(ParseAttempt::Accept(text))
  }
}

/// Two digits, read inside an attempt that rolls back unless both are there: an operand parser
/// whose decline takes back everything its reads did to the input.
#[derive(Debug, Clone, Copy)]
pub struct Rolling;

impl<'inp, L, Ctx> TryParseInput<'inp, L, String, Ctx> for Rolling
where
  L: Lexer<'inp, Token = Tok>,
  Ctx: ParseContext<'inp, L>,
{
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx>,
  ) -> Result<ParseAttempt<String>, <Ctx::Emitter as Emitter<'inp, L>>::Error> {
    let digits = input.attempt(|input| {
      let mut text = String::new();
      while text.len() < 2 {
        let tok = input
          .try_expect(|tok| tok.data().0.is_ascii_digit())
          .ok()
          .flatten()?;
        text.push(char::from(tok.data().0));
      }
      Some(text)
    });
    Ok(match digits {
      Some(text) => ParseAttempt::Accept(text),
      None => ParseAttempt::Decline,
    })
  }
}

/// Installs a lexer-side limit of one token, then reads two digits as [`Rolling`] does: an operand
/// parser that re-keys the lexer before it trips, and rolls back to a point after the re-key.
#[derive(Debug, Clone, Copy)]
pub struct Rekeying;

impl<'inp, Ctx> TryParseInput<'inp, Limited<'inp>, String, Ctx> for Rekeying
where
  Ctx: ParseContext<'inp, Limited<'inp>>,
{
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, Limited<'inp>, Ctx>,
  ) -> Result<ParseAttempt<String>, <Ctx::Emitter as Emitter<'inp, Limited<'inp>>>::Error> {
    input.set_state(TokenLimiter::with_limitation(1));
    Rolling.try_parse_input(input)
  }
}

/// A digit, after which it peeks two tokens ahead and accepts whatever the window shows: an operand
/// parser that accepts over a stop its lookahead met.
#[derive(Debug, Clone, Copy)]
pub struct Lookahead;

impl<'inp, const SKIP: bool, Ctx> TryParseInput<'inp, ByteLexer<'inp, SKIP>, String, Ctx>
  for Lookahead
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
{
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  ) -> Result<ParseAttempt<String>, <Ctx::Emitter as Emitter<'inp, ByteLexer<'inp, SKIP>>>::Error>
  {
    let Some(tok) = input.try_expect(|tok| tok.data().0.is_ascii_digit())? else {
      return Ok(ParseAttempt::Decline);
    };
    let _ = input.peek::<U2>()?;
    Ok(ParseAttempt::Accept(char::from(tok.data().0).to_string()))
  }
}

// ── The typed driver's folds ──────────────────────────────────────────────────────────────────

/// An operator token with its span, as the drivers hand one over.
pub type Tk = Spanned<Tok, SimpleSpan>;

/// An infix operator, as the drivers hand one over.
pub type InfixOp = Precedenced<PrattInfix<Tk, Tk, Tk>, u8>;

/// `-1` folds to `(-1)`.
pub fn fold_prefix<'inp, const SKIP: bool, Ctx>(
  _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  operand: String,
  operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
) -> Result<String, Fail>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  Ok(format!(
    "({}{operand})",
    char::from(operator.into_data().into_data().0)
  ))
}

/// `1+2` folds to `(1+2)`.
pub fn fold_infix<'inp, const SKIP: bool, Ctx>(
  _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  left: String,
  right: String,
  operator: InfixOp,
) -> Result<String, Fail>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  let (PrattInfix::Left(tok) | PrattInfix::Right(tok) | PrattInfix::Neither(tok)) =
    operator.into_data();
  Ok(format!("({left}{}{right})", char::from(tok.into_data().0)))
}

/// `1!` folds to `(1!)`.
pub fn fold_postfix<'inp, const SKIP: bool, Ctx>(
  _input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
  operand: String,
  operator: Precedenced<Spanned<Tok, SimpleSpan>, u8>,
) -> Result<String, Fail>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  Ok(format!(
    "({operand}{})",
    char::from(operator.into_data().into_data().0)
  ))
}

// ── Where a parse left the input ──────────────────────────────────────────────────────────────

/// The committed frontier after a parse, the next token there and its offset, and how many tokens
/// the input had charged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Landing {
  pub handback: usize,
  pub next: Option<(u8, usize)>,
  pub spent: usize,
}

/// Reads where `input` stands: its frontier and its charge first, then the next token, which
/// reading moves.
pub fn landing<'inp, const SKIP: bool, Ctx>(
  input: &mut InputRef<'inp, '_, ByteLexer<'inp, SKIP>, Ctx>,
) -> Result<Landing, Fail>
where
  Ctx: ParseContext<'inp, ByteLexer<'inp, SKIP>>,
  Ctx::Emitter: Emitter<'inp, ByteLexer<'inp, SKIP>, Error = Fail>,
{
  let handback = input.span().end();
  let spent = input.token_budget().spent();
  let next = input.next()?.map(|tok| (tok.data().0, tok.span().start()));
  Ok(Landing {
    handback,
    next,
    spent,
  })
}
