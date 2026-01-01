use crate::{
  container::Container as ContainerT,
  emitter::{DelimitedEmitter, FullContainerEmitter},
};

use super::*;

impl<'inp, L, P, Open, Close, O, Condition, Container, Ctx, Delim, W, Lang: ?Sized>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    DelimitedBy<RepeatedOnCondition<P, Condition, O, W, L, Ctx, Lang>, Open, Close, Delim>,
    Container,
    Ctx,
    Lang,
  >
where
  Open: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
  Delim: Clone,
  L: Lexer<'inp>,
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
  W: Window,
  Ctx::Emitter: DelimitedEmitter<'inp, Delim, L, Lang> + FullContainerEmitter<'inp, O, L, Lang>,
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
    DelimitedBy::new_in(
      &mut self.parser.parser,
      &self.parser.left_classifier,
      &self.parser.right_classifier,
      &self.parser.delimiter,
    )
    .parse_repeated(inp, &mut self.container, |_, _, _| Ok(()))
  }
}
