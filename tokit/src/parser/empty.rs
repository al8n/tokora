use super::*;

/// A parser that always succeeds without consuming input, returning `()`.
///
/// This parser is the identity element for parser sequencing - it does nothing,
/// consumes nothing, and always succeeds immediately. It's useful for:
/// - **Default values**: Optional parsers that default to doing nothing
/// - **Conditional logic**: Placeholder in choice combinators
/// - **Testing**: Minimal parser for test scaffolding
///
/// # Examples
///
/// ## Optional Prefix
///
/// ```ignore
/// use tokit::parser::{empty, ParseInput};
///
/// // Parse optional 'pub' keyword
/// let parser = parse_pub()
///     .or_not()  // Returns Option<PubKeyword>
///     .unwrap_or_else(|| empty());  // Default to empty if absent
///
/// // Or simpler with peek_then_choice_or_not
/// let parser = (parse_pub(),)
///     .peek_then_choice_or_not(|peeked, _| {
///         match peeked.front() {
///             Some(Token::Pub) => Ok(Some(0)),
///             _ => Ok(None),  // Use empty implicitly
///         }
///     });
/// ```
///
/// ## Placeholder in Choices
///
/// ```ignore
/// // Parse statement: either a let-binding or nothing
/// let parser = peek_then_choice((
///     parse_let_stmt(),
///     empty().map(|_| Statement::Empty),
/// ));
/// ```
///
/// ## Default Behavior
///
/// ```ignore
/// // Conditional parsing
/// let parser = if needs_prefix {
///     parse_prefix().map(Some)
/// } else {
///     empty().map(|_| None)
/// };
/// ```
///
/// ## Testing
///
/// ```ignore
/// // Minimal parser for testing combinators
/// #[test]
/// fn test_map_combinator() {
///     let parser = empty().map(|_| 42);
///     assert_eq!(parser.parse(""), Ok(42));
/// }
/// ```
///
/// # When to Use
///
/// - **Defaults**: When an optional parser should default to no-op
/// - **Type matching**: When you need a parser that returns `()` for sequencing
/// - **Placeholders**: In development before implementing the real parser
///
/// **Don't use** when you actually want to fail - use a parser that returns an error instead.
///
/// # Performance
///
/// - **Memory**: Zero-sized type
/// - **Runtime**: O(1) - instant success, no input consumption
///
/// # See Also
///
/// - [`Todo`] - Placeholder that panics (for unimplemented parsers)
/// - [`or_not`](OrNot) - Make parsers optional
/// - [`map`](crate::parser::ParseInput::map) - Transform `()` to other types
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Empty(());

impl Empty {
  /// Creates a parser that always succeeds without consuming any input.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self(())
  }
}

impl<'inp, L, Ctx, Lang> ParseInput<'inp, L, (), Ctx, Lang> for Empty
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    _inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(())
  }
}
