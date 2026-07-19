//! Emitter capability for the unclosed-delimiter diagnostic.
//!
//! [`UnclosedEmitter`] is the delimiter-family twin of
//! [`FullContainerEmitter`](crate::emitter::FullContainerEmitter): the additive,
//! atomically-composable emit surface the delimited many-builders
//! (`.delimited::<D>().collect()`) reach for when an opener has been committed but the
//! matching closer never arrives before end-of-input.

use crate::{Lexer, error::Unclosed};

use super::Emitter;

/// An emitter that handles the [`Unclosed`] diagnostic — an opening delimiter that was
/// committed but whose matching closer never arrived before end-of-input.
///
/// This is the delimiter-family analogue of
/// [`FullContainerEmitter`](crate::emitter::FullContainerEmitter): an additive sub-trait the
/// delimited many-builders (`.delimited::<D>().collect()`) require so an unterminated list is
/// reported *through the emitter* rather than silently accepted. Following the house emit
/// discipline, a fail-fast emitter ([`Fatal`](crate::emitter::Fatal)) turns the emission into
/// `Err` via the `From<Unclosed<…>>` conversion; a recovering emitter
/// ([`Verbose`](crate::emitter::Verbose)) records it and lets the parse return the elements
/// collected so far; a dropping emitter ([`Silent`](crate::emitter::Silent),
/// [`Ignored`](crate::utils::marker::Ignored)) discards it.
///
/// The [`Unclosed`] carries the **opening** delimiter's span — so the diagnostic points at the
/// opener that was never closed — and the delimiter pair's name
/// ([`Delimiter::name`](crate::delimiter::Delimiter::name)).
pub trait UnclosedEmitter<'a, L, Lang: ?Sized = ()>: Emitter<'a, L, Lang> {
  /// Emits the [`Unclosed`] diagnostic for a delimiter whose opener was committed but whose
  /// closer never arrived before end-of-input.
  ///
  /// The `Delimiter` type parameter is the type-level delimiter tag carried by
  /// [`Unclosed`]; the diagnostic's span is the opener's span and its name is the delimiter
  /// pair's name.
  fn emit_unclosed<Delimiter>(
    &mut self,
    err: Unclosed<Delimiter, L::Span, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
    Self::Error: From<Unclosed<Delimiter, L::Span, Lang>>;
}

impl<'a, L, U, Lang: ?Sized> UnclosedEmitter<'a, L, Lang> for &mut U
where
  U: UnclosedEmitter<'a, L, Lang>,
{
  #[inline(always)]
  fn emit_unclosed<Delimiter>(
    &mut self,
    err: Unclosed<Delimiter, L::Span, Lang>,
  ) -> Result<(), Self::Error>
  where
    L: Lexer<'a>,
    Self::Error: From<Unclosed<Delimiter, L::Span, Lang>>,
  {
    (**self).emit_unclosed(err)
  }
}
