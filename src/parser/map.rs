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
pub struct Map<F, MapFn, O> {
  parser: F,
  map_fn: MapFn,
  _marker: PhantomData<O>,
}

impl<F, MapFn, O> Map<F, MapFn, O> {
  /// Creates a new `Map` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(parser: F, map_fn: MapFn) -> Self {
    Self {
      parser,
      map_fn,
      _marker: PhantomData,
    }
  }
}

impl<'inp, F, MapFn, L, O, NO, E, C> ParseInput<'inp, L, NO, E, C> for Map<F, MapFn, O>
where
  F: ParseInput<'inp, L, O, E, C>,
  MapFn: FnMut(O) -> NO,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> NO {
    let result = self.parser.parse_input(input);
    (self.map_fn)(result)
  }
}

/// A parser that transforms the `Ok` variant of a `Result` output.
///
/// This is similar to `Map`, but only transforms successful results,
/// leaving errors unchanged. This is particularly useful when working
/// with parsers that return `Result<T, E>`.
///
/// # Type Parameters
///
/// - `F`: The inner parser
/// - `MapFn`: The mapping function for successful values
/// - `T`: The success type of the inner parser's Result
/// - `E`: The error type (passed through unchanged)
///
/// # Examples
///
/// ```ignore
/// // Parse a number token and extract its value
/// let parser = Any::parser()
///     .map_ok(|tok| match tok {
///         Token::Number(n) => n,
///         _ => 0.0,
///     });
/// ```
pub struct MapOk<F, MapFn, T, E> {
  parser: F,
  map_fn: MapFn,
  _marker: PhantomData<(T, E)>,
}

impl<F, MapFn, T, E> MapOk<F, MapFn, T, E> {
  /// Creates a new `MapOk` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(parser: F, map_fn: MapFn) -> Self {
    Self {
      parser,
      map_fn,
      _marker: PhantomData,
    }
  }
}

impl<'inp, F, MapFn, L, T, NT, Err, Em, C> ParseInput<'inp, L, Result<NT, Err>, Em, C>
  for MapOk<F, MapFn, T, Err>
where
  F: ParseInput<'inp, L, Result<T, Err>, Em, C>,
  MapFn: FnMut(T) -> NT,
  L: Lexer<'inp>,
  Em: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, Em, C>) -> Result<NT, Err> {
    self.parser.parse_input(input).map(&mut self.map_fn)
  }
}

impl<F, L, O, Error, E, C> With<F, Parser<(), L, O, Error, ParserOptions<L, E, C>>> {
  /// Apply a mapping function to transform the output of this parser.
  ///
  /// This creates a new parser that applies the given function to the result
  /// of this parser, transforming the output type.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let parser = Any::parser()
  ///     .map(|token| token.kind());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map<MapFn, NO>(
    self,
    map_fn: MapFn,
  ) -> With<Map<F, MapFn, O>, Parser<(), L, NO, Error, ParserOptions<L, E, C>>>
  where
    MapFn: FnMut(O) -> NO,
  {
    With::new(
      Map::new(self.primary, map_fn),
      Parser {
        f: (),
        opts: self.secondary.opts,
        _marker: PhantomData,
      },
    )
  }
}

impl<F, L, T, Error, E, C> With<F, Parser<(), L, Result<T, Error>, Error, ParserOptions<L, E, C>>> {
  /// Apply a mapping function to transform the `Ok` variant of a `Result` output.
  ///
  /// This is similar to `.map()`, but only transforms successful results,
  /// leaving errors unchanged. This is useful when working with parsers
  /// that return `Result<T, Error>`.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let parser = Any::parser()
  ///     .map_ok(|token| match token {
  ///         Token::Number(n) => n,
  ///         _ => 0,
  ///     });
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map_ok<MapFn, NT>(
    self,
    map_fn: MapFn,
  ) -> With<MapOk<F, MapFn, T, Error>, Parser<(), L, Result<NT, Error>, Error, ParserOptions<L, E, C>>>
  where
    MapFn: FnMut(T) -> NT,
  {
    With::new(
      MapOk::new(self.primary, map_fn),
      Parser {
        f: (),
        opts: self.secondary.opts,
        _marker: PhantomData,
      },
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::{DummyLexer, DummyToken};

  use super::*;

  fn assert_map_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Result<(), ()>, ()> {
    Any::parser().map(|_tok: Result<DummyToken, ()>| Ok(()))
  }

  fn assert_map_ok_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Result<(), ()>, ()> {
    Any::parser().map_ok(|_tok: DummyToken| ())
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_map_parse_impl();
    let _ = assert_map_ok_parse_impl();
  }
}
