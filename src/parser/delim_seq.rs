use super::*;

/// A parser that parses a construct delimited by left and right tokens.
///
/// See also: [`DelimSepSeq`]
pub struct DelimitedSeparatedBy<P, SepClassifier, Condition, Open, Close, Delim, O, W, Config = SeparatedByOptions> {
  parser: SeparatedBy<P, SepClassifier, Condition, O, W, Config>,
  left_classifier: Open,
  right_classifier: Close,
  delimiter: Delim,
  _m: PhantomData<O>,
  _window: PhantomData<W>,
}

