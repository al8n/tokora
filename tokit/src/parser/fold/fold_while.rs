use super::*;

/// A fold parser that accumulates results while a condition is met, with a fallible accumulator.
#[derive(Clone, Debug, Copy)]
pub struct FoldWhile<F, Condition, Init, Acc, O, W, L, Ctx, Lang: ?Sized = ()> {
  f: F,
  condition: Condition,
  init: Init,
  acc: Acc,
  _m: PhantomData<O>,
  _cap: PhantomData<W>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, Condition, Init, Acc, O, W, L, Ctx, Lang: ?Sized>
  FoldWhile<F, Condition, Init, Acc, O, W, L, Ctx, Lang>
{
  /// Creates a new `FoldWhile` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(f: F, condition: Condition, init: Init, acc: Acc) -> Self {
    Self {
      f,
      condition,
      init,
      acc,
      _m: PhantomData,
      _cap: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<'inp, F, Condition, Init, Acc, O, W, L, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for FoldWhile<F, Condition, Init, Acc, O, W, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  Init: FnMut() -> O,
  Acc: FnMut(O, O) -> O,
  W: Window,
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
      let (peeked, emitter) = inp.peek_with_emitter::<W>()?;
      match self.condition.decide(peeked, emitter)? {
        Action::Stop => break,
        Action::Continue => {
          output = (self.acc)(output, self.f.parse_input(inp)?);
        }
      }
    }

    Ok(output)
  }
}

/// A fold parser that accumulates results while a condition is met, with a fallible accumulator.
#[derive(Clone, Debug, Copy)]
pub struct TryFoldWhile<F, Condition, Init, Acc, O, W, L, Ctx, Lang: ?Sized = ()> {
  f: F,
  condition: Condition,
  init: Init,
  acc: Acc,
  _m: PhantomData<O>,
  _cap: PhantomData<W>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, Condition, Init, Acc, O, W, L, Ctx, Lang: ?Sized>
  TryFoldWhile<F, Condition, Init, Acc, O, W, L, Ctx, Lang>
{
  /// Creates a new `FoldWhile` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(f: F, condition: Condition, init: Init, acc: Acc) -> Self {
    Self {
      f,
      condition,
      init,
      acc,
      _m: PhantomData,
      _cap: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<'inp, F, Condition, Init, Acc, O, W, L, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for TryFoldWhile<F, Condition, Init, Acc, O, W, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  Init: FnMut() -> O,
  Acc: FnMut(O, O) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  W: Window,
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
      let (peeked, emitter) = inp.peek_with_emitter::<W>()?;
      match self.condition.decide(peeked, emitter)? {
        Action::Stop => break,
        Action::Continue => {
          let new = self.f.parse_input(inp)?;
          output = (self.acc)(output, new)?;
        }
      }
    }

    Ok(output)
  }
}

/// A fold parser that accumulates results while a condition is met, with a fallible accumulator
/// and access to parsing state.
#[derive(Clone, Debug, Copy)]
pub struct TryFoldWhileWith<F, Condition, Init, Acc, O, W, L, Ctx, Lang: ?Sized = ()> {
  f: F,
  condition: Condition,
  init: Init,
  acc: Acc,
  _m: PhantomData<O>,
  _cap: PhantomData<W>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, Condition, Init, Acc, O, W, L, Ctx, Lang: ?Sized>
  TryFoldWhileWith<F, Condition, Init, Acc, O, W, L, Ctx, Lang>
{
  /// Creates a new `FoldWhile` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(f: F, condition: Condition, init: Init, acc: Acc) -> Self {
    Self {
      f,
      condition,
      init,
      acc,
      _m: PhantomData,
      _cap: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<'inp, F, Condition, Init, Acc, O, W, L, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang>
  for TryFoldWhileWith<F, Condition, Init, Acc, O, W, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  Init: FnMut() -> O,
  Acc: FnMut(
    O,
    O,
    ParseState<'_, 'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  W: Window,
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
      let (peeked, emitter) = inp.peek_with_emitter::<W>()?;
      match self.condition.decide(peeked, emitter)? {
        Action::Stop => break,
        Action::Continue => {
          let cursor = inp.cursor().clone();
          let new = self.f.parse_input(inp)?;
          let state = ParseState::new(inp, cursor);
          output = (self.acc)(output, new, state)?;
        }
      }
    }

    Ok(output)
  }
}
