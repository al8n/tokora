use core::{convert::Infallible, fmt, hash::Hash, ops::AddAssign};

pub use cache::*;
pub use checkpoint::Checkpoint;
pub use cursor::Cursor;
pub use emitter::Emitter;
pub use input_ref::InputRef;
pub use source::Source;
pub use token::{
  DelimiterToken, IdentifierToken, KeywordToken, Lexed, LitToken, Logos, OperatorToken,
  PunctuatorToken, Token, TriviaToken,
};

// #[cfg(feature = "logos")]
pub use self::logos::LogosLexer;

pub(crate) use input::Input;

use crate::utils::Spanned;

/// The token related structures and traits
pub mod token;

/// The source related structures and traits
pub mod source;

/// The emitter related structures and traits
pub mod emitter;

mod cache;
mod checkpoint;
mod cursor;
mod input;
mod input_ref;
mod logos;

/// a
pub trait IntoLexer<'inp, T: ?Sized> {
  /// a
  type Lexer;

  /// a
  fn into_lexer(self) -> Self::Lexer
  where
    Self: 'inp,
    T: Token<'inp>;
}

impl<'inp, T, L> IntoLexer<'inp, T> for L
where
  T: Token<'inp>,
  L: Lexer<'inp, Token = T>,
{
  type Lexer = L;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_lexer(self) -> Self::Lexer {
    self
  }
}

/// A trait for lexers
pub trait Lexer<'inp>: 'inp {
  /// The state of the lexer.
  type State: State;
  /// The source type of the lexer.
  type Source: super::Source<Self::Offset> + ?Sized;
  /// The token type produced by the lexer.
  type Token: Token<'inp>;

  /// The span type of the lexer.
  type Span: fmt::Debug + Span<Offset = Self::Offset> + Ord + Clone + Hash;
  /// The offset type of the source.
  type Offset: Default + fmt::Debug + Ord + Clone + Hash;

  /// Lexes the input source and returns a tokenizer.
  fn new(src: &'inp Self::Source) -> Self
  where
    Self::State: Default;

  /// Lexes the input source with the given initial state and returns a tokenizer.
  fn with_state(src: &'inp Self::Source, state: Self::State) -> Self;

  /// Checks the current state of the lexer for errors.
  ///
  /// If the state is valid, returns `Ok(())`, otherwise returns an error.
  fn check(&self) -> Result<(), <Self::Token as Token<'inp>>::Error>;

  /// Returns a reference to the current state of the lexer.
  fn state(&self) -> &Self::State;

  /// Returns a mutable reference to the current state of the lexer.
  fn state_mut(&mut self) -> &mut Self::State;

  /// Consumes the lexer and returns the current state.
  fn into_state(self) -> Self::State;

  /// Returns a reference to the source being lexed.
  fn source(&self) -> &'inp Self::Source;

  /// Get the range for the current token in `Source`.
  fn span(&self) -> Self::Span;

  // /// Returns the offset in the source for the given cursor.
  // fn offset(&self, cursor: &Self::Cursor) -> Self::Offset;

  /// Returns the slice of the current token in the source.
  fn slice(&self) -> <Self::Source as Source<Self::Offset>>::Slice<'inp>;

  /// Lexes the next token from the input source.
  fn lex(&mut self) -> Option<Result<Self::Token, <Self::Token as Token<'inp>>::Error>>;

  /// Bumps the end of currently lexed token by `n` offsets.
  ///
  /// # Panics
  ///
  /// Panics if adding `n` to current offset would place the `Lexer` beyond the last byte,
  /// or in the middle of an UTF-8 code point (does not apply when lexing raw `&[u8]`).
  fn bump(&mut self, n: &Self::Offset);
}

/// A trait for types that can be lexed from the input.
///
/// This trait provides a standardized way to lex (tokenize) an entire input
/// into a structured type. It's useful for types that represent complete
/// lexical structures that can be built from an input source.
///
/// # Type Parameters
///
/// - `I`: The input type to lex from (e.g., `&str`, `&[u8]`)
/// - `Error`: The error type returned when lexing fails
///
/// # Example
///
/// ```rust,ignore
/// use logosky::Lexable;
///
/// struct Document {
///     tokens: Vec<Token>,
/// }
///
/// impl Lexable<&str, LexError> for Document {
///     fn lex(input: &str) -> Result<Self, LexError> {
///         // Lex the entire input into a Document
///         let tokens = tokenize(input)?;
///         Ok(Document { tokens })
///     }
/// }
/// ```
pub trait Lexable<I, Error> {
  /// Lexes `Self` from the given input.
  ///
  /// This method consumes the input and attempts to construct `Self` by
  /// lexing the entire input. It returns an error if the input cannot be
  /// successfully lexed.
  ///
  /// # Errors
  ///
  /// Returns an error if the input is malformed or cannot be lexed according
  /// to the rules of the implementing type.
  fn lex(input: I) -> Result<Self, Error>
  where
    Self: Sized;
}

/// The state trait for lexers
pub trait State: core::fmt::Debug + Clone {
  /// The error type of the state.
  type Error: Clone;

  /// Checks the state for errors.
  fn check(&self) -> Result<(), Self::Error>;
}

impl State for () {
  type Error = ();

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn check(&self) -> Result<(), Self::Error> {
    Ok(())
  }
}

impl State for Infallible {
  type Error = Infallible;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn check(&self) -> Result<(), Self::Error> {
    Ok(())
  }
}

/// A cached token with its associated extras.
pub struct CachedToken<'a, L: Lexer<'a>> {
  token: Spanned<Lexed<'a, L::Token>, L::Span>,
  state: L::State,
}

impl<'a, L: Lexer<'a>> Clone for CachedToken<'a, L>
where
  L::State: Clone,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    Self {
      token: self.token.clone(),
      state: self.state.clone(),
    }
  }
}

