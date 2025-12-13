use crate::parser::sep::{LeadingSpec, TrailingSpec};

use super::*;

mod parse_input;

/// A parser that parses a construct delimited by left and right tokens.
///
/// See also: [`DelimSepSeq`]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DelimitedSeparatedBy<
  P,
  SepClassifier,
  Condition,
  Open,
  Close,
  Delim,
  O,
  W,
  L,
  Ctx,
  Options = SeparatedByOptions,
  Lang: ?Sized = (),
> {
  parser: SeparatedBy<P, SepClassifier, Condition, O, W, L, Ctx, Options, Lang>,
  left_classifier: Open,
  right_classifier: Close,
  delimiter: Delim,
  _m: PhantomData<O>,
  _window: PhantomData<W>,
}

impl<
  P,
  SepClassifier,
  Condition,
  Open,
  Close,
  Delim,
  O,
  Trailing,
  Leading,
  Max,
  Min,
  Window,
  L,
  Ctx,
  Lang: ?Sized,
>
  DelimitedSeparatedBy<
    P,
    SepClassifier,
    Condition,
    Open,
    Close,
    Delim,
    O,
    Window,
    L,
    Ctx,
    SeparatedByOptions<Trailing, Leading, Max, Min>,
    Lang,
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

impl<P, SepClassifier, Condition, Open, Close, Delim, O, W, L, Ctx, Options, Lang: ?Sized>
  DelimitedSeparatedBy<P, SepClassifier, Condition, Open, Close, Delim, O, W, L, Ctx, Options, Lang>
{
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

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn new_in(
    parser: SeparatedBy<P, SepClassifier, Condition, O, W, L, Ctx, Options, Lang>,
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
