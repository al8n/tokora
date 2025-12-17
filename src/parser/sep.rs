use core::marker::PhantomData;

use derive_more::{IsVariant, TryUnwrap, Unwrap};

// use crate::{Check, punct::*};

use crate::lexer::Span;

use super::*;

pub use allow_leading::*;
pub use allow_trailing::*;
pub use require_leading::*;
pub use require_trailing::*;

mod parse;

mod allow_leading;
mod allow_trailing;
mod require_leading;
mod require_trailing;

/// Leading-separator markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Deny(());

/// Leading-separator markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Allow(());

/// Requires a leading separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Require(());

/// A type-safe alias for configuring `SeparatedBy` parsers.
///
/// Canonical configuration layout: `With<With<Trailing, Leading>, With<Maximum, Minimum>>`.
pub type SeparatedByOptions<Trailing = (), Leading = (), Max = (), Min = ()> =
  With<With<Trailing, Leading>, With<Max, Min>>;

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
/// - `SepClassifier`: Separator checker (e.g., [`Comma`], custom punctuator)
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
/// use tokit::parser::{SeparatedBy, ParseInput};
/// use generic_arraydeque::typenum::U1;
///
/// // Parse: element, element, element
/// let parser = SeparatedBy::comma::<MyLexer, U1, Ctx>(
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
/// let parser = SeparatedBy::comma::<MyLexer, U1, Ctx>(
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
/// let parser = SeparatedBy::comma::<MyLexer, U1, Ctx>(
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
/// let parser = SeparatedBy::comma::<MyLexer, U1, Ctx>(
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
/// let parser = SeparatedBy::semicolon::<MyLexer, U1, Ctx>(
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
/// The parser emits errors via the [`SeparatedByEmitter`](crate::emitter::SeparatedByEmitter) trait:
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
/// - [`delimited_by`](SeparatedBy::delimited_by) - Wrap in delimiters (e.g., `[...]` or `{...}`)
/// - [`repeated`](Repeated) - Repeat without separators
/// - [`collect`](SeparatedBy::collect) - Collect into a container
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SeparatedBy<F, SepClassifier, Condition, O, Window, L, Ctx, Lang: ?Sized = ()> {
  pub(super) f: F,
  pub(super) sep: SepClassifier,
  pub(super) condition: Condition,
  pub(super) _m: PhantomData<O>,
  pub(super) _decision_window: PhantomData<Window>,
  pub(super) _l: PhantomData<L>,
  pub(super) _ctx: PhantomData<Ctx>,
  pub(super) _lang: PhantomData<Lang>,
}

