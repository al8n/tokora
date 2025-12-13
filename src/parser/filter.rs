use super::*;

/// A parser that validates the output of another parser based on a validation function.
///
/// This combinator parses successfully only if the inner parser succeeds AND
/// the validation function returns `Ok(())`. If validation fails with `Err(e)`,
/// that error is propagated.
///
/// The validator receives both the parsed value and its span, allowing for
/// context-aware error construction.
///
/// # Type Parameters
///
/// - `P`: The inner parser
/// - `F`: The validation function
/// - `O`: The output type
///
/// # Examples
///
/// ```ignore
/// use tokit::parser::{any, ParseInput};
///
/// // Parse any token, but only accept identifiers
/// let parser = any::<MyLexer>()
///     .filter(|tok, span| {
///         if tok.is_identifier() {
///             Ok(())
///         } else {
///             Err(UnexpectedToken::new(*span, tok.kind()))
///         }
///     });
///
/// // Parse a number, but only accept even numbers
/// let parser = parse_number()
///     .filter(|&n, span| {
///         if n % 2 == 0 {
///             Ok(())
///         } else {
///             Err(OddNumberError::new(*span))
///         }
///     });
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Filter<P, F> {
  parser: P,
  filter: F,
}

impl<P, F> Filter<P, F> {
  /// Creates a new `Validate` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new<'inp, L, O, Ctx>(parser: P, filter: F) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseInput<'inp, L, O, Ctx, ()>,
    F: FnMut(&O) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self::of(parser, filter)
  }

  /// Creates a new `Validate` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of<'inp, L, O, Ctx, Lang>(parser: P, filter: F) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Lang: ?Sized,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    F: FnMut(&O) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    Self { parser, filter }
  }
}

impl<'inp, P, F, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for Filter<P, F>
where
  P: ParseInput<'inp, L, O, Ctx, Lang>,
  F: FnMut(&O) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self
      .parser
      .parse_input(input)
      .and_then(|output| (self.filter)(&output).map(|_| output))
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_filter_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::new().apply(Any::new().filter(|_tok: &DummyToken| Ok(())))
  }

  fn assert_filter_parse_with_ctx_impl<'inp>() -> impl Parse<'inp, DummyLexer, DummyToken, ()> {
    Parser::with_context(()).apply(Any::new().filter(|_tok: &DummyToken| Ok(())))
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_filter_parse_impl();
    let _ = assert_filter_parse_with_ctx_impl();
  }
}
