use super::*;

/// A parser that accepts any token with optional padding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Padded<P>(P);

impl<P> Padded<P> {
  /// Creates a parser that accepts any token with optional padding.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(parser: P) -> Self {
    Self(parser)
  }
}

impl<'inp, P, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for Padded<P>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, <Ctx>::Emitter, <Ctx>::Cache, Lang>,
  ) -> Result<O, <<Ctx>::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    inp.sync_until(|t, _| !t.is_trivia(), || None)?;
    let output = self.0.parse_input(inp)?;
    inp.sync_until(|t, _| !t.is_trivia(), || None)?;
    Ok(output)
  }
}
