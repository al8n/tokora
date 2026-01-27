use core::marker::PhantomData;

use crate::{TryParseInput, try_parse_input::ParseAttempt};

use super::*;

pub use fold_while::*;

mod fold_while;

/// A fold parser combinator.
#[derive(Debug, Clone)]
pub struct Fold<P, Init, Acc, L, O, Ctx, Lang: ?Sized = ()> {
  parser: P,
  init: Init,
  acc: Acc,
  _output: PhantomData<O>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<P, Init, Acc, O, L, Ctx, Lang: ?Sized> Fold<P, Init, Acc, L, O, Ctx, Lang> {
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
    }
  }
}

impl<'inp, P, Init, Acc, O, L, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for Fold<P, Init, Acc, O, L, Ctx, Lang>
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

/// A fold parser combinator that accept a fallible accumulator.
#[derive(Debug, Clone)]
pub struct TryFold<P, Init, Acc, L, O, Ctx, Lang: ?Sized = ()> {
  parser: P,
  init: Init,
  acc: Acc,
  _output: PhantomData<O>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<P, Init, Acc, O, L, Ctx, Lang: ?Sized> TryFold<P, Init, Acc, L, O, Ctx, Lang> {
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
    }
  }
}

impl<'inp, P, Init, Acc, O, L, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for TryFold<P, Init, Acc, O, L, Ctx, Lang>
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

    while let ParseAttempt::Accept(value) = self.parser.try_parse_input(inp)? {
      let cursor = inp.cursor().clone();
      output = (self.acc)(output, value, ParseState::new(inp, cursor))?;
    }
    Ok(output)
  }
}
