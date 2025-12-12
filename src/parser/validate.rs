use super::*;

/// A parser that validates the output with full location context (span and slice).
///
/// This combinator is similar to `Filter` but provides the validator with both
/// the span and source slice, enabling more detailed error messages that can
/// reference the exact source text.
///
/// The validator receives the parsed value, its span, and the source slice,
/// allowing for comprehensive context-aware error construction.
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
/// use logosky::parser::{any, ParseInput};
///
/// // Parse an identifier and validate it's not a reserved keyword
/// let parser = parse_identifier()
///     .validate(|name, span, slice| {
///         if KEYWORDS.contains(name) {
///             Err(ReservedKeywordError::new(*span, slice.to_string()))
///         } else {
///             Ok(())
///         }
///     });
///
/// // Parse a number and validate range with source context
/// let parser = parse_number()
///     .validate(|&n, span, slice| {
///         if n >= 0 && n <= 255 {
///             Ok(())
///         } else {
///             Err(OutOfRangeError::new(*span, slice.to_string(), 0, 255))
///         }
///     });
/// ```
pub struct Validate<P, F> {
  parser: P,
  validator: F,
}

impl<P, F> Validate<P, F> {
  /// Creates a new `Validate` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new<'inp, L, O, Ctx>(parser: P, validator: F) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, ()>,
    P: ParseInput<'inp, L, O, Ctx, ()>,
    F: FnMut(&O) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
  {
    Self { parser, validator }
  }

  /// Creates a new `Validate` combinator for the specified language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of<'inp, L, O, Ctx, Lang>(parser: P, validator: F) -> Self
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Lang: ?Sized,
    P: ParseInput<'inp, L, O, Ctx, Lang>,
    F: FnMut(&O) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  {
    Self { parser, validator }
  }
}

impl<'inp, P, F, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for Validate<P, F>
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
    input: &mut InputRef<'inp, '_, L, Ctx::Emitter, Ctx::Cache, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    self
      .parser
      .parse_input(input)
      .and_then(|output| (self.validator)(&output).map(|_| output))
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{DummyLexer, DummyToken};

  use super::*;

  fn assert_validate_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Spanned<DummyToken>, ()> {
    Parser::new().apply(
      Any::new()
        .spanned()
        .validate(|_tok: &Spanned<DummyToken>| Ok(())),
    )
  }

  fn assert_validate_parse_with_ctx_impl<'inp>()
  -> impl Parse<'inp, DummyLexer, Spanned<DummyToken>, ()> {
    Parser::with_context(()).apply(
      Any::new()
        .spanned()
        .validate(|_tok: &Spanned<DummyToken>| Ok(())),
    )
  }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_validate_parse_impl();
    let _ = assert_validate_parse_with_ctx_impl();
  }
}
