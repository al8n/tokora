use crate::parser::sep::{LeadingSpec, TrailingSpec};

use super::*;

mod parse_input;

// struct DecisionWithClose<'a, D, Close>(With<&'a mut D, &'a mut Close>);

// impl<'inp, D, Close, L, E, W, Lang> Decision<'inp, L, E, W, Lang>
//   for DecisionWithClose<'_, D, Close>
// where
//   L: Lexer<'inp>,
//   D: Decision<'inp, L, E, W, Lang>,
//   Close: Check<L::Token, Result<(), <L::Token as Token<'inp>>::Kind>>,
//   W: Window,
//   Lang: ?Sized,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn decide(&mut self, toks: Peeked<'_, 'inp, L, W>, emitter: &mut E) -> Result<Action, E::Error>
//   where
//     E: Emitter<'inp, L, Lang>,
//   {
//     if let Some(t) = toks.front() {
//       let t = t
//         .as_maybe_ref()
//         .map(|t| t.token().copied(), |t| t.token())
//         .into_inner();

//       if let Lexed::Token(t) = t.data() {
//         if self.0.secondary().check(t).is_ok() {
//           return Ok(Action::End);
//         }
//       }
//     }

//     self.0.primary_mut().decide(toks, emitter)
//   }
// }

/// A parser that parses a construct delimited by left and right tokens.
///
/// See also: [`DelimSepSeq`]
pub struct DelimitedSeparatedBy<
  P,
  SepClassifier,
  Condition,
  Open,
  Close,
  Delim,
  O,
  W,
  Options = SeparatedByOptions,
> {
  parser: SeparatedBy<P, SepClassifier, Condition, O, W, Options>,
  left_classifier: Open,
  right_classifier: Close,
  delimiter: Delim,
  _m: PhantomData<O>,
  _window: PhantomData<W>,
}

impl<P, SepClassifier, Condition, Open, Close, Delim, O, Trailing, Leading, Max, Min, Window>
  DelimitedSeparatedBy<
    P,
    SepClassifier,
    Condition,
    Open,
    Close,
    Delim,
    O,
    Window,
    SeparatedByOptions<Trailing, Leading, Max, Min>,
  >
{
  /// Returns the specification for leading separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn leading(&self) -> SepFixSpec
  where
    Leading: LeadingSpec,
  {
    self.parser.leading()
  }

  /// Returns the specification for trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn trailing(&self) -> SepFixSpec
  where
    Trailing: TrailingSpec,
  {
    self.parser.trailing()
  }

  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn minimum(&self) -> usize
  where
    Min: MinSpec,
  {
    self.parser.minimum()
  }

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn maximum(&self) -> usize
  where
    Max: MaxSpec,
  {
    self.parser.maximum()
  }
}

impl<P, SepClassifier, Condition, Open, Close, Delim, O, W, Options>
  DelimitedSeparatedBy<P, SepClassifier, Condition, Open, Close, Delim, O, W, Options>
{
  /// Collects the parsed elements into the specified container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn collect<Container>(self) -> Collect<Self, Container>
  where
    Container: Default,
  {
    Collect::new(self, Container::default())
  }

  /// Collects the parsed elements with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn collect_with<Container>(self, container: Container) -> Collect<Self, Container> {
    Collect::new(self, container)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(
    parser: SeparatedBy<P, SepClassifier, Condition, O, W, Options>,
    left: Open,
    right: Close,
    delim: Delim,
  ) -> Self {
    Self {
      parser,
      left_classifier: left,
      right_classifier: right,
      delimiter: delim,
      _m: PhantomData,
      _window: PhantomData,
    }
  }
}
