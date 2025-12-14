use core::marker::PhantomData;

use super::*;

mod parse_input;

/// A type-safe alias for configuring `Repeated` parsers.
///
/// Canonical configuration layout: `With<With<Trailing, Leading>, With<Maximum, Minimum>>`.
pub type RepeatedOptions<Max = (), Min = ()> = With<PhantomData<()>, With<Max, Min>>;

impl<MAX, MIN> MaxSpec for RepeatedOptions<MAX, MIN>
where
  MAX: MaxSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    self.secondary.primary.maximum()
  }
}

impl<MAX, MIN> MinSpec for RepeatedOptions<MAX, MIN>
where
  MIN: MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    self.secondary.secondary.minimum()
  }
}

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
/// The parser emits errors via the [`RepeatedEmitter`](crate::emitter::RepeatedEmitter) trait:
/// - Too few elements (below minimum)
/// - Too many elements (above maximum)
/// - Container full
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
pub struct Repeated<F, Condition, O, W, L, Ctx, Config = RepeatedOptions, Lang: ?Sized = ()> {
  pub(super) f: F,
  pub(super) condition: Condition,
  pub(super) config: Config,
  _m: PhantomData<O>,
  _cap: PhantomData<W>,
  _l: PhantomData<L>,
  _ctx: PhantomData<Ctx>,
  _lang: PhantomData<Lang>,
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized>
  Repeated<F, Condition, O, W, L, Ctx, RepeatedOptions, Lang>
{
  /// Creates a new `Repeated` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(f: F, condition: Condition) -> Self {
    Self::new_in(f, condition)
  }

  /// Creates a new `Repeated` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new_in(f: F, condition: Condition) -> Self {
    Self {
      f,
      condition,
      config: RepeatedOptions::new(PhantomData, With::new((), ())),
      _m: PhantomData,
      _cap: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<F, Condition, O, Options, W, L, Ctx, Lang: ?Sized>
  Repeated<F, Condition, O, W, L, Ctx, Options, Lang>
{
  /// Collects the parsed elements into the specified container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn collect<Container>(self) -> Collect<Self, Container, (), ()>
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
  ) -> Collect<Self, Container, (), ()> {
    Collect::new(self, container)
  }

  /// Creates a new `Delimited` parser with the given delimiters and separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimited_by<Open, Close, Delim>(
    self,
    left: Open,
    right: Close,
    delim: Delim,
  ) -> DelimitedBy<F, Condition, Open, Close, Delim, O, W, L, Ctx, Options, Lang> {
    DelimitedBy::new_in(self, left, right, delim)
  }
}

impl<F, Condition, O, Max, Min, W, L, Ctx, Lang: ?Sized>
  Repeated<F, Condition, O, W, L, Ctx, RepeatedOptions<Max, Min>, Lang>
{
  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(
    self,
    n: Min::Options,
  ) -> Repeated<F, Condition, O, W, L, Ctx, RepeatedOptions<Max, Minimum>, Lang>
  where
    Min: Apply<Minimum>,
  {
    Repeated {
      f: self.f,
      condition: self.condition,
      config: RepeatedOptions::new(
        self.config.primary,
        With::new(
          self.config.secondary.primary,
          Min::apply(self.config.secondary.secondary, n),
        ),
      ),
      _m: PhantomData,
      _cap: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(
    self,
    n: Max::Options,
  ) -> Repeated<F, Condition, O, W, L, Ctx, RepeatedOptions<Maximum, Min>, Lang>
  where
    Max: Apply<Maximum>,
  {
    Repeated {
      f: self.f,
      condition: self.condition,
      config: RepeatedOptions::new(
        self.config.primary,
        With::new(
          Max::apply(self.config.secondary.primary, n),
          self.config.secondary.secondary,
        ),
      ),
      _m: PhantomData,
      _cap: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn minimum(&self) -> usize
  where
    Min: MinSpec,
  {
    Min::minimum(&self.config.secondary.secondary)
  }

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn maximum(&self) -> usize
  where
    Max: MaxSpec,
  {
    Max::maximum(&self.config.secondary.primary)
  }
}
