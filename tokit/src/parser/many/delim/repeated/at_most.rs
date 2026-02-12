use crate::{container::Container as ContainerT, emitter::TooManyEmitter, error::syntax::TooMany};

use super::*;

impl<'inp, L, P, O, Container, Ctx, Delim, Lang: ?Sized> ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<DelimitedBy<AtMost<Repeated<P, O, L, Ctx, Lang>>, Delim>, Container, Ctx, Lang>
where
  Delim: Delimiter<'inp, L, Lang>,
  L: Lexer<'inp>,
  P: TryParseInput<'inp, L, O, Ctx, Lang>,
  Ctx::Emitter: FullContainerEmitter<'inp, L, Lang> + TooManyEmitter<'inp, L, Lang>,
  Ctx: ParseContext<'inp, L, Lang>,
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
    let max = self.parser.parser.maximum().get();

    DelimitedBy::<_, Delim>::new_in(self.parser.parser.parser_mut()).parse_repeated(
      inp,
      &mut self.container,
      |nums, inp, span| {
        if nums > max {
          inp
            .emitter()
            .emit_too_many(TooMany::of(span.clone(), nums, max))?;
        }
        Ok(())
      },
    )
  }
}
