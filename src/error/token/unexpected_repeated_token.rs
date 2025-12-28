use core::marker::PhantomData;

use crate::{
  lexer::{Lexer, Span},
  utils::SimpleSpan,
};

use super::Repeated;

/// An error indicating that an unexpected repeated tokens were found during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnexpectedRepeatedToken<T, S = SimpleSpan, Lang: ?Sized = ()> {
  span: S,
  count: usize,
  _t: PhantomData<T>,
  _lang: PhantomData<Lang>,
}

impl<T, S> UnexpectedRepeatedToken<T, S> {
  /// Creates a new `UnexpectedRepeatedToken` error.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(span: S, count: usize) -> Self {
    Self::of(span, count)
  }
}

impl<T, S, Lang: ?Sized> UnexpectedRepeatedToken<T, S, Lang> {
  /// Creates a new `UnexpectedRepeatedToken` error for the given language.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn of(span: S, count: usize) -> Self {
    Self {
      span,
      count,
      _lang: PhantomData,
      _t: PhantomData,
    }
  }

  /// Returns the reference to the span covering the repeated tokens.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_ref(&self) -> &S {
    &self.span
  }

  /// Returns the mutable reference to the span covering the repeated tokens.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span_mut(&mut self) -> &mut S {
    &mut self.span
  }

  /// Returns the span covering the repeated tokens.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn span(&self) -> S
  where
    S: Copy,
  {
    self.span
  }

  /// Returns the number of repeated tokens found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn count(&self) -> usize {
    self.count
  }

  /// Expands the span covering the repeated tokens.
  ///
  /// The `span` parameter is used to extend the end of the current span,
  /// and the `count` parameter indicates how many additional repeated tokens were found.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn expand(&mut self, span: S, count: usize) -> &mut Self
  where
    S: Span,
  {
    *self.span.end_mut() = span.end();
    self.count += count;
    self
  }
}

impl<T, S, Lang: ?Sized> From<UnexpectedRepeatedToken<T, S, Lang>> for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: UnexpectedRepeatedToken<T, S, Lang>) -> Self {}
}

/// A type alias for an `UnexpectedPrefix` error indicating a repeated punctuator was found for a given lexer and separator.
pub type UnexpectedRepeatedOf<'inp, Sep, L, Lang = ()> =
  UnexpectedRepeatedToken<<L as Lexer<'inp>>::Token, <L as Lexer<'inp>>::Span, Repeated<Sep, Lang>>;