impl<'a, L: Lexer<'a>> CachedToken<'a, L> {
  /// Creates a new cached token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new(token: Spanned<Lexed<'a, L::Token>, L::Span>, state: L::State) -> Self {
    Self { token, state }
  }

  /// Returns a reference to the token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn token(&self) -> &Spanned<Lexed<'a, L::Token>, L::Span> {
    &self.token
  }

  /// Consumes the cached token and returns the lexed token.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_token(self) -> Spanned<Lexed<'a, L::Token>, L::Span> {
    self.token
  }

  /// Returns a reference to the state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn state(&self) -> &L::State {
    &self.state
  }

  /// Consumes the cached token and returns the extras.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(clippy::type_complexity)]
  pub fn into_components(self) -> (Spanned<Lexed<'a, L::Token>, L::Span>, L::State) {
    (self.token, self.state)
  }
}

/// A trait representing a span in the source code.
pub trait Span {
  /// The offset type of the span.
  type Offset: Ord + Clone + Hash;

  /// Creates a new span from the given start and end offsets.
  fn new(start: Self::Offset, end: Self::Offset) -> Self;

  /// Consumes the span and returns it.
  fn into_range(self) -> core::ops::Range<Self::Offset>
  where
    Self: Sized;

  /// Returns the start offset of the span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start(&self) -> Self::Offset {
    self.start_ref().clone()
  }

  /// Returns the start offset of the span.
  fn start_ref(&self) -> &Self::Offset;

  /// Returns the mutable reference to the start offset of the span.
  fn start_mut(&mut self) -> &mut Self::Offset;

  /// Returns the end offset of the span.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end(&self) -> Self::Offset {
    self.end_ref().clone()
  }

  /// Returns the end offset of the span.
  fn end_ref(&self) -> &Self::Offset;

  /// Returns the mutable reference to the end offset of the span.
  fn end_mut(&mut self) -> &mut Self::Offset;

  /// Bumps the span by `n` offsets.
  fn bump(&mut self, n: &Self::Offset);
}

impl Span for core::ops::Range<usize> {
  type Offset = usize;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn new(start: Self::Offset, end: Self::Offset) -> Self {
    start..end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start_ref(&self) -> &Self::Offset {
    &self.start
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end_ref(&self) -> &Self::Offset {
    &self.end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start_mut(&mut self) -> &mut Self::Offset {
    &mut self.start
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end_mut(&mut self) -> &mut Self::Offset {
    &mut self.end
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn bump(&mut self, n: &Self::Offset) {
    self.end += *n;
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_range(self) -> core::ops::Range<Self::Offset> {
    self.start..self.end
  }
}

impl<O> Span for crate::utils::Span<O>
where
  O: Ord + Clone + Hash + for<'a> AddAssign<&'a O>,
{
  type Offset = O;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn new(start: Self::Offset, end: Self::Offset) -> Self {
    crate::utils::Span::new(start, end)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start_ref(&self) -> &Self::Offset {
    self.start_ref()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn start_mut(&mut self) -> &mut Self::Offset {
    self.start_mut()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end_ref(&self) -> &Self::Offset {
    self.end_ref()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn end_mut(&mut self) -> &mut Self::Offset {
    self.end_mut()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn bump(&mut self, n: &Self::Offset) {
    self.bump(n);
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_range(self) -> core::ops::Range<Self::Offset> {
    self.start..self.end
  }
}

/// A black hole cache that discards all tokens.
///
/// `BlackHole` implements the [`Cache`] trait but doesn't actually store any tokens.
/// All tokens pushed to it are immediately discarded. This is useful when you want to
/// process tokens in a streaming fashion without maintaining a lookahead buffer.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlackHole;

impl<O> From<O> for BlackHole
where
  (): From<O>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(_: O) -> Self {
    BlackHole
  }
}
