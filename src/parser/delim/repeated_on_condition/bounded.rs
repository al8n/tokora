use crate::{
  container::Container as ContainerT,
  emitter::{DelimitedEmitter, TooFewEmitter, TooManyEmitter},
  error::syntax::{TooFew, TooMany},
};

use super::*;

impl<'inp, L, P, Open, Close, O, Condition, Container, Ctx, Delim, W, Lang: ?Sized>
  ParseInput<'inp, L, Container, Ctx, Lang>
  for Collect<
    DelimitedBy<Bounded<RepeatedWhile<P, Condition, O, W, L, Ctx, Lang>>, Open, Close, Delim>,
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
  Ctx::Emitter: DelimitedEmitter<'inp, Delim, L, Lang>
    + TooManyEmitter<'inp, O, L, Lang>
    + TooFewEmitter<'inp, O, L, Lang>,
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

    DelimitedBy::new_in(
      self.parser.parser.parser_mut(),
      &self.parser.left_classifier,
      &self.parser.right_classifier,
      &self.parser.delimiter,
    )
    .parse_repeated(inp, &mut self.container, |nums, inp, span| {
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
    })
  }
}
