use crate::{
  container::Container as ContainerT,
  emitter::{DelimitedEmitter, TooFewEmitter, TooManyEmitter},
  error::syntax::{TooFew, TooMany},
};

use super::*;

impl<'inp, L, P, O, Container, Ctx, Delim, Lang: ?Sized> ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<DelimitedBy<Bounded<Repeated<P, O, L, Ctx, Lang>>, Delim>, Container, Ctx, Lang>
where
  Delim: DelimiterSelector<'inp, L, Lang>,
  L: Lexer<'inp>,
  P: TryParseInput<'inp, L, O, Ctx, Lang>,
  Ctx::Emitter: DelimitedEmitter<'inp, Delim, L, Lang>
    + TooManyEmitter<'inp, L, Lang>
    + TooFewEmitter<'inp, L, Lang>,
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
    let min = self.parser.parser.minimum().get();

    DelimitedBy::new_in(self.parser.parser.parser_mut()).parse_repeated(
      inp,
      &mut self.container,
      |nums, inp, span| {
        if min > nums {
          inp
            .emitter()
            .emit_too_few(TooFew::of(span.clone(), nums, min))?;
        }

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
