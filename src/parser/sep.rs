use core::marker::PhantomData;

use derive_more::{IsVariant, TryUnwrap, Unwrap};

use crate::{Check, punct::*};

use super::*;

mod parser_input;

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

/// A parser that parses a sequence of elements separated by a specific separator.
pub struct SeqSep<F, SepClassifier, Condition, O, const PEEK: usize, Config = SeqSepOptions> {
  f: F,
  sep: SepClassifier,
  condition: Condition,
  config: Config,
  _m: PhantomData<O>,
}

impl<F, SepClassifier, Condition, O, const PEEK: usize>
  SeqSep<F, SepClassifier, Condition, O, PEEK>
{
  /// Creates a new `SeqSep` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(f: F, sep_classifier: SepClassifier, condition: Condition) -> Self {
    Self::new_in(f, sep_classifier, condition)
  }

  /// Creates a new `SeqSep` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new_in(f: F, sep_classifier: SepClassifier, condition: Condition) -> Self {
    assert!(
      PEEK > 0,
      "the maximum size of peek buf must be greater than zero"
    );

    Self {
      f,
      sep: sep_classifier,
      condition,
      config: SeqSepOptions::new(With::new((), ()), With::new((), ())),
      _m: PhantomData,
    }
  }
}

impl<F, SepClassifier, Condition, O, Options, const PEEK: usize>
  SeqSep<F, SepClassifier, Condition, O, PEEK, Options>
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

impl<F, SepClassifier, Condition, O, Trailing, Leading, Max, Min, const PEEK: usize>
  SeqSep<F, SepClassifier, Condition, O, PEEK, SeqSepOptions<Trailing, Leading, Max, Min>>
{
  /// Allows trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(
    self,
  ) -> SeqSep<F, SepClassifier, Condition, O, PEEK, SeqSepOptions<Allow, Leading, Max, Min>>
  where
    Trailing: Apply<Allow>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
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
  ) -> SeqSep<F, SepClassifier, Condition, O, PEEK, SeqSepOptions<Require, Leading, Max, Min>>
  where
    Trailing: Apply<Require>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
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
  ) -> SeqSep<F, SepClassifier, Condition, O, PEEK, SeqSepOptions<Trailing, Allow, Max, Min>>
  where
    Leading: Apply<Allow>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
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
  ) -> SeqSep<F, SepClassifier, Condition, O, PEEK, SeqSepOptions<Trailing, Require, Max, Min>>
  where
    Leading: Apply<Require>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
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
  ) -> SeqSep<F, SepClassifier, Condition, O, PEEK, SeqSepOptions<Trailing, Leading, Max, Minimum>>
  where
    Min: Apply<Minimum>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
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
  ) -> SeqSep<F, SepClassifier, Condition, O, PEEK, SeqSepOptions<Trailing, Leading, Maximum, Min>>
  where
    Max: Apply<Maximum>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
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
        impl<F, Condition, O, const PEEK: usize> SeqSep<F, $sep, Condition, O, PEEK> {
          #[doc = "Creates a new sequence with [" $sep:snake "](crate::punct::" $sep ") separator parser."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< $sep:snake >]<'inp, L, Ctx>(f: F, condition: Condition) -> Self
          where
            L: Lexer<'inp>,
            Ctx: ParseContext<'inp, L, ()>,
            $sep: Check<L::Token>,
            Condition: FnMut(
              &[MaybeRef<'_, CachedToken<'inp, L>>],
              &mut Ctx::Emitter,
            ) -> Result<Action, <Ctx::Emitter as Emitter<'inp, L, ()>>::Error>,
          {
            Self::new_in(f, <$sep>::PHANTOM, condition)
          }
        }

        impl<F, Condition, O, const PEEK: usize, Lang> SeqSep<F, $sep<(), (), Lang>, Condition, O, PEEK> {
          #[doc = "Creates a new sequence with [" $sep:snake "](crate::punct::" $sep ") separator parser of a specific language."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< $sep:snake _of >]<'inp, L, Ctx>(f: F, condition: Condition) -> Self
          where
            L: Lexer<'inp>,
            $sep<(), (), Lang>: Check<L::Token>,
            Ctx: ParseContext<'inp, L, Lang>,
            Condition: FnMut(
              &[MaybeRef<'_, CachedToken<'inp, L>>],
              &mut Ctx::Emitter,
            ) -> Result<Action, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>,
          {
            Self::new_in(f, <$sep>::PHANTOM.change_language_const(), condition)
          }
        }

        #[cfg(test)]
        const _: () = {
          use crate::DummyLexer;

          fn __assert_parse_impl__<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
            Parser::with_parser(SeqSep::<_, _, _, _, 1>::comma::<DummyLexer, ()>(Any::new(), |_toks: &PeekBuf<'inp, '_, DummyLexer>, _| Ok(Action::Continue))
            .collect::<()>())
          }

          fn __assert_parse_with_ctx_impl__<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
            Parser::with_parser_and_context(SeqSep::<_, _, _, _, 1>::comma::<DummyLexer, ()>(Any::new(), |_toks: &PeekBuf<'inp, '_, DummyLexer>, _| Ok(Action::Continue))
            .collect::<()>(), ())
          }
        };
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
