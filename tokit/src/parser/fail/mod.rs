use crate::{TryParseInput, try_parse_input::ParseAttempt};

use super::*;

/// A parser that always fails with the given error fn.
#[derive(Debug, Clone, Copy)]
pub struct Fail<F, L, O, Ctx, Lang: ?Sized = ()> {
  err: F,
  _l: PhantomData<L>,
  _o: PhantomData<O>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, O, L, Ctx, Lang: ?Sized> Fail<F, L, O, Ctx, Lang> {
  /// Creates a new `Fail` parser.
  #[inline(always)]
  pub(crate) const fn new(err: F) -> Self {
    Self {
      err,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _o: PhantomData,
    }
  }
}

impl<'inp, F, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for Fail<F, L, O, Ctx, Lang>
where
  F: FnMut() -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[inline(always)]
  fn parse_input(
    &mut self,
    _input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Err((self.err)())
  }
}

impl<'inp, F, L, O, Ctx, Lang> TryParseInput<'inp, L, O, Ctx, Lang> for Fail<F, L, O, Ctx, Lang>
where
  F: FnMut() -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[inline(always)]
  fn try_parse_input(
    &mut self,
    _input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    Err((self.err)())
  }
}

/// A parser that always fails with the given error fn with state information.
#[derive(Debug, Clone, Copy)]
pub struct FailWith<F, L, O, Ctx, Lang: ?Sized = ()> {
  err: F,
  _l: PhantomData<L>,
  _o: PhantomData<O>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, O, L, Ctx, Lang: ?Sized> FailWith<F, L, O, Ctx, Lang> {
  /// Creates a new `FailWith` parser.
  #[inline(always)]
  pub(crate) const fn new(err: F) -> Self {
    Self {
      err,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _o: PhantomData,
    }
  }
}

impl<'inp, F, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for FailWith<F, L, O, Ctx, Lang>
where
  F: FnMut(
    ParseState<'_, 'inp, '_, L, Ctx, Lang>,
  ) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[inline(always)]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let start = input.cursor().clone();
    Err((self.err)(ParseState::new(input, start)))
  }
}

impl<'inp, F, L, O, Ctx, Lang> TryParseInput<'inp, L, O, Ctx, Lang> for FailWith<F, L, O, Ctx, Lang>
where
  F: FnMut(
    ParseState<'_, 'inp, '_, L, Ctx, Lang>,
  ) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[inline(always)]
  fn try_parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<ParseAttempt<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let start = input.cursor().clone();
    Err((self.err)(ParseState::new(input, start)))
  }
}

/// Creates a new `Fail` parser.
#[must_use]
#[inline(always)]
pub const fn fail<'inp, F, L, O, Ctx>(err: F) -> Fail<F, L, O, Ctx>
where
  F: FnMut() -> <Ctx::Emitter as Emitter<'inp, L>>::Error,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L>,
{
  fail_of(err)
}

/// Creates a new `Fail` parser for the specified language.
#[must_use]
#[inline(always)]
pub const fn fail_of<'inp, F, L, O, Ctx, Lang: ?Sized>(err: F) -> Fail<F, L, O, Ctx, Lang>
where
  F: FnMut() -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  Fail::new(err)
}

/// Creates a new `FailWith` parser.
#[must_use]
#[inline(always)]
pub const fn fail_with<'inp, F, L, O, Ctx>(err: F) -> FailWith<F, L, O, Ctx>
where
  F: FnMut(ParseState<'_, 'inp, '_, L, Ctx>) -> <Ctx::Emitter as Emitter<'inp, L>>::Error,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L>,
{
  fail_with_of(err)
}

/// Creates a new `FailWith` parser for the specified language.
#[must_use]
#[inline(always)]
pub const fn fail_with_of<'inp, F, L, O, Ctx, Lang: ?Sized>(err: F) -> FailWith<F, L, O, Ctx, Lang>
where
  F: FnMut(
    ParseState<'_, 'inp, '_, L, Ctx, Lang>,
  ) -> <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  FailWith::new(err)
}

#[cfg(test)]
#[allow(warnings)]
#[cfg(feature = "std")]
mod tests;
