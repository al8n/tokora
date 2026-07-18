use core::marker::PhantomData;

use crate::{TryParseInput, try_parse_input::ParseAttempt};

use super::*;

pub use fold_while::*;

mod fold_while;

#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
pub use rfold::*;

#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
mod rfold;

#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
pub use rfold_while::*;

#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "alloc", feature = "std"))))]
mod rfold_while;

/// A fold parser combinator.
#[derive(Debug, Clone)]
pub struct Fold<P, Init, Acc, L, O, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  parser: P,
  init: Init,
  acc: Acc,
  _output: PhantomData<O>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _cmpl: PhantomData<Cmpl>,
}

impl<P, Init, Acc, O, L, Ctx, Lang: ?Sized, Cmpl> Fold<P, Init, Acc, L, O, Ctx, Lang, Cmpl> {
  /// Creates a new fold parser combinator.
  pub(crate) fn new(parser: P, init: Init, acc: Acc) -> Self {
    Self {
      parser,
      init,
      acc,
      _output: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _cmpl: PhantomData,
    }
  }
}

impl<'inp, P, Init, Acc, O, L, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for Fold<P, Init, Acc, L, O, Ctx, Lang>
where
  P: TryParseInput<'inp, L, O, Ctx, Lang>,
  Init: FnMut() -> O,
  Acc: FnMut(O, O) -> O,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let mut output = (self.init)();
    while let ParseAttempt::Accept(value) = self.parser.try_parse_input(inp)? {
      output = (self.acc)(output, value);
    }
    Ok(output)
  }
}

/// A fold parser combinator that accepts a fallible accumulator.
#[derive(Debug, Clone)]
pub struct TryFold<P, Init, Acc, L, O, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  parser: P,
  init: Init,
  acc: Acc,
  _output: PhantomData<O>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _cmpl: PhantomData<Cmpl>,
}

impl<P, Init, Acc, O, L, Ctx, Lang: ?Sized, Cmpl> TryFold<P, Init, Acc, L, O, Ctx, Lang, Cmpl> {
  /// Creates a new fold parser combinator.
  pub(crate) fn new(parser: P, init: Init, acc: Acc) -> Self {
    Self {
      parser,
      init,
      acc,
      _output: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _cmpl: PhantomData,
    }
  }
}

impl<'inp, P, Init, Acc, O, L, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for TryFold<P, Init, Acc, L, O, Ctx, Lang>
where
  P: TryParseInput<'inp, L, O, Ctx, Lang>,
  Init: FnMut() -> O,
  Acc: FnMut(O, O) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let mut output = (self.init)();
    while let ParseAttempt::Accept(value) = self.parser.try_parse_input(inp)? {
      output = (self.acc)(output, value)?;
    }
    Ok(output)
  }
}

/// A fold parser combinator that accepts a fallible accumulator with access to parsing state.
#[derive(Debug, Clone)]
pub struct TryFoldWith<P, Init, Acc, L, O, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  parser: P,
  init: Init,
  acc: Acc,
  _output: PhantomData<O>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _cmpl: PhantomData<Cmpl>,
}

impl<P, Init, Acc, O, L, Ctx, Lang: ?Sized, Cmpl> TryFoldWith<P, Init, Acc, L, O, Ctx, Lang, Cmpl> {
  /// Creates a new fold parser combinator.
  pub(crate) fn new(parser: P, init: Init, acc: Acc) -> Self {
    Self {
      parser,
      init,
      acc,
      _output: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _cmpl: PhantomData,
    }
  }
}

impl<'inp, P, Init, Acc, O, L, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for TryFoldWith<P, Init, Acc, L, O, Ctx, Lang>
where
  P: TryParseInput<'inp, L, O, Ctx, Lang>,
  Init: FnMut() -> O,
  Acc: FnMut(
    O,
    O,
    ParseState<'_, 'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let mut output = (self.init)();
    loop {
      let cursor = inp.cursor().clone();
      match self.parser.try_parse_input(inp)? {
        ParseAttempt::Accept(value) => {
          output = (self.acc)(output, value, ParseState::new(inp, cursor))?;
        }
        ParseAttempt::Decline => break,
      }
    }
    Ok(output)
  }
}
