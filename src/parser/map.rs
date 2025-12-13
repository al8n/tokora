use core::marker::PhantomData;

use super::*;

/// A parser that transforms the output of another parser using a mapping function.
///
/// This combinator applies a function to the successful output of a parser,
/// allowing you to transform the parsed value into a different type.
///
/// # Type Parameters
///
/// - `F`: The inner parser
/// - `MapFn`: The mapping function
/// - `O`: The output type of the inner parser
///
/// # Examples
///
/// ```ignore
/// // Parse a token and extract just its kind
/// let parser = Any::parser()
///     .map(|tok| tok.kind());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Map<F, G, L, Ctx, O, O2, Lang: ?Sized = ()> {
  parser: F,
  map_fn: G,
  _o: PhantomData<O>,
  _o2: PhantomData<O2>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, G, L, Ctx, O, O2, Lang: ?Sized> Map<F, G, L, Ctx, O, O2, Lang> {
  /// Creates a new `Map` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(parser: F, map_fn: G) -> Self {
    Self {
      parser,
      map_fn,
      _o: PhantomData,
      _o2: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<'inp, F, G, L, O, U, Ctx, Lang> ParseInput<'inp, L, U, Ctx, Lang>
  for Map<F, G, L, Ctx, O, U, Lang>
where
  F: ParseInput<'inp, L, O, Ctx, Lang>,
  G: FnMut(O) -> U,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self.parser.parse_input(input).map(&mut self.map_fn)
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_map_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
    Parser::new().apply(Any::new().map(|_tok: DummyToken| ()))
  }

  fn assert_map_parse_with_ctx_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
    Parser::with_context(()).apply(Any::new().map(|_tok: DummyToken| ()))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_map_parse_impl();
    let _ = assert_map_parse_with_ctx_impl();
  }
}
