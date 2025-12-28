use super::*;

/// A trait bound for emitters that handle separated-by syntax errors.
pub trait FromFullContainerError<'a, O: ?Sized, L, Lang: ?Sized = ()> {
  /// Creates an emitter error from a full container error.
  fn from_full_container(err: FullContainer<O, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>;
}

impl<'a, T, O: ?Sized, L, Lang: ?Sized> FromFullContainerError<'a, O, L, Lang> for T
where
  L: Lexer<'a>,
  T: From<FullContainer<O, L::Span, Lang>>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_full_container(err: FullContainer<O, L::Span, Lang>) -> Self
  where
    L: Lexer<'a>,
  {
    err.into()
  }
}

/// An emitter that handles errors related to containers do not have enough capacity for repeated parsers.
pub trait FullContainerEmitter<'a, O: ?Sized, L, Lang: ?Sized = ()>: Emitter<'a, L, Lang> {
  /// Emits an error indicating that the given container is full, and cannot accept more elements.
  fn emit_full_container(
    &mut self,
    err: FullContainer<O, L::Span, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>;
}

impl<'a, O, L, U, Lang: ?Sized> FullContainerEmitter<'a, O, L, Lang> for &mut U
where
  U: FullContainerEmitter<'a, O, L, Lang>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn emit_full_container(&mut self, err: FullContainer<O, L::Span, Lang>) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
  {
    (**self).emit_full_container(err)
  }
}
