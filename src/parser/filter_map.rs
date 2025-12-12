use core::marker::PhantomData;

use super::*;

/// A parser that validates and transforms the output of another parser.
///
/// This combinator is similar to `Filter` but also transforms the value.
/// It parses successfully only if the inner parser succeeds AND the mapping
/// function returns `Ok(value)`. If the function returns `Err(e)`, that error
/// is propagated.
///
/// The mapper receives both the parsed value and its span, allowing for
/// context-aware transformation and error construction.
///
/// # Type Parameters
///
/// - `P`: The inner parser
/// - `F`: The mapping/validation function
/// - `O`: The input output type
///
/// # Examples
///
/// ```ignore
/// use tokit::parser::{any, ParseInput};
///
/// // Parse a token and extract its text, failing if empty
/// let parser = any::<MyLexer>()
///     .filter_map(|tok, span| {
///         let text = tok.text();
///         if !text.is_empty() {
///             Ok(text)
///         } else {
///             Err(EmptyTokenError::new(*span))
///         }
///     });
///
/// // Parse a number string and convert to integer
/// let parser = parse_string()
///     .filter_map(|s, span| {
///         s.parse::<i32>()
///             .map_err(|_| InvalidNumberError::new(*span))
///     });
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FilterMap<P, F, O> {
  parser: P,
  mapper: F,
  _marker: PhantomData<O>,
}

impl<P, F, O> FilterMap<P, F, O> {
  /// Creates a new `Validate` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new<'inp, L, U, Ctx>(parser: P, filter: F) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseInput<'inp, L, O, Ctx, ()>,
    F: FnMut(O) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self::of(parser, filter)
  }

  /// Creates a new `Validate` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of<'inp, L, U, Ctx, Lang>(parser: P, filter: F) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Lang: ?Sized,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    F: FnMut(O) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    Self {
      parser,
      mapper: filter,
      _marker: PhantomData,
    }
  }
}

impl<'inp, P, F, L, O, U, Ctx, Lang> ParseInput<'inp, L, U, Ctx, Lang> for FilterMap<P, F, O>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  F: FnMut(O) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<U, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self.parser.parse_input(input).and_then(&mut self.mapper)
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_filter_map_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
    Parser::new().apply(Any::new().filter_map(|_tok: DummyToken| Ok(())))
  }

  fn assert_filter_map_parse_with_ctx_impl<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
    Parser::with_context(()).apply(Any::new().filter_map(|_tok: DummyToken| Ok(())))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_filter_map_parse_impl();
    let _ = assert_filter_map_parse_with_ctx_impl();
  }
}
