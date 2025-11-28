#![allow(clippy::type_complexity)]

use core::marker::PhantomData;

use crate::{
  Cache, Emitter, Lexed, Lexer, Token,
  lexer::{Input, InputRef},
  utils::{Spanned, marker::Noop},
};

pub use any::*;
pub use sep::{SepFixSpec, SeqSep, SeqSepAction, SeqSepOptions, comma_seq};

mod any;
mod sep;

/// Shorthand for the result type of a parser returning a result.
pub type ParseResult<'inp, O, L, E> = Result<
  Spanned<O, <L as Lexer<'inp>>::Span>,
  Spanned<<E as Emitter<'inp, L>>::Error, <L as Lexer<'inp>>::Span>,
>;

// /// a
// pub struct ParseOptions<E, C = (), S = ()> {
//   state: S,
//   emitter: E,
//   cache_opts: C,
// }

mod sealed {
  use super::*;

  pub trait Sealed<'inp, L, O, E, C> {}

  impl<'inp, F, L, O, E, C> Sealed<'inp, L, O, E, C> for F
  where
    F: FnMut(&mut InputRef<'inp, '_, L, E, C>) -> O,
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {
  }

  impl<'inp, F, L, O, E, C> Sealed<'inp, L, O, E, C> for Parser<F, L, O, E::Error, ParserOptions<E::Error, C::Options, E, C>>
  where
    F: ParseInput<'inp, L, O, E, C>,
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {
  }
}

/// Core trait implemented by every parser combinator.
///
/// This mirrors the ergonomics of libraries like `winnow`: a parser is
/// simply something that can mutate an [`InputRef`] and either produce
/// a value or a spanned error using the configured `Emitter`.
pub trait ParseInput<'inp, L, O, E, C>: sealed::Sealed<'inp, L, O, E, C> {
  /// Try to parse from the given input.
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> O
  where
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>;
}

impl<'inp, F, L, O, E, C> ParseInput<'inp, L, O, E, C> for F
where
  F: FnMut(&mut InputRef<'inp, '_, L, E, C>) -> O,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> O {
    (self)(input)
  }
}

/// m
pub type ParserOptions<Error, Options = (), E = Noop<Error>, C = ()> = With<With<E, PhantomData<Error>>, With<Options, PhantomData<C>>>; 

/// Lightweight wrapper around a parsing function.
pub struct Parser<F, L, O, Error, Options = ParserOptions<Error>> {
  f: F,
  opts: Options,
  _marker: PhantomData<(L, O, Error)>,
}

impl<F, L, O, Error> core::ops::Deref for Parser<F, L, O, Error> {
  type Target = F;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    &self.f
  }
}

impl<F, L, O, Error> core::ops::DerefMut for Parser<F, L, O, Error> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.f
  }
}

impl<L> Default for Parser<(), L, (), ()> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, O, Error> Parser<(), L, O, Error> {
  /// A parser without any behavior.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      f: (),
      opts: With::new(With::new(Noop::new(), PhantomData), With::new((), PhantomData)),
      _marker: PhantomData,
    }
  }
}

impl<F, L, O, Error> Parser<F, L, O, Error> {
  /// A parser without any behavior.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with(f: F) -> Self {
    Self {
      f,
      opts: With::new(With::new(Noop::new(), PhantomData), With::new((), PhantomData)),
      _marker: PhantomData,
    }
  }
}

impl<L, O, Error> Parser<(), L, O, Error> {
  /// Apply a new emitter to the parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_emitter<'inp, E>(self, emitter: E) -> Parser<(), L, O, Error, ParserOptions<Error, (), E>>
  where
    E: Emitter<'inp, L, Error = Error>,
    L: Lexer<'inp>,
  {
    Parser {
      f: self.f,
      opts: With::new(With::new(emitter, PhantomData), self.opts.secondary),
      _marker: PhantomData,
    }
  }

  /// Apply new cache options to the parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_cache<'inp, C>(self, options: C::Options) -> Parser<(), L, O, Error, ParserOptions<Error, C::Options, Noop<Error>, C>>
  where
    C: Cache<'inp, L>,
    L: Lexer<'inp>,
  {
    Parser {
      f: self.f,
      opts: With::new(self.opts.primary, With::new(options, PhantomData)),
      _marker: PhantomData,
    }
  }
}

