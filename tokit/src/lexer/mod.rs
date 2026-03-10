use core::{fmt, hash::Hash};

use derive_more::{IsVariant, TryUnwrap, Unwrap};

use crate::span::{Span, Spanned};

use super::{Source, State, token::Token};

/// A module containing integrations with the `logos` lexer library.
#[cfg(any(feature = "logos_0_16", feature = "logos_0_15", feature = "logos_0_14"))]
#[cfg_attr(
  docsrs,
  doc(cfg(any(feature = "logos_0_16", feature = "logos_0_15", feature = "logos_0_14")))
)]
mod logos;

#[cfg(any(feature = "logos_0_16", feature = "logos_0_15", feature = "logos_0_14"))]
#[cfg_attr(
  docsrs,
  doc(cfg(any(feature = "logos_0_16", feature = "logos_0_15", feature = "logos_0_14")))
)]
pub use logos::*;

/// The result of lexing a single token: either a successful token or an error.
///
/// `Lexed` represents the output of the lexing process for a single position in the input.
/// It can either be a successfully recognized [`Token`] with its span information, or a
/// lexing error that occurred at that position.
///
/// # Error Handling Strategy
///
/// `Lexed` enables **error recovery** during lexing by keeping errors in the token stream
/// rather than immediately aborting. This allows you to:
///
/// - Continue lexing after an error to find multiple issues in one pass
/// - Collect all lexing errors before reporting them to the user
/// - Implement "best-effort" parsing that tolerates some malformed input
/// - Provide better diagnostics by showing multiple errors at once
///
/// # Convenience Methods
///
/// Thanks to the `derive_more` macros, `Lexed` provides several utility methods:
///
/// - `is_token()` / `is_error()`: Check which variant this is
/// - `unwrap_token()` / `unwrap_error()`: Unwrap to get the inner value (panics if wrong variant)
/// - `try_unwrap_token()` / `try_unwrap_error()`: Safe unwrapping that returns `Option`
/// - `unwrap_token_ref()` / `unwrap_error_ref()`: Get a reference to the inner value
///
/// ## Examples
///
/// ## Basic Usage
///
/// ```rust,ignore
/// use tokit::{Lexed, TokenExt};
///
/// let mut lexer = logos::Lexer::<MyTokens>::new(input);
///
/// while let Some(lexed) = MyToken::lex(&mut lexer) {
///     match lexed {
///         Lexed::Token(spanned_token) => {
///             println!("Token: {:?} at {:?}", spanned_token.data(), spanned_token.span());
///         }
///         Lexed::Error(err) => {
///             eprintln!("Lexing error: {:?}", err);
///         }
///     }
/// }
/// ```
///
/// ## Error Collection
///
/// ```rust,ignore
/// let mut tokens = Vec::new();
/// let mut errors = Vec::new();
///
/// let mut lexer = logos::Lexer::<MyTokens>::new(input);
/// while let Some(lexed) = MyToken::lex(&mut lexer) {
///     match lexed {
///         Lexed::Token(tok) => tokens.push(tok),
///         Lexed::Error(err) => errors.push(err),
///     }
/// }
///
/// if !errors.is_empty() {
///     report_lexing_errors(&errors);
/// }
/// ```
///
/// ## Using Convenience Methods
///
/// ```rust,ignore
/// if lexed.is_token() {
///     let token = lexed.unwrap_token_ref();
///     process_token(token);
/// }
///
/// // Safe unwrapping
/// if let Some(token) = lexed.try_unwrap_token() {
///     use_token(token);
/// }
/// ```
#[derive(Debug, PartialEq, IsVariant, Unwrap, TryUnwrap)]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
pub enum Lexed<'a, T: Token<'a>> {
  /// A successfully recognized token with its span information.
  Token(T),

  /// A lexing error that occurred during tokenization.
  ///
  /// The error type is determined by the Logos lexer's error type. It typically
  /// contains information about what went wrong and where in the input it occurred.
  Error(T::Error),
}

impl<'a, T> Clone for Lexed<'a, T>
where
  T: Token<'a>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    match self {
      Self::Token(tok) => Self::Token(tok.clone()),
      Self::Error(err) => Self::Error(err.clone()),
    }
  }
}

impl<'a, T> Copy for Lexed<'a, T>
where
  T: Token<'a> + Copy,
  T::Error: Copy,
{
}

