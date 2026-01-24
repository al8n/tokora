use crate::{
  emitter::{SeparatedEmitter, TooFewEmitter, TooManyEmitter},
  error::syntax::{TooFew, TooMany},
  input::Checkpoint,
};

use super::*;

/// Combines two values in a type-safe way.
///
/// This type is used throughout the parser system for:
///
/// - Wrapping parser functions with base parsers: `With<F, Parser<()>>`
/// - Building configuration structures: `With<E, C>` for emitter + cache
/// - Nested configurations: `With<PhantomData<L>, With<E, C>>` for ParserOptions
///
/// # Type Parameters
///
/// - `P`: The primary value (typically a parser function or marker)
/// - `S`: The secondary value (typically configuration or a base parser)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct With<P, S> {
  pub(crate) primary: P,
  pub(crate) secondary: S,
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

  /// Maps the primary value using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map_primary<U, F>(self, f: F) -> With<U, S>
  where
    F: FnOnce(P) -> U,
  {
    With {
      primary: f(self.primary),
      secondary: self.secondary,
    }
  }

  /// Maps the secondary value using the given function.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn map_secondary<U, F>(self, f: F) -> With<P, U>
  where
    F: FnOnce(S) -> U,
  {
    With {
      primary: self.primary,
      secondary: f(self.secondary),
    }
  }
}

impl With<Minimum, Maximum> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn check<'inp, 'closure, L, Ctx, Lang: ?Sized>(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    num_elems: usize,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: SeparatedEmitter<'inp, L, Lang>
      + TooFewEmitter<'inp, L, Lang>
      + TooManyEmitter<'inp, L, Lang>,
  {
    let full_span = inp.span_since(ckp.cursor());
    let minimum = self.primary().get();
    let maximum = self.secondary().get();
    if num_elems < minimum {
      inp
        .emitter()
        .emit_too_few(TooFew::of(full_span.clone(), num_elems, minimum))?;
    }

    if num_elems > maximum {
      inp
        .emitter()
        .emit_too_many(TooMany::of(full_span.clone(), num_elems, maximum))?;
    }
    Ok(full_span)
  }
}

impl Minimum {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn check<'inp, 'closure, L, Ctx, Lang: ?Sized>(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    num_elems: usize,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: SeparatedEmitter<'inp, L, Lang> + TooFewEmitter<'inp, L, Lang>,
  {
    let full_span = inp.span_since(ckp.cursor());
    let minimum = self.get();
    if num_elems < minimum {
      inp
        .emitter()
        .emit_too_few(TooFew::of(full_span.clone(), num_elems, minimum))?;
    }
    Ok(full_span)
  }
}

impl Maximum {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn check<'inp, 'closure, L, Ctx, Lang: ?Sized>(
    &self,
    inp: &mut InputRef<'inp, 'closure, L, Ctx, Lang>,
    ckp: &Checkpoint<'inp, 'closure, L>,
    num_elems: usize,
  ) -> Result<L::Span, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Ctx::Emitter: SeparatedEmitter<'inp, L, Lang> + TooManyEmitter<'inp, L, Lang>,
  {
    let full_span = inp.span_since(ckp.cursor());
    let maximum = self.get();
    if num_elems > maximum {
      inp
        .emitter()
        .emit_too_many(TooMany::of(full_span.clone(), num_elems, maximum))?;
    }
    Ok(full_span)
  }
}