impl<'inp, L, O, E, C> Parser<(), L, O, E::Error, ParserOptions<E::Error, C::Options, E, C>>
where
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  /// Apply a new parsing function to the parser.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn apply<F>(self, f: F) -> Parser<F, L, O, E::Error, ParserOptions<E::Error, C::Options, E, C>> {
    Parser {
      f,
      opts: self.opts,
      _marker: PhantomData,
    }
  }
}

impl<F, L, O, Error> Parser<F, L, O, Error> {
  /// Convert to a configurable parser with all defaults.
  ///
  /// This allows you to configure emitter and cache options in any order
  /// before calling `.parse()`.
  ///
  /// # Example
  /// ```ignore
  /// parser.configured()
  ///   .with_emitter(my_emitter)
  ///   .with_cache(my_cache_opts)
  ///   .parse(src)
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn configured<'inp>(self) -> Configured<'inp, Self, L>
  where
    L: Lexer<'inp>,
  {
    Configured {
      parser: self,
      config: With::new((), ()),
      _marker: PhantomData,
    }
  }
}

// impl<'inp, F, L, O, E, C> ParseInput<'inp, L, O, E, C> for Parser<F, L, O, E::Error>
// where
//   F: ParseInput<'inp, L, O, E, C>,
//   L: Lexer<'inp>,
//   E: Emitter<'inp, L>,
//   C: Cache<'inp, L>,
// {
//   #[cfg_attr(not(tarpaulin), inline(always))]
//   fn parse_input(&mut self, input: &mut InputRef<'inp, '_, L, E, C>) -> O {
//     self.f.parse_input(input)
//   }
// }

/// Unified configuration for parser execution.
///
/// This type allows you to configure emitter and cache options in any order
/// before executing the parser. It eliminates the combinatorial explosion of
/// wrapper types like `WithEmitter<WithCache<P>>` vs `WithCache<WithEmitter<P>>`.
///
/// Uses `With<E, C>` to hold configuration, where `()` represents "use default".
/// The final normalized form is always `With<E, C>` where E and C are concrete types or `()`.
pub struct Configured<'inp, P, L: Lexer<'inp>, Config = With<(), ()>> {
  parser: P,
  config: Config,
  _marker: PhantomData<(&'inp L::Source, L)>,
}

impl<'inp, P, L, E, C> Configured<'inp, P, L, With<E, C>>
where
  L: Lexer<'inp>,
{
  /// Set a custom emitter (replaces any previous emitter config).
  ///
  /// Transforms from `With<_, C>` to `With<WithEmitter<NE>, C>`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_emitter<NE>(self, emitter: NE) -> Configured<'inp, P, L, With<NE, C>> {
    Configured {
      parser: self.parser,
      config: With::new(emitter, self.config.secondary),
      _marker: PhantomData,
    }
  }

  /// Set custom cache options (replaces any previous cache config).
  ///
  /// Transforms from `With<E, _>` to `With<E, WithCache<NC>>`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_cache<NC>(self, options: NC::Options) -> Configured<'inp, P, L, With<E, With<NC::Options, PhantomData<NC>>>>
  where
    NC: Cache<'inp, L>,
    L: Lexer<'inp>,
  {
    Configured {
      parser: self.parser,
      config: With::new(self.config.primary, With::new(options, PhantomData)),
      _marker: PhantomData,
    }
  }
}

/// Entry-point trait: run a parser against a source.
///
/// This provides the ergonomic `.parse()` API similar to Chumsky and
/// Winnow. Implementations wire up `Input`, `Emitter`, and `Cache`
/// before delegating to [`ParseInput`].
pub trait Parse<'inp, L, O, Error>: Sized {
  /// Parse using the lexer's default state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse(self, src: &'inp L::Source) -> O
  where
    L: Lexer<'inp>,
    L::State: Default,
  {
    self.parse_with_state(src, L::State::default())
  }

  /// Parse using an explicit lexer state.
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> O
  where
    L: Lexer<'inp>;
}

