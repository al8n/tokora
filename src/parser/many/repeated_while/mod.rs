use core::marker::PhantomData;

use crate::delimiter::DelimiterSelector;

use super::*;

mod at_least;
mod at_most;
mod bounded;
mod unbounded;

/// A parser that repeatedly applies an element parser until a condition signals to stop.
///
/// This combinator repeatedly parses elements **without separators** until the `condition`
/// function returns [`Action::Stop`]. It provides fine-grained control over:
/// - **When to stop**: User-defined lookahead-based decision function
/// - **Repetition bounds**: Minimum and maximum number of elements
/// - **Delimiters**: Can wrap in delimiters like `[...]` or `{...}`
///
/// Unlike [`SeparatedWhile`] which expects delimiters between elements, `RepeatedWhile` parses
/// consecutive elements with no separators.
///
/// # Type Parameters
///
/// - `F`: The element parser
/// - `Condition`: Decision function that determines when to stop parsing (receives lookahead)
/// - `O`: Output type of the element parser
/// - `W`: Lookahead window size for the condition
/// - `L`: Lexer type
/// - `Ctx`: Parse context
/// - `Config`: Configuration options (min/max bounds)
/// - `Lang`: Language marker type (default `()`)
///
/// # Examples
///
/// ## Basic Repetition
///
/// ```ignore
/// use tokit::parser::{ParseInput, RepeatedWhile, Action};
/// use generic_arraydeque::typenum::U1;
///
/// // Parse numbers until we hit a non-number token
/// let parser = number_parser()
///     .repeated(|mut peeked: Peeked<_, _, U1>, _| {
///         match peeked.front() {
///             None => Ok(Action::Stop),
///             Some(Token::Number(_)) => Ok(Action::Continue),
///             _ => Ok(Action::Stop),
///         }
///     })
///     .collect::<Vec<_>>();
///
/// // Input: "123 456 789 abc"
/// // Output: Ok(vec![123, 456, 789])
/// ```
///
/// ## With Bounds
///
/// ```ignore
/// // Parse at least 1, at most 10 elements
/// let parser = element_parser()
///     .repeated(stop_condition)
///     .at_least(Minimum::new(1))
///     .at_most(Maximum::new(10))
///     .collect::<Vec<_>>();
/// ```
///
/// ## Delimited Repetition
///
/// ```ignore
/// // Parse: [element element element]
/// let parser = element_parser()
///     .repeated(stop_condition)
///     .delimited_by(
///         |t| matches!(t, Token::BracketOpen),
///         |t| matches!(t, Token::BracketClose),
///         Delimiter::Bracket
///     )
///     .collect::<Vec<_>>();
///
/// // Input: "[1 2 3 4]"
/// // Output: Ok(vec![1, 2, 3, 4])
/// ```
///
/// ## Stop on Specific Token
///
/// ```ignore
/// use generic_arraydeque::typenum::U1;
///
/// // Parse tokens until we see a semicolon
/// let parser = token_parser()
///     .repeated::<_, U1>(|mut peeked, _| {
///         match peeked.front() {
///             Some(Token::Semicolon) | None => Ok(Action::Stop),
///             _ => Ok(Action::Continue),
///         }
///     })
///     .collect::<Vec<_>>();
/// ```
///
/// # How It Works
///
/// 1. **Parse first element**
/// 2. **Loop**:
///    - Call `condition` with lookahead to check if we should continue
///    - If `Action::Continue`: parse next element
///    - If `Action::Stop`: break
/// 3. **Validate** min/max bounds
/// 4. **Collect** parsed elements into container
///
/// # Difference from `SeparatedWhile`
///
/// | Feature | `RepeatedWhile` | `SeparatedWhile` |
/// |---------|-----------|---------------|
/// | **Separators** | ❌ No separators | ✅ Elements separated by delimiter |
/// | **Use Case** | Consecutive elements | Comma/semicolon-separated lists |
/// | **Example** | `1 2 3 4` | `1, 2, 3, 4` |
///
/// # Error Handling
///
/// The parser emits errors via the traits:
/// - [`TooFewEmitter`](crate::emitter::TooFewEmitter): Too few elements (below minimum)
/// - [`TooManyEmitter`](crate::emitter::TooManyEmitter): Too many elements (above maximum)
///
/// # Performance
///
/// - **Memory**: O(1) for the parser itself (elements collected into container)
/// - **Parsing**: O(n) where n is the number of elements
/// - **Lookahead**: O(W) per iteration where W is the window size
///
/// # See Also
///
/// - [`SeparatedWhile`] - Parse elements with separators (e.g., commas)
/// - [`delimited_by`](RepeatedWhile::delimited_by) - Wrap in delimiters
/// - [`Collect`](crate::parser::Collect) - Wrapper for collecting elements into a container
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RepeatedWhile<F, Condition, O, W, L, Ctx, Lang: ?Sized = ()> {
  pub(super) f: F,
  pub(super) condition: Condition,
  _m: PhantomData<O>,
  _cap: PhantomData<W>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized> RepeatedWhile<F, Condition, O, W, L, Ctx, Lang> {
  /// Creates a new `RepeatedWhile` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(f: F, condition: Condition) -> Self {
    Self::new_in(f, condition)
  }

  /// Creates a new `RepeatedWhile` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new_in(f: F, condition: Condition) -> Self {
    Self {
      f,
      condition,
      _m: PhantomData,
      _cap: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized> RepeatedWhile<F, Condition, O, W, L, Ctx, Lang> {
  /// Delimits the parser with the given open and close classifiers and delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimited<'inp, Delim>(self) -> DelimitedBy<Self, Delim>
  where
    Delim: DelimiterSelector<'inp, L, Lang>,
  {
    DelimitedBy::new_in(self)
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized> RepeatedWhile<F, Condition, O, W, L, Ctx, Lang> {
  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, n: usize) -> AtLeast<RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>> {
    self.apply(Minimum::new(n))
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, n: usize) -> AtMost<RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>> {
    self.apply(Maximum::new(n))
  }

  /// Sets both the minimum and maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bounded(
    self,
    min: usize,
    max: usize,
  ) -> Bounded<RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>> {
    self.apply(With::new(Maximum::new(max), Minimum::new(min)))
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized> Apply<AtLeast<Self>>
  for RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>
{
  type Options = Minimum;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> AtLeast<Self> {
    AtLeast::new(self, options.get())
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized> Apply<AtMost<Self>>
  for RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>
{
  type Options = Maximum;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> AtMost<Self> {
    AtMost::new(self, options.get())
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized> Apply<Bounded<Self>>
  for RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>
{
  type Options = With<Maximum, Minimum>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Bounded<Self> {
    Bounded::new(self, options.primary.get(), options.secondary.get())
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized>
  Apply<Bounded<RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>>>
  for AtMost<RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>>
{
  type Options = Minimum;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(
    self,
    options: Self::Options,
  ) -> Bounded<RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>> {
    Bounded::new(self.parser, self.maximum.get(), options.get())
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized>
  Apply<Bounded<RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>>>
  for AtLeast<RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>>
{
  type Options = Maximum;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(
    self,
    options: Self::Options,
  ) -> Bounded<RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>> {
    Bounded::new(self.parser, options.get(), self.minimum.get())
  }
}

impl<'inp, 'c, L, F, Condition, O, Ctx, Lang: ?Sized, W>
  RepeatedWhile<F, Condition, O, W, L, Ctx, Lang>
{
  fn parse<Container>(
    &mut self,
    inp: &mut InputRef<'inp, 'c, L, Ctx, Lang>,
    container: &mut Container,
    on_stop: impl FnOnce(
      usize,
      &mut InputRef<'inp, 'c, L, Ctx, Lang>,
      &L::Span,
    ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    F: ParseInput<'inp, L, O, Ctx, Lang>,
    Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
    W: Window,
    Ctx::Emitter: Emitter<'inp, L, Lang>,
    Ctx: ParseContext<'inp, L, Lang>,
    Container: crate::container::Container<O>,
  {
    let ckp = inp.save();
    let mut nums = 0;

    loop {
      let (peeked, emitter) = inp.peek_with_emitter::<W>()?;

      match self.condition.decide(peeked, emitter) {
        Err(err) => return Err(err),
        Ok(action) => match action {
          Action::Stop => {
            let span = inp.span_since(ckp.cursor());
            return on_stop(nums, inp, &span).map(|_| span);
          }
          Action::Continue => {
            container.push(self.f.parse_input(inp)?);
            nums += 1;
          }
        },
      }
    }
  }
}
