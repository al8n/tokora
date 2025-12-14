use super::*;

/// A placeholder parser that panics when executed - for unimplemented parsers.
///
/// This parser is a development tool for **incrementally building parsers**. It allows
/// you to sketch out parser structure with type-correct placeholders that will panic
/// if accidentally executed.
///
/// Similar to Rust's `todo!()` macro, this parser:
/// - ✅ **Compiles**: Type-checks correctly in parser combinators
/// - ❌ **Panics at runtime**: Crashes if actually called during parsing
/// - 📍 **Tracks location**: Shows where the `Todo` was created (via `#[track_caller]`)
///
/// # Type Parameters
///
/// - `O`: The output type the parser would produce if implemented
///
/// # Examples
///
/// ## Sketching Parser Structure
///
/// ```ignore
/// use tokit::parser::{todo_parser, ParseInput};
///
/// // Define high-level structure first
/// fn parse_function<'inp>() -> impl Parse<'inp, MyLexer, Function> {
///     todo_parser()  // Implement later
/// }
///
/// // Use in larger parser
/// let parser = parse_struct()
///     .or(parse_function())  // Compiles but panics if called
///     .or(parse_enum());
/// ```
///
/// ## Type-Driven Development
///
/// ```ignore
/// // Build parser incrementally
/// struct Module {
///     imports: Vec<Import>,
///     items: Vec<Item>,
/// }
///
/// fn parse_module<'inp>() -> impl Parse<'inp, MyLexer, Module> {
///     todo_parser::<Import>()
///         .repeated(stop_condition)
///         .collect::<Vec<_>>()
///         .then(parse_item().repeated(stop_condition).collect())
///         .map(|(imports, items)| Module { imports, items })
/// }
///
/// // Compiles! Implement parse_item later.
/// ```
///
/// ## Testing Partial Implementations
///
/// ```ignore
/// // Test the parts you've implemented
/// fn parse_statement<'inp>() -> impl Parse<'inp, MyLexer, Statement> {
///     peek_then_choice((
///         parse_let(),       // ✅ Implemented
///         parse_if(),        // ✅ Implemented
///         todo_parser(),     // 🚧 For/while loops - TODO
///     ))
/// }
///
/// // Tests pass as long as they don't hit the todo
/// ```
///
/// # Panics
///
/// **Always panics** when `parse_input()` is called:
/// ```text
/// thread 'main' panicked at 'not yet implemented', src/parser.rs:42:5
/// ```
///
/// The panic location points to where `.parse()` was called (thanks to `#[track_caller]`).
///
/// # When to Use
///
/// ✅ **Good uses**:
/// - Prototyping parser structure before implementation
/// - Incremental development of large grammars
/// - Placeholder for rarely-used grammar branches
///
/// ❌ **Don't use for**:
/// - Production code (obviously - it panics!)
/// - Parsers you intend to execute (use [`Empty`] for no-ops)
/// - Error handling (use proper error types)
///
/// # Comparison with Empty
///
/// | Parser | Runtime Behavior | Use Case |
/// |--------|-----------------|----------|
/// | [`Empty`] | Always succeeds | Actual no-op parser |
/// | **`Todo`** | Always panics | Development placeholder |
///
/// # Performance
///
/// - **Memory**: Zero-sized type
/// - **Runtime**: N/A - panics before doing work
///
/// # See Also
///
/// - [`Empty`] - Parser that actually does nothing (doesn't panic)
/// - `todo!()` macro - Rust's built-in todo placeholder
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Todo<O: ?Sized>(PhantomData<O>);

impl<O: ?Sized> Todo<O> {
  /// Creates a parser that is not yet implemented.
  ///
  /// **Panics** if `parse_input()` is called.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  pub const fn new() -> Self {
    Self(PhantomData)
  }
}

impl<'inp, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for Todo<O>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    _inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    todo!()
  }
}
