use super::*;

/// A parser that repeatedly applies another parser, buffers outputs, and folds them in reverse order.
#[derive(Debug, Clone, Copy)]
pub struct RFold<P, Init, Acc, L, O, Ctx, Lang: ?Sized = ()> {
  parser: P,
  init: Init,
  acc: Acc,
  _output: PhantomData<O>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<P, Init, Acc, O, L, Ctx, Lang: ?Sized> RFold<P, Init, Acc, L, O, Ctx, Lang> {
  /// Creates a new `RFold` parser combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
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
  for RFold<P, Init, Acc, L, O, Ctx, Lang>
where
  P: TryParseInput<'inp, L, O, Ctx, Lang>,
  Init: FnMut() -> O,
  Acc: FnMut(O, O) -> O,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    let mut buf = std::vec::Vec::new();
    while let ParseAttempt::Accept(value) = self.parser.try_parse_input(inp)? {
      buf.push(value);
    }

    Ok(buf.into_iter().rfold((self.init)(), &mut self.acc))
  }
}
