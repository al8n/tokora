//! The [`labelled`] combinator — attach a *"while parsing X"* diagnostic context to a sub-parse.
//!
//! Wrapping a parser in [`labelled`] pushes a `&'static str` onto the emitter's open-label
//! stack for the duration of the sub-parse and pops it afterwards, so every diagnostic the
//! sub-parse records is stamped with the enclosing context (see [`Verbose`](crate::emitter::Verbose)).
//! Labels are **captured into the emission log at emit time**: each recorded diagnostic carries a
//! snapshot of the labels open when it was emitted, so an emitter rewind that drops an entry drops
//! its labels with it, and a later re-emission re-derives its labels from the then-current stack.
//!
//! The stack lives on the emitter — the one party present at every emit site, including
//! parser-level emissions that have no input access — so a [`labelled`] scope needs nothing beyond
//! the push/pop pair. Around a non-collecting emitter ([`Fatal`](crate::emitter::Fatal),
//! [`Silent`](crate::emitter::Silent)) both calls are inlined-away no-ops, so `labelled(name, p)`
//! reduces to exactly `p`.

use crate::{
  Emitter, InputRef, Lexer, ParseContext, ParseInput, TryParseInput, input::Completeness,
  try_parse::input::ParseAttempt,
};

/// Wraps `parser` with the diagnostic context `name`: for the duration of the sub-parse, `name`
/// is pushed onto the emitter's open-label stack (a *"while parsing X"* context), and every
/// diagnostic recorded during the sub-parse is stamped with the labels open at emit time.
///
/// `name` is `&'static str` — parser names are static, so opening a label never allocates. The
/// label is popped on **every** exit of the sub-parse — success, error, and panic unwind alike —
/// because the pop lives in a drop guard that both paths run, so the live stack always mirrors
/// the nesting of `labelled` scopes even for a host that catches. (Before that guard existed, an
/// unwind skipped the pop and every later diagnostic on that emitter carried the stale context
/// for the emitter's whole life: the label stack is plain emitter state and survives
/// `catch_unwind`.)
///
/// With a non-collecting emitter the push/pop pair are no-ops that inline away, so this wrapper is
#[cfg_attr(
  any(feature = "std", feature = "alloc"),
  doc = " zero-cost there; a collecting emitter such as [`Verbose`](crate::emitter::Verbose) snapshots the"
)]
#[cfg_attr(
  not(any(feature = "std", feature = "alloc")),
  doc = " zero-cost there; a collecting emitter such as `Verbose` snapshots the"
)]
/// open labels into each diagnostic and exposes them per-diagnostic via
#[cfg_attr(
  any(feature = "std", feature = "alloc"),
  doc = " [`Verbose::labels`](crate::emitter::Verbose::labels)."
)]
#[cfg_attr(
  not(any(feature = "std", feature = "alloc")),
  doc = " `Verbose::labels`."
)]
///
/// ```
/// # #[cfg(all(feature = "logos_0_16", feature = "std"))]
/// # fn demo<P>(inner: P) -> tokora::Labelled<P> {
/// // Diagnostics emitted inside `inner` are stamped "while parsing a list".
/// tokora::labelled("while parsing a list", inner)
/// # }
/// ```
///
/// When the `trace` feature is on, entering a `labelled` scope also fires a single trace leaf event
/// naming the label at the current depth, so the label context shows up in the parse transcript
/// alongside [`traced`](crate::traced) — keeping the two DX systems coherent.
#[inline(always)]
pub fn labelled<P>(name: &'static str, parser: P) -> Labelled<P> {
  Labelled { name, parser }
}

/// The parser wrapper produced by [`labelled`].
///
/// Delegates to the inner parser, bracketing its run with an
/// [`enter_label`](Emitter::enter_label) / [`exit_label`](Emitter::exit_label) pair on the
/// emitter. Implements both [`ParseInput`] and [`TryParseInput`], so it can wrap either kind of
/// parser — including the element of a `repeated`/`separated` driver.
#[derive(Debug, Clone, Copy)]
pub struct Labelled<P> {
  name: &'static str,
  parser: P,
}

/// The RAII half of a [`labelled`] scope: it owns the reborrowed input for the sub-parse and
/// pops the label in its `Drop`, so the **normal path and the unwind path run the same code**
/// and `exit_label` happens exactly once on both. No disarm flag exists, because there is no
/// path on which the pop should be skipped or repeated.
///
/// It is private to this module: the guard is the mechanism, not a surface.
struct LabelScope<'g, 'inp, 'closure, L, Ctx, Lang: ?Sized, Cmpl>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  input: &'g mut InputRef<'inp, 'closure, L, Ctx, Lang, Cmpl>,
}

impl<'inp, L, Ctx, Lang: ?Sized, Cmpl> Drop for LabelScope<'_, 'inp, '_, L, Ctx, Lang, Cmpl>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  fn drop(&mut self) {
    self.input.emitter().exit_label();
  }
}

impl<'inp, L, O, Ctx, Lang, P, Cmpl> ParseInput<'inp, L, O, Ctx, Lang, Cmpl> for Labelled<P>
where
  Lang: ?Sized,
  P: ParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  #[inline]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    input.emitter().enter_label(self.name);
    // The guard goes up BEFORE the trace event: `trace_event!` formats, and a panicking
    // formatter between the push and the guard would reopen the exact window this closes.
    let scope = LabelScope { input };
    trace_event!(scope.input, self.name);
    let res = self.parser.parse_input(scope.input);
    drop(scope);
    res
  }
}

impl<'inp, L, O, Ctx, Lang, P, Cmpl> TryParseInput<'inp, L, O, Ctx, Lang, Cmpl> for Labelled<P>
where
  Lang: ?Sized,
  P: TryParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Cmpl: Completeness,
{
  #[inline]
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    input.emitter().enter_label(self.name);
    let scope = LabelScope { input };
    trace_event!(scope.input, self.name);
    let res = self.parser.try_parse_input(scope.input);
    drop(scope);
    res
  }
}

#[cfg(all(test, feature = "logos_0_16", feature = "std"))]
mod tests;
