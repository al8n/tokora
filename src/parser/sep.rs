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

/// A type-safe alias for configuring `SeparatedBy` parsers.
///
/// Canonical configuration layout: `With<With<Trailing, Leading>, With<Maximum, Minimum>>`.
pub type SeparatedByOptions<Trailing = (), Leading = (), Max = (), Min = ()> =
  With<With<Trailing, Leading>, With<Max, Min>>;

/// A parser that parses a sequence of elements separated by a specific separator.
pub struct SeparatedBy<F, SepClassifier, Condition, O, Window, Config = SeparatedByOptions> {
  pub(super) f: F,
  pub(super) sep: SepClassifier,
  pub(super) condition: Condition,
  pub(super) config: Config,
  pub(super) _m: PhantomData<O>,
  pub(super) _decision_window: PhantomData<Window>,
}

impl<F, SepClassifier, Condition, O, W: Window> SeparatedBy<F, SepClassifier, Condition, O, W> {
  /// Creates a new `SeparatedBy` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(f: F, sep_classifier: SepClassifier, condition: Condition) -> Self {
    Self::new_in(f, sep_classifier, condition)
  }

  /// Creates a new `SeparatedBy` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new_in(f: F, sep_classifier: SepClassifier, condition: Condition) -> Self {
    Self {
      f,
      sep: sep_classifier,
      condition,
      config: SeparatedByOptions::new(With::new((), ()), With::new((), ())),
      _m: PhantomData,
      _decision_window: PhantomData,
    }
  }
}

impl<F, SepClassifier, Condition, O, Options, Window>
  SeparatedBy<F, SepClassifier, Condition, O, Window, Options>
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(super) const fn as_mut(
    &mut self,
  ) -> SeparatedBy<&mut F, &mut SepClassifier, &mut Condition, O, Window, &mut Options> {
    SeparatedBy {
      f: &mut self.f,
      sep: &mut self.sep,
      condition: &mut self.condition,
      config: &mut self.config,
      _m: PhantomData,
      _decision_window: PhantomData,
    }
  }

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

  /// Creates a new `DelimitedSeparatedBy` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn delimited_by<Open, Close, Delim>(
    self,
    left: Open,
    right: Close,
    delim: Delim,
  ) -> DelimitedSeparatedBy<F, SepClassifier, Condition, Open, Close, Delim, O, Window, Options> {
    DelimitedSeparatedBy::new_in(self, left, right, delim)
  }
}

