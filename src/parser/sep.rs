use core::{marker::PhantomData, mem::MaybeUninit};

use derive_more::IsVariant;

use crate::{Check, emitter::{SeparatedByEmitter, TrailingSeparatorEmitter}, utils::Span};

use super::*;
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

impl<F, Sep, O, Container, Trailing, Leading, Max, Min> SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, Leading, Max, Min>> {
  /// Allows trailing separators.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(self) -> SeqSep<F, Sep, O, Container, SeqSepOptions<AllowTrailing, Leading, Max, Min>>
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
  pub fn require_trailing(self) -> SeqSep<F, Sep, O, Container, SeqSepOptions<RequireTrailing, Leading, Max, Min>>
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
  pub fn allow_leading(self) -> SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, AllowLeading, Max, Min>>
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
  pub fn require_leading(self) -> SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, RequireLeading, Max, Min>>
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
  pub fn at_least(self, n: Min::Options) -> SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, Leading, Max, Minimum>>
  where
    Min: Next<Minimum>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      config: SeqSepOptions::new(
        self.config.primary,
        With::new(self.config.secondary.primary, Min::next(self.config.secondary.secondary, n)),
      ),
      _m: PhantomData,
    }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, n: Max::Options) -> SeqSep<F, Sep, O, Container, SeqSepOptions<Trailing, Leading, Maximum, Min>>
  where
    Max: Next<Maximum>,
  {
    SeqSep {
      f: self.f,
      sep: self.sep,
      config: SeqSepOptions::new(
        self.config.primary,
        With::new(Max::next(self.config.secondary.primary, n), self.config.secondary.secondary),
      ),
      _m: PhantomData,
    }
  }
}


impl<'inp, L, F, Sep, O, Output, Container, E, C, Config> sealed::Sealed<'inp, L, Output, E, C>
  for SeqSep<F, Sep, O, Container, Config>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, E, C>,
  Sep: Check<L::Token, SeqSepHint>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
enum State<S> {
  Start,
  Element,
  Separator,
  RepeatedSeparator(S),
}

// No trailing, no leading, unbounded
impl<'inp, L, F, Sep, O, Container, E, C> ParseInput<'inp, L, ParseResult<'inp, Container, L, E>, E, C> for SeqSep<F, Sep, O, Container, Init>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, E, C>,
  Sep: Check<L::Token, SeqSepHint>,
  E: SeparatedByEmitter<'inp, L>,
  C: Cache<'inp, L>,
  Container: Default + super::Container<O>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, inp: &mut InputRef<'inp, '_, L, E, C>) -> ParseResult<'inp, Container, L, E>
  where
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {
    let mut container = Container::default();
    let mut state = State::Start;
    let ckp = inp.save();

    loop {
      // peek two tokens ahead
      let peeked = inp.peek_one();

      match peeked {
        None => {
          let trailings = match state {
            State::Separator => inp.span().clone(),
            State::RepeatedSeparator(span) => span,
            _ => return Ok(Spanned::new(inp.span_since(ckp.cursor()), container)),
          };

          // if the emitter treat trailing separator error as a non-fatal error, emit it
          // otherwise, return an error
          let span = inp.span_since(ckp.cursor());
          inp.emitter().emit_trailing_separator(span, trailings.clone())?;

          return Ok(Spanned::new(inp.span_since(ckp.cursor()), container));
        },
        Some(tok) => {
          let tok = tok.as_ref();
          match tok.token().data() {
            Lexed::Error(_) => {
              // if the next token is an error token, emit the error.
              let nxt = inp.next().expect("peeked token already confirmed there must be a token");
              inp.emit_token_error(nxt.map_data(|s| s.unwrap_error()))?;
              continue;
            },
            Lexed::Token(tok) => {
              match self.sep.check(tok) {
                SeqSepHint::End => {
                  let trailings = match state {
                    State::Separator => inp.span().clone(),
                    State::RepeatedSeparator(span) => span,
                    _ => return Ok(Spanned::new(inp.span_since(ckp.cursor()), container)),
                  };

                  let span = inp.span_since(ckp.cursor());
                  // TODO(al8n): improve the trailing error, add info about the separator
                  inp.emitter().emit_trailing_separator(span.clone(), trailings)?;
                  return Ok(Spanned::new(span, container));
                }
                SeqSepHint::Separator => {
                  match &state {
                    State::Start => todo!(),
                    State::Element => todo!(),
                    State::Separator => todo!(),
                    State::RepeatedSeparator(_) => todo!(),
                  }
                }
              }


            },
          }
        }
      }
    }

    Ok(Spanned::new(inp.span_since(ckp.cursor()), container))
  }
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
