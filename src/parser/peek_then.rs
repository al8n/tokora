use core::mem::MaybeUninit;

use mayber::MaybeRef;

use crate::CachedToken;

use super::*;

/// a
pub struct PeekThen<P, H, T, const N: usize> {
  parser: P,
  handler: H,
  _token: PhantomData<T>,
}

impl<'inp, P, H, L, O, Ctx, Lang, const N: usize> ParseInput<'inp, L, Option<O>, Ctx, Lang>
  for PeekThen<P, H, L::Token, N>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  H: FnMut(
    &mut [MaybeRef<'_, CachedToken<'_, L>>],
  ) -> Result<bool, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<Option<O>, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let mut buf = [const { MaybeUninit::uninit() }; N];
    let output = inp.peek(&mut buf);

    if (self.handler)(output)? {
      self.parser.parse_input(inp).map(Some)
    } else {
      Ok(None)
    }
  }
}