impl<F, SepClassifier, Condition, O, Trailing, Leading, Max, Min, Window>
  SeparatedBy<
    F,
    SepClassifier,
    Condition,
    O,
    Window,
    SeparatedByOptions<Trailing, Leading, Max, Min>,
  >
{
  /// Allows trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(
    self,
  ) -> SeparatedBy<
    F,
    SepClassifier,
    Condition,
    O,
    Window,
    SeparatedByOptions<Allow, Leading, Max, Min>,
  >
  where
    Trailing: Apply<Allow>,
  {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
      config: SeparatedByOptions::new(
        With::new(Allow(()), self.config.primary.secondary),
        self.config.secondary,
      ),
      _m: PhantomData,
      _decision_window: PhantomData,
    }
  }

  /// Requires a trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_trailing(
    self,
  ) -> SeparatedBy<
    F,
    SepClassifier,
    Condition,
    O,
    Window,
    SeparatedByOptions<Require, Leading, Max, Min>,
  >
  where
    Trailing: Apply<Require>,
  {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
      config: SeparatedByOptions::new(
        With::new(Require(()), self.config.primary.secondary),
        self.config.secondary,
      ),
      _m: PhantomData,
      _decision_window: PhantomData,
    }
  }

  /// Allows leading separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_leading(
    self,
  ) -> SeparatedBy<
    F,
    SepClassifier,
    Condition,
    O,
    Window,
    SeparatedByOptions<Trailing, Allow, Max, Min>,
  >
  where
    Leading: Apply<Allow>,
  {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
      config: SeparatedByOptions::new(
        With::new(self.config.primary.primary, Allow(())),
        self.config.secondary,
      ),
      _m: PhantomData,
      _decision_window: PhantomData,
    }
  }

  /// Requires a leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_leading(
    self,
  ) -> SeparatedBy<
    F,
    SepClassifier,
    Condition,
    O,
    Window,
    SeparatedByOptions<Trailing, Require, Max, Min>,
  >
  where
    Leading: Apply<Require>,
  {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
      config: SeparatedByOptions::new(
        With::new(self.config.primary.primary, Require(())),
        self.config.secondary,
      ),
      _m: PhantomData,
      _decision_window: PhantomData,
    }
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(
    self,
    n: Min::Options,
  ) -> SeparatedBy<
    F,
    SepClassifier,
    Condition,
    O,
    Window,
    SeparatedByOptions<Trailing, Leading, Max, Minimum>,
  >
  where
    Min: Apply<Minimum>,
  {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
      config: SeparatedByOptions::new(
        self.config.primary,
        With::new(
          self.config.secondary.primary,
          Min::apply(self.config.secondary.secondary, n),
        ),
      ),
      _m: PhantomData,
      _decision_window: PhantomData,
    }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(
    self,
    n: Max::Options,
  ) -> SeparatedBy<
    F,
    SepClassifier,
    Condition,
    O,
    Window,
    SeparatedByOptions<Trailing, Leading, Maximum, Min>,
  >
  where
    Max: Apply<Maximum>,
  {
    SeparatedBy {
      f: self.f,
      sep: self.sep,
      condition: self.condition,
      config: SeparatedByOptions::new(
        self.config.primary,
        With::new(
          Max::apply(self.config.secondary.primary, n),
          self.config.secondary.secondary,
        ),
      ),
      _m: PhantomData,
      _decision_window: PhantomData,
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
        impl<F, Condition, O> SeparatedBy<F, $sep, Condition, O, ()> {
          #[doc = "Creates a new sequence with [" $sep:snake "](crate::punct::" $sep ") separator parser."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< $sep:snake >]<'inp, L, W, Ctx>(f: F, condition: Condition) -> SeparatedBy<F, $sep, Condition, O, W>
          where
            L: Lexer<'inp>,
            Ctx: ParseContext<'inp, L, ()>,
            $sep: Check<L::Token>,
            Condition: Decision<'inp, L, Ctx::Emitter, W, ()>,
            W: Window,
          {
            SeparatedBy::new_in(f, <$sep>::PHANTOM, condition)
          }
        }

        impl<F, Condition, O, Lang: ?Sized> SeparatedBy<F, $sep<(), (), Lang>, Condition, O, ()> {
          #[doc = "Creates a new sequence with [" $sep:snake "](crate::punct::" $sep ") separator parser of a specific language."]
          #[cfg_attr(not(tarpaulin), inline(always))]
          pub const fn [< $sep:snake _of >]<'inp, L, W, Ctx>(f: F, condition: Condition) -> SeparatedBy<F, $sep, Condition, O, W>
          where
            L: Lexer<'inp>,
            $sep<(), (), Lang>: Check<L::Token>,
            Ctx: ParseContext<'inp, L, Lang>,
            Condition: Decision<'inp, L, Ctx::Emitter, W, Lang>,
            W: Window,
          {
            SeparatedBy::new_in(f, <$sep>::PHANTOM.change_language_const(), condition)
          }
        }

        #[cfg(test)]
        const _: () = {
          use crate::lexer::DummyLexer;
          use generic_arraydeque::typenum::U1;

          fn __assert_parse_impl__<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
            Parser::with_parser(
              SeparatedBy::[< $sep:snake >]::<DummyLexer, U1, ()>(
                Any::new(),
                |_toks: Peeked<'_, '_, DummyLexer, U1>, _: &mut Fatal<()>| Ok(Action::Continue),
              )
              .collect::<()>(),
            )
          }

          fn __assert_parse_with_ctx_impl__<'inp>() -> impl Parse<'inp, DummyLexer, (), ()> {
            Parser::with_parser_and_context(SeparatedBy::[< $sep:snake >]::<DummyLexer, U1, ()>(
                Any::new(),
                |_toks: Peeked<'_, '_, DummyLexer, U1>, _: &mut Fatal<()>| Ok(Action::Continue),
              )
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
  /// the span is the start of the
  Leadings(S),
  Separator(Spanned<T, S>),
  RepeatedSeparator(S),
}
