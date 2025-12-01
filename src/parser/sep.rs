use core::marker::PhantomData;

use derive_more::{IsVariant, TryUnwrap, Unwrap};

use crate::{Check, Token, punct::*, utils::Expected};

use super::*;

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
enum SeqSepAction<'a, Kind> {
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

impl<'a, Kind> From<Action<'a, Kind>> for SeqSepAction<'a, Kind> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(action: Action<'a, Kind>) -> Self {
    match action {
      Action::End => Self::End,
      Action::Continue => Self::Continue,
      Action::Skip => Self::Skip,
      Action::Unexpected(expected) => Self::Unexpected(expected),
    }
  }
}

struct SeqSepClassifier<Sep, Element> {
  sep: Sep,
  element: Element,
}

impl<Sep, Element> SeqSepClassifier<Sep, Element> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(sep: Sep, element: Element) -> Self {
    Self { sep, element }
  }
}

impl<'a, Sep, Element, T> Check<T, SeqSepAction<'a, T::Kind>> for SeqSepClassifier<Sep, Element>
where
  T: Token<'a>,
  Sep: Check<T>,
  Element: Check<T, Action<'a, T::Kind>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn check(&self, target: &T) -> SeqSepAction<'a, T::Kind> {
    if self.sep.check(target) {
      return SeqSepAction::Separator;
    }

    SeqSepAction::from(self.element.check(target))
  }
}

/// A parser that parses a sequence of elements separated by a specific separator.
pub struct SeqSep<F, SepClassifier, ElementClassifier, O, Config = SeqSepOptions> {
  f: F,
  classifier: SeqSepClassifier<SepClassifier, ElementClassifier>,
  config: Config,
  _m: PhantomData<(O, Config)>,
}

impl<F, SepClassifier, ElementClassifier, O> SeqSep<F, SepClassifier, ElementClassifier, O> {
  /// Creates a new `SeqSep` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    f: F,
    sep_classifier: SepClassifier,
    element_classifier: ElementClassifier,
  ) -> Self {
    Self::with_container(f, sep_classifier, element_classifier)
  }

  /// Creates a new `SeqSep` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn with_container(
    f: F,
    sep_classifier: SepClassifier,
    element_classifier: ElementClassifier,
  ) -> Self {
    Self {
      f,
      classifier: SeqSepClassifier::new(sep_classifier, element_classifier),
      config: SeqSepOptions::new(With::new((), ()), With::new((), ())),
      _m: PhantomData,
    }
  }
}

impl<F, SepClassifier, ElementClassifier, O, Options>
  SeqSep<F, SepClassifier, ElementClassifier, O, Options>
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
}

impl<F, SepClassifier, ElementClassifier, O, Trailing, Leading, Max, Min>
  SeqSep<F, SepClassifier, ElementClassifier, O, SeqSepOptions<Trailing, Leading, Max, Min>>
{
  /// Allows trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(
    self,
  ) -> SeqSep<F, SepClassifier, ElementClassifier, O, SeqSepOptions<Allow, Leading, Max, Min>>
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
  ) -> SeqSep<F, SepClassifier, ElementClassifier, O, SeqSepOptions<Require, Leading, Max, Min>>
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
  ) -> SeqSep<F, SepClassifier, ElementClassifier, O, SeqSepOptions<Trailing, Allow, Max, Min>>
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
  ) -> SeqSep<F, SepClassifier, ElementClassifier, O, SeqSepOptions<Trailing, Require, Max, Min>>
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
  ) -> SeqSep<F, SepClassifier, ElementClassifier, O, SeqSepOptions<Trailing, Leading, Max, Minimum>>
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
  ) -> SeqSep<F, SepClassifier, ElementClassifier, O, SeqSepOptions<Trailing, Leading, Maximum, Min>>
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

macro_rules! sep_by {
  ($(
    $(#[$meta:meta])*
    $sep:ident
  ),+$(,)?) => {
    paste::paste! {
      $(
        impl<F, ElementClassifier, O> SeqSep<F, $sep, ElementClassifier, O> {
          #[doc = "Creates a new sequence with [" $sep:snake "](crate::punct::" $sep ") separator parser."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< $sep:snake >]<'inp, L>(f: F, element_classifier: ElementClassifier) -> Self
          where
            L: Lexer<'inp>,
            $sep: Check<L::Token>,
            ElementClassifier: Check<L::Token, Action<'inp, <L::Token as Token<'inp>>::Kind>>,
          {
            Self::with_container(f, <$sep>::PHANTOM, element_classifier)
          }
        }

        impl<F, ElementClassifier, O, Lang> SeqSep<F, $sep<(), (), Lang>, ElementClassifier, O> {
          #[doc = "Creates a new sequence with [" $sep:snake "](crate::punct::" $sep ") separator parser of a specific language."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< $sep:snake _of >]<'inp, L>(f: F, element_classifier: ElementClassifier) -> Self
          where
            L: Lexer<'inp>,
            $sep<(), (), Lang>: Check<L::Token>,
            ElementClassifier: Check<L::Token, Action<'inp, <L::Token as Token<'inp>>::Kind>>,
          {
            Self::with_container(f, <$sep>::PHANTOM.change_language_const(), element_classifier)
          }
        }
      )*
    }
  };
}

sep_by!(
  Comma,
  Semicolon,
  Dot,
  Colon,
  Pipe,
  Ampersand,
  Hyphen,
  Underscore,
  DoubleColon,
  Arrow,
  FatArrow,
  Tilde,
  Trivia,
  Slash,
  BackSlash,
  Percent,
  Dollar,
  Hash,
  At,
);

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

#[cfg(test)]
mod tests {
  use crate::{DummyLexer, DummyToken};

  use super::*;

  fn assert_comma_parse_impl_with_all<'inp>() -> impl Parse<'inp, DummyLexer, Result<(), ()>, ()> {
    SeqSep::comma::<DummyLexer>(Any::new(), |_tok: &DummyToken| Action::Continue)
      .collect::<()>()
      .into_parser()
      .with_cache::<()>(())
      .with_emitter(Fatal::new())
  }

  // fn assert_expect_parse_impl_with_emitter<'inp>()
  // -> impl Parse<'inp, DummyLexer, Result<DummyToken, ()>, ()> {
  //   Expect::parser::<'inp, DummyLexer, ()>(|_tok: &DummyToken| Ok(())).with_emitter(Fatal::new())
  // }

  // fn assert_expect_parse_impl_with_cache<'inp>()
  // -> impl Parse<'inp, DummyLexer, Result<DummyToken, ()>, ()> {
  //   Expect::parser::<'inp, DummyLexer, ()>(|_tok: &DummyToken| Ok(())).with_cache::<()>(())
  // }

  // fn assert_expect_parse_impl<'inp>() -> impl Parse<'inp, DummyLexer, Result<DummyToken, ()>, ()> {
  //   Expect::parser::<'inp, DummyLexer, ()>(|_tok: &DummyToken| Ok(()))
  // }

  #[test]
  fn assert_parse_impl() {
    let _ = assert_comma_parse_impl_with_all();
    // let _ = assert_expect_parse_impl_with_all();
    // let _ = assert_expect_parse_impl_with_emitter();
    // let _ = assert_expect_parse_impl_with_cache();
  }
}
