use super::*;

/// A parser that unwraps `Option<T>` output to `T`, panicking on `None`.
///
/// This combinator converts a parser that returns `Option<O>` into one that returns `O`
/// by calling `.unwrap()` on the result. If the inner parser returns `None`, this will
/// **panic** with a message indicating the source location of the `.unwrapped()` call.
///
/// # When to Use
///
/// Use this when:
/// - You've already validated that a value must be present (e.g., with [`or_not`](OrNot))
/// - During development/debugging to find logic errors
/// - In situations where `None` represents a programming error, not a parsing error
///
/// **Warning**: This is similar to `.unwrap()` in Rust - it will panic if the value is `None`.
/// For error handling, prefer using [`filter_map`](FilterMap) or working with `Option` directly.
///
/// # Type Parameters
///
/// - `P`: The inner parser (must output `Option<O>`)
/// - `O`: The unwrapped output type
/// - `Ctx`: Parse context
/// - `Lang`: Language marker type (default `()`)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```ignore
/// use tokit::parser::{ParseInput, OrNot};
///
/// // Parse an optional element, then unwrap it
/// let parser = optional_element()
///     .unwrapped();  // Panics if None
///
/// // This is equivalent to:
/// let parser = optional_element()
///     .map(|opt| opt.unwrap());
/// ```
///
/// ## With `or_not`
///
/// ```ignore
/// // Parse optional whitespace, default to empty if not found
/// let parser = whitespace()
///     .or_not()           // Returns Option<Whitespace>
///     .unwrap_or_default();  // Better than .unwrapped() - no panic
///
/// // Or use unwrapped if you know it must be present:
/// let parser = required_element()
///     .or_not()           // Made optional for some reason
///     .unwrapped();       // Convert back to required
/// ```
///
/// # Panics
///
/// Panics if the inner parser returns `Ok(None)`. The panic message includes the
/// source location of the `.unwrapped()` call (tracked via `#[track_caller]`).
///
/// # See Also
///
/// - [`or_not`](OrNot) - Makes a parser optional (returns `Option<T>`)
/// - [`filter_map`](FilterMap) - Transform and filter with error handling
/// - [`map`](Map) - Transform output without unwrapping
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Unwrapped<P, O, Ctx, Lang: ?Sized = ()> {
  pub(crate) parser: P,
  pub(crate) _m: PhantomData<O>,
  pub(crate) _ctx: PhantomData<Ctx>,
  pub(crate) _lang: PhantomData<Lang>,
}

impl<P, O, Ctx, Lang: ?Sized> Unwrapped<P, O, Ctx, Lang> {
  /// Creates a new `Unwrapped` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  pub(super) const fn new(parser: P) -> Self {
    Self {
      parser,
      _m: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<'inp, P, L, O, Ctx, Lang> ParseInput<'inp, L, O, Ctx, Lang> for Unwrapped<P, O, Ctx, Lang>
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  P: ParseInput<'inp, L, Option<O>, Ctx, Lang>,
  Lang: ?Sized,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    inp: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<O, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    self.parser.parse_input(inp).map(Option::unwrap)
  }
}
