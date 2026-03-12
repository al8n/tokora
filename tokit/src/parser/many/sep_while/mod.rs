use core::marker::PhantomData;

use derive_more::IsVariant;

use crate::{delimiter::Delimiter, punct::Punctuator};

use super::*;

mod parse;

mod delim;

/// A parser that parses a sequence of elements separated by a delimiter.
///
/// This combinator parses repeated occurrences of an element parser, expecting each
/// element to be separated by a delimiter (e.g., comma, semicolon). It provides
/// fine-grained control over:
/// - **Leading separators**: Allow/deny/require separators before the first element
/// - **Trailing separators**: Allow/deny/require separators after the last element
/// - **Repetition bounds**: Minimum and maximum number of elements
///
/// # Type Parameters
///
/// - `F`: The element parser
/// - `Sep`: Separator checker (e.g., comma punctuator, custom classifier)
/// - `Condition`: Decision function that determines when to stop parsing
/// - `O`: Output type of the element parser
/// - `Window`: Lookahead window size for the condition
/// - `L`: Lexer type
/// - `Ctx`: Parse context
/// - `Config`: Configuration options (trailing/leading/min/max)
/// - `Lang`: Language marker type (default `()`)
///
/// # Examples
///
/// ## Basic Comma-Separated List
///
/// ```ignore
/// use tokit::parser::{SeparatedWhile, ParseInput};
/// use generic_arraydeque::typenum::U1;
///
/// // Parse: element, element, element
/// let parser = SeparatedWhile::comma::<MyLexer, U1, Ctx>(
///     element_parser(),
///     |peeked, _| match peeked.front() {
///         None => Ok(Action::Stop),
///         Some(Token::Comma) => Ok(Action::Continue),
///         _ => Ok(Action::Stop),
///     }
/// ).collect::<Vec<_>>();
///
/// // Input: "1, 2, 3"
/// // Output: Ok(vec![1, 2, 3])
/// ```
///
/// ## With Trailing Separator
///
/// ```ignore
/// // Parse: element, element, element,  (trailing comma allowed)
/// let parser = SeparatedWhile::comma::<MyLexer, U1, Ctx>(
///     element_parser(),
///     stop_condition
/// )
/// .allow_trailing()   // Allow trailing comma
/// .collect::<Vec<_>>();
///
/// // Input: "1, 2, 3,"
/// // Output: Ok(vec![1, 2, 3])
/// ```
///
/// ## With Leading Separator
///
/// ```ignore
/// // Parse: , element, element  (leading comma allowed)
/// let parser = SeparatedWhile::comma::<MyLexer, U1, Ctx>(
///     element_parser(),
///     stop_condition
/// )
/// .allow_leading()    // Allow leading comma
/// .collect::<Vec<_>>();
///
/// // Input: ", 1, 2"
/// // Output: Ok(vec![1, 2])
/// ```
///
/// ## With Bounds
///
/// ```ignore
/// // Parse at least 1, at most 5 elements
/// let parser = SeparatedWhile::comma::<MyLexer, U1, Ctx>(
///     element_parser(),
///     stop_condition
/// )
/// .at_least(Minimum::new(1))
/// .at_most(Maximum::new(5))
/// .collect::<Vec<_>>();
/// ```
///
/// ## Custom Separator
///
/// ```ignore
/// // Parse elements separated by semicolons
/// let parser = SeparatedWhile::semicolon::<MyLexer, U1, Ctx>(
///     element_parser(),
///     stop_condition
/// ).collect::<Vec<_>>();
///
/// // Input: "a;b;c"
/// // Output: Ok(vec![a, b, c])
/// ```
///
/// # How It Works
///
/// 1. **Parse first element** (unless leading separator is required)
/// 2. **Loop**:
///    - Call `condition` to check if we should continue
///    - If `Action::Continue`: parse separator, then element
///    - If `Action::Stop`: break
/// 3. **Validate** trailing separator rules
/// 4. **Collect** parsed elements into container
///
/// # Error Handling
///
/// The parser emits errors via the [`SeparatedEmitter`](crate::emitter::SeparatedEmitter) trait:
/// - Missing separator between elements
/// - Unexpected leading separator (when denied)
/// - Unexpected trailing separator (when denied)
/// - Missing element after separator
/// - Too few or too many elements (when bounds set)
///
/// # Performance
///
/// - **Memory**: O(1) for the parser itself (elements collected into container)
/// - **Parsing**: O(n) where n is the number of elements
/// - **Lookahead**: O(W) per iteration where W is the window size
///
/// # See Also
///
/// - [`delimited`](SeparatedWhile::delimited) - Wrap in delimiters (e.g., `[...]` or `{...}`)
/// - [`repeated`](RepeatedWhile) - Repeat without separators
/// - [`Collect`](crate::parser::Collect) - Wrapper for collecting elements into a container
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SeparatedWhile<F, Sep, Condition, O, Window, L, Ctx, Lang: ?Sized = ()> {
  pub(super) f: F,
  pub(super) condition: Condition,
  pub(super) _sep: PhantomData<Sep>,
  pub(super) _m: PhantomData<O>,
  pub(super) _decision_window: PhantomData<Window>,
  pub(super) _l: PhantomData<L>,
  pub(super) _ctx: PhantomData<Ctx>,
  pub(super) _lang: PhantomData<Lang>,
}

