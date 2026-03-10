#![cfg(all(feature = "std", feature = "logos"))]
#![allow(warnings)]

//! Tests that directly exercise emitter trait method implementations for
//! Ignored, Silent, Fatal, and Verbose emitters to cover many small uncovered
//! code paths (each file has 0-4 uncovered lines).

mod common;

use common::{TestLexer, Token, TokenKind};

use tokit::{
  Emitter, Lexer, Token as TokenTrait,
  emitter::{
    Fatal, FromSeparatedError, FromUnexpectedLeadingSeparatorError,
    FromUnexpectedTrailingSeparatorError, FullContainerEmitter, Ignored, PrattEmitter,
    SeparatedEmitter, Silent, TooFewEmitter, TooManyEmitter, UnexpectedLeadingSeparatorEmitter,
    UnexpectedTrailingSeparatorEmitter, Verbose,
  },
  error::{
    UnexpectedEoLhs, UnexpectedEoRhs, UnexpectedEot,
    syntax::{FullContainer, MissingSyntax, TooFew, TooMany},
    token::{MissingToken, UnexpectedToken},
  },
  span::{SimpleSpan, Spanned},
  utils::CowStr,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Helper error type for Fatal/Verbose emitters that need FromEmitterError etc.
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
enum TestError {
  UnexpectedToken,
  LexerError,
  TooFew,
  TooMany,
  FullContainer,
  MissingSeparator,
  MissingElement,
  UnexpectedLeadingSep,
  UnexpectedTrailingSep,
  UnexpectedEoLhs,
  UnexpectedEoRhs,
}

impl From<()> for TestError {
  fn from(_: ()) -> Self {
    TestError::LexerError
  }
}

impl<'a, T, Kind: Clone, S, Lang: ?Sized> From<UnexpectedToken<'a, T, Kind, S, Lang>>
  for TestError
{
  fn from(_: UnexpectedToken<'a, T, Kind, S, Lang>) -> Self {
    TestError::UnexpectedToken
  }
}

impl<S, Lang: ?Sized> From<FullContainer<S, Lang>> for TestError {
  fn from(_: FullContainer<S, Lang>) -> Self {
    TestError::FullContainer
  }
}

impl<S, Lang: ?Sized> From<TooFew<S, Lang>> for TestError {
  fn from(_: TooFew<S, Lang>) -> Self {
    TestError::TooFew
  }
}

impl<S, Lang: ?Sized> From<TooMany<S, Lang>> for TestError {
  fn from(_: TooMany<S, Lang>) -> Self {
    TestError::TooMany
  }
}

impl From<UnexpectedEot> for TestError {
  fn from(_: UnexpectedEot) -> Self {
    TestError::LexerError
  }
}

impl<O, Lang: ?Sized> From<UnexpectedEoLhs<O, Lang>> for TestError {
  fn from(_: UnexpectedEoLhs<O, Lang>) -> Self {
    TestError::UnexpectedEoLhs
  }
}

impl<O, Lang: ?Sized> From<UnexpectedEoRhs<O, Lang>> for TestError {
  fn from(_: UnexpectedEoRhs<O, Lang>) -> Self {
    TestError::UnexpectedEoRhs
  }
}

impl<'inp> FromSeparatedError<'inp, TestLexer<'inp>> for TestError {
  fn from_missing_separator(
    _: CowStr,
    _: tokit::error::token::MissingTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    TestError::MissingSeparator
  }

  fn from_missing_element(_: tokit::error::syntax::MissingSyntaxOf<'inp, TestLexer<'inp>>) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    TestError::MissingElement
  }
}

impl<'inp> FromUnexpectedLeadingSeparatorError<'inp, TestLexer<'inp>> for TestError {
  fn from_unexpected_leading_separator(
    _: CowStr,
    _: tokit::error::token::UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    TestError::UnexpectedLeadingSep
  }
}

