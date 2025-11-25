use core::marker::PhantomData;

use super::{ParseInput, With, sealed};
use crate::{Cache, Check, Emitter, InputRef, Lexer};

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
pub struct AllowLeading;
/// Requires a leading separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequireLeading;

/// Trailing-separator markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllowTrailing;
/// Requires a trailing separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequireTrailing;

/// The Initial configuration layout for `SeqSep`.
pub type Init = ConfigOf<(), (), (), ()>;

/// Canonical configuration layout: `With<With<Trailing, Leading>, With<Maximum, Minimum>>`.
pub type ConfigOf<Trailing, Leading, Max, Min> = With<With<Trailing, Leading>, With<Max, Min>>;

/// A parser that parses a sequence of elements separated by a specific separator.
pub struct SeqSep<F, Sep, End, O, Config = Init> {
  f: F,
  sep: Sep,
  end: End,
  config: Config,
  _m: PhantomData<(O, Config)>,
}

impl<F, Sep, End, O> SeqSep<F, Sep, End, O> {
  /// Creates a new `SeqSep` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(f: F, is_sep: Sep, is_end: End) -> Self {
    Self::new_in(f, is_sep)
  }

  const fn new_in(f: F, sep: Sep, is_end: End) -> Self {
    Self {
      f,
      sep,
      end: is_end,
      config: ConfigOf::new(With::new((), ()), With::new((), ())),
      _m: PhantomData,
    }
  }
}

impl<F, V, O, Ld, Max, Min> SeqSep<F, V, O, ConfigOf<(), Ld, Max, Min>> {
  /// Allows trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(self) -> SeqSep<F, V, O, ConfigOf<AllowTrailing, Ld, Max, Min>> {
    SeqSep {
      f: self.f,
      sep: self.sep,
      end: self.end,
      config: ConfigOf::new(
        With::new(AllowTrailing, self.config.primary.secondary),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }

  /// Requires a trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_trailing(self) -> SeqSep<F, V, O, ConfigOf<RequireTrailing, Ld, Max, Min>> {
    SeqSep {
      f: self.f,
      sep: self.sep,
      end: self.end,
      config: ConfigOf::new(
        With::new(RequireTrailing, self.config.primary.secondary),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }
}

impl<F, Sep, O, Tl, Max, Min> SeqSep<F, Sep, O, ConfigOf<Tl, (), Max, Min>> {
  /// Allows leading separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_leading(self) -> SeqSep<F, Sep, O, ConfigOf<Tl, AllowLeading, Max, Min>> {
    SeqSep {
      f: self.f,
      sep: self.sep,
      end: self.end,
      config: ConfigOf::new(
        With::new(self.config.primary.primary, AllowLeading),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }

  /// Requires a leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_leading(self) -> SeqSep<F, Sep, O, ConfigOf<Tl, RequireLeading, Max, Min>> {
    SeqSep {
      f: self.f,
      sep: self.sep,
      end: self.end,
      config: ConfigOf::new(
        With::new(self.config.primary.primary, RequireLeading),
        self.config.secondary,
      ),
      _m: PhantomData,
    }
  }
}

impl<F, V, O, Tl, Ld, Max> SeqSep<F, V, O, ConfigOf<Tl, Ld, Max, ()>> {
  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, n: usize) -> SeqSep<F, V, O, ConfigOf<Tl, Ld, Max, Minimum>> {
    SeqSep {
      f: self.f,
      sep: self.sep,
      end: self.end,
      config: ConfigOf::new(
        self.config.primary,
        With::new(self.config.secondary.primary, Minimum(n)),
      ),
      _m: PhantomData,
    }
  }
}

impl<F, V, O, Tl, Ld, Min> SeqSep<F, V, O, ConfigOf<Tl, Ld, (), Min>> {
  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, n: usize) -> SeqSep<F, V, O, ConfigOf<Tl, Ld, Maximum, Min>> {
    SeqSep {
      f: self.f,
      sep: self.sep,
      end: self.end,
      config: ConfigOf::new(
        self.config.primary,
        With::new(Maximum(n), self.config.secondary.secondary),
      ),
      _m: PhantomData,
    }
  }
}

impl<'inp, L, F, Sep, End, O, E, C, Config> sealed::Sealed<'inp, L, O, E, C>
  for SeqSep<F, Sep, End, O, Config>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, E, C>,
  Sep: Check<L::Token>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
}

impl<'inp, L, F, Sep, End, O, E, C> ParseInput<'inp, L, O, E, C> for SeqSep<F, Sep, End, O, Init>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, E, C>,
  Sep: Check<L::Token>,
  End: Check<L::Token>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> O
  where
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {
  }
}
