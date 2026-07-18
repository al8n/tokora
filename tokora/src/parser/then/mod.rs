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
pub struct Then<F, T, O, U, L, Ctx, Lang: ?Sized = (), Cmpl = Complete> {
  parser: F,
  then: T,
  _o: PhantomData<O>,
  _u: PhantomData<U>,
  _l: PhantomData<L>,
  _lang: PhantomData<Lang>,
  _ctx: PhantomData<Ctx>,
  _cmpl: PhantomData<Cmpl>,
}

impl<F, T, O, U, L, Ctx, Lang: ?Sized, Cmpl> Then<F, T, O, U, L, Ctx, Lang, Cmpl> {
  /// Creates a new `Then` combinator.
  #[inline(always)]
  pub(crate) const fn new(parser: F, then: T) -> Self {
    Self {
      parser,
      then,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
      _o: PhantomData,
      _u: PhantomData,
      _cmpl: PhantomData,
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
  #[inline(always)]
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
mod tests;
