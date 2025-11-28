use crate::{Check, emitter::SeparatedByEmitter, error::token::UnexpectedToken, punct::Comma};

use super::*;

impl<'inp, T, Classifier, Lang> Check<T, SeqSepAction<'inp, T::Kind>>
  for Comma<(), Classifier, Lang>
where
  T: Token<'inp>,
  Classifier: Fn(&T) -> SeqSepAction<'inp, T::Kind>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn check(&self, target: &T) -> SeqSepAction<'inp, T::Kind> {
    self.content()(target)
  }
}

/// A parser that accepts an empty comma-separated sequence.
pub const fn comma_seq<'inp, F, Classifier, L, O, Container, Error>(
  parser: F,
  classifier: Classifier,
) -> SeqSep<F, Comma<(), Classifier>, O, Container>
where
  F: ParseInput<'inp, L, ParseResult<'inp, O, L, Noop<Error>>, Noop<Error>, DefaultCache<'inp, L>>,
  Classifier: Check<L::Token, SeqSepAction<'inp, <L::Token as Token<'inp>>::Kind>>,
  L: Lexer<'inp>,
  Error: From<<L::Token as Token<'inp>>::Error>
    + From<UnexpectedToken<'inp, L::Token, <L::Token as Token<'inp>>::Kind, L::Span>>,
  Noop<Error>: SeparatedByEmitter<'inp, O, Classifier, L>,
{
  SeqSep::new(parser, Comma::with_content((), classifier))
}
