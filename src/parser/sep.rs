use core::marker::PhantomData;

use derive_more::IsVariant;

use super::*;

mod init;

/// A marker type representing the maximum number of elements allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Maximum(pub usize);

impl Maximum {
  /// The maximum possible value for `Maximum`.
  pub const MAX: Self = Self(usize::MAX);

  /// Returns the maximum number of elements allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

/// A marker type representing the minimum number of elements required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Minimum(pub usize);

impl Minimum {
  /// The minimum possible value for `Minimum`.
  pub const MIN: Self = Self(0);

  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

/// Leading-separator markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DenyLeading;

/// Leading-separator markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllowLeading;

/// Requires a leading separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequireLeading;

/// Denies trailing separators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DenyTrailing;

/// Trailing-separator markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllowTrailing;
/// Requires a trailing separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequireTrailing;

/// The Initial configuration layout for `SeqSep`.
pub type Init = SeqSepOptions<(), (), (), ()>;

/// Canonical configuration layout: `With<With<Trailing, Leading>, With<Maximum, Minimum>>`.
pub type SeqSepOptions<Trailing, Leading, Max, Min> = With<With<Trailing, Leading>, With<Max, Min>>;

/// A hint used during parsing sequences with separators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
pub enum SeqSepHint {
  /// Indicates the start of the sequence, hint to stop.
  End,
  /// Indicates a separator was found, hint to parse another element.
  Separator,
  /// Indicates an element was found, hint to continue parsing.
  Continue,
}

/// A parser that parses a sequence of elements separated by a specific separator.
pub struct SeqSep<F, Sep, O, Container, Config = Init> {
  f: F,
  sep: Sep,
  config: Config,
  _m: PhantomData<(O, Config, Container)>,
}

impl<F, Sep, O, Container> SeqSep<F, Sep, O, Container> {
  /// Creates a new `SeqSep` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(f: F, sep: Sep) -> Self {
    Self::with_container(f, sep)
  }

  /// Creates a new `SeqSep` parser with the given container.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn with_container(f: F, sep: Sep) -> Self {
    Self {
      f,
      sep,
      config: SeqSepOptions::new(With::new((), ()), With::new((), ())),
      _m: PhantomData,
    }
  }
}

impl<F, Sep, O, Container, Trailing, Leading, Max, Min>
  SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, Leading, Max, Min>>
{
  /// Allows trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(
    self,
  ) -> SeqSep<F, Sep, O, Container, SeqSepOptions<AllowTrailing, Leading, Max, Min>>
  where
    Trailing: Next<AllowTrailing>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      config: SeqSepOptions::new(
        With::new(AllowTrailing, self.config.primary.secondary),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }

  /// Requires a trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_trailing(
    self,
  ) -> SeqSep<F, Sep, O, Container, SeqSepOptions<RequireTrailing, Leading, Max, Min>>
  where
    Trailing: Next<RequireTrailing>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      config: SeqSepOptions::new(
        With::new(RequireTrailing, self.config.primary.secondary),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }

  /// Allows leading separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_leading(
    self,
  ) -> SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, AllowLeading, Max, Min>>
  where
    Leading: Next<AllowLeading>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      config: SeqSepOptions::new(
        With::new(self.config.primary.primary, AllowLeading),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }

  /// Requires a leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_leading(
    self,
  ) -> SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, RequireLeading, Max, Min>>
  where
    Leading: Next<RequireLeading>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      config: SeqSepOptions::new(
        With::new(self.config.primary.primary, RequireLeading),
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
  ) -> SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, Leading, Max, Minimum>>
  where
    Min: Next<Minimum>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      config: SeqSepOptions::new(
        self.config.primary,
        With::new(
          self.config.secondary.primary,
          Min::next(self.config.secondary.secondary, n),
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
  ) -> SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, Leading, Maximum, Min>>
  where
    Max: Next<Maximum>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      config: SeqSepOptions::new(
        self.config.primary,
        With::new(
          Max::next(self.config.secondary.primary, n),
          self.config.secondary.secondary,
        ),
      ),
      _m: PhantomData,
    }
  }
}

impl<L, F, Sep, O, Output, Container, E, C, Config> sealed::Sealed<'_, L, Output, E, C>
  for SeqSep<F, Sep, O, Container, Config>
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
enum State<S> {
  Start,
  Element,
  Separator(S),
  RepeatedSeparator(S),
}

impl Next<AllowLeading> for () {
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(self, _options: Self::Options) -> AllowLeading {
    AllowLeading
  }
}

impl Next<RequireLeading> for () {
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(self, _options: Self::Options) -> RequireLeading {
    RequireLeading
  }
}

impl Next<AllowTrailing> for () {
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(self, _options: Self::Options) -> AllowTrailing {
    AllowTrailing
  }
}

impl Next<RequireTrailing> for () {
  type Options = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(self, _options: Self::Options) -> RequireTrailing {
    RequireTrailing
  }
}

impl Next<Maximum> for () {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(self, options: Self::Options) -> Maximum {
    Maximum(options)
  }
}

impl Next<Minimum> for () {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(self, options: Self::Options) -> Minimum {
    Minimum(options)
  }
}

impl Next<Maximum> for Maximum {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(self, options: Self::Options) -> Maximum {
    Maximum(options)
  }
}

impl Next<Minimum> for Minimum {
  type Options = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn next(self, options: Self::Options) -> Minimum {
    Minimum(options)
  }
}

enum Spec {
  Deny,
  Allow,
  Require,
}

trait Leading {
  const SPEC: Spec;
}

impl Leading for DenyLeading {
  const SPEC: Spec = Spec::Deny;
}

impl Leading for AllowLeading {
  const SPEC: Spec = Spec::Allow;
}

impl Leading for RequireLeading {
  const SPEC: Spec = Spec::Require;
}

trait Trailing {
  const SPEC: Spec;
}

impl Trailing for DenyTrailing {
  const SPEC: Spec = Spec::Deny;
}

impl Trailing for AllowTrailing {
  const SPEC: Spec = Spec::Allow;
}

trait Min {
  fn minimum(&self) -> usize;
}

impl Min for Minimum {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    self.0
  }
}

impl Min for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn minimum(&self) -> usize {
    0
  }
}

trait Max {
  fn maximum(&self) -> usize;
}

impl Max for Maximum {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    self.0
  }
}

impl Max for () {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn maximum(&self) -> usize {
    usize::MAX
  }
}
