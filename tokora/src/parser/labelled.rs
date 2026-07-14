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
  Emitter, InputRef, Lexer, ParseContext, ParseInput, TryParseInput, try_parse_input::ParseAttempt,
};

/// Wraps `parser` with the diagnostic context `name`: for the duration of the sub-parse, `name`
/// is pushed onto the emitter's open-label stack (a *"while parsing X"* context), and every
/// diagnostic recorded during the sub-parse is stamped with the labels open at emit time.
///
/// `name` is `&'static str` — parser names are static, so opening a label never allocates. The
/// label is popped when the sub-parse returns, on both the success and error paths, so the live
/// stack always mirrors the nesting of `labelled` scopes.
///
/// With a non-collecting emitter the push/pop pair are no-ops that inline away, so this wrapper is
/// zero-cost there; a collecting emitter such as [`Verbose`](crate::emitter::Verbose) snapshots the
/// open labels into each diagnostic and exposes them per-diagnostic via
/// [`Verbose::labels`](crate::emitter::Verbose::labels).
///
/// ```
/// # #[cfg(all(feature = "logos", feature = "std"))]
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

impl<'inp, L, O, Ctx, Lang, P> ParseInput<'inp, L, O, Ctx, Lang> for Labelled<P>
where
  Lang: ?Sized,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[inline]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    input.emitter().enter_label(self.name);
    trace_event!(input, self.name);
    let res = self.parser.parse_input(input);
    input.emitter().exit_label();
    res
  }
}

impl<'inp, L, O, Ctx, Lang, P> TryParseInput<'inp, L, O, Ctx, Lang> for Labelled<P>
where
  Lang: ?Sized,
  P: TryParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[inline]
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    input.emitter().enter_label(self.name);
    trace_event!(input, self.name);
    let res = self.parser.try_parse_input(input);
    input.emitter().exit_label();
    res
  }
}

#[cfg(all(test, feature = "trace", feature = "logos", feature = "std"))]
mod trace_tests {
  use crate::{
    InputRef, ParseInput, Token, cache::DefaultCache, emitter::Silent,
    error::token::UnexpectedToken, input::Input, lexer::LogosLexer,
  };

  #[derive(Debug, Clone, PartialEq)]
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

  #[derive(Debug, Clone, PartialEq, crate::logos::Logos)]
  #[logos(crate = crate::logos, skip r"[ \t\r\n]+")]
  enum Tok {
    #[regex(r"[0-9]+")]
    Num,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  enum Kind {
    Num,
  }
  impl core::fmt::Display for Kind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      write!(f, "number")
    }
  }

  impl Token<'_> for Tok {
    type Kind = Kind;
    type Error = Err;
    fn kind(&self) -> Kind {
      Kind::Num
    }
    fn is_trivia(&self) -> bool {
      false
    }
  }

  type Lex<'a> = LogosLexer<'a, Tok>;
  type Cx<'a> = (Silent<Err>, DefaultCache<'a, Lex<'a>>);

  fn eat_num<'inp>(inp: &mut InputRef<'inp, '_, Lex<'inp>, Cx<'inp>>) -> Result<bool, Err> {
    inp.try_expect(|_| true).map(|tok| tok.is_some())
  }

  // With the `trace` feature on, entering a `labelled` scope fires exactly one trace leaf line
  // naming the label at the current depth — keeping the label DX coherent with `traced`.
  #[test]
  fn labelled_fires_a_single_trace_leaf_naming_the_label() {
    let mut emitter = Silent::<Err>::new();
    let mut input = Input::<Lex<'_>, Cx<'_>>::with_state_and_cache(
      "12",
      (),
      DefaultCache::<'_, Lex<'_>>::default(),
    );
    let mut inp = input.as_ref(&mut emitter);

    let mut parser = crate::labelled("while parsing item", eat_num);
    let (res, lines) = crate::trace::capture(|| parser.parse_input(&mut inp));

    assert_eq!(res, Ok(true));
    // The label surfaces as a leaf line (`·`), naming the label.
    assert!(
      lines
        .iter()
        .any(|l| l.contains("\u{b7}") && l.contains("while parsing item")),
      "labelled fires a trace leaf naming the label: {lines:#?}"
    );
  }
}
