use core::marker::PhantomData;

use super::*;

mod parse;

/// A parser that repeatedly applies an element parser until a condition signals to stop.
///
/// This combinator repeatedly parses elements **without separators** until the `condition`
/// function returns [`Action::Stop`]. It provides fine-grained control over:
/// - **When to stop**: User-defined lookahead-based decision function
/// - **Repetition bounds**: Minimum and maximum number of elements
/// - **Delimiters**: Can wrap in delimiters like `[...]` or `{...}`
///
/// Unlike [`SeparatedBy`] which expects delimiters between elements, `Repeated` parses
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
/// use tokit::parser::{ParseInput, Repeated, Action};
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
/// # Difference from `SeparatedBy`
///
/// | Feature | `Repeated` | `SeparatedBy` |
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
/// - [`SeparatedBy`] - Parse elements with separators (e.g., commas)
/// - [`delimited_by`](Repeated::delimited_by) - Wrap in delimiters
/// - [`collect`](Repeated::collect) - Collect into a container
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Repeated<F, O, L, Ctx, Lang: ?Sized = ()> {
  pub(super) f: F,
  _m: PhantomData<O>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, O, L, Ctx, Lang: ?Sized> Repeated<F, O, L, Ctx, Lang> {
  /// Creates a new `Repeated` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn new(f: F) -> Self {
    Self {
      f,
      _m: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<F, O, L, Ctx, Lang: ?Sized> Repeated<F, O, L, Ctx, Lang> {
  /// Collects the parsed elements into the specified container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn collect<Container>(self) -> Collect<Self, Container, Ctx, Lang>
  where
    Container: Default,
  {
    Collect::new(self, Container::default())
  }

  /// Collects the parsed elements with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn collect_with<Container>(
    self,
    container: Container,
  ) -> Collect<Self, Container, Ctx, Lang> {
    Collect::new(self, container)
  }

  /// Delimits the parser with the given open and close classifiers and delimiter.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimited_by<Open, Close, Delim>(
    self,
    left: Open,
    right: Close,
    delim: Delim,
  ) -> DelimitedBy<Self, Open, Close, Delim> {
    DelimitedBy::new_in(self, left, right, delim)
  }
}

impl<F, O, L, Ctx, Lang: ?Sized> Repeated<F, O, L, Ctx, Lang> {
  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, n: usize) -> AtLeast<Repeated<F, O, L, Ctx, Lang>> {
    self.apply(Minimum::new(n))
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, n: usize) -> AtMost<Repeated<F, O, L, Ctx, Lang>> {
    self.apply(Maximum::new(n))
  }

  /// Sets both the minimum and maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn bounded(self, min: usize, max: usize) -> Bounded<Repeated<F, O, L, Ctx, Lang>> {
    self.apply(With::new(Maximum::new(max), Minimum::new(min)))
  }
}

impl<F, O, L, Ctx, Lang: ?Sized> Apply<AtLeast<Self>> for Repeated<F, O, L, Ctx, Lang> {
  type Options = Minimum;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> AtLeast<Self> {
    AtLeast::new(self, options.get())
  }
}

impl<F, O, L, Ctx, Lang: ?Sized> Apply<AtMost<Self>> for Repeated<F, O, L, Ctx, Lang> {
  type Options = Maximum;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> AtMost<Self> {
    AtMost::new(self, options.get())
  }
}

impl<F, O, L, Ctx, Lang: ?Sized> Apply<Bounded<Self>> for Repeated<F, O, L, Ctx, Lang> {
  type Options = With<Maximum, Minimum>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Bounded<Self> {
    Bounded::new(self, options.primary.get(), options.secondary.get())
  }
}

impl<F, O, L, Ctx, Lang: ?Sized> Apply<Bounded<Repeated<F, O, L, Ctx, Lang>>>
  for AtMost<Repeated<F, O, L, Ctx, Lang>>
{
  type Options = Minimum;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Bounded<Repeated<F, O, L, Ctx, Lang>> {
    Bounded::new(self.parser, self.maximum.get(), options.get())
  }
}

impl<F, O, L, Ctx, Lang: ?Sized> Apply<Bounded<Repeated<F, O, L, Ctx, Lang>>>
  for AtLeast<Repeated<F, O, L, Ctx, Lang>>
{
  type Options = Maximum;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Bounded<Repeated<F, O, L, Ctx, Lang>> {
    Bounded::new(self.parser, options.get(), self.minimum.get())
  }
}
