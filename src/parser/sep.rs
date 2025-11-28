use core::marker::PhantomData;

use derive_more::{IsVariant, TryUnwrap, Unwrap};

use crate::utils::Expected;

use super::*;

pub use comma::*;

mod comma;
mod parser_input;

/// A marker type representing the maximum number of elements allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Maximum(pub usize);

impl Maximum {
  /// The maximum possible value for `Maximum`.
  pub const MAX: Self = Self::new(usize::MAX);

  /// Creates a new `Maximum`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(n: usize) -> Self {
    Self(n)
  }

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

/// A marker type representing the minimum number of elements required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Minimum(usize);

impl Minimum {
  /// The minimum possible value for `Minimum`.
  pub const MIN: Self = Self::new(0);

  /// Creates a new `Minimum`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(n: usize) -> Self {
    Self(n)
  }

  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

/// Leading-separator markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Deny(());

/// Leading-separator markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Allow(());

/// Requires a leading separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Require(());

/// A type-safe alias for configuring `SeqSep` parsers.
///
/// Canonical configuration layout: `With<With<Trailing, Leading>, With<Maximum, Minimum>>`.
pub type SeqSepOptions<Trailing = (), Leading = (), Max = (), Min = ()> =
  With<With<Trailing, Leading>, With<Max, Min>>;

/// A hint used during parsing sequences with separators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
pub enum SeqSepAction<'a, Kind> {
  /// Indicates the start of the sequence, hint to stop.
  End,
  /// Indicates a separator was found, hint to parse another element.
  Separator,
  /// Indicates a token belongs to an element was found, hint to continue parsing.
  Continue,
  /// Indicates that we should skip the token, useful for trivial tokens like whitespace, comments, etc.
  Skip,
  /// Indicates this is an unexpected token, but this token should not terminate the parsing.
  Unexpected(Option<Expected<'a, Kind>>),
}

/// A parser that parses a sequence of elements separated by a specific separator.
pub struct SeqSep<F, Classifier, O, Container, Config = SeqSepOptions> {
  f: F,
  classifier: Classifier,
  config: Config,
  _m: PhantomData<(O, Config, Container)>,
}

impl<F, Classifier, O, Container> SeqSep<F, Classifier, O, Container> {
  /// Creates a new `SeqSep` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(f: F, classifier: Classifier) -> Self {
    Self::with_container(f, classifier)
  }

  /// Creates a new `SeqSep` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn with_container(f: F, classifier: Classifier) -> Self {
    Self {
      f,
      classifier,
      config: SeqSepOptions::new(With::new((), ()), With::new((), ())),
      _m: PhantomData,
    }
  }
}

