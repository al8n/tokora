use derive_more::{From, Into, Display};

use crate::PunctuatorToken;

use super::*;

/// A marker type representing the maximum number of elements allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Display)]
#[display("{_0}")]
pub struct Maximum(usize);

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Display)]
#[display("{_0}")]
pub struct Minimum(usize);

impl Minimum {
  /// The minimum possible value for `Minimum`.
  pub const MIN: Self = Self(0);

  /// Returns the minimum number of elements required.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn get(&self) -> usize {
    self.0
  }
}

/// A marker type representing a trailing separator is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Display)]
#[display("")]
pub struct AllowTrailing(());

/// A marker type representing a leading separator is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Display)]
#[display("")]
pub struct AllowLeading(());

/// A marker type representing a leading separator must be present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Display)]
#[display("")]
pub struct RequireLeading(());

/// A marker type representing a trailing separator must be present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Display)]
#[display("")]
pub struct RequireTrailing(());

/// A parser that parses a sequence of elements separated by a specific separator.
pub struct SeqSep<F, V, O, Config = ()> {
  f: F,
  valid: V,
  config: Config,
  _m: PhantomData<O>,
}

impl<F, V, O> SeqSep<F, V, O> {
  /// Creates a new `SeqSep` parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(f: F, is_sep: V) -> Self {
    Self::new_in(f, is_sep)
  }

  /// Sets whether leading separators are allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(self) -> SeqSep<F, V, O, AllowTrailing> {
    SeqSep { f: self.f, valid: self.valid, config: AllowTrailing(()), _m: PhantomData }
  }

  /// Sets whether trailing separators are allowed.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_leading(self) -> SeqSep<F, V, O, AllowLeading> {
    SeqSep { f: self.f, valid: self.valid, config: AllowLeading(()), _m: PhantomData }
  }

  /// Sets requirement for leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_leading(self) -> SeqSep<F, V, O, RequireLeading> {
    SeqSep { f: self.f, valid: self.valid, config: RequireLeading(()), _m: PhantomData }
  }

  /// Sets requirement for trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn require_trailing(self) -> SeqSep<F, V, O, RequireTrailing> {
    SeqSep { f: self.f, valid: self.valid, config: RequireTrailing(()), _m: PhantomData }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, n: usize) -> SeqSep<F, V, O, Minimum> {
    SeqSep { config: Minimum(n), f: self.f, valid: self.valid, _m: PhantomData }
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, n: usize) -> SeqSep<F, V, O, Maximum> {
    SeqSep { config: Maximum(n), f: self.f, valid: self.valid, _m: PhantomData }
  }

  const fn new_in(
    f: F,
    valid: V,
  ) -> Self {
    Self {
      f,
      valid,
      config: (),
      _m: PhantomData,
    }
  }
}

impl<F, V, O> SeqSep<F, V, O, AllowLeading> {
  /// Allows trailing separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_trailing(self) -> SeqSep<F, V, O, With<AllowLeading, AllowTrailing>> {
    SeqSep { f: self.f, valid: self.valid, config: With::new(self.config, AllowTrailing(())), _m: PhantomData }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, n: usize) -> SeqSep<F, V, O, With<AllowLeading, Minimum>> {
    SeqSep { config: With::new(self.config, Minimum(n)), f: self.f, valid: self.valid, _m: PhantomData }
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, n: usize) -> SeqSep<F, V, O, With<AllowLeading, Maximum>> {
    SeqSep { config: With::new(self.config, Maximum(n)), f: self.f, valid: self.valid, _m: PhantomData }
  }
}

impl<F, V, O> SeqSep<F, V, O, AllowTrailing> {
  /// Allows leading separator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn allow_leading(self) -> SeqSep<F, V, O, With<AllowLeading, AllowTrailing>> {
    SeqSep { f: self.f, valid: self.valid, config: With::new(AllowLeading(()), self.config), _m: PhantomData }
  }

  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, n: usize) -> SeqSep<F, V, O, With<AllowTrailing, Minimum>> {
    SeqSep { config: With::new(self.config, Minimum(n)), f: self.f, valid: self.valid, _m: PhantomData }
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, n: usize) -> SeqSep<F, V, O, With<AllowTrailing, Maximum>> {
    SeqSep { config: With::new(self.config, Maximum(n)), f: self.f, valid: self.valid, _m: PhantomData }
  }
}


