//! The flat driver's CST seam: one node of the caller's kind around every expression.

use crate::{
  Emitter, InputRef, Lexer, ParseContext, Token as _,
  cst::EventMark,
  emitter::CstEmitter,
  input::{Complete, Completeness},
};

use super::super::{NoCst, sealed::Sealed};

/// The steps a seam supplies the flat driver: the entry, and the two exits.
///
/// Its module is private, so it cannot be named outside the crate, and it is hidden so that it is
/// no part of the documented surface.
#[doc(hidden)]
pub trait Seam<'inp, L, Ctx, Lang: ?Sized, Cmpl> {
  /// What the entry hands the exit.
  type Open;

  /// The entry, before anything of the expression is read.
  fn open(
    &self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>,
  ) -> Result<Self::Open, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness;

  /// The exit of a parse that returns `Ok`.
  fn finish(&self, open: Self::Open, input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>)
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness;

  /// The exit of a parse that returns `Err`.
  fn demote(&self, open: Self::Open, input: &mut InputRef<'inp, '_, L, Ctx, Lang, Cmpl>)
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
    Cmpl: Completeness;
}

/// The flat driver's CST seam.
///
/// Sealed: [`NoCst`], the default, and [`WithFlatKind`], which
/// [`with_cst_kind`](super::Flat::with_cst_kind) configures, are its two implementations, both at
/// [`Complete`] alone.
pub trait FlatCst<'inp, L, Ctx, Lang: ?Sized, Cmpl = Complete>:
  Sealed + Seam<'inp, L, Ctx, Lang, Cmpl>
{
}

/// No seam: no node, no event, and no bound on the emitter.
impl<'inp, L, Ctx, Lang: ?Sized> Seam<'inp, L, Ctx, Lang, Complete> for NoCst {
  type Open = ();

  #[inline(always)]
  fn open(
    &self,
    _input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<(), <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error>
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
    Ok(())
  }

  #[inline(always)]
  fn finish(&self, _open: (), _input: &mut InputRef<'inp, '_, L, Ctx, Lang>)
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
  }

  #[inline(always)]
  fn demote(&self, _open: (), _input: &mut InputRef<'inp, '_, L, Ctx, Lang>)
  where
    L: Lexer<'inp>,
    Ctx: ParseContext<'inp, L, Lang>,
  {
  }
}

impl<L, Ctx, Lang: ?Sized> FlatCst<'_, L, Ctx, Lang> for NoCst {}

/// The seam [`with_cst_kind`](super::Flat::with_cst_kind) configures: one node of one kind around
/// every expression.
///
/// The node opens once the leading trivia is crossed, so that trivia lands outside it. The read
/// that finds the expression over is made inside it, so the trailing trivia lands inside. A parse
/// that returns `Err` leaves no node: its tokens are the parent's. Its implementation carries the
/// `Ctx::Emitter: CstEmitter` bound, so a driver with a kind over an emitter without the event
/// channel does not compile.
#[derive(Debug, Clone, Copy)]
pub struct WithFlatKind {
  kind: u16,
}

impl WithFlatKind {
  /// The seam for nodes of `kind`.
  #[inline]
  pub(super) const fn new(kind: u16) -> Self {
    Self { kind }
  }
}

impl Sealed for WithFlatKind {}

impl<'inp, L, Ctx, Lang: ?Sized> Seam<'inp, L, Ctx, Lang, Complete> for WithFlatKind
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: CstEmitter<'inp, L, Lang>,
{
  type Open = EventMark;

  #[inline(always)]
  fn open(
    &self,
    input: &mut InputRef<'inp, '_, L, Ctx, Lang>,
  ) -> Result<EventMark, <Ctx::Emitter as Emitter<'inp, L, Lang>>::Error> {
    // Leading trivia is crossed before the node opens, so it lands in the parent. The token the
    // skip stops at is staged, and the driver's first read takes it.
    input.skip_while(|tok| tok.is_trivia())?;
    Ok(input.emitter().cst_start(self.kind))
  }

  #[inline(always)]
  fn finish(&self, _open: EventMark, input: &mut InputRef<'inp, '_, L, Ctx, Lang>) {
    input.emitter().cst_finish(self.kind);
  }

  #[inline(always)]
  fn demote(&self, open: EventMark, input: &mut InputRef<'inp, '_, L, Ctx, Lang>) {
    input.emitter().cst_demote(open, self.kind);
  }
}

impl<'inp, L, Ctx, Lang: ?Sized> FlatCst<'inp, L, Ctx, Lang> for WithFlatKind
where
  L: Lexer<'inp>,
  Ctx: ParseContext<'inp, L, Lang>,
  Ctx::Emitter: CstEmitter<'inp, L, Lang>,
{
}
