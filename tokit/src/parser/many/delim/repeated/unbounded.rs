use crate::container::Container as ContainerT;

use super::*;

impl<'inp, L, P, O, Container, Ctx, Delim, Lang: ?Sized> ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<DelimitedBy<Repeated<P, O, L, Ctx, Lang>, Delim>, Container, Ctx, Lang>
where
  Delim: Delimiter<'inp, L, Lang>,
  L: Lexer<'inp>,
  P: TryParseInput<'inp, L, O, Ctx, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: FullContainerEmitter<'inp, L, Lang>,
  <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error: From<UnexpectedEot<L::Offset, Lang>>,
  Container: Default + ContainerT<O> + DelimiterHandler<'inp, L>,
{
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<Container, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    DelimitedBy::<_, Delim>::new_in(&mut self.parser.parser).parse_repeated(
      inp,
      &mut self.container,
      |_, _, _| Ok(()),
    )
  }
}
