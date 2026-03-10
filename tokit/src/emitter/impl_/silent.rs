use core::marker::PhantomData;

use crate::{error::syntax::MissingSyntaxOf, span::Spanned};

use super::super::*;

mod full_container;
mod pratt;
mod separator;
mod too_few;
mod too_many;
mod unexpected_leading_separator;
mod unexpected_trailing_separator;

/// A silent emitter that treats all errors as non-fatal, and ignores them.
///
/// Compared to [`Ignored`](super::ignored::Ignored) emitter, the error type is preserved.
///
/// `Silent` is a **complete implementation** of all atomic emitter traits, providing a pre-built bundle
/// for error-ignoring behavior. It implements all emitter traits ([`Emitter`](super::super::Emitter),
/// [`TooFewEmitter`](super::super::TooFewEmitter), [`TooManyEmitter`](super::super::TooManyEmitter),
/// etc.) with consistent silent behavior.
///
/// For custom error handling, you can implement only the atomic emitter traits you need rather than
/// using this pre-built bundle.
pub struct Silent<T: ?Sized, Lang: ?Sized = ()> {
  _marker: PhantomData<T>,
  _lang: PhantomData<Lang>,
}

impl<T: ?Sized, Lang: ?Sized> Silent<T, Lang> {
  /// Creates a new `Silent`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      _marker: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<T: ?Sized, Lang: ?Sized> Default for Silent<T, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self {
      _marker: PhantomData,
      _lang: PhantomData,
    }
  }
}

impl<T: ?Sized, Lang: ?Sized> core::fmt::Debug for Silent<T, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Silent")
  }
}

impl<T: ?Sized, Lang: ?Sized> Clone for Silent<T, Lang> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: ?Sized, Lang: ?Sized> Copy for Silent<T, Lang> {}

impl<'a, L, E, Lang: ?Sized> Emitter<'a, L, Lang> for Silent<E, Lang> {
  type Error = E;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_lexer_error(
    &mut self,
    _: Spanned<<L::Token as Token<'a>>::Error, L::Span>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_error(&mut self, _: Spanned<Self::Error, L::Span>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_unexpected_token(&mut self, _: UnexpectedTokenOf<'a, L, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    Ok(())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn rewind(&mut self, cursor: &Cursor<'a, '_, L>)
  where
    L: Lexer<'a>,
  {
    let _ = cursor;
  }
}

#[cfg(test)]
const _: () = {
  use crate::lexer::DummyLexer;

  const fn assert_noop_separated_by_emitter<'a, L, Error, E>()
  where
    L: Lexer<'a>,
    E: SeparatedEmitter<'a, L, Error = Error>,
  {
  }

  assert_noop_separated_by_emitter::<'_, DummyLexer, (), Silent<()>>();
};

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::DummyLexer;
  use crate::span::SimpleSpan;

  #[test]
  fn silent_new() {
    let _s: Silent<()> = Silent::new();
  }

  #[test]
  fn silent_default() {
    let _s: Silent<()> = Silent::default();
  }

  #[test]
  fn silent_debug() {
    let s: Silent<()> = Silent::new();
    assert_eq!(format!("{:?}", s), "Silent");
  }

  #[test]
  fn silent_clone_and_copy() {
    let s: Silent<()> = Silent::new();
    let s2 = s.clone();
    let s3 = s;
    let _ = (s2, s3);
  }

  #[test]
  fn silent_emit_lexer_error_returns_ok() {
    let mut s: Silent<()> = Silent::new();
    let spanned = Spanned::new(SimpleSpan::new(0usize, 5usize), ());
    let result = <Silent<()> as Emitter<'_, DummyLexer>>::emit_lexer_error(&mut s, spanned);
    assert!(result.is_ok());
  }

  #[test]
  fn silent_emit_error_returns_ok() {
    let mut s: Silent<()> = Silent::new();
    let spanned = Spanned::new(SimpleSpan::new(0usize, 5usize), ());
    let result = <Silent<()> as Emitter<'_, DummyLexer>>::emit_error(&mut s, spanned);
    assert!(result.is_ok());
  }

  #[test]
  fn silent_emit_unexpected_token_returns_ok() {
    use crate::error::token::UnexpectedToken;
    use crate::lexer::DummyToken;

    let mut s: Silent<()> = Silent::new();
    let ut: UnexpectedToken<'_, DummyToken, DummyToken, SimpleSpan> =
      UnexpectedToken::new(SimpleSpan::new(0usize, 1usize));
    let result = <Silent<()> as Emitter<'_, DummyLexer>>::emit_unexpected_token(&mut s, ut);
    assert!(result.is_ok());
  }

  #[test]
  fn silent_with_lang_type() {
    struct MyLang;
    let _s: Silent<(), MyLang> = Silent::new();
    let _s2: Silent<(), MyLang> = Silent::default();
    assert_eq!(format!("{:?}", _s), "Silent");
  }
}