impl<F, Sep, Condition, O, W, L, Ctx, Lang: ?Sized> Copy
  for SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>
where
  F: Copy,
  Sep: Copy,
  Condition: Copy,
{
}

impl<F, Sep, Condition, O, W, L, Ctx, Lang: ?Sized> Clone
  for SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang>
where
  F: Clone,
  Sep: Clone,
  Condition: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    Self {
      f: self.f.clone(),
      condition: self.condition.clone(),
      _sep: PhantomData,
      _m: PhantomData,
      _decision_window: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<F, Condition, O, W, L, Ctx, Lang: ?Sized>
  SeparatedWhile<F, (), Condition, O, W, L, Ctx, Lang>
{
  /// Creates a new `SeparatedWhile` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new<Sep>(
    f: F,
    condition: Condition,
  ) -> SeparatedWhile<F, Sep, Condition, O, W, L, Ctx, Lang> {
    SeparatedWhile {
      f,
      condition,
      _sep: PhantomData,
      _m: PhantomData,
      _decision_window: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<F, Sep, Condition, O, Window, L, Ctx, Lang: ?Sized>
  SeparatedWhile<F, Sep, Condition, O, Window, L, Ctx, Lang>
{
  /// Creates a mutable reference version of this `SeparatedWhile` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut(
    &mut self,
  ) -> SeparatedWhile<&mut F, Sep, &mut Condition, O, Window, L, Ctx, Lang> {
    SeparatedWhile {
      f: &mut self.f,
      condition: &mut self.condition,
      _sep: PhantomData,
      _m: PhantomData,
      _decision_window: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn at_least(self, minimum: usize) -> AtLeast<Self> {
    AtLeast::new(self, minimum)
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn at_most(self, maximum: usize) -> AtMost<Self> {
    AtMost::new(self, maximum)
  }

  /// Sets both the minimum and maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn bounded(self, minimum: usize, maximum: usize) -> Bounded<Self> {
    Bounded::new(self, maximum, minimum)
  }

  /// Sets allows trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn allow_trailing(self) -> AllowTrailing<Self> {
    AllowTrailing::new(self)
  }

  /// Sets requires trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn require_trailing(self) -> RequireTrailing<Self> {
    RequireTrailing::new(self)
  }

  /// Sets allows leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn allow_leading(self) -> AllowLeading<Self> {
    AllowLeading::new(self)
  }

  /// Sets requires leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn require_leading(self) -> RequireLeading<Self> {
    RequireLeading::new(self)
  }

  /// Creates a new `Delimited` parser with the given delimiters and separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimited<'inp, Delim>(self) -> DelimitedBy<Self, Delim>
  where
    Delim: Delimiter<'inp, L, Lang>,
  {
    DelimitedBy::<_, Delim>::new(self)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
pub(super) enum State<T, S> {
  Start,
  Element,
  Leading(Spanned<T, S>),
  Separator(Spanned<T, S>),
}
