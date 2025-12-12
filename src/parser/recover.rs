use super::*;

/// A parser that attempts to recover from a fatal error using a recovery parser.
pub struct Recover<P, R> {
  parser: P,
  recoverer: R,
}

impl<P, R> Recover<P, R> {
  /// Creates a new `Recover` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(parser: P, recoverer: R) -> Self {
    Self { parser, recoverer }
  }
}

impl<'inp, P, R, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for Recover<P, R>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  R: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let ckp = inp.save();
    match self.parser.parse_input(inp) {
      Ok(output) => Ok(output),
      Err(_) => {
        inp.go(ckp);
        self.recoverer.parse_input(inp)
      }
    }
  }
}

/// A parser that attempts to recover from a fatal error using a recovery parser.
pub struct InplaceRecover<P, R> {
  parser: P,
  recoverer: R,
}

impl<P, R> InplaceRecover<P, R> {
  /// Creates a new `InplaceRecover` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(parser: P, recoverer: R) -> Self {
    Self { parser, recoverer }
  }
}

impl<'inp, P, R, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for InplaceRecover<P, R>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  R: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    match self.parser.parse_input(inp) {
      Ok(output) => Ok(output),
      Err(_) => self.recoverer.parse_input(inp),
    }
  }
}
