use super::*;
use crate::{
  Emitter, ParseContext, Token,
  cache::DefaultCache,
  emitter::{Silent, Verbose},
  error::token::UnexpectedToken,
  input::Input,
  lexer::LogosLexer,
  span::Spanned,
};

#[derive(Debug, Clone, PartialEq)]
struct LexErr;

impl From<()> for LexErr {
  fn from(_: ()) -> Self {
    LexErr
  }
}

// `LexErr` never carries an incomplete or terminal signal, so it takes the blanket `false`
// default for both discrimination hooks.
impl crate::error::MaybeIncomplete for LexErr {}
impl crate::error::MaybeTerminal for LexErr {}

#[derive(Debug, Clone, PartialEq, Eq, crate::logos::Logos)]
#[logos(crate = crate::logos, skip r"[ \t\r\n]+")]
enum Tok {
  #[regex(r"[0-9]+")]
  Num,
}

impl core::fmt::Display for Tok {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "number")
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NumKind;

impl core::fmt::Display for NumKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "number")
  }
}

impl Token<'_> for Tok {
  type Kind = NumKind;
  type Error = LexErr;

  const SCAN_LOOKAHEAD: crate::ScanLookahead = crate::ScanLookahead::Unbounded;

  fn kind(&self) -> NumKind {
    NumKind
  }

  fn is_trivia(&self) -> bool {
    false
  }
}

type Lex<'a> = LogosLexer<'a, Tok>;
type Ctx<'a> = (Silent<LexErr>, DefaultCache<'a, Lex<'a>>);
// The emitter's error type, spelled as the trait's own projection so the manual
// `ParseInput`/`RecoverInput` impls below match their trait signatures exactly (it
// resolves to `LexErr`, but the compatibility check wants the projection, not the
// resolved type).
type EmitErr<'a> =
  <<Ctx<'a> as ParseContext<'a, Lex<'a>, ()>>::Emitter as Emitter<'a, Lex<'a>, ()>>::Error;

/// The primary parser: consume one token and succeed. It always returns `Ok` (even at
/// end of input, where it consumes nothing), so every application takes `Recover`'s
/// success path — the path that must not leak its checkpoint.
struct ConsumeOne;

impl<'inp> ParseInput<'inp, Lex<'inp>, (), Ctx<'inp>, ()> for ConsumeOne {
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, Lex<'inp>, Ctx<'inp>, ()>,
  ) -> Result<(), EmitErr<'inp>> {
    let _ = inp.next()?;
    Ok(())
  }
}

/// The recovery parser: unreachable here, since the primary always succeeds.
struct NeverRecover;

impl<'inp> RecoverInput<'inp, Lex<'inp>, (), Ctx<'inp>, ()> for NeverRecover {
  fn recover_input(
    &mut self,
    _inp: &mut InputRef<'inp, '_, Lex<'inp>, Ctx<'inp>, ()>,
    _err: EmitErr<'inp>,
  ) -> Result<(), EmitErr<'inp>> {
    unreachable!("the primary always succeeds on this input, so recovery never runs")
  }
}