impl<'a, T: Token<'a>> Lexed<'a, T> {
  /// Lexes the next token from the given lexer, returning `None` if the input is exhausted.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn lex<L>(lexer: &mut L) -> Option<Self>
  where
    L: super::Lexer<'a, Token = T>,
  {
    lexer.lex().map(|res| res.into())
  }

  /// Lexes the next token from the given lexer, returning `None` if the input is exhausted.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn lex_spanned<L>(lexer: &mut L) -> Option<Spanned<Self, L::Span>>
  where
    L: super::Lexer<'a, Token = T>,
  {
    lexer
      .lex()
      .map(|res| Spanned::new(lexer.span(), res.into()))
  }

  /// Returns the contained [`Lexed::Token`] value, consuming the `self` value.
  ///
  /// # Panics
  ///
  /// Panics if the value is a [`Lexed::Error`] with a custom panic message provided by
  /// `msg`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  pub fn expect_token(self, msg: &str) -> T {
    match self {
      Self::Token(tok) => tok,
      Self::Error(_) => panic!("{msg}"),
    }
  }

  /// Returns the contained [`Lexed::Error`] value, consuming the `self` value.
  ///
  /// # Panics
  ///
  /// Panics if the value is a [`Lexed::Token`] with a custom panic message provided by
  /// `msg`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  pub fn expect_error(self, msg: &str) -> T::Error {
    match self {
      Self::Token(_) => panic!("{msg}"),
      Self::Error(err) => err,
    }
  }

  /// Returns the reference of the contained [`Lexed::Token`] value.
  ///
  /// # Panics
  ///
  /// Panics if the value is a [`Lexed::Error`] with a custom panic message provided by
  /// `msg`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  pub fn expect_token_ref(&self, msg: &str) -> &T {
    match self {
      Self::Token(tok) => tok,
      Self::Error(_) => panic!("{msg}"),
    }
  }

  /// Returns the reference of the contained [`Lexed::Error`] value.
  ///
  /// # Panics
  ///
  /// Panics if the value is a [`Lexed::Token`] with a custom panic message provided by
  /// `msg`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  pub fn expect_error_ref(&self, msg: &str) -> &T::Error {
    match self {
      Self::Token(_) => panic!("{msg}"),
      Self::Error(err) => err,
    }
  }

  /// Returns the mutable reference of the contained [`Lexed::Token`] value.
  ///
  /// # Panics
  ///
  /// Panics if the value is a [`Lexed::Error`] with a custom panic message provided by
  /// `msg`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  pub fn expect_token_mut(&mut self, msg: &str) -> &mut T {
    match self {
      Self::Token(tok) => tok,
      Self::Error(_) => panic!("{msg}"),
    }
  }

  /// Returns the mutable reference of the contained [`Lexed::Error`] value.
  ///
  /// # Panics
  ///
  /// Panics if the value is a [`Lexed::Token`] with a custom panic message provided by
  /// `msg`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[track_caller]
  pub fn expect_error_mut(&mut self, msg: &str) -> &mut T::Error {
    match self {
      Self::Token(_) => panic!("{msg}"),
      Self::Error(err) => err,
    }
  }
}

impl<'a, T: 'a> core::fmt::Display for Lexed<'a, T>
where
  T: Token<'a> + core::fmt::Display,
  T::Error: core::fmt::Display,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Token(tok) => ::core::fmt::Display::fmt(tok, f),
      Self::Error(err) => err.fmt(f),
    }
  }
}

impl<'a, T: Token<'a>> From<Result<T, T::Error>> for Lexed<'a, T> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(value: Result<T, T::Error>) -> Self {
    match value {
      Ok(tok) => Self::Token(tok),
      Err(err) => Self::Error(err),
    }
  }
}

impl<'a, T: Token<'a>> From<Lexed<'a, T>> for Result<T, T::Error> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(value: Lexed<'a, T>) -> Self {
    match value {
      Lexed::Token(tok) => Ok(tok),
      Lexed::Error(err) => Err(err),
    }
  }
}

