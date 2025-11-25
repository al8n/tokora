#![allow(clippy::type_complexity)]

use core::marker::PhantomData;

use crate::{
  Cache, DefaultCache, Emitter, Lexed, Lexer, Noop, Token, lexer::{Input, InputRef}, utils::Spanned
};

pub use any::*;

mod any;

mod sealed {
  use super::*;

  pub trait Sealed<'inp, L, O, E, C> {}

  impl<'inp, F, L, O, E, C> sealed::Sealed<'inp, L, O, E, C> for F
  where
    F: FnMut(&mut InputRef<'inp, '_, L, E, C>) -> O,
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {}

  impl<'inp, F, L, O, E, C> Sealed<'inp, L, O, E, C> for Parser<F, L, O, E::Error>
  where
    F: ParseInput<'inp, L, O, E, C>,
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {
  }

  impl<'inp, L, O, E, P, C> Sealed<'inp, L, O, E, C> for WithEmitter<P, E>
  where
    P: ParseInput<'inp, L, O, E, C>,
    L: Lexer<'inp>,
    E: Emitter<'inp, L>,
    C: Cache<'inp, L>,
  {
  }

  impl<'inp, L, O, E, P, C> Sealed<'inp, L, O, E, C> for WithCache<'inp, P, L, C>
  where
    P: ParseInput<'inp, L, O, E, C>,
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
pub trait ParseInput<'inp, L, O, E, C>:
  sealed::Sealed<'inp, L, O, E, C>
{
  /// Try to parse from the given input.
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, E, C>,
  ) -> O
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
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, E, C>,
  ) -> O {
    (self)(input)
  }
}

/// Lightweight wrapper around a parsing function.
#[repr(transparent)]
pub struct Parser<F, L, O, Error> {
  f: F,
  _marker: PhantomData<(L, O, Error)>,
}

impl<F, L, O, Error> Parser<F, L, O, Error> {
  /// Wrap a parsing function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(f: F) -> Self {
    Self {
      f,
      _marker: PhantomData,
    }
  }

  /// Attach a custom emitter to the parser.
  pub fn with_emitter<E>(self, emitter: E) -> WithEmitter<Self, E> {
    WithEmitter {
      inner: self,
      emitter,
    }
  }

  /// Attach custom cache options to the parser.
  pub fn with_cache<'inp, C>(self, options: C::Options) -> WithCache<'inp, Self, L, C>
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
}

impl<'inp, F, L, O, E, C> ParseInput<'inp, L, O, E, C>
  for Parser<F, L, O, E::Error>
where
  F: ParseInput<'inp, L, O, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, E, C>,
  ) -> O {
    self.f.parse_input(input)
  }
}

/// Parser configured with a concrete emitter.
pub struct WithEmitter<P, E> {
  inner: P,
  emitter: E,
}

impl<P, E> WithEmitter<P, E> {
  /// Attach cache options after an emitter has been selected.
  pub fn with_cache<'inp, L, C>(self, options: C::Options) -> WithEmitter<WithCache<'inp, P, L, C>, E>
  where
    L: Lexer<'inp>,
    C: Cache<'inp, L>,
  {
    WithEmitter {
      inner: WithCache {
        inner: self.inner,
        cache_opts: options,
        _marker: PhantomData,
      },
      emitter: self.emitter,
    }
  }
}

impl<'inp, P, L, O, E, C> ParseInput<'inp, L, O, E, C>
  for WithEmitter<P, E>
where
  P: ParseInput<'inp, L, O, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, E, C>,
  ) -> O {
    self.inner.parse_input(input)
  }
}

/// Parser configured with a concrete cache.
pub struct WithCache<'inp, P, L: Lexer<'inp>, C: Cache<'inp, L>> {
  inner: P,
  cache_opts: C::Options,
  _marker: PhantomData<fn() -> (&'inp L::Source, L, C)>,
}

impl<'inp, P, L, C> WithCache<'inp, P, L, C>
where
  L: Lexer<'inp>,
  C: Cache<'inp, L>,
{
  /// Attach an emitter after cache options have been selected.
  pub fn with_emitter<E>(self, emitter: E) -> WithEmitter<Self, E> {
    WithEmitter {
      inner: self,
      emitter,
    }
  }
}

impl<'inp, P, L, O, E, C> ParseInput<'inp, L, O, E, C>
  for WithCache<'inp, P, L, C>
where
  P: ParseInput<'inp, L, O, E, C>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_input(
    &mut self,
    input: &mut InputRef<'inp, '_, L, E, C>,
  ) -> O {
    self.inner.parse_input(input)
  }
}