impl<F, SepClassifier, Condition, O, W, L, Ctx, Lang: ?Sized>
  SeparatedBy<F, SepClassifier, Condition, O, W, L, Ctx, Lang>
{
  /// Creates a new `SeparatedBy` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new(f: F, sep_classifier: SepClassifier, condition: Condition) -> Self {
    Self {
      f,
      sep: sep_classifier,
      condition,
      _m: PhantomData,
      _decision_window: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<F, SepClassifier, Condition, O, Window, L, Ctx, Lang: ?Sized>
  SeparatedBy<F, SepClassifier, Condition, O, Window, L, Ctx, Lang>
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn as_mut(
    &mut self,
  ) -> SeparatedBy<&mut F, &mut SepClassifier, &mut Condition, O, Window, L, Ctx, Lang> {
    SeparatedBy {
      f: &mut self.f,
      sep: &mut self.sep,
      condition: &mut self.condition,
      _m: PhantomData,
      _decision_window: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

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

  // /// Creates a new `DelimitedSeparatedBy` parser.
  // #[cfg_attr(not(tarpaulin), inline(always))]
  // pub const fn delimited_by<Open, Close, Delim>(
  //   self,
  //   left: Open,
  //   right: Close,
  //   delim: Delim,
  // ) -> DelimitedSeparatedBy<
  //   F,
  //   SepClassifier,
  //   Condition,
  //   Open,
  //   Close,
  //   Delim,
  //   O,
  //   Window,
  //   L,
  //   Ctx,
  //   Lang,
  // > {
  //   DelimitedSeparatedBy::new_in(self, left, right, delim)
  // }
}

impl<F, SepClassifier, Condition, O, Window, L, Ctx, Lang: ?Sized>
  SeparatedBy<F, SepClassifier, Condition, O, Window, L, Ctx, Lang>
{
  /// Allows trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(self) -> SeparatedBy<F, SepClassifier, Condition, O, Window, L, Ctx, Lang> {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
      _m: PhantomData,
      _decision_window: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Requires a trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_trailing(
    self,
  ) -> SeparatedBy<F, SepClassifier, Condition, O, Window, L, Ctx, Lang> {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
      _m: PhantomData,
      _decision_window: PhantomData,
      _ctx: PhantomData,
      _l: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Allows leading separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_leading(self) -> SeparatedBy<F, SepClassifier, Condition, O, Window, L, Ctx, Lang> {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
      _m: PhantomData,
      _decision_window: PhantomData,
      _l: PhantomData,
      _ctx: PhantomData,
      _lang: PhantomData,
    }
  }

  /// Requires a leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_leading(
    self,
  ) -> SeparatedBy<F, SepClassifier, Condition, O, Window, L, Ctx, Lang> {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
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
}

// macro_rules! sep_by {
//   ($(
//     $(#[$meta:meta])*
//     $sep:ident
//   ),+$(,)?) => {
//     paste::paste! {
//       $(
//         impl<F, Condition, O> SeparatedBy<F, $sep, Condition, O, (), (), ()> {
//           #[doc = "Creates a new sequence with [" $sep:snake "](crate::punct::" $sep ") separator parser."]
//           #[cfg_attr(not(tarpaulin), inline(always))]
//           pub const fn [< $sep:snake >]<'inp, L, Ctx, W>(f: F, condition: Condition) -> SeparatedBy<F, $sep, Condition, O, W, L, Ctx>
//           where
//             L: Lexer<'inp>,
//             Ctx: ParseContext<'inp, L, ()>,
//             $sep: Check<L::Token>,
//             Condition: Decision<'inp, L, Ctx::Emitter, W, ()>,
//             W: Window,
//           {
//             SeparatedBy::new_in(f, <$sep>::PHANTOM, condition)
//           }
//         }

//         impl<F, Condition, O, Lang: ?Sized> SeparatedBy<F, $sep<(), (), Lang>, Condition, O, ()> {
//           #[doc = "Creates a new sequence with [" $sep:snake "](crate::punct::" $sep ") separator parser of a specific language."]
//           #[cfg_attr(not(tarpaulin), inline(always))]
//           pub const fn [< $sep:snake _of >]<'inp, L, W, Ctx>(f: F, condition: Condition) -> SeparatedBy<F, $sep, Condition, O, W>
//           where
//             L: Lexer<'inp>,
//             $sep<(), (), Lang>: Check<L::Token>,
//             Ctx: ParseContext<'inp, L, Lang>,
//             Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
//             W: Window,
//           {
//             SeparatedBy::new_in(f, <$sep>::PHANTOM.change_language_const(), condition)
//           }
//         }

//         #[cfg(test)]
//         const _: () = {
//           use crate::lexer::DummyLexer;
//           use generic_arraydeque::typenum::U1;

//           fn __assert_parse_impl__<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
//             Parser::with_parser(
//               SeparatedBy::[< $sep:snake >]::<DummyLexer, U1, ()>(
//                 Any::new(),
//                 |_toks: Peeked<'_, '_, DummyLexer, U1>, _: &mut Fatal<()>| Ok(Action::Continue),
//               )
//               .collect::<()>(),
//             )
//           }

//           fn __assert_parse_with_ctx_impl__<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
//             Parser::with_parser_and_context(SeparatedBy::[< $sep:snake >]::<DummyLexer, U1, ()>(
//                 Any::new(),
//                 |_toks: Peeked<'_, '_, DummyLexer, U1>, _: &mut Fatal<()>| Ok(Action::Continue),
//               )
//               .collect::<()>(), ())
//           }
//         };
//       )*
//     }
//   };
// }

// sep_by!(
//   Comma,
//   Semicolon,
//   Dot,
//   Colon,
//   Pipe,
//   Ampersand,
//   Hyphen,
//   Underscore,
//   DoubleColon,
//   Arrow,
//   FatArrow,
//   Tilde,
//   Trivia,
//   Slash,
//   BackSlash,
//   Percent,
//   Dollar,
//   Hash,
//   At,
// );

impl Apply<Allow> for () {
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, _: Self::Options) -> Allow {
    Allow(())
  }
}

impl Apply<Require> for () {
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, _: Self::Options) -> Require {
    Require(())
  }
}

/// Specification for leading/trailing separators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Unwrap, TryUnwrap)]
pub enum SepFixSpec {
  /// Denies leading/trailing separators.
  Deny(Deny),
  /// Allows leading/trailing separators.
  Allow(Allow),
  /// Requires leading/trailing separators.
  Require(Require),
}

pub(super) trait LeadingSpec {
  fn leading(&self) -> SepFixSpec;
}

impl<T: LeadingSpec> LeadingSpec for &mut T {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn leading(&self) -> SepFixSpec {
    (**self).leading()
  }
}

impl LeadingSpec for Deny {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn leading(&self) -> SepFixSpec {
    SepFixSpec::Deny(*self)
  }
}

impl LeadingSpec for Allow {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn leading(&self) -> SepFixSpec {
    SepFixSpec::Allow(*self)
  }
}

impl LeadingSpec for Require {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn leading(&self) -> SepFixSpec {
    SepFixSpec::Require(*self)
  }
}

pub(super) trait TrailingSpec {
  fn trailing(&self) -> SepFixSpec;
}

impl<T: TrailingSpec> TrailingSpec for &mut T {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn trailing(&self) -> SepFixSpec {
    (**self).trailing()
  }
}

impl TrailingSpec for Deny {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn trailing(&self) -> SepFixSpec {
    SepFixSpec::Deny(*self)
  }
}

impl TrailingSpec for Allow {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn trailing(&self) -> SepFixSpec {
    SepFixSpec::Allow(*self)
  }
}

impl TrailingSpec for Require {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn trailing(&self) -> SepFixSpec {
    SepFixSpec::Require(*self)
  }
}

impl TrailingSpec for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn trailing(&self) -> SepFixSpec {
    SepFixSpec::Deny(Deny(()))
  }
}