#[cfg_attr(not(tarpaulin), inline(always))]
fn drive<'inp, P, L, O, E, C>(
  mut parser: P,
  src: &'inp L::Source,
  state: L::State,
  mut emitter: E,
  cache: C,
) -> O
where
  P: ParseInput<'inp, L, O, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  let mut input = Input::with_state_and_cache(src, state, cache);
  let mut input_ref = input.as_ref(&mut emitter);
  parser.parse_input(&mut input_ref)
}

impl<'inp, F, L, O, E, C> Parse<'inp, L, O, E::Error> for Parser<F, L, O, E::Error, ParserOptions<E::Error, C::Options, E, C>>
where
  F: ParseInput<'inp, L, O, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(mut self, src: &'inp L::Source, state: L::State) -> O {
    let cache = C::with_options(self.opts.secondary.primary);
    let mut emitter = self.opts.primary.primary;

    let mut input = Input::with_state_and_cache(src, state, cache);
    let mut input_ref = input.as_ref(&mut emitter);
    self.f.parse_input(&mut input_ref)
  }
}

// =============================================================================
// Single Parse implementation for Configured using helper traits
// =============================================================================

impl<'inp, P, L, O, EC, CC> sealed::Sealed<'inp, L, O, EC, CC>
for Configured<'inp, P, L, With<EC, With<CC::Options, PhantomData<CC>>>>
where
  L: Lexer<'inp>,
  EC: Emitter<'inp, L>,
  CC: Cache<'inp, L>,
  P: ParseInput<'inp, L, O, EC, CC>,
{
}

impl<'inp, P, L, O, EC, CC> Parse<'inp, L, O, EC::Error>
for Configured<'inp, P, L, With<EC, With<CC::Options, PhantomData<CC>>>>
where
  L: Lexer<'inp>,
  EC: Emitter<'inp, L>,
  CC: Cache<'inp, L>,
  P: ParseInput<'inp, L, O, EC, CC>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> O {
    let emitter = self.config.primary;
    let cache = CC::with_options(self.config.secondary.primary);
    drive(self.parser, src, state, emitter, cache)
  }
}

/// Trait for computing the next state
pub trait Apply<State> {
  /// The options for computing the next state
  type Options;

  /// Computes the next state given the options.
  fn apply(self, options: Self::Options) -> State;
}

/// Trait for container types used in parsers.
pub trait Container<T> {
  /// Push an item into the container.
  fn push(&mut self, item: T);

  /// Returns the first item in the container, if any.
  fn first(&self) -> Option<&T>;

  /// Returns the last item in the container, if any.
  fn last(&self) -> Option<&T>;

  /// Returns the number of items in the container.
  fn len(&self) -> usize;

  /// Returns `true` if the container is empty.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

macro_rules! blackhole {
  ($ty:ty) => {
    impl<T> Container<T> for $ty {
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn push(&mut self, _: T) {}

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn first(&self) -> Option<&T> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn last(&self) -> Option<&T> {
        None
      }

      #[cfg_attr(not(tarpaulin), inline(always))]
      fn len(&self) -> usize {
        0
      }
    }
  };
}

blackhole!(());
blackhole!(core::marker::PhantomData<T>);
blackhole!(crate::utils::marker::Ignored<T>);
blackhole!(crate::lexer::BlackHole);

/// Shorthand for building a [`Parser`] from a closure.
pub const fn parser<'inp, L, O, E, C, F>(f: F) -> Parser<F, L, O, E::Error>
where
  F: FnMut(&mut InputRef<'inp, '_, L, E, C>) -> O,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  Parser::with(f)
}

/// With something else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct With<P, S> {
  primary: P,
  secondary: S,
}

impl<P, S> With<P, S> {
  /// Create a new `With` combinator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(primary: P, secondary: S) -> Self {
    Self { primary, secondary }
  }

  /// Returns a reference to the primary.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn primary(&self) -> &P {
    &self.primary
  }

  /// Returns a reference to the secondary.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn secondary(&self) -> &S {
    &self.secondary
  }

  /// Returns a mutable reference to the primary.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn primary_mut(&mut self) -> &mut P {
    &mut self.primary
  }

  /// Returns a mutable reference to the secondary.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn secondary_mut(&mut self) -> &mut S {
    &mut self.secondary
  }
}

#[cfg(test)]
mod tests {
  #![allow(warnings)]