/// Entry-point trait: run a parser against a source.
///
/// This provides the ergonomic `.parse()` API similar to Chumsky and
/// Winnow. Implementations wire up `Input`, `Emitter`, and `Cache`
/// before delegating to [`ParseInput`].
pub trait Parse<'inp, L, O, E>: Sized {
  /// Parse using the lexer's default state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse(self, src: &'inp L::Source) -> O
  where
    L: Lexer<'inp>,
    L::State: Default,
    E: Emitter<'inp, L>,
  {
    self.parse_with_state(src, L::State::default())
  }

  /// Parse using an explicit lexer state.
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> O
  where
    L: Lexer<'inp>,
    E: Emitter<'inp, L>;
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

impl<'inp, F, L, O, Error> Parse<'inp, L, O, Noop<Error>> for Parser<F, L, O, Error>
where
  F: ParseInput<'inp, L, O, Noop<Error>, DefaultCache<'inp, L>>,
  L: Lexer<'inp>,
  Error: From<<L::Token as Token<'inp>>::Error>,
  Noop<Error>: Emitter<'inp, L, Error = Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> O {
    let cache = <DefaultCache<'inp, L> as Cache<'inp, L>>::new();
    let emitter = Noop::<Error>::default();
    drive(self, src, state, emitter, cache)
  }
}

impl<'inp, F, L, O, E> Parse<'inp, L, O, E> for WithEmitter<Parser<F, L, O, E::Error>, E>
where
  F: ParseInput<'inp, L, O, E, DefaultCache<'inp, L>>,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> O
  where
    L: Lexer<'inp>,
  {
    let cache = <DefaultCache<'inp, L> as Cache<'inp, L>>::new();
    drive(self.inner, src, state, self.emitter, cache)
  }
}

impl<'inp, F, L, O, C, Error> Parse<'inp, L, O, Noop<Error>>
  for WithCache<'inp, Parser<F, L, O, Error>, L, C>
where
  F: ParseInput<'inp, L, O, Noop<Error>, C>,
  L: Lexer<'inp>,
  C: Cache<'inp, L>,
  Error: From<<L::Token as Token<'inp>>::Error>,
  Noop<Error>: Emitter<'inp, L, Error = Error>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> O
  where
    L: Lexer<'inp>,
  {
    let cache = C::with_options(self.cache_opts);
    let emitter = Noop::<Error>::default();
    drive(self.inner, src, state, emitter, cache)
  }
}

impl<'inp, F, L, O, E, C> Parse<'inp, L, O, E> for WithEmitter<WithCache<'inp, Parser<F, L, O, E::Error>, L, C>, E>
where
  F: ParseInput<'inp, L, O, E, C>,
  L: Lexer<'inp>,
  C: Cache<'inp, L>,
  E: Emitter<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> O
  where
    L: Lexer<'inp>,
  {
    let cache = C::with_options(self.inner.cache_opts);
    drive(self.inner.inner, src, state, self.emitter, cache)
  }
}

impl<'inp, F, L, O, E, C> Parse<'inp, L, O, E> for WithCache<'inp, WithEmitter<Parser<F, L, O, E::Error>, E>, L, C>
where
  F: ParseInput<'inp, L, O, E, C>,
  L: Lexer<'inp>,
  C: Cache<'inp, L>,
  E: Emitter<'inp, L>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn parse_with_state(self, src: &'inp L::Source, state: L::State) -> O
  where
    L: Lexer<'inp>,
  {
    let cache = C::with_options(self.cache_opts);
    drive(self.inner.inner, src, state, self.inner.emitter, cache)
  }
}

/// Shorthand for building a [`Parser`] from a closure.
pub const fn parser<'inp, L, O, E, C, F>(f: F) -> Parser<F, L, O, E::Error>
where
  F: FnMut(&mut InputRef<'inp, '_, L, E, C>) -> O,
  L: Lexer<'inp>,
  E: Emitter<'inp, L>,
  C: Cache<'inp, L>,
{
  Parser::new(f)
}

#[cfg(test)]
mod tests {
  #![allow(warnings)]

  use logos::*;
  use super::{Token as TokenT, *};

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

  #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
  enum TokenKind {
    Bool,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    Null,
    Number,
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

  const fn assert_any_parse_impl<'inp>() -> impl Parse<'inp, JsonLexer<'inp>, Option<Spanned<Lexed<'inp, Token>>>, Noop<()>> {
    any()
  }

  #[test]
  fn t() {
    let src = "{}";

    let tok = any::<JsonLexer<'_>>().parse(src);
  }
}