impl<'inp> FromUnexpectedTrailingSeparatorError<'inp, TestLexer<'inp>> for TestError {
  fn from_unexpected_trailing_separator(
    _: CowStr,
    _: tokit::error::token::UnexpectedTokenOf<'inp, TestLexer<'inp>>,
  ) -> Self
  where
    TestLexer<'inp>: Lexer<'inp>,
  {
    TestError::UnexpectedTrailingSep
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Ignored emitter: direct trait method calls
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn ignored_full_container_emitter() {
  let mut ign = Ignored::default();
  let err = FullContainer::new(SimpleSpan::new(0usize, 5usize), 10, 5);
  let result =
    <Ignored as FullContainerEmitter<'_, TestLexer<'_>>>::emit_full_container(&mut ign, err);
  assert!(result.is_ok());
}

#[test]
fn ignored_too_few_emitter() {
  let mut ign = Ignored::default();
  let err = TooFew::new(SimpleSpan::new(0usize, 5usize), 1, 3);
  let result = <Ignored as TooFewEmitter<'_, TestLexer<'_>>>::emit_too_few(&mut ign, err);
  assert!(result.is_ok());
}

#[test]
fn ignored_too_many_emitter() {
  let mut ign = Ignored::default();
  let err = TooMany::new(SimpleSpan::new(0usize, 5usize), 10, 5);
  let result = <Ignored as TooManyEmitter<'_, TestLexer<'_>>>::emit_too_many(&mut ign, err);
  assert!(result.is_ok());
}

#[test]
fn ignored_separated_emitter_missing_separator() {
  let mut ign = Ignored::default();
  let name = CowStr::from_static("comma");
  let err: MissingToken<'_, TokenKind, usize> = MissingToken::new(0usize);
  let result =
    <Ignored as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_separator(&mut ign, name, err);
  assert!(result.is_ok());
}

#[test]
fn ignored_separated_emitter_missing_element() {
  let mut ign = Ignored::default();
  let err = MissingSyntax::new(0usize);
  let result =
    <Ignored as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_element(&mut ign, err);
  assert!(result.is_ok());
}

#[test]
fn ignored_unexpected_leading_separator_emitter() {
  let mut ign = Ignored::default();
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Ignored as UnexpectedLeadingSeparatorEmitter<'_, TestLexer<'_>>>::emit_unexpected_leading_separator(&mut ign, name, err);
  assert!(result.is_ok());
}

#[test]
fn ignored_unexpected_trailing_separator_emitter() {
  let mut ign = Ignored::default();
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Ignored as UnexpectedTrailingSeparatorEmitter<'_, TestLexer<'_>>>::emit_unexpected_trailing_separator(&mut ign, name, err);
  assert!(result.is_ok());
}

#[test]
fn ignored_pratt_emitter_lhs() {
  let mut ign = Ignored::default();
  let err = UnexpectedEoLhs::eolhs(0usize);
  let result =
    <Ignored as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_lhs(&mut ign, err);
  assert!(result.is_ok());
}

#[test]
fn ignored_pratt_emitter_rhs() {
  let mut ign = Ignored::default();
  let err = UnexpectedEoRhs::eorhs(0usize);
  let result =
    <Ignored as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_rhs(&mut ign, err);
  assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Silent emitter: direct trait method calls
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn silent_full_container_emitter() {
  let mut s = Silent::<TestError>::new();
  let err = FullContainer::new(SimpleSpan::new(0usize, 5usize), 10, 5);
  let result = <Silent<TestError> as FullContainerEmitter<'_, TestLexer<'_>>>::emit_full_container(
    &mut s, err,
  );
  assert!(result.is_ok());
}

#[test]
fn silent_too_few_emitter() {
  let mut s = Silent::<TestError>::new();
  let err = TooFew::new(SimpleSpan::new(0usize, 5usize), 1, 3);
  let result = <Silent<TestError> as TooFewEmitter<'_, TestLexer<'_>>>::emit_too_few(&mut s, err);
  assert!(result.is_ok());
}

#[test]
fn silent_too_many_emitter() {
  let mut s = Silent::<TestError>::new();
  let err = TooMany::new(SimpleSpan::new(0usize, 5usize), 10, 5);
  let result = <Silent<TestError> as TooManyEmitter<'_, TestLexer<'_>>>::emit_too_many(&mut s, err);
  assert!(result.is_ok());
}

#[test]
fn silent_separated_emitter_missing_separator() {
  let mut s = Silent::<TestError>::new();
  let name = CowStr::from_static("comma");
  let err: MissingToken<'_, TokenKind, usize> = MissingToken::new(0usize);
  let result = <Silent<TestError> as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_separator(
    &mut s, name, err,
  );
  assert!(result.is_ok());
}

#[test]
fn silent_separated_emitter_missing_element() {
  let mut s = Silent::<TestError>::new();
  let err = MissingSyntax::new(0usize);
  let result =
    <Silent<TestError> as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_element(&mut s, err);
  assert!(result.is_ok());
}

#[test]
fn silent_unexpected_leading_separator_emitter() {
  let mut s = Silent::<TestError>::new();
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Silent<TestError> as UnexpectedLeadingSeparatorEmitter<'_, TestLexer<'_>>>::emit_unexpected_leading_separator(&mut s, name, err);
  assert!(result.is_ok());
}

#[test]
fn silent_unexpected_trailing_separator_emitter() {
  let mut s = Silent::<TestError>::new();
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Silent<TestError> as UnexpectedTrailingSeparatorEmitter<'_, TestLexer<'_>>>::emit_unexpected_trailing_separator(&mut s, name, err);
  assert!(result.is_ok());
}

#[test]
fn silent_pratt_emitter_lhs() {
  let mut s = Silent::<TestError>::new();
  let err = UnexpectedEoLhs::eolhs(0usize);
  let result =
    <Silent<TestError> as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_lhs(&mut s, err);
  assert!(result.is_ok());
}

#[test]
fn silent_pratt_emitter_rhs() {
  let mut s = Silent::<TestError>::new();
  let err = UnexpectedEoRhs::eorhs(0usize);
  let result =
    <Silent<TestError> as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_rhs(&mut s, err);
  assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fatal emitter: direct trait method calls
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn fatal_full_container_emitter() {
  let mut f = Fatal::<TestError>::new();
  let err = FullContainer::new(SimpleSpan::new(0usize, 5usize), 10, 5);
  let result =
    <Fatal<TestError> as FullContainerEmitter<'_, TestLexer<'_>>>::emit_full_container(&mut f, err);
  assert!(result.is_err());
}

#[test]
fn fatal_too_few_emitter() {
  let mut f = Fatal::<TestError>::new();
  let err = TooFew::new(SimpleSpan::new(0usize, 5usize), 1, 3);
  let result = <Fatal<TestError> as TooFewEmitter<'_, TestLexer<'_>>>::emit_too_few(&mut f, err);
  assert!(result.is_err());
}

#[test]
fn fatal_too_many_emitter() {
  let mut f = Fatal::<TestError>::new();
  let err = TooMany::new(SimpleSpan::new(0usize, 5usize), 10, 5);
  let result = <Fatal<TestError> as TooManyEmitter<'_, TestLexer<'_>>>::emit_too_many(&mut f, err);
  assert!(result.is_err());
}

#[test]
fn fatal_separated_emitter_missing_separator() {
  let mut f = Fatal::<TestError>::new();
  let name = CowStr::from_static("comma");
  let err: MissingToken<'_, TokenKind, usize> = MissingToken::new(0usize);
  let result = <Fatal<TestError> as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_separator(
    &mut f, name, err,
  );
  assert!(result.is_err());
}

#[test]
fn fatal_separated_emitter_missing_element() {
  let mut f = Fatal::<TestError>::new();
  let err = MissingSyntax::new(0usize);
  let result =
    <Fatal<TestError> as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_element(&mut f, err);
  assert!(result.is_err());
}

#[test]
fn fatal_unexpected_leading_separator_emitter() {
  let mut f = Fatal::<TestError>::new();
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Fatal<TestError> as UnexpectedLeadingSeparatorEmitter<'_, TestLexer<'_>>>::emit_unexpected_leading_separator(&mut f, name, err);
  assert!(result.is_err());
}

#[test]
fn fatal_unexpected_trailing_separator_emitter() {
  let mut f = Fatal::<TestError>::new();
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Fatal<TestError> as UnexpectedTrailingSeparatorEmitter<'_, TestLexer<'_>>>::emit_unexpected_trailing_separator(&mut f, name, err);
  assert!(result.is_err());
}

#[test]
fn fatal_pratt_emitter_lhs() {
  let mut f = Fatal::<TestError>::new();
  let err = UnexpectedEoLhs::eolhs(0usize);
  let result =
    <Fatal<TestError> as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_lhs(&mut f, err);
  assert!(result.is_err());
}

#[test]
fn fatal_pratt_emitter_rhs() {
  let mut f = Fatal::<TestError>::new();
  let err = UnexpectedEoRhs::eorhs(0usize);
  let result =
    <Fatal<TestError> as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_rhs(&mut f, err);
  assert!(result.is_err());
}

#[test]
fn fatal_emit_lexer_error() {
  let mut f = Fatal::<TestError>::new();
  let spanned = Spanned::new(SimpleSpan::new(0usize, 5usize), ());
  let result = <Fatal<TestError> as Emitter<'_, TestLexer<'_>>>::emit_lexer_error(&mut f, spanned);
  assert!(result.is_err());
}

#[test]
fn fatal_emit_unexpected_token() {
  let mut f = Fatal::<TestError>::new();
  let ut = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Fatal<TestError> as Emitter<'_, TestLexer<'_>>>::emit_unexpected_token(&mut f, ut);
  assert!(result.is_err());
}

#[test]
fn fatal_clone_copy_debug() {
  let f = Fatal::<TestError>::new();
  let f2 = f.clone();
  let f3 = f;
  let _ = (f2, f3);
  let _ = Fatal::<TestError>::default();
  let dbg = format!("{:?}", Fatal::<TestError>::new());
  assert_eq!(dbg, "Fatal");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Verbose emitter: direct trait method calls
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn verbose_full_container_emitter() {
  let mut v = Verbose::<TestError>::new();
  let err = FullContainer::new(SimpleSpan::new(0usize, 5usize), 10, 5);
  let result = <Verbose<TestError> as FullContainerEmitter<'_, TestLexer<'_>>>::emit_full_container(
    &mut v, err,
  );
  // Verbose full_container returns Err (fatal) just like Fatal
  assert!(result.is_err());
}

#[test]
fn verbose_too_few_emitter() {
  let mut v = Verbose::<TestError>::new();
  let err = TooFew::new(SimpleSpan::new(0usize, 5usize), 1, 3);
  let result = <Verbose<TestError> as TooFewEmitter<'_, TestLexer<'_>>>::emit_too_few(&mut v, err);
  assert!(result.is_ok());
  assert_eq!(v.errors().len(), 1);
}

#[test]
fn verbose_too_many_emitter() {
  let mut v = Verbose::<TestError>::new();
  let err = TooMany::new(SimpleSpan::new(0usize, 5usize), 10, 5);
  let result =
    <Verbose<TestError> as TooManyEmitter<'_, TestLexer<'_>>>::emit_too_many(&mut v, err);
  assert!(result.is_ok());
  assert_eq!(v.errors().len(), 1);
}

#[test]
fn verbose_separated_emitter_missing_separator() {
  let mut v = Verbose::<TestError>::new();
  let name = CowStr::from_static("comma");
  let err: MissingToken<'_, TokenKind, usize> = MissingToken::new(0usize);
  let result = <Verbose<TestError> as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_separator(
    &mut v, name, err,
  );
  assert!(result.is_ok());
  assert_eq!(v.errors().len(), 1);
}

#[test]
fn verbose_separated_emitter_missing_element() {
  let mut v = Verbose::<TestError>::new();
  let err = MissingSyntax::new(0usize);
  let result =
    <Verbose<TestError> as SeparatedEmitter<'_, TestLexer<'_>>>::emit_missing_element(&mut v, err);
  assert!(result.is_ok());
  assert_eq!(v.errors().len(), 1);
}

#[test]
fn verbose_unexpected_leading_separator_emitter() {
  let mut v = Verbose::<TestError>::new();
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Verbose<TestError> as UnexpectedLeadingSeparatorEmitter<'_, TestLexer<'_>>>::emit_unexpected_leading_separator(&mut v, name, err);
  assert!(result.is_ok());
  assert_eq!(v.errors().len(), 1);
}

#[test]
fn verbose_unexpected_trailing_separator_emitter() {
  let mut v = Verbose::<TestError>::new();
  let name = CowStr::from_static("comma");
  let err = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Verbose<TestError> as UnexpectedTrailingSeparatorEmitter<'_, TestLexer<'_>>>::emit_unexpected_trailing_separator(&mut v, name, err);
  assert!(result.is_ok());
  assert_eq!(v.errors().len(), 1);
}

#[test]
fn verbose_pratt_emitter_lhs() {
  let mut v = Verbose::<TestError>::new();
  let err = UnexpectedEoLhs::eolhs(0usize);
  let result = <Verbose<TestError> as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_lhs(
    &mut v, err,
  );
  // Verbose pratt returns Err (fatal) just like Fatal
  assert!(result.is_err());
}

#[test]
fn verbose_pratt_emitter_rhs() {
  let mut v = Verbose::<TestError>::new();
  let err = UnexpectedEoRhs::eorhs(0usize);
  let result = <Verbose<TestError> as PrattEmitter<'_, TestLexer<'_>>>::emit_unexpected_end_of_rhs(
    &mut v, err,
  );
  // Verbose pratt returns Err (fatal) just like Fatal
  assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Ignored emitter: Emitter base trait methods
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn ignored_emit_unexpected_token() {
  let mut ign = Ignored::default();
  let ut = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Ignored as Emitter<'_, TestLexer<'_>>>::emit_unexpected_token(&mut ign, ut);
  assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Silent emitter: Emitter base trait methods
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn silent_emit_lexer_error() {
  let mut s = Silent::<TestError>::new();
  let spanned = Spanned::new(SimpleSpan::new(0usize, 5usize), ());
  let result = <Silent<TestError> as Emitter<'_, TestLexer<'_>>>::emit_lexer_error(&mut s, spanned);
  assert!(result.is_ok());
}

#[test]
fn silent_emit_error() {
  let mut s = Silent::<TestError>::new();
  let spanned = Spanned::new(SimpleSpan::new(0usize, 5usize), TestError::LexerError);
  let result = <Silent<TestError> as Emitter<'_, TestLexer<'_>>>::emit_error(&mut s, spanned);
  assert!(result.is_ok());
}

#[test]
fn silent_emit_unexpected_token() {
  let mut s = Silent::<TestError>::new();
  let ut = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result = <Silent<TestError> as Emitter<'_, TestLexer<'_>>>::emit_unexpected_token(&mut s, ut);
  assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Verbose emitter: Emitter base trait methods (emit_unexpected_token)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn verbose_emit_unexpected_token() {
  let mut v = Verbose::<TestError>::new();
  let ut = UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
  let result =
    <Verbose<TestError> as Emitter<'_, TestLexer<'_>>>::emit_unexpected_token(&mut v, ut);
  assert!(result.is_ok());
  assert_eq!(v.errors().len(), 1);
}