  use super::{Token as TokenT, *};
  use crate::{BlackHole, parser::sep::comma_seq, punct::Comma, utils::marker::Ignored};
  use derive_more::Display;
  use logos::*;

  #[derive(Debug, Logos, Clone)]
  #[logos(skip r"[ \t\r\n\f]+")]
  enum Token {
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

    #[token("{")]
    BraceOpen,

    #[token("}")]
    BraceClose,

    #[token("[")]
    BracketOpen,

    #[token("]")]
    BracketClose,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("null")]
    Null,

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),

    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
    String(String),
  }

  #[derive(Debug, Display, PartialEq, Eq, Clone, Copy, Hash)]
  enum TokenKind {
    #[display("bool")]
    Bool,

    #[display("{{")]
    BraceOpen,
    #[display("}}")]
    BraceClose,
    #[display("[")]
    BracketOpen,
    #[display("]")]
    BracketClose,
    #[display(":")]
    Colon,
    #[display(",")]
    Comma,
    #[display("null")]
    Null,
    #[display("number")]
    Number,
    #[display("string")]
    String,
  }

  impl From<&Token> for TokenKind {
    fn from(token: &Token) -> Self {
      match token {
        Token::Bool(_) => TokenKind::Bool,
        Token::BraceOpen => TokenKind::BraceOpen,
        Token::BraceClose => TokenKind::BraceClose,
        Token::BracketOpen => TokenKind::BracketOpen,
        Token::BracketClose => TokenKind::BracketClose,
        Token::Colon => TokenKind::Colon,
        Token::Comma => TokenKind::Comma,
        Token::Null => TokenKind::Null,
        Token::Number(_) => TokenKind::Number,
        Token::String(_) => TokenKind::String,
      }
    }
  }

  impl TokenT<'_> for Token {
    type Kind = TokenKind;

    type Error = ();

    fn kind(&self) -> Self::Kind {
      TokenKind::from(self)
    }
  }

  type JsonLexer<'a> = crate::LogosLexer<'a, Token, Token>;

  fn assert_any_parse_impl<'inp>()
  -> impl Parse<'inp, JsonLexer<'inp>, Option<Spanned<Lexed<'inp, Token>>>, ()> {
    Parser::new().with_emitter(Noop::new()).with_cache::<'_, ()>(()).apply(Any)
  }

  // fn assert_configured_api_compiles<'inp>()
  // -> Configured<'inp, Parser<Any, JsonLexer<'inp>, Option<Spanned<Lexed<'inp, Token>>>, ()>, JsonLexer<'inp>, With<WithEmitter<Noop<()>>, ()>> {
  //   Parser::any().configured()
  // }

  fn token<'inp, L>(inp: &mut InputRef<'inp, '_, L, Noop<()>, ()>) -> ParseResult<'inp, Token, L, Noop<()>>
  where
    L: crate::Lexer<'inp, Token = Token>,
  {
    match inp.next() {
      Some(Spanned { span, data: tok }) => {
        match tok {
          Lexed::Token(tok) => Ok(Spanned { span, data: tok }),
          Lexed::Error(e) => Err(Spanned { span, data: e }),
        }
      },
      None => todo!(),
    }
  }

  fn assert_comma_seq_parse_impl<'inp>()
  -> impl Parse<'inp, JsonLexer<'inp>, ParseResult<'inp, (), JsonLexer<'inp>, Noop<()>>, ()> {
    Parser::new().apply(comma_seq::<_, _, JsonLexer<'inp>, Token, (), Noop<()>, ()>(
      parser(token),
      |t: &Token| {
        if let TokenKind::Comma = t.kind() {
          SeqSepAction::Separator
        } else {
          SeqSepAction::Continue
        }
      }
    ))
    // .configured()
    // .with_emitter(Noop::new())
    // .with_cache::<()>(())
    // .parse()
  }

  // #[test]
  // fn t() {
  //   let src = "{}";

  //   let tok = Parser::any::<JsonLexer<'_>, ()>().parse(src);
  //   let a = Parse::parse(Parser::comma_seq::<'_, _, JsonLexer<'_>, Option<Spanned<Lexed<'_, Token>>>, (), ()>(Parser::any()), src);
  // }
}