impl LeadingSpec for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn leading(&self) -> SepFixSpec {
    SepFixSpec::Deny(Deny(()))
  }
}

impl<T, L, MAX, MIN> MaxSpec for SeparatedByOptions<T, L, MAX, MIN>
where
  MAX: MaxSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    self.secondary.primary.maximum()
  }
}

impl<T, L, MAX, MIN> MinSpec for SeparatedByOptions<T, L, MAX, MIN>
where
  MIN: MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    self.secondary.secondary.minimum()
  }
}

impl<T, L, MAX, MIN> TrailingSpec for SeparatedByOptions<T, L, MAX, MIN>
where
  T: TrailingSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn trailing(&self) -> SepFixSpec {
    T::trailing(&self.primary.primary)
  }
}

impl<T, L, MAX, MIN> LeadingSpec for SeparatedByOptions<T, L, MAX, MIN>
where
  L: LeadingSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn leading(&self) -> SepFixSpec {
    L::leading(&self.primary.secondary)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
pub(super) enum State<T, S> {
  Start,
  Element,
  Leading(Spanned<T, S>),
  Separator(Spanned<T, S>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CachedRepeatedSep<S> {
  // the number of consecutive separators
  count: usize,
  // the current cached separator token.
  span: S,
}

impl<S> CachedRepeatedSep<S> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S) -> Self {
    Self { count: 1, span }
  }

  /// Returns the number of consecutive separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn count(&self) -> usize {
    self.count
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_span(self) -> S {
    self.span
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn expand(&mut self, end: S::Offset)
  where
    S: Span,
  {
    self.count += 1;
    *self.span.end_mut() = end;
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CachedSepTok<T, S> {
  // the number of consecutive separators
  count: usize,
  // the current cached separator token.
  tok: Spanned<T, S>,
}

impl<T, S> From<Spanned<T, S>> for CachedSepTok<T, S> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(tok: Spanned<T, S>) -> Self {
    Self::new(tok)
  }
}

impl<T, S> CachedSepTok<T, S> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(tok: Spanned<T, S>) -> Self {
    Self { count: 1, tok }
  }

  /// Returns the number of consecutive separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn count(&self) -> usize {
    self.count
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_components(self) -> (S, T) {
    self.tok.into_components()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_inner(self) -> Spanned<T, S> {
    self.tok
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn update_replace(&mut self, tok: Spanned<T, S>) -> Spanned<T, S> {
    self.count += 1;
    core::mem::replace(&mut self.tok, tok)
  }
}