impl<F, Classifier, O, Container, Trailing, Leading, Max, Min>
  SeqSep<F, Classifier, O, Container, SeqSepOptions<Trailing, Leading, Max, Min>>
{
  /// Attach a custom emitter to the parser.
  pub fn with_emitter<E>(self, emitter: E) -> WithEmitter<Self, E> {
    WithEmitter {
      inner: self,
      emitter,
    }
  }

  /// Attach custom cache options to the parser.
  pub fn with_cache<'inp, L, C>(self, options: C::Options) -> WithCache<'inp, Self, L, C>
  where
    L: Lexer<'inp>,
    C: Cache<'inp, L>,
  {
    WithCache {
      inner: self,
      cache_opts: options,
      _marker: PhantomData,
    }
  }

  /// Allows trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(
    self,
  ) -> SeqSep<F, Classifier, O, Container, SeqSepOptions<Allow, Leading, Max, Min>>
  where
    Trailing: Apply<Allow>,
  {
    SeqSep {
      f: self.f,
      classifier: self.classifier,
      config: SeqSepOptions::new(
        With::new(Allow(()), self.config.primary.secondary),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }

  /// Requires a trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_trailing(
    self,
  ) -> SeqSep<F, Classifier, O, Container, SeqSepOptions<Require, Leading, Max, Min>>
  where
    Trailing: Apply<Require>,
  {
    SeqSep {
      f: self.f,
      classifier: self.classifier,
      config: SeqSepOptions::new(
        With::new(Require(()), self.config.primary.secondary),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }

  /// Allows leading separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_leading(
    self,
  ) -> SeqSep<F, Classifier, O, Container, SeqSepOptions<Trailing, Allow, Max, Min>>
  where
    Leading: Apply<Allow>,
  {
    SeqSep {
      f: self.f,
      classifier: self.classifier,
      config: SeqSepOptions::new(
        With::new(self.config.primary.primary, Allow(())),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }

  /// Requires a leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_leading(
    self,
  ) -> SeqSep<F, Classifier, O, Container, SeqSepOptions<Trailing, Require, Max, Min>>
  where
    Leading: Apply<Require>,
  {
    SeqSep {
      f: self.f,
      classifier: self.classifier,
      config: SeqSepOptions::new(
        With::new(self.config.primary.primary, Require(())),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(
    self,
    n: Min::Options,
  ) -> SeqSep<F, Classifier, O, Container, SeqSepOptions<Trailing, Leading, Max, Minimum>>
  where
    Min: Apply<Minimum>,
  {
    SeqSep {
      f: self.f,
      classifier: self.classifier,
      config: SeqSepOptions::new(
        self.config.primary,
        With::new(
          self.config.secondary.primary,
          Min::apply(self.config.secondary.secondary, n),
        ),
      ),
      _m: PhantomData,
    }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(
    self,
    n: Max::Options,
  ) -> SeqSep<F, Classifier, O, Container, SeqSepOptions<Trailing, Leading, Maximum, Min>>
  where
    Max: Apply<Maximum>,
  {
    SeqSep {
      f: self.f,
      classifier: self.classifier,
      config: SeqSepOptions::new(
        self.config.primary,
        With::new(
          Max::apply(self.config.secondary.primary, n),
          self.config.secondary.secondary,
        ),
      ),
      _m: PhantomData,
    }
  }

  /// Returns the specification for leading separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn leading(&self) -> SepFixSpec
  where
    Leading: LeadingSpec,
  {
    Leading::leading(&self.config.primary.secondary)
  }

  /// Returns the specification for trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(private_bounds)]
  pub fn trailing(&self) -> SepFixSpec
  where
    Trailing: TrailingSpec,
  {
    Trailing::trailing(&self.config.primary.primary)
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

impl<L, F, Classifier, O, Output, Container, E, C, Config> sealed::Sealed<'_, L, Output, E, C>
  for SeqSep<F, Classifier, O, Container, Config>
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
enum State<T, S> {
  Start,
  Element,
  Leading(Spanned<T, S>),
  /// the span is the start of the
  Leadings(S),
  Separator(Spanned<T, S>),
  RepeatedSeparator(S),
}

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

impl Apply<Maximum> for () {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Maximum {
    Maximum(options)
  }
}

impl Apply<Minimum> for () {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Minimum {
    Minimum(options)
  }
}

impl Apply<Maximum> for Maximum {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Maximum {
    Maximum(options)
  }
}

impl Apply<Minimum> for Minimum {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn apply(self, options: Self::Options) -> Minimum {
    Minimum(options)
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

trait LeadingSpec {
  fn leading(&self) -> SepFixSpec;
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

trait TrailingSpec {
  fn trailing(&self) -> SepFixSpec;
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

trait MinSpec {
  fn minimum(&self) -> usize;
}

impl MinSpec for Minimum {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    self.0
  }
}

impl MinSpec for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    0
  }
}

trait MaxSpec {
  fn maximum(&self) -> usize;
}

impl MaxSpec for Maximum {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    self.0
  }
}

impl MaxSpec for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    usize::MAX
  }
}

impl<T, L, MAX, MIN> MaxSpec for SeqSepOptions<T, L, MAX, MIN>
where
  MAX: MaxSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    self.secondary.primary.maximum()
  }
}

impl<T, L, MAX, MIN> MinSpec for SeqSepOptions<T, L, MAX, MIN>
where
  MIN: MinSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    self.secondary.secondary.minimum()
  }
}

impl<T, L, MAX, MIN> TrailingSpec for SeqSepOptions<T, L, MAX, MIN>
where
  T: TrailingSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn trailing(&self) -> SepFixSpec {
    T::trailing(&self.primary.primary)
  }
}

impl<T, L, MAX, MIN> LeadingSpec for SeqSepOptions<T, L, MAX, MIN>
where
  L: LeadingSpec,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn leading(&self) -> SepFixSpec {
    L::leading(&self.primary.secondary)
  }
}