/// A trait to convert a type into a lexer.
pub trait IntoLexer<'inp, T: ?Sized> {
  /// The lexer type.
  type Lexer;

  /// Converts `self` into a lexer.
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
  fn new(src: &'inp Self::Source) -> Self;

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
/// use tokit::Lexable;
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

#[cfg(test)]
#[allow(warnings)]
mod tests {
  use super::*;

  // Use DummyToken defined below

  #[test]
  fn lexed_from_ok_result() {
    let result: Result<DummyToken, ()> = Ok(DummyToken);
    let lexed: Lexed<'_, DummyToken> = result.into();
    assert!(lexed.is_token());
    assert!(!lexed.is_error());
  }

  #[test]
  fn lexed_from_err_result() {
    let result: Result<DummyToken, ()> = Err(());
    let lexed: Lexed<'_, DummyToken> = result.into();
    assert!(!lexed.is_token());
    assert!(lexed.is_error());
  }

  #[test]
  fn lexed_into_result_ok() {
    let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    let result: Result<DummyToken, ()> = lexed.into();
    assert!(result.is_ok());
  }

  #[test]
  fn lexed_into_result_err() {
    let lexed = Lexed::<'_, DummyToken>::Error(());
    let result: Result<DummyToken, ()> = lexed.into();
    assert!(result.is_err());
  }

  #[test]
  fn lexed_expect_token() {
    let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    let tok = lexed.expect_token("should be a token");
    assert_eq!(tok, DummyToken);
  }

  #[test]
  #[should_panic(expected = "not a token")]
  fn lexed_expect_token_panics_on_error() {
    let lexed = Lexed::<'_, DummyToken>::Error(());
    lexed.expect_token("not a token");
  }

  #[test]
  fn lexed_expect_error() {
    let lexed = Lexed::<'_, DummyToken>::Error(());
    lexed.expect_error("should be an error");
  }

  #[test]
  #[should_panic(expected = "not an error")]
  fn lexed_expect_error_panics_on_token() {
    let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    lexed.expect_error("not an error");
  }

  #[test]
  fn lexed_expect_token_ref() {
    let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    let tok = lexed.expect_token_ref("should be a token");
    assert_eq!(tok, &DummyToken);
  }

  #[test]
  #[should_panic(expected = "not a token")]
  fn lexed_expect_token_ref_panics_on_error() {
    let lexed = Lexed::<'_, DummyToken>::Error(());
    lexed.expect_token_ref("not a token");
  }

  #[test]
  fn lexed_expect_error_ref() {
    let lexed = Lexed::<'_, DummyToken>::Error(());
    let _err = lexed.expect_error_ref("should be an error");
  }

  #[test]
  #[should_panic(expected = "not an error")]
  fn lexed_expect_error_ref_panics_on_token() {
    let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    lexed.expect_error_ref("not an error");
  }

  #[test]
  fn lexed_expect_token_mut() {
    let mut lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    let tok = lexed.expect_token_mut("should be a token");
    assert_eq!(tok, &mut DummyToken);
  }

  #[test]
  #[should_panic(expected = "not a token")]
  fn lexed_expect_token_mut_panics_on_error() {
    let mut lexed = Lexed::<'_, DummyToken>::Error(());
    lexed.expect_token_mut("not a token");
  }

  #[test]
  fn lexed_expect_error_mut() {
    let mut lexed = Lexed::<'_, DummyToken>::Error(());
    let _err = lexed.expect_error_mut("should be an error");
  }

  #[test]
  #[should_panic(expected = "not an error")]
  fn lexed_expect_error_mut_panics_on_token() {
    let mut lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    lexed.expect_error_mut("not an error");
  }

  // Display test removed: DummyToken::Error = () which doesn't impl Display

  #[test]
  fn lexed_clone() {
    let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    let cloned = lexed.clone();
    assert_eq!(lexed, cloned);
  }

  #[test]
  fn lexed_try_unwrap() {
    let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    assert!(lexed.try_unwrap_token().is_ok());

    let lexed = Lexed::<'_, DummyToken>::Error(());
    assert!(lexed.try_unwrap_error().is_ok());
  }

  #[test]
  fn lexed_unwrap_ref() {
    let lexed = Lexed::<'_, DummyToken>::Token(DummyToken);
    assert_eq!(lexed.unwrap_token_ref(), &DummyToken);

    let lexed = Lexed::<'_, DummyToken>::Error(());
    assert_eq!(lexed.unwrap_error_ref(), &());
  }
}

#[cfg(test)]
pub(crate) struct DummyLexer;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display)]
#[display("DummyToken")]
pub(crate) struct DummyToken;

#[cfg(test)]
const _: () = {
  use crate::token::{LitToken, PunctuatorToken};

  impl Token<'_> for DummyToken {
    type Kind = Self;
    type Error = ();

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn kind(&self) -> Self::Kind {
      *self
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_trivia(&self) -> bool {
      true
    }
  }

  impl PunctuatorToken<'_> for DummyToken {}

  impl LitToken<'_> for DummyToken {}

  impl<'inp> Lexer<'inp> for DummyLexer {
    type State = ();

    type Source = str;

    type Token = DummyToken;

    type Span = crate::span::SimpleSpan;

    type Offset = usize;

    fn new(_: &'inp Self::Source) -> Self
    where
      Self::State: Default,
    {
      todo!()
    }

    fn with_state(_: &'inp Self::Source, _: Self::State) -> Self {
      todo!()
    }

    fn check(&self) -> Result<(), <Self::Token as Token<'inp>>::Error> {
      todo!()
    }

    fn state(&self) -> &Self::State {
      todo!()
    }

    fn state_mut(&mut self) -> &mut Self::State {
      todo!()
    }

    fn into_state(self) -> Self::State {
      todo!()
    }

    fn source(&self) -> &'inp Self::Source {
      todo!()
    }

    fn span(&self) -> Self::Span {
      todo!()
    }

    fn slice(&self) -> <Self::Source as Source<Self::Offset>>::Slice<'inp> {
      todo!()
    }

    fn lex(&mut self) -> Option<Result<Self::Token, <Self::Token as Token<'inp>>::Error>> {
      todo!()
    }

    fn bump(&mut self, _: &Self::Offset) {
      todo!()
    }
  }
};
