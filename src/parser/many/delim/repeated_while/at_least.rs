use crate::{
  container::Container as ContainerT,
  emitter::{SeparatedEmitter, TooFewEmitter},
  error::syntax::TooFew,
};

use super::*;

impl<'inp, L, P, O, Condition, Container, Ctx, Delim, W, Lang: ?Sized>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    DelimitedBy<AtLeast<RepeatedWhile<P, Condition, O, W, L, Ctx, Lang>>, Delim>,
    Container,
    Ctx,
    Lang,
  >
where
  Delim: DelimiterSelector<'inp, L, Lang>,
  L: Lexer<'inp>,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  W: Window,
  Ctx::Emitter: SeparatedEmitter<'inp, Delim, L, Lang> + TooFewEmitter<'inp, L, Lang>,
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
    let min = self.parser.parser.minimum().get();

    DelimitedBy::<_, Delim>::new_in(self.parser.parser.parser_mut()).parse_repeated(
      inp,
      &mut self.container,
      |nums, inp, span| {
        if min > nums {
          inp
            .emitter()
            .emit_too_few(TooFew::of(span.clone(), nums, min))?;
        }

        Ok(())
      },
    )
  }
}
