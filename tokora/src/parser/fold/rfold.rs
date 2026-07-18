use super::*;

/// A parser that repeatedly applies another parser, buffers outputs, and folds them in reverse order.
#[derive(Debug, Clone, Copy)]
pub struct RFold<P, Init, Acc, L, O, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  parser: P,
  init: Init,
  acc: Acc,
  _output: PhantomData<O>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
  _cmpl: PhantomData<Cmpl>,
}

impl<P, Init, Acc, O, L, Ctx, Lang: ?Sized, Cmpl> RFold<P, Init, Acc, L, O, Ctx, Lang, Cmpl> {
  /// Creates a new `RFold` parser combinator.
  #[inline(always)]
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

impl<'inp, P, Init, Acc, O, L, Ctx, Lang, Cmpl> ParseInput<'inp, L, O, Ctx, Lang, Cmpl>
  for RFold<P, Init, Acc, L, O, Ctx, Lang, Cmpl>
where
  P: TryParseInput<'inp, L, O, Ctx, Lang, Cmpl>,
  Init: FnMut() -> O,
  Acc: FnMut(O, O) -> O,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
  Cmpl: Completeness,
{
  #[inline(always)]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
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