impl<F, V, O> SeqSep<F, V, O, With<AllowLeading, AllowTrailing>> {
  /// Sets the maximum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_least(self, n: usize) -> SeqSep<F, V, O, With<With<AllowLeading, AllowTrailing>, Minimum>> {
    SeqSep { config: With::new(self.config, Minimum(n)), f: self.f, valid: self.valid, _m: PhantomData }
  }

  /// Sets the minimum number of elements to parse.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn at_most(self, n: usize) -> SeqSep<F, V, O, With<With<AllowLeading, AllowTrailing>, Maximum>> {
    SeqSep { config: With::new(self.config, Maximum(n)), f: self.f, valid: self.valid, _m: PhantomData }
  }
}

impl<F, V, O> SeqSep<F, V, O, With<AllowTrailing, Minimum>> {
  
}


impl<'inp, L, F, V, O, E, C, Container, Config> sealed::Sealed<'inp, L, Container, E, C> for SeqSep<F, V, O, Config>
where
  L: Lexer<'inp>,
  F: ParseInput<'inp, L, O, E, C>,
  V: Fn(&L::Token) -> bool,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
}

// enum StateMachine {
//   Sep,
//   SepOrElem,
//   Elem,
  
// }

// impl<'inp, L, F, V, O, E, C, Container, Config> ParseInput<'inp, L, Result<Container, E::Error>, E, C> for SeqSep<F, V, O, Config>
// where
//   L: Lexer<'inp>,
//   F: ParseInput<'inp, L, O, E, C>,
//   V: Fn(&L::Token) -> bool,
//   E: Emitter<'inp, L>,
//   C: Cache<'inp, L>,
//   Container: Default,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn parse_input(
//     &mut self,
//     inp: &mut InputRef<'inp, '_, L, E, C>,
//   ) -> Result<Container, E::Error> {
//     let mut cur = inp.save();
//     let mut num = 0;
//     let mut container = Container::default();
//     let mut state = if self.allow_leading {
//       StateMachine::Elem
//     } else {
//       StateMachine::SepOrElem
//     };

//     loop {
//       match inp.peek_one() {
//         None if num == 0 && self.min == 0 => return Ok(container),
//         None if num > self.max => {
//           inp.emitter().emit_too_many();
//         },
//         None => {},
//         Some(_) => todo!(),
//       }
//     }
//   }
// }

// /// A parser that accepts any input, returning a parser for a sequence of elements separated by a specific separator.
// #[cfg_attr(not(tarpaulin), inline(always))]
// pub const fn seq<'inp, F, V, L, O, E>(f: F, is_sep: V) -> Parser<SeqSep<F, V, O>, L, Option<Spanned<Lexed<'inp, L::Token>, L::Span>>, E::Error>
// where
//   L: Lexer<'inp>,
//   E: Emitter<'inp, L>,
//   F: Parse<'inp, L, O, E>,
//   V: Fn(&L::Token) -> bool,
// {
//   Parser::new(SeqSep::new(f, is_sep))
// }

// /// A parser that accepts any input, returning a parser for a sequence of elements separated by commas.
// #[cfg_attr(not(tarpaulin), inline(always))]
// pub const fn comma_seq<'inp, F, L, O, E>(f: F) -> Parser<SeqSep<F, impl Fn(&L::Token) -> bool, O>, L, Option<Spanned<Lexed<'inp, L::Token>, L::Span>>, E::Error>
// where
//   L: Lexer<'inp>,
//   L::Token: PunctuatorToken<'inp>,
//   E: Emitter<'inp, L>,
//   F: Parse<'inp, L, O, E>,
// {
//   seq(f, PunctuatorToken::is_comma)
// }

// #[test]
// fn t() {
//   seq(any(), |_| true);
// }