#[test]
fn recover_success_does_not_grow_lineage_stack() {
  // One input, one parse session: a `Recover`-wrapped sub-parser applied repeatedly.
  // Every application takes the success path, which keeps (commits) its progress — so the
  // live-checkpoint lineage stack must return to its baseline length after each iteration.
  // Before the fix, `Recover`'s `Ok` branch left the `save()`d id on the stack, so this
  // count grew by one per successful parse without bound.
  let src = "1 ".repeat(128);
  let mut input = Input::<Lex<'_>, Ctx<'_>, ()>::with_state_and_context(
    src.as_str(),
    (),
    crate::input::InputContext::new(
      Silent::<LexErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let baseline = inp.live_checkpoints_len();

  let mut rec = Recover::<_, _, (), Lex<'_>, Ctx<'_>, ()>::new(ConsumeOne, NeverRecover);

  for i in 0..100 {
    rec
      .parse_input(&mut inp)
      .expect("the primary parser always succeeds");
    assert_eq!(
      inp.live_checkpoints_len(),
      baseline,
      "iteration {i}: a committed Recover success must not leave its checkpoint id on the \
       lineage stack (found {} live, baseline {baseline})",
      inp.live_checkpoints_len(),
    );
  }
}

// ── The never-recoverable law: Recover re-raises an Incomplete, untouched ──────

// An error type that can carry an incomplete signal (`Incomplete`), a terminal scanner stop
// (`Terminal`), or an ordinary failure (`Other`). `is_incomplete()` and `is_terminal()` are what
// `Recover` keys the never-recoverable law (and its terminal dual) off of.
#[derive(Debug, Clone, PartialEq)]
enum RecErr {
  Incomplete,
  Terminal,
  Other,
}

impl From<()> for RecErr {
  fn from(_: ()) -> Self {
    RecErr::Other
  }
}

// The two conversions the collecting `Verbose` emitter asks of an error type (`FromEmitterError`):
// the lexer's own error, and an unexpected token. Neither is an incomplete signal.
impl From<LexErr> for RecErr {
  fn from(_: LexErr) -> Self {
    RecErr::Other
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>> for RecErr {
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    RecErr::Other
  }
}

// The exact construction path W5's frontier rules use (`SurfaceIncomplete` builds the emitter
// error as `Error::from(Incomplete::new(off))`); pairing it with `MaybeIncomplete` below keeps
// construct-and-detect coherent, which is what makes the never-recoverable law hold end to end.
impl From<crate::error::Incomplete<usize>> for RecErr {
  fn from(_: crate::error::Incomplete<usize>) -> Self {
    RecErr::Incomplete
  }
}

// The terminal twin of the incomplete conversion: a terminal-marked end-of-input value (the way
// `try_expect_or_stop` and the delimited close build one) maps to `Terminal`, a genuine one to
// `Other`. Pairing it with `MaybeTerminal` keeps construct-and-detect coherent, which is what makes
// the terminal dual of the never-recoverable law hold end to end.
impl From<crate::error::UnexpectedEot<usize>> for RecErr {
  fn from(e: crate::error::UnexpectedEot<usize>) -> Self {
    if e.is_terminal() {
      RecErr::Terminal
    } else {
      RecErr::Other
    }
  }
}

impl crate::error::MaybeTerminal for RecErr {
  fn is_terminal(&self) -> bool {
    matches!(self, RecErr::Terminal)
  }
}

impl crate::error::MaybeIncomplete for RecErr {
  fn is_incomplete(&self) -> bool {
    matches!(self, RecErr::Incomplete)
  }
}

type RCtx<'a> = (Silent<RecErr>, DefaultCache<'a, Lex<'a>>);
type REmitErr<'a> =
  <<RCtx<'a> as ParseContext<'a, Lex<'a>, ()>>::Emitter as Emitter<'a, Lex<'a>, ()>>::Error;

/// A primary parser that fails outright with a chosen error, without consuming input.
struct FailWith(RecErr);

impl<'inp> ParseInput<'inp, Lex<'inp>, (), RCtx<'inp>, ()> for FailWith {
  fn parse_input(
    &mut self,
    _inp: &mut InputRef<'inp, '_, Lex<'inp>, RCtx<'inp>, ()>,
  ) -> Result<(), REmitErr<'inp>> {
    Err(self.0.clone())
  }
}

/// A recoverer that records whether it was invoked and otherwise succeeds.
struct FlagRecover<'f> {
  invoked: &'f core::cell::Cell<bool>,
}

impl<'inp> RecoverInput<'inp, Lex<'inp>, (), RCtx<'inp>, ()> for FlagRecover<'_> {
  fn recover_input(
    &mut self,
    _inp: &mut InputRef<'inp, '_, Lex<'inp>, RCtx<'inp>, ()>,
    _err: REmitErr<'inp>,
  ) -> Result<(), REmitErr<'inp>> {
    self.invoked.set(true);
    Ok(())
  }
}

