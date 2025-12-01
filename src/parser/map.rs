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
pub struct Map<A, U, F> {
  parser: A,
  map_fn: F,
  _m: PhantomData<U>,
}

impl<A, F, U> Map<A, U, F> {
  /// Creates a new `Map` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(parser: A, map_fn: F) -> Self {
    Self {
      parser,
      map_fn,
      _m: PhantomData,
    }
  }
}

impl<'inp, A, F, L, O, U, E, C> ParseInput<'inp, L, U, E, C> for Map<A, O, F>
where
  A: ParseInput<'inp, L, O, E, C>,
  F: FnMut(O) -> U,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> Result<U, E::Error> {
    self.parser.parse_input(input).map(&mut self.map_fn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{DummyLexer, DummyToken};

  use super::*;

  fn assert_map_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
    Parser::new().apply(Any::new().map(|_tok: DummyToken| ()))
  }

  fn assert_map_parse_with_cache_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
    Parser::new()
      .with_cache::<()>(())
      .apply(Any::new().map(|_tok: DummyToken| ()))
  }

  fn assert_map_parse_with_emitter_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
    Parser::new()
      .with_emitter::<Fatal<()>>(Fatal::new())
      .with_cache::<()>(())
      .apply(Any::new().map(|_tok: DummyToken| ()))
  }


  #[test]
  fn assert_parse_impl() {
    let _ = assert_map_parse_impl();
  }
}
