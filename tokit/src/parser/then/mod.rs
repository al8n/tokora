use core::marker::PhantomData;

use crate::{TryParseInput, try_parse_input::ParseAttempt};

use super::*;

pub use and_then::*;
pub use and_then_with::*;
pub use ignore_then::*;
pub use then_ignore::*;
pub use then_value::*;

mod and_then;
mod and_then_with;
mod ignore_then;
mod then_ignore;
mod then_value;

/// A parser that sequentially composes two parsers.
///
/// This combinator runs the first parser, then runs the second parser,
/// returning both results as a tuple.
///
/// See also [`AndThen`] and [`AndThenWith`] for variants that use the output
/// of the first parser to determine the second parser.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Then<F, T, O, U, L, Ctx, Lang: ?Sized = ()> {
  parser: F,
  then: T,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _lang: PhantomData<Lang>,
  _ctx: PhantomData<Ctx>,
}

impl<F, T, O, U, L, Ctx, Lang: ?Sized> Then<F, T, O, U, L, Ctx, Lang> {
  /// Creates a new `Then` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(parser: F, then: T) -> Self {
    Self {
      parser,
      then,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _o: PhantomData,
      _u: PhantomData,
    }
  }
}

impl<'inp, F, T, L, O, U, Ctx, Lang> ParseInput<'inp, L, (O, U), Ctx, Lang>
  for Then<F, T, O, U, L, Ctx, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  T: ParseInput<'inp, L, U, Ctx, Lang>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<(O, U), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    let a = self.parser.parse_input(input)?;
    let b = self.then.parse_input(input)?;
    Ok((a, b))
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_ignore_then_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Any::new().ignore_then(Any::new()))
  }

  fn assert_then_ignore_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Any::new().then_ignore(Any::new()))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_ignore_then_parse_impl();
    let _ = assert_then_ignore_parse_impl();
  }
}