// The law, value-asserted: when the primary fails with an `Incomplete`, `Recover` re-raises it
// verbatim on the `Err` channel and NEVER invokes the recoverer.
#[test]
fn recover_reraises_incomplete_and_never_invokes_recoverer() {
  let mut input = Input::<Lex<'_>, RCtx<'_>, ()>::with_state_and_context(
    "1 2 3",
    (),
    crate::input::InputContext::new(
      Silent::<RecErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let invoked = core::cell::Cell::new(false);
  let mut rec = Recover::<_, _, (), Lex<'_>, RCtx<'_>, ()>::new(
    FailWith(RecErr::Incomplete),
    FlagRecover { invoked: &invoked },
  );

  let r = rec.parse_input(&mut inp);
  assert_eq!(
    r,
    Err(RecErr::Incomplete),
    "an Incomplete is re-raised untouched on the Err channel"
  );
  assert!(
    !invoked.get(),
    "the recoverer must never run for an Incomplete"
  );
}

// The W5 partial-input incomplete: the value the frontier rules actually surface is built via
// `From<Incomplete>` (not the hand-written `RecErr::Incomplete` literal above). Prove that value
// reports itself incomplete and that `Recover` re-raises it untouched — the never-recoverable law
// wired to the real surfacing mechanism.
#[test]
fn recover_reraises_a_from_incomplete_built_error() {
  let surfaced: RecErr = crate::error::Incomplete::new(7usize).into();
  assert!(
    surfaced.is_incomplete(),
    "an error built the way the frontier rules build it must report itself incomplete"
  );

  let mut input = Input::<Lex<'_>, RCtx<'_>, ()>::with_state_and_context(
    "1 2 3",
    (),
    crate::input::InputContext::new(
      Silent::<RecErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let invoked = core::cell::Cell::new(false);
  let mut rec = Recover::<_, _, (), Lex<'_>, RCtx<'_>, ()>::new(
    FailWith(surfaced.clone()),
    FlagRecover { invoked: &invoked },
  );

  let r = rec.parse_input(&mut inp);
  assert_eq!(
    r,
    Err(surfaced),
    "the partial-mode incomplete is re-raised untouched"
  );
  assert!(
    !invoked.get(),
    "the recoverer never runs for a partial-mode incomplete"
  );
}

// The scoping guard: an ordinary (non-incomplete) failure still recovers as before — the law
// does not swallow every error, only the incomplete sentinel.
#[test]
fn recover_still_recovers_an_ordinary_error() {
  let mut input = Input::<Lex<'_>, RCtx<'_>, ()>::with_state_and_context(
    "1 2 3",
    (),
    crate::input::InputContext::new(
      Silent::<RecErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let invoked = core::cell::Cell::new(false);
  let mut rec = Recover::<_, _, (), Lex<'_>, RCtx<'_>, ()>::new(
    FailWith(RecErr::Other),
    FlagRecover { invoked: &invoked },
  );

  let r = rec.parse_input(&mut inp);
  assert_eq!(r, Ok(()), "an ordinary error is recovered");
  assert!(invoked.get(), "the recoverer runs for an ordinary error");
}

// ── The terminal dual: Recover re-raises a terminal scanner stop, untouched ──────
//
// A terminal stop is not a malformed construct — no amount of skipping or re-attempting clears a
// tripped limit — so, like an Incomplete, it must re-raise on the `Err` channel rather than reach
// the recoverer, which would otherwise synthesize a value from a recovery that should never have
// begun.

// The law, value-asserted: when the primary fails with a terminal stop, `Recover` re-raises it
// verbatim and NEVER invokes the recoverer.
#[test]
fn recover_reraises_terminal_and_never_invokes_recoverer() {
  let mut input = Input::<Lex<'_>, RCtx<'_>, ()>::with_state_and_context(
    "1 2 3",
    (),
    crate::input::InputContext::new(
      Silent::<RecErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let invoked = core::cell::Cell::new(false);
  let mut rec = Recover::<_, _, (), Lex<'_>, RCtx<'_>, ()>::new(
    FailWith(RecErr::Terminal),
    FlagRecover { invoked: &invoked },
  );

  let r = rec.parse_input(&mut inp);
  assert_eq!(
    r,
    Err(RecErr::Terminal),
    "a terminal stop is re-raised untouched on the Err channel"
  );
  assert!(
    !invoked.get(),
    "the recoverer must never run for a terminal stop"
  );
}

// The value the attempt/decline leaves actually surface is built via `From<UnexpectedEot>` with
// the terminal marker set (the way `try_expect_or_stop` and the delimited close build it). Prove
// that value reports itself terminal — while a genuine end of input does not — and that `Recover`
// re-raises it untouched: the terminal dual wired to the real surfacing mechanism.
#[test]
fn recover_reraises_a_from_terminal_built_error() {
  let surfaced: RecErr = crate::error::UnexpectedEot::eot_of(7usize)
    .into_terminal()
    .into();
  assert_eq!(
    surfaced,
    RecErr::Terminal,
    "a terminal-marked end-of-input value maps to the terminal variant"
  );
  assert!(
    surfaced.is_terminal(),
    "an error built the way the leaves build it must report itself terminal"
  );
  let genuine: RecErr = crate::error::UnexpectedEot::eot_of(7usize).into();
  assert!(
    !genuine.is_terminal(),
    "a genuine end of input is not a terminal stop"
  );

  let mut input = Input::<Lex<'_>, RCtx<'_>, ()>::with_state_and_context(
    "1 2 3",
    (),
    crate::input::InputContext::new(
      Silent::<RecErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let invoked = core::cell::Cell::new(false);
  let mut rec = Recover::<_, _, (), Lex<'_>, RCtx<'_>, ()>::new(
    FailWith(surfaced.clone()),
    FlagRecover { invoked: &invoked },
  );

  let r = rec.parse_input(&mut inp);
  assert_eq!(r, Err(surfaced), "the terminal stop is re-raised untouched");
  assert!(
    !invoked.get(),
    "the recoverer never runs for a terminal stop"
  );
}

// ── The same law, binding the in-place path: InplaceRecover re-raises an Incomplete ──────
//
// The in-place path never backtracks, so it cannot undo a recovery it should never have begun —
// which is precisely why the check must happen *before* the handler runs. These run over the
// collecting `Verbose` emitter so the "no emissions" half of the law is observable: the handler
// records a diagnostic whenever it runs, so an untouched pass-through must leave the log empty.

type VCtx<'a> = (Verbose<RecErr>, DefaultCache<'a, Lex<'a>>);
type VEmitErr<'a> =
  <<VCtx<'a> as ParseContext<'a, Lex<'a>, ()>>::Emitter as Emitter<'a, Lex<'a>, ()>>::Error;

/// The `Verbose`-context twin of `FailWith`: fails outright with a chosen error, consuming
/// nothing, so every application takes the recovery path.
struct VFailWith(RecErr);

impl<'inp> ParseInput<'inp, Lex<'inp>, (), VCtx<'inp>, ()> for VFailWith {
  fn parse_input(
    &mut self,
    _inp: &mut InputRef<'inp, '_, Lex<'inp>, VCtx<'inp>, ()>,
  ) -> Result<(), VEmitErr<'inp>> {
    Err(self.0.clone())
  }
}

/// An in-place recoverer that records that it ran — and emits while doing so, so a run is
/// visible on the diagnostic channel too — then succeeds.
struct InplaceFlagRecover<'f> {
  invoked: &'f core::cell::Cell<bool>,
}

impl<'inp> InplaceRecoverInput<'inp, Lex<'inp>, (), VCtx<'inp>, ()> for InplaceFlagRecover<'_> {
  fn inplace_recover_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, Lex<'inp>, VCtx<'inp>, ()>,
    _cursor: Cursor<'inp, '_, Lex<'inp>>,
    err: VEmitErr<'inp>,
  ) -> Result<(), VEmitErr<'inp>> {
    self.invoked.set(true);
    let span = *inp.span();
    <Verbose<RecErr> as Emitter<'inp, Lex<'inp>>>::emit_error(
      inp.emitter(),
      Spanned::new(span, err),
    )?;
    Ok(())
  }
}

/// The input reference the in-place law tests run on, over the verbose (collecting) context.
type VIr<'inp, 'closure> = InputRef<'inp, 'closure, Lex<'inp>, VCtx<'inp>, ()>;

/// The number of diagnostics the input's emitter is holding.
fn vdiags(inp: &mut VIr<'_, '_>) -> usize {
  inp.emitter().errors().values().map(|g| g.len()).sum()
}

// The law, value-asserted on the in-place path: an `Incomplete` is re-raised verbatim, the
// recoverer never runs, and nothing is emitted.
#[test]
fn inplace_recover_reraises_incomplete_and_never_invokes_recoverer() {
  let mut input = Input::<Lex<'_>, VCtx<'_>, ()>::with_state_and_context(
    "1 2 3",
    (),
    crate::input::InputContext::new(
      Verbose::<RecErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let invoked = core::cell::Cell::new(false);
  let mut rec = InplaceRecover::<_, _, (), Lex<'_>, VCtx<'_>, ()>::new(
    VFailWith(RecErr::Incomplete),
    InplaceFlagRecover { invoked: &invoked },
  );

  let r = rec.parse_input(&mut inp);
  assert_eq!(
    r,
    Err(RecErr::Incomplete),
    "an Incomplete is re-raised untouched on the Err channel"
  );
  assert!(
    !invoked.get(),
    "the recoverer must never run for an Incomplete"
  );
  assert_eq!(vdiags(&mut inp), 0, "and nothing is emitted on its behalf");
}

// The W5 partial-input incomplete: the value the frontier rules actually surface is built via
// `From<Incomplete>`, not a hand-written literal. The in-place path re-raises that one too.
#[test]
fn inplace_recover_reraises_a_from_incomplete_built_error() {
  let surfaced: RecErr = crate::error::Incomplete::new(7usize).into();
  assert!(
    surfaced.is_incomplete(),
    "an error built the way the frontier rules build it must report itself incomplete"
  );

  let mut input = Input::<Lex<'_>, VCtx<'_>, ()>::with_state_and_context(
    "1 2 3",
    (),
    crate::input::InputContext::new(
      Verbose::<RecErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let invoked = core::cell::Cell::new(false);
  let mut rec = InplaceRecover::<_, _, (), Lex<'_>, VCtx<'_>, ()>::new(
    VFailWith(surfaced.clone()),
    InplaceFlagRecover { invoked: &invoked },
  );

  let r = rec.parse_input(&mut inp);
  assert_eq!(
    r,
    Err(surfaced),
    "the partial-mode incomplete is re-raised untouched"
  );
  assert!(
    !invoked.get(),
    "the recoverer never runs for a partial-mode incomplete"
  );
  assert_eq!(vdiags(&mut inp), 0, "and nothing is emitted on its behalf");
}

// The scoping guard, in-place flavour: an ordinary failure still recovers exactly as before.
#[test]
fn inplace_recover_still_recovers_an_ordinary_error() {
  let mut input = Input::<Lex<'_>, VCtx<'_>, ()>::with_state_and_context(
    "1 2 3",
    (),
    crate::input::InputContext::new(
      Verbose::<RecErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let invoked = core::cell::Cell::new(false);
  let mut rec = InplaceRecover::<_, _, (), Lex<'_>, VCtx<'_>, ()>::new(
    VFailWith(RecErr::Other),
    InplaceFlagRecover { invoked: &invoked },
  );

  let r = rec.parse_input(&mut inp);
  assert_eq!(r, Ok(()), "an ordinary error is recovered in place");
  assert!(invoked.get(), "the recoverer runs for an ordinary error");
  assert_eq!(vdiags(&mut inp), 1, "and its diagnostic is on the log");
}

// The terminal dual on the in-place path: a terminal scanner stop is re-raised verbatim, the
// recoverer never runs, and nothing is emitted. The in-place path cannot undo a recovery it should
// never have begun, so re-raising before the handler runs is the only correct order.
#[test]
fn inplace_recover_reraises_terminal_and_never_invokes_recoverer() {
  let mut input = Input::<Lex<'_>, VCtx<'_>, ()>::with_state_and_context(
    "1 2 3",
    (),
    crate::input::InputContext::new(
      Verbose::<RecErr>::new(),
      DefaultCache::<'_, Lex<'_>>::default(),
    ),
  );
  let mut inp = input.as_ref();

  let invoked = core::cell::Cell::new(false);
  let mut rec = InplaceRecover::<_, _, (), Lex<'_>, VCtx<'_>, ()>::new(
    VFailWith(RecErr::Terminal),
    InplaceFlagRecover { invoked: &invoked },
  );

  let r = rec.parse_input(&mut inp);
  assert_eq!(
    r,
    Err(RecErr::Terminal),
    "a terminal stop is re-raised untouched on the Err channel"
  );
  assert!(
    !invoked.get(),
    "the recoverer must never run for a terminal stop"
  );
  assert_eq!(vdiags(&mut inp), 0, "and nothing is emitted on its behalf");
}
